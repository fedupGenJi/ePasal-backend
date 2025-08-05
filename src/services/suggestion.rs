use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::FromRow;
use serde::Deserialize;

#[derive(Serialize, FromRow)]
pub struct LaptopResponse {
    id: i32,
    image: Option<String>,
    display_name: String,
    show_price: String,
    tag: String,
}

#[derive(Deserialize)]
pub struct SuggestionQuery {
    search: Option<String>,
}

#[get("/api/productshow/suggestion")]
async fn get_suggestion(
    pool: web::Data<PgPool>,
    query: web::Query<SuggestionQuery>,
) -> impl Responder {
    if let Some(search) = &query.search {
        let sql = r#"
            SELECT 
                id,
                face_image_url as image,
                display_name,
                show_price::TEXT,
                product_authentication as tag
            FROM laptop_details
            WHERE to_tsvector('english', 
                coalesce(brand_name, '') || ' ' || 
                coalesce(model_name, '') || ' ' || 
                coalesce(display_name, '') || ' ' || 
                coalesce(product_type, '') || ' ' || 
                coalesce(product_authentication, '') || ' ' || 
                coalesce(suitable_for, '') || ' ' || 
                coalesce(color, '') || ' ' || 
                coalesce(processor_generation, '') || ' ' || 
                coalesce(processor, '') || ' ' || 
                coalesce(processor_series, '') || ' ' || 
                coalesce(ram_type, '') || ' ' || 
                coalesce(storage_type, '') || ' ' || 
                coalesce(graphic, '') || ' ' || 
                coalesce(display, '') || ' ' || 
                coalesce(display_type, '') || ' ' || 
                coalesce(power_supply, '') || ' ' || 
                coalesce(battery, '') || ' ' || 
                coalesce(warranty, '')
            ) @@ plainto_tsquery('english', $1)
            ORDER BY ts_rank(
                to_tsvector('english', 
                    coalesce(brand_name, '') || ' ' || 
                    coalesce(model_name, '') || ' ' || 
                    coalesce(display_name, '') || ' ' || 
                    coalesce(product_type, '') || ' ' || 
                    coalesce(product_authentication, '') || ' ' || 
                    coalesce(suitable_for, '') || ' ' || 
                    coalesce(color, '') || ' ' || 
                    coalesce(processor_generation, '') || ' ' || 
                    coalesce(processor, '') || ' ' || 
                    coalesce(processor_series, '') || ' ' || 
                    coalesce(ram_type, '') || ' ' || 
                    coalesce(storage_type, '') || ' ' || 
                    coalesce(graphic, '') || ' ' || 
                    coalesce(display, '') || ' ' || 
                    coalesce(display_type, '') || ' ' || 
                    coalesce(power_supply, '') || ' ' || 
                    coalesce(battery, '') || ' ' || 
                    coalesce(warranty, '')
                ), plainto_tsquery('english', $1)
            ) DESC
            LIMIT 5
        "#;

        let laptops = sqlx::query_as::<_, LaptopResponse>(sql)
            .bind(search)
            .fetch_all(pool.get_ref())
            .await;

        match laptops {
            Ok(results) => HttpResponse::Ok().json(results),
            Err(e) => {
                eprintln!("DB error: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    } else {
        HttpResponse::Ok().json(Vec::<LaptopResponse>::new())
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_suggestion);
}