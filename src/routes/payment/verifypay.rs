use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct VerifyPaymentRequest {
    pidx: String,
    status: String,
}

#[post("/api/payment/verify")]
pub async fn verify_payment(
    data: web::Json<VerifyPaymentRequest>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let result = sqlx::query!(
    "SELECT email, laptop_id FROM khalti_temp_payments WHERE pidx = $1",
    data.pidx
)
.fetch_optional(db.get_ref())
.await;

    match result {
        Ok(Some(row)) => {
            let email = row.email;
            let laptop_id: i32 = row.laptop_id.parse::<i32>()
    .expect("Failed to parse laptop_id as i32");
            // email-sending logic
            if let Err(err) = send_payment_status_email(&email, &data.status).await {
                eprintln!("Failed to send email: {}", err);
            }
            //store database
            if data.status == "Completed" {
            if let Err(err) = record_laptop_sale(db.get_ref(), laptop_id).await {
                eprintln!("Failed to record laptop sale: {}", err);
            }
        }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Payment verified"
            }))
        }
        Ok(None) => HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Invalid or expired payment link"
        })),
        Err(err) => {
            eprintln!("DB error: {}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Internal server error"
            }))
        }
    }
}

use anyhow::{Result, Context};
use lettre::{Message, SmtpTransport, Transport, message::Mailbox};
use std::env;

pub async fn send_payment_status_email(email: &str, status: &str) -> Result<()> {

    let smtp_email = env::var("SMTP_EMAIL").context("Missing SMTP_EMAIL")?;
    let smtp_password = env::var("SMTP_PASSWORD").context("Missing SMTP_PASSWORD")?;
    let smtp_server = env::var("SMTP_SERVER").context("Missing SMTP_SERVER")?;

    let subject = "E-Pasal Payment Status";
    let body = match status {
        "Completed" => "Your order is set to depart soon.",
        "User canceled" => "So sorry we could not make a deal.",
        _ => "Undefined payment status.",
    };

    let email = Message::builder()
        .from(smtp_email.parse::<Mailbox>()?)
        .to(email.parse::<Mailbox>()?)
        .subject(subject)
        .body(String::from(body))?;

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        smtp_email,
        smtp_password,
    );

    let mailer = SmtpTransport::relay(&smtp_server)?
        .credentials(creds)
        .build();

    mailer.send(&email).context("Failed to send email")?;

    Ok(())
}

pub async fn record_laptop_sale(db: &PgPool, laptop_id: i32) -> Result<()> {
    let mut tx = db.begin().await?;

    let row = sqlx::query!(
        "SELECT show_price FROM laptop_details WHERE id = $1",
        laptop_id
    )
    .fetch_one(&mut *tx)
    .await?;

    let price = row.show_price;

    sqlx::query!(
        "INSERT INTO laptops_sold (laptop_id, price_at_sale) VALUES ($1, $2)",
        laptop_id,
        price
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE laptop_details SET quantity = quantity - 1 WHERE id = $1",
        laptop_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_payment);
}