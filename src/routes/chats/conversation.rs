use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use super::chatbot::process_bot_message;

use crate::routes::chats::messages::{Message, NewMessage};

#[get("/messages")]
pub async fn get_messages(
    db: web::Data<PgPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user_id = match query.get("userId") {
        Some(id) => id.clone(),
        None => return HttpResponse::BadRequest().body("Missing userId"),
    };

    let result = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE user_id = $1 ORDER BY timestamp ASC",
    )
    .bind(user_id)
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => {
            eprintln!("Error fetching messages: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/messages")]
pub async fn post_message(
    db: web::Data<PgPool>,
    body: web::Json<NewMessage>,
) -> impl Responder {
    let msg = body.into_inner();

    let exists_result = sqlx::query_scalar::<_, Option<String>>(
        "SELECT user_id FROM user_bot_settings WHERE user_id = $1",
    )
    .bind(&msg.user_id)
    .fetch_optional(db.get_ref())
    .await;

    if let Err(e) = exists_result {
        eprintln!("Check user_bot_settings error: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    if exists_result.unwrap().is_none() {
        let insert_result = sqlx::query(
            "INSERT INTO user_bot_settings (user_id) VALUES ($1)",
        )
        .bind(&msg.user_id)
        .execute(db.get_ref())
        .await;

        if let Err(e) = insert_result {
            eprintln!("Insert into user_bot_settings error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    let bot_enabled_result = sqlx::query_scalar::<_, bool>(
        "SELECT bot_enabled FROM user_bot_settings WHERE user_id = $1",
    )
    .bind(&msg.user_id)
    .fetch_one(db.get_ref())
    .await;

    let bot_enabled = match bot_enabled_result {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Fetch bot_enabled error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let receiver = if bot_enabled { "bot" } else { "admin" };

    let result = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (user_id, content, timestamp, sender, receiver) 
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(&msg.user_id)
    .bind(&msg.content)
    .bind(msg.timestamp)
    .bind(&msg.sender)
    .bind(receiver)
    .fetch_one(db.get_ref())
    .await;

    match result {
    Ok(saved) => {
        if bot_enabled {
            // Fire and forget
            let user_id = msg.user_id.clone();
            let content = msg.content.clone();
            let db_clone = db.clone();

            actix_web::rt::spawn(async move {
                if let Err(e) =process_bot_message(&user_id, &content, db_clone.get_ref()).await {
                    eprintln!("Bot processing error: {:?}", e);
                }
            });
        }

        HttpResponse::Ok().json(serde_json::json!({
            "message": saved
        }))
    }
    Err(e) => {
        eprintln!("Insert message error: {:?}", e);
        HttpResponse::InternalServerError().finish()
    }
}
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_messages);
    cfg.service(post_message);
}