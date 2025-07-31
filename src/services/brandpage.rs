use actix_web::web;
use actix_web::get;
use sqlx::PgPool;
use actix_web::Responder;
use actix_web::HttpResponse;
use crate::services::toppicks::LaptopRaw;
use crate::services::toppicks::LaptopFrontend;
use num_format::Locale;
use num_format::ToFormattedString;

#[get("/api/brand/{brand_name}")]
pub async fn laptops_by_brand(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> impl Responder {
    let brand_name_raw = path.into_inner().to_lowercase();

    // Define allowed brands and normalize mapping (if needed)
    let valid_brands: std::collections::HashMap<&str, &str> = [
        ("acer", "acer"),
        ("asus", "asus"),
        ("lenovo", "lenovo"),
        ("msi", "msi"),
    ]
    .iter()
    .cloned()
    .collect();

    let Some(valid_brand) = valid_brands.get(brand_name_raw.as_str()) else {
        return HttpResponse::NotFound().body("Brand not supported");
    };

    let rows = sqlx::query_as::<_, LaptopRaw>(
        r#"
        SELECT id, face_image_url, product_authentication, show_price::FLOAT8 AS show_price, display_name
        FROM laptop_details
        WHERE LOWER(brand_name) = $1 AND face_image_url IS NOT NULL
        ORDER BY RANDOM()
        LIMIT 12;
        "#
    )
    .bind(*valid_brand)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(laptops) => {
            let mapped: Vec<LaptopFrontend> = laptops
                .into_iter()
                .map(|item| LaptopFrontend {
                    id: item.id,
                    image: item.face_image_url,
                    tag: item.product_authentication,
                    show_price: format!(
                        "Rs{}",
                        (item.show_price as u64).to_formatted_string(&Locale::en)
                    ),
                    display_name: item.display_name,
                })
                .collect();

            HttpResponse::Ok().json(mapped)
        }
        Err(e) => {
            eprintln!("Error fetching laptops by brand: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch laptops")
        }
    }
}


pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(laptops_by_brand);
}