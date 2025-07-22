use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Serialize)]
struct InventoryItem {
    id: i32,
    name: String,
    image: String,
    product_type: String,
    quantity: i32,
}

#[get("/api/inventory")]
async fn get_inventory(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query(
        r#"
        SELECT id, brand_name, model_name, model_year, face_image_url, product_authentication, quantity
        FROM laptop_details
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let items: Vec<InventoryItem> = rows.into_iter().map(|row| {
                let id: i32 = row.get("id");
                let brand_name: Option<String> = row.get("brand_name");
                let model_name: Option<String> = row.get("model_name");
                let model_year: Option<i32> = row.get("model_year");
                let face_image_url: Option<String> = row.get("face_image_url");
                let product_authentication: Option<String> = row.get("product_authentication");
                let quantity: Option<i32> = row.get("quantity");

                let model_year_val = model_year.unwrap_or(0);
                let brand = brand_name.unwrap_or_else(|| "".to_string());
                let model = model_name.unwrap_or_else(|| "".to_string());
                let name = format!("{} {} {}", brand, model, model_year_val).trim().to_string();

                let image = match face_image_url {
                    Some(url) if !url.trim().is_empty() => url,
                    _ => String::new(),
                };

                let product_type = product_authentication.unwrap_or_else(|| "Unknown".to_string());

                InventoryItem {
                    id,
                    name,
                    image,
                    product_type,
                    quantity: quantity.unwrap_or(0),
                }
            }).collect();

            HttpResponse::Ok().json(items)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("Failed to fetch inventory: {}", e)
        })),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_inventory);
}