use actix_web::{web, HttpResponse, Responder};
use crate::databases::auth::temp_user::SignupData;
use crate::databases::auth::tempdb::{insert_temp_user, user_exists};
use crate::services::email::send_code_email;
use sqlx::PgPool;
use serde_json::json;

pub async fn signup(
    data: web::Json<SignupData>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let user = data.into_inner();

    if user.email.is_empty() || user.password.is_empty() {
        return HttpResponse::BadRequest().body("Email and password required");
    }

    match user_exists(&db_pool, &user.email, &user.number).await {
    Ok(true) => return HttpResponse::Conflict().body("User already exists"),
    Err(e) => {
        eprintln!("❌ Error checking if user exists: {:?}", e);
        return HttpResponse::InternalServerError().body("DB query failed");
    }
    _ => {}
}

    match insert_temp_user(&db_pool, user).await {
        Ok(temp_user) => {
            if let Err(e) = send_code_email(&temp_user.email, &temp_user.code).await {
                return HttpResponse::InternalServerError().body(format!("Email failed: {}", e));
            }

            HttpResponse::Ok().json(json!({ "temp_id": temp_user.temp_id }))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB insert failed: {}", e)),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/signup", web::post().to(signup));
}