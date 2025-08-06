use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use serde::Serialize;
use num_traits::ToPrimitive;

#[derive(Serialize)]
struct LaptopSummary {
    id: i32,
    brand_name: String,
    model_name: String,
    face_image_url: Option<String>,
    total_quantity_sold: i64,
    total_revenue: f64,
}

#[get("/api/admin/dashboard")]
async fn get_dashboard_data(db_pool: web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query!(
    r#"
    SELECT 
        ld.id,
        ld.brand_name,
        ld.model_name,
        ld.face_image_url,
        SUM(ls.quantity) AS total_quantity_sold,
        SUM(ls.price_at_sale * ls.quantity) AS total_revenue
    FROM 
        laptop_details ld
    LEFT JOIN
        laptops_sold ls ON ld.id = ls.laptop_id
    GROUP BY 
        ld.id, ld.brand_name, ld.model_name, ld.face_image_url
    HAVING 
        SUM(ls.quantity) > 0
    ORDER BY
        total_quantity_sold DESC
    LIMIT 10
    "#
)
.fetch_all(db_pool.get_ref())
.await;

    match rows {
        Ok(records) => {
            let result: Vec<LaptopSummary> = records.into_iter().map(|r| LaptopSummary {
                id: r.id,
                brand_name: r.brand_name,
                model_name: r.model_name,
                face_image_url: r.face_image_url,
                total_quantity_sold: r.total_quantity_sold.unwrap_or(0),
                total_revenue: r.total_revenue
                    .and_then(|bd| bd.to_f64())
                    .unwrap_or(0.0),
            }).collect();

            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            eprintln!("DB query error: {:?}", e);
            HttpResponse::InternalServerError().body("Error fetching dashboard data")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_dashboard_data);
}