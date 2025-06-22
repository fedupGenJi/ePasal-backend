use actix_web::{web, HttpResponse, Responder};
use crate::databases::auth::temp_user::SignupData;
use crate::databases::auth::tempdb::{insert_temp_user, user_exists, check_conflict_field};
use crate::services::email::send_code_email;
use sqlx::PgPool;
use serde_json::json;
use rand_core::OsRng;
use argon2::Argon2;
use argon2::password_hash::{SaltString, PasswordHasher};

pub async fn signup(
    data: web::Json<SignupData>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let mut user = data.into_inner();

    if user.email.is_empty() || user.password.is_empty() {
        return HttpResponse::BadRequest().body("Email and password required");
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = match argon2.hash_password(user.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Hashing error: {}", e)),
    };

    user.password = hashed_password;

match user_exists(&db_pool, &user.email, &user.number).await {
    Ok(true) => {
        match check_conflict_field(&db_pool, &user.email, &user.number).await {
            Ok(field) => {
                let message = match field.as_str() {
                    "email" => "Email already exists",
                    "phoneNumber" => "Phone number already exists",
                    _ => "User already exists",
                };
                return HttpResponse::Conflict().body(message);
            }
            Err(e) => {
                eprintln!("❌ Error checking conflict field: {:?}", e);
                return HttpResponse::InternalServerError().body("DB query failed");
            }
        }
    }
    Ok(false) => {
         match insert_temp_user(&db_pool, user).await {
        Ok(temp_user) => {
            if let Err(e) = send_code_email(&temp_user.email, &temp_user.code).await {
                return HttpResponse::InternalServerError().body(format!("Email failed: {}", e));
            }

            HttpResponse::Ok().json(json!({
                "temp_id": temp_user.temp_id,
                "otp": temp_user.code 
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB insert failed: {}", e)),
    }
    }
    Err(e) => {
        eprintln!("❌ Error checking if user exists: {:?}", e);
        return HttpResponse::InternalServerError().body("DB query failed");
    }
}
   
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/signup", web::post().to(signup));
}