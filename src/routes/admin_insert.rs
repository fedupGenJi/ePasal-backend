use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Responder};
use futures_util::StreamExt as _;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Deserialize, Debug)]
pub struct LaptopForm {
    pub brand_name: String,
    pub model_name: String,
    pub model_year: i32,
    pub product_type: String,
    pub cost_price: f64,
    pub ram: i32,
    pub ram_type: String,
    pub storage: i32,
    pub storage_type: String,
    pub processor: String,
    pub processor_series: String,
    pub graphic_ram: i32,
    pub graphic: String,
    pub warranty: String,
    pub display: String,
    pub display_type: String,
    pub quantity: i32,
    pub touchscreen: bool,
}

#[post("/api/insertion")]
pub async fn insert_laptop(
    pool: web::Data<PgPool>,
    mut multipart: Multipart,
) -> impl Responder {
    let mut form_data: Option<LaptopForm> = None;
    let mut face_image: Option<Vec<u8>> = None;
    let mut side_images: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::BadRequest().body(format!("Error reading field: {}", e));
            }
        };

        let name = field
    .content_disposition()
    .get_name()
    .map(|n| n.to_string())
    .unwrap_or_default();

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
            "faceImage" => {
                face_image = Some(data);
            }
            "sideImages[]" => {
                side_images.push(data);
            }
            _ => {
                //ignore
            }
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

let inserted_laptop_id: i32 = match sqlx::query(
    r#"
    INSERT INTO laptop_details (
        brand_name, model_name, model_year, product_type, cost_price,
        ram, ram_type, storage, storage_type,
        processor, processor_series, graphic_ram, graphic,
        warranty, display, display_type, face_image, quantity, touchscreen
    )
    VALUES (
        $1, $2, $3, $4, $5,
        $6, $7, $8, $9,
        $10, $11, $12, $13,
        $14, $15, $16, $17, $18, $19
    )
    RETURNING id
    "#
)
.bind(&form.brand_name)
.bind(&form.model_name)
.bind(form.model_year)
.bind(&form.product_type)
.bind(form.cost_price)
.bind(form.ram)
.bind(&form.ram_type)
.bind(form.storage)
.bind(&form.storage_type)
.bind(&form.processor)
.bind(&form.processor_series)
.bind(form.graphic_ram)
.bind(&form.graphic)
.bind(&form.warranty)
.bind(&form.display)
.bind(&form.display_type)
.bind(&face_image)
.bind(form.quantity)
.bind(form.touchscreen)
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

    for img in side_images {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO laptop_side_images (laptop_id, image)
            VALUES ($1, $2)
            "#
        )
        .bind(inserted_laptop_id)
        .bind(img)
        .execute(pool.get_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to insert side image: {}", e)
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