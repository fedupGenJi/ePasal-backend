use actix_web::{get, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use sqlx::PgPool;

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
    let result = sqlx::query!(
        r#"
        SELECT id, brand_name, model_name, model_year, face_image, product_type, quantity
        FROM laptop_details
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let items: Vec<InventoryItem> = rows
                .into_iter()
                .map(|row| InventoryItem {
                    id: row.id,
                    name: format!(
                        "{} {} {}",
                        row.brand_name,
                        row.model_name,
                        row.model_year.unwrap_or(0)
                    ),
                    image: general_purpose::STANDARD.encode(row.face_image.unwrap_or_default()),
                    product_type: row.product_type.unwrap_or_else(|| "Unknown".to_string()),
                    quantity: row.quantity.unwrap_or(0),
                })
                .collect();

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