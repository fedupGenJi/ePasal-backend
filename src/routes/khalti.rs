use actix_web::{post, web, HttpResponse, Responder, get, HttpRequest};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct InitiatePaymentRequest {
    product_id: String,
    product_name: String,
    price: u32,
    customer_info: CustomerInfo,
}

#[derive(Deserialize, Serialize)]
pub struct CustomerInfo {
    name: String,
    email: String,
    phone: String,
}

#[derive(Serialize)]
struct KhaltiPayload<'a> {
    return_url: &'a str,
    website_url: &'a str,
    amount: u32,
    purchase_order_id: &'a str,
    purchase_order_name: &'a str,
    customer_info: &'a CustomerInfo,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pidx: String,
}

#[post("/api/payment/khalti/initiate")]
pub async fn initiate_khalti_payment(
    payload: web::Json<InitiatePaymentRequest>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let client = Client::new();

    let khalti_secret_key = env::var("KHALTI_SECRET_KEY").unwrap_or_default();
    let backend_url = std::env::var("BACKEND_URL").expect("BACKEND_URL must be set");
let return_url = format!("{}/api/payment/khalti/verify", backend_url);

    let base_url = std::env::var("BASE_URL").expect("BASE_URL must be set");
let website_url = format!("{}/payment/status", base_url);


    let khalti_payload = KhaltiPayload {
        return_url: &website_url,
        website_url: &return_url,
        amount: payload.price * 100,
        purchase_order_id: &payload.product_id,
        purchase_order_name: &payload.product_name,
        customer_info: &payload.customer_info,
    };

    let res = client
        .post("https://a.khalti.com/api/v2/epayment/initiate/")
        .header("Authorization", format!("Key {}", khalti_secret_key))
        .json(&khalti_payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            println!("Khalti Response ({}): {}", status, text);

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(pidx) = json.get("pidx").and_then(|v| v.as_str()) {
                    let email = &payload.customer_info.email;
                    let laptop_id = &payload.product_id;

                    if let Err(e) = sqlx::query!(
                        "INSERT INTO khalti_temp_payments (pidx, email, laptop_id) VALUES ($1, $2, $3)",
                        pidx,
                        email,
                        laptop_id
                    )
                    .execute(db.get_ref())
                    .await
                    {
                        eprintln!("Failed to store pidx/email/laptop_id: {:?}", e);
                    }
                }
            }

            HttpResponse::Ok()
                .content_type("application/json")
                .body(text)
        }
        Err(err) => {
            eprintln!("Error calling Khalti API: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to initiate payment")
        }
    }
}

#[get("/api/payment/khalti/verify")]
pub async fn verify_khalti_payment(query: web::Query<VerifyQuery>) -> impl Responder {
    let client = Client::new();
    let khalti_secret_key = env::var("KHALTI_SECRET_KEY").unwrap_or_default();

    let res = client
        .post("https://a.khalti.com/api/v2/epayment/lookup/")
        .header("Authorization", format!("Key {}", khalti_secret_key))
        .json(&serde_json::json!({ "pidx": query.pidx }))
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            println!("Khalti Verification Response ({}): {}", status, text);

//this part could be used in future enhancement of the payment gateway (lookup)

            HttpResponse::Ok()
                .content_type("application/json")
                .body(text)
        }
        Err(err) => {
            eprintln!("Error verifying payment: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to verify payment")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(initiate_khalti_payment);
    cfg.service(verify_khalti_payment);
}