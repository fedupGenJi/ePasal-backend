use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SignupData {
    name: String,
    number: String,
    email: String,
    password: String,
}

// POST /signup handler
pub async fn signup(data: web::Json<SignupData>) -> impl Responder {
    println!("Received signup data: {:?}", data);

    // Simulate success or failure conditionally
    if data.email.is_empty() || data.password.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "status": "error", "message": "Email and password required" }));
    }

    HttpResponse::Ok()
        .json(serde_json::json!({ "status": "success", "message": "Signup successful" }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/signup", web::post().to(signup));
}
