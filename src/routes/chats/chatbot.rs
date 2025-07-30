use sqlx::{Row, PgPool};
use chrono::Utc;
use serde_json::{json, Value};
use reqwest::Client;
use std::collections::HashSet;
use std::env;
use sqlx::types::BigDecimal;
use num_traits::cast::ToPrimitive;

fn extract_json_from_text(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if start < end {
        Some(text[start..=end].to_string())
    } else {
        None
    }
}

pub async fn process_bot_message(user_id: &str, user_message: &str, db: &PgPool) -> anyhow::Result<String> {
    let system_prompt = r#"
    You are a backend assistant for an e-commerce laptop store in Nepal. Shop name is epasal.

DO NOT generate product listings or product data.
ONLY return a JSON object like:
{
  "action": "search",
  "filters": {
    "brand_name": "acer",
    "ram": 16,
    "graphic": "rtx 3050",
    "show_price": { "lte": 180000 }
  }
}

NEVER include descriptions, prices, or recommendations yourself.
NEVER guess what laptops are available.

User filters may include:
- brand_name, model_name, model_year, display_name, product_type, suitable_for, color,
  processor_generation, processor, processor_series, ram, ram_type, storage, storage_type,
  graphic, graphic_ram, display, display_type, touchscreen, power_supply, battery, warranty,
  show_price (in NPR)

Once enough filters are collected from the user, return the JSON.
Otherwise, keep asking clarifying questions to get more filter info.

Until enough information is gathered, keep the conversation friendly and natural.
"#;

    println!("Fetching previous messages for user_id: {}", user_id);
    let rows = sqlx::query("SELECT sender, content FROM messages WHERE user_id = $1 ORDER BY timestamp ASC")
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let mut messages = Vec::new();

    messages.push(json!({
        "role": "system",
        "content": system_prompt.trim()
    }));

    for row in rows {
        let sender: String = row.try_get("sender")?;
        let content: String = row.try_get("content")?;
        let role = match sender.as_str() {
            "user" => "user",
            "bot" => "assistant",
            _ => "user",
        };
        messages.push(json!({
            "role": role,
            "content": content
        }));
    }

    messages.push(json!({
        "role": "user",
        "content": user_message
    }));

    println!("Calling chat API with current user message...");
    let client = Client::new();
    let res = client
        .post("http://localhost:11434/api/chat")
        .json(&json!({
            "model": "gemma3",
            "messages": messages,
            "stream": false
        }))
        .send()
        .await?;

    let response_json: Value = res.json().await?;
    //println!("{}",response_json);
    let bot_response = response_json["message"]["content"]
        .as_str()
        .unwrap_or("Sorry, I didn’t understand that.")
        .to_string();

    let json_candidate = extract_json_from_text(&bot_response);

if let Some(json_str) = json_candidate {
    if let Ok(value) = serde_json::from_str::<Value>(&json_str) {
        if value["action"] == "search" {
            let filters_obj = value.get("filters").and_then(|f| f.as_object());
            let filters = match filters_obj {
                Some(f) => f,
                None => {
                    let ask_more = "Could you please give me more details like your budget, RAM, storage, or use case (e.g., gaming, study, editing)?";
                    sqlx::query("INSERT INTO messages (user_id, content, timestamp, sender, receiver) VALUES ($1, $2, $3, 'bot', 'user')")
                        .bind(user_id)
                        .bind(ask_more)
                        .bind(Utc::now())
                        .execute(db)
                        .await?;
                    return Ok(ask_more.to_string());
                }
            };

            let mut query = String::from("SELECT id, display_name, show_price FROM laptop_details WHERE 1=1");
            let mut args = Vec::new();

            println!("Building SQL query with filters...");

            let numeric_columns = HashSet::from(["ram", "graphic_ram", "show_price", "model_year", "storage"]);

            for (key, val) in filters {
                if numeric_columns.contains(key.as_str()) {
                    if val.is_object() {
                        if let Some(obj) = val.as_object() {
                            for (op, v) in obj {
                                let val_num = v.as_i64().unwrap_or(0);
                                match op.as_str() {
                                    "lte" => {
                                        query.push_str(&format!(" AND {} <= ${}", key, args.len() + 1));
                                        args.push(val_num.to_string());
                                    }
                                    "gte" => {
                                        query.push_str(&format!(" AND {} >= ${}", key, args.len() + 1));
                                        args.push(val_num.to_string());
                                    }
                                    "eq" => {
                                        query.push_str(&format!(" AND {} = ${}", key, args.len() + 1));
                                        args.push(val_num.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else if val.is_number() {
                        let val_num = val.as_i64().unwrap_or(0);
                        query.push_str(&format!(" AND {} = ${}", key, args.len() + 1));
                        args.push(val_num.to_string());
                    } else if val.is_string() {
                        if let Ok(val_num) = val.as_str().unwrap_or("").parse::<i64>() {
                            query.push_str(&format!(" AND {} = ${}", key, args.len() + 1));
                            args.push(val_num.to_string());
                        }
                    }
                } else {
                    if let Some(val_str) = val.as_str() {
                        query.push_str(&format!(" AND {} ILIKE ${}", key, args.len() + 1));
                        args.push(format!("%{}%", val_str));
                    }
                }
            }

            if args.len() <= 2 {
                let ask_more = "Could you please tell me a bit more, like your budget, RAM, storage, or intended use (e.g., gaming, study, editing)? This will help me suggest the best laptops for you.";
                sqlx::query("INSERT INTO messages (user_id, content, timestamp, sender, receiver) VALUES ($1, $2, $3, 'bot', 'user')")
                    .bind(user_id)
                    .bind(ask_more)
                    .bind(Utc::now())
                    .execute(db)
                    .await?;
                return Ok(ask_more.to_string());
            }

            query.push_str(" ORDER BY show_price ASC LIMIT 4");

            let mut q = sqlx::query(&query);
            for arg in &args {
    if let Ok(int_val) = arg.parse::<i64>() {
        q = q.bind(int_val);
    } 
    else if let Ok(float_val) = arg.parse::<f64>() {
        q = q.bind(float_val);
    } 
    else {
        q = q.bind(arg);
    }
}

            println!("Executing SQL query to fetch laptops...");
            let rows = q.fetch_all(db).await?;

            if rows.is_empty() {
                return Ok("Sorry, no laptops matched your preferences. Would you like to try different filters?".to_string());
            }

            // Build the response links
            println!("Generating response links for found laptops...");
            let base_url = env::var("BASE_URL").unwrap_or_else(|_| "https://localhost:5173".to_string());
            let mut links = Vec::new();
for row in rows {
    let id: i32 = row.try_get("id")?;
    let name: String = row.try_get("display_name")?;
    let price: BigDecimal = row.try_get("show_price")?;
    let price_f64 = price.to_f64().unwrap_or(0.0);

    links.push(format!(
        "- [{}](https://{}/products?id={}) - NPR {:.2}",
        name,
        base_url.trim_start_matches("https://"),
        id,
        price_f64
    ));
}

let response_text = format!(
    "Here are some laptops I found for you:\n{}",
    links.join("\n")
);
            // Save bot reply in DB
            sqlx::query("INSERT INTO messages (user_id, content, timestamp, sender, receiver) VALUES ($1, $2, $3, 'bot', 'user')")
                .bind(user_id)
                .bind(&response_text)
                .bind(Utc::now())
                .execute(db)
                .await?;

            return Ok(response_text);
        } else {
            sqlx::query("INSERT INTO messages (user_id, content, timestamp, sender, receiver) VALUES ($1, $2, $3, 'bot', 'user')")
                .bind(user_id)
                .bind(&bot_response)
                .bind(Utc::now())
                .execute(db)
                .await?;

            return Ok(bot_response);
        }
    } else {
        sqlx::query("INSERT INTO messages (user_id, content, timestamp, sender, receiver) VALUES ($1, $2, $3, 'bot', 'user')")
            .bind(user_id)
            .bind(&bot_response)
            .bind(Utc::now())
            .execute(db)
            .await?;

        return Ok(bot_response);
    }
}else{
    sqlx::query("INSERT INTO messages (user_id, content, timestamp, sender, receiver) VALUES ($1, $2, $3, 'bot', 'user')")
    .bind(user_id)
    .bind(&bot_response)
    .bind(Utc::now())
    .execute(db)
    .await?;

return Ok(bot_response);
}
}