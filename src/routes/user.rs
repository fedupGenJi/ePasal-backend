use actix_web::{web, HttpResponse, Responder};
use sqlx::{PgPool, FromRow};
use serde::Serialize;

#[derive(Debug, Serialize, FromRow)]
pub struct UserInfo {
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub phonenumber: String,
    pub balance: f32,
}

pub async fn get_user_info(
    user_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    let result = sqlx::query_as::<_, UserInfo>(
        r#"
        SELECT 
            id AS user_id,
            name,
            email,
            phoneNumber,
            balance
        FROM logininfo
        WHERE id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(db_pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(e) => {
            eprintln!("❌ Failed to fetch user info: {:?}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/user/{user_id}", web::get().to(get_user_info));
}