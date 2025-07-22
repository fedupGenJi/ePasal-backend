// handlers/conversation.rs
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::databases::auth::messages::{Message, NewMessage};

#[get("/messages")]
pub async fn get_messages(
    db: web::Data<PgPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user_id = match query.get("userId") {
        Some(id) => id.clone(),
        None => return HttpResponse::BadRequest().body("Missing userId"),
    };

    let result = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE user_id = $1 ORDER BY timestamp ASC",
    )
    .bind(user_id)
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => {
            eprintln!("Error fetching messages: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/messages")]
pub async fn post_message(
    db: web::Data<PgPool>,
    body: web::Json<NewMessage>,
) -> impl Responder {
    let msg = body.into_inner();

    let result = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (user_id, content, timestamp, sender) 
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&msg.user_id)
    .bind(&msg.content)
    .bind(msg.timestamp)
    .bind(&msg.sender)
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(saved) => HttpResponse::Ok().json(saved),
        Err(e) => {
            eprintln!("Insert error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_messages);
    cfg.service(post_message);
}