use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use serde::Deserialize;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use crate::databases::auth::logindb::{get_user_by_gmail};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub gmail: String,
    pub password: String,
}

pub async fn login(
    data: web::Json<LoginRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let LoginRequest { gmail, password } = data.into_inner();

    match get_user_by_gmail(&db_pool, &gmail).await {
        Ok(Some(user)) => {
            let parsed_hash = match PasswordHash::new(&user.hashed_password) {
                Ok(hash) => hash,
                Err(_) => return HttpResponse::InternalServerError().body("Password hash parsing failed"),
            };

            if Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok() {
                HttpResponse::Ok().json(json!({
                    "message": "Login successful",
                    "user_id": user.id
                }))
            } else {
                HttpResponse::Unauthorized().body("Password does not match")
            }
        }
        Ok(None) => HttpResponse::NotFound().body("No user found with that Gmail"),
        Err(e) => {
            eprintln!("❌ DB query error: {:?}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login));
}