use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Responder};
use futures_util::StreamExt as _;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use std::{fs, io::Write};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct LaptopForm {
    pub brand_name: String,
    pub display_name: String,
    pub product_authetication: String,
    pub model_name: String,
    pub model_year: i32,
    pub product_type: String,
    pub suitable_for: String,
    pub color: String,
    pub ram: i32,
    pub ram_type: String,
    pub processor: String,
    pub processor_series: String,
    pub processor_generation: String,
    pub storage: i32,
    pub storage_type: String,
    pub warranty: String,
    pub graphic: String,
    pub graphic_ram: i32,
    pub display: String,
    pub display_type: String,
    pub battery: String,
    pub power_supply: String,
    pub touchscreen: bool,
    pub cost_price: f64,
    pub quantity: i32,
}

#[post("/api/insertion")]
pub async fn insert_laptop(pool: web::Data<PgPool>, mut multipart: Multipart) -> impl Responder {
    let mut form_data: Option<LaptopForm> = None;
    let mut face_image: Option<Vec<u8>> = None;
    let mut side_images: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().body(format!("Error reading field: {}", e)),
        };

        let name = field.content_disposition().get_name().map(|n| n.to_string()).unwrap_or_default();

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => return HttpResponse::BadRequest().body(format!("Error reading chunk: {}", e)),
            };
            data.extend_from_slice(&chunk);
        }

        match name.as_str() {
            "form" => {
                match serde_json::from_slice::<LaptopForm>(&data) {
                    Ok(f) => form_data = Some(f),
                    Err(e) => return HttpResponse::BadRequest().body(format!("Invalid form JSON: {}", e)),
                }
            }
            "faceImage" => face_image = Some(data),
            "sideImages[]" => side_images.push(data),
            _ => {}
        }
    }

    let form = match form_data {
        Some(f) => f,
        None => return HttpResponse::BadRequest().body("Missing form data"),
    };

    let face_image = match face_image {
        Some(img) => img,
        None => return HttpResponse::BadRequest().body("Missing faceImage"),
    };

    if laptop_exists(&pool, &form).await {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Laptop already exists in the database"
        }));
    }

    // Save face image
    let face_path = format!("uploads/laptops/face/{}.jpg", Uuid::new_v4());
    fs::create_dir_all("uploads/laptops/face").ok();
    if let Err(e) = fs::File::create(&face_path).and_then(|mut f| f.write_all(&face_image)) {
        return HttpResponse::InternalServerError().body(format!("Error saving face image: {}", e));
    }

    let inserted_laptop_id: i32 = match sqlx::query(
        r#"
        INSERT INTO laptop_details (
            brand_name, display_name, product_authetication, model_name, model_year,
            product_type, suitable_for, color, ram, ram_type,
            processor, processor_series, processor_generation, storage, storage_type,
            warranty, graphic, graphic_ram, display, display_type,
            battery, power_supply, touchscreen, cost_price, quantity,
            face_image_url
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15,
            $16, $17, $18, $19, $20,
            $21, $22, $23, $24, $25,
            $26
        )
        RETURNING id
        "#,
    )
    .bind(&form.brand_name)
    .bind(&form.display_name)
    .bind(&form.product_authetication)
    .bind(&form.model_name)
    .bind(form.model_year)
    .bind(&form.product_type)
    .bind(&form.suitable_for)
    .bind(&form.color)
    .bind(form.ram)
    .bind(&form.ram_type)
    .bind(&form.processor)
    .bind(&form.processor_series)
    .bind(&form.processor_generation)
    .bind(form.storage)
    .bind(&form.storage_type)
    .bind(&form.warranty)
    .bind(&form.graphic)
    .bind(form.graphic_ram)
    .bind(&form.display)
    .bind(&form.display_type)
    .bind(&form.battery)
    .bind(&form.power_supply)
    .bind(form.touchscreen)
    .bind(form.cost_price)
    .bind(form.quantity)
    .bind(&face_path)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => match row.try_get("id") {
            Ok(id) => id,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to retrieve inserted laptop ID: {}", e)
                }));
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to insert laptop: {}", e)
            }));
        }
    };

    // Save side images
    fs::create_dir_all("uploads/laptops/side").ok();
    for img in side_images {
        let path = format!("uploads/laptops/side/{}.jpg", Uuid::new_v4());
        if let Err(e) = fs::File::create(&path).and_then(|mut f| f.write_all(&img)) {
            return HttpResponse::InternalServerError().body(format!("Error saving side image: {}", e));
        }

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO laptop_side_images (laptop_id, image_url)
            VALUES ($1, $2)
            "#
        )
        .bind(inserted_laptop_id)
        .bind(&path)
        .execute(pool.get_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to insert side image path: {}", e)
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Laptop inserted successfully"
    }))
}

async fn laptop_exists(pool: &PgPool, form: &LaptopForm) -> bool {
    let row = sqlx::query(
        r#"
        SELECT id FROM laptop_details
        WHERE brand_name = $1 AND model_name = $2 AND model_year = $3 AND product_type = $4
        "#,
    )
    .bind(&form.brand_name)
    .bind(&form.model_name)
    .bind(form.model_year)
    .bind(&form.product_type)
    .fetch_optional(pool)
    .await;

    matches!(row, Ok(Some(_)))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(insert_laptop);
}