use actix_web::{get, HttpResponse, Responder};
use sqlx::PgPool;
use num_format::{Locale, ToFormattedString};
use actix_web::web;

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct LaptopRaw {
    pub id: i32,
    pub face_image_url: String,
    pub product_authentication: String,
    pub show_price: f64,
    pub display_name: String,
}

#[derive(serde::Serialize)]
pub struct LaptopFrontend {
    pub id: i32,
    pub image: String,
    pub show_price: String,
    pub tag: String,
    pub display_name: String,
}

#[get("/api/top-picks")]
pub async fn top_picks(pool: actix_web::web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query_as::<_, LaptopRaw>(
        r#"
       SELECT 
    id, face_image_url, product_authentication, show_price::FLOAT8 AS show_price, display_name
FROM laptop_details
WHERE face_image_url IS NOT NULL
ORDER BY RANDOM()
LIMIT 15;
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(laptops) => {
            let mapped: Vec<LaptopFrontend> = laptops
                .into_iter()
                .map(|item| LaptopFrontend {
                    id: item.id,
                    image: item.face_image_url,
                    show_price: format!("Rs{}", (item.show_price as u64).to_formatted_string(&Locale::en)),
                    tag: item.product_authentication,
                    display_name: item.display_name,
                })
                .collect();

            HttpResponse::Ok().json(mapped)
        }
        Err(e) => {
            eprintln!("Error fetching top picks: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch top picks")
        }
    }
}


pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(top_picks);
}