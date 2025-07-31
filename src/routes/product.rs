use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use num_traits::ToPrimitive;

#[derive(Serialize)]
pub struct ProductDetails {
    brand_name: String,
    display_name: String,
    model_name: String,
    model_year: Option<i32>,
    product_type: Option<String>,
    product_authetication: Option<String>,
    suitable_for: Option<String>,
    color: Option<String>,
    ram: Option<i32>,
    ram_type: Option<String>,
    processor: Option<String>,
    processor_series: Option<String>,
    processor_generation: Option<String>,
    storage: Option<i32>,
    storage_type: Option<String>,
    warranty: Option<String>,
    graphic: Option<String>,
    graphic_ram: Option<i32>,
    display: Option<String>,
    display_type: Option<String>,
    battery: Option<String>,
    power_supply: Option<String>,
    touchscreen: Option<bool>,
    cost_price: f64,
    quantity: i32,
    face_image: Option<String>,
    side_images: Vec<String>,
}

#[get("/api/products/{id}")]
async fn get_product(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    let product = sqlx::query!(
        "
        SELECT brand_name, display_name, model_name, model_year, product_type, product_authentication, suitable_for,
               color, processor_generation, processor, processor_series, ram, ram_type, storage, storage_type,
               warranty, graphic, graphic_ram, display, display_type, battery, power_supply, touchscreen, cost_price,
               quantity, face_image_url
        FROM laptop_details
        WHERE id = $1
        ",
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    let product = match product {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().body("Product not found"),
        Err(e) => {
            eprintln!("DB query error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let side_images_result = sqlx::query!(
        "SELECT image_url FROM laptop_side_images WHERE laptop_id = $1 ORDER BY id ASC",
        id
    )
    .fetch_all(pool.get_ref())
    .await;

    let side_images = match side_images_result {
        Ok(images) => images.into_iter()
                            .filter_map(|r| r.image_url)
                            .collect(),
        Err(e) => {
            eprintln!("Error fetching side images: {:?}", e);
            Vec::new()
        }
    };

    let response = ProductDetails {
    brand_name: product.brand_name,
    display_name: product.display_name.unwrap_or_else(|| "Unknown".to_string()),
    model_name: product.model_name,
    model_year: product.model_year,
    product_type: product.product_type,
    product_authetication: product.product_authentication,
    suitable_for: product.suitable_for,
    color: product.color,
    ram: product.ram,
    ram_type: product.ram_type,
    processor: product.processor,
    processor_series: product.processor_series,
    processor_generation: product.processor_generation,
    storage: product.storage,
    storage_type: product.storage_type,
    warranty: product.warranty,
    graphic: product.graphic,
    graphic_ram: product.graphic_ram,
    display: product.display,
    display_type: product.display_type,
    battery: product.battery,
    power_supply: product.power_supply,
    touchscreen: product.touchscreen,
    cost_price: product.cost_price.to_f64().unwrap_or(0.0),
    quantity: product.quantity.unwrap_or(0),
    face_image: product.face_image_url,
    side_images: side_images,
};

    HttpResponse::Ok().json(response)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_product);
}