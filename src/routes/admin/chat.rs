use actix_web::{get,post, web, HttpResponse, Responder};
use sqlx::{PgPool, Row};
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize)]
struct User {
    id: String,
    name: String,
}

#[derive(Serialize, FromRow)]
struct Message {
    id: i32,
    user_id: String,
    content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    sender: String,
    receiver: String,
}

#[derive(Serialize)]
struct ChatResponse {
    messages: Vec<Message>,
    bot_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub user_id: String,
    pub content: String,
    pub sender: String,
    pub receiver: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[get("/api/admin/users")]
async fn get_users(db: web::Data<PgPool>) -> impl Responder {
    let query = r#"
        SELECT 
            user_bot_settings.user_id AS id,
            logininfo.name 
        FROM user_bot_settings
        JOIN logininfo ON logininfo.id::TEXT = user_bot_settings.user_id
        ORDER BY logininfo.name
    "#;

    let users_result = sqlx::query(query)
        .map(|row: sqlx::postgres::PgRow| User {
            id: row.get("id"),
            name: row.get("name"),
        })
        .fetch_all(db.get_ref())
        .await;

    match users_result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Error fetching users: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch users")
        }
    }
}


#[get("/api/admin/chats/{user_id}")]
async fn get_messages(path: web::Path<String>, db: web::Data<PgPool>) -> impl Responder {
    let user_id = path.into_inner();

    let messages_res = sqlx::query_as::<_, Message>(
    "SELECT id, user_id, content, timestamp, sender, receiver FROM messages WHERE user_id = $1 ORDER BY timestamp ASC"
)
.bind(&user_id)
.fetch_all(db.get_ref())
.await;

    let bot_enabled_res = sqlx::query_scalar::<_, bool>(
    "SELECT bot_enabled FROM user_bot_settings WHERE user_id = $1"
)
.bind(&user_id)
    .fetch_optional(db.get_ref())
    .await;

    match (messages_res, bot_enabled_res) {
        (Ok(messages), Ok(bot_enabled_opt)) => {
            let bot_enabled = bot_enabled_opt.unwrap_or(false);

            let response = ChatResponse {
                messages,
                bot_enabled,
            };
            HttpResponse::Ok().json(response)
        }
        (Err(e), _) => {
            eprintln!("Error fetching messages: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch messages")
        }
        (_, Err(e)) => {
            eprintln!("Error fetching bot_enabled status: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch bot status")
        }
    }
}

#[derive(Deserialize)]
struct BotStatusPayload {
    bot_enabled: bool,
}

#[post("/api/admin/bot_status/{user_id}")]
async fn update_bot_status(
    path: web::Path<String>,
    db: web::Data<PgPool>,
    payload: web::Json<BotStatusPayload>,
) -> impl Responder {
    let user_id = path.into_inner();
    let bot_enabled = payload.bot_enabled;

    let result = sqlx::query(
    r#"
    INSERT INTO user_bot_settings (user_id, bot_enabled)
    VALUES ($1, $2)
    ON CONFLICT (user_id) DO UPDATE
    SET bot_enabled = EXCLUDED.bot_enabled
    "#
)
.bind(user_id)
.bind(bot_enabled)
.execute(db.get_ref())
.await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Bot status updated successfully"),
        Err(e) => {
            eprintln!("Error updating bot status: {}", e);
            HttpResponse::InternalServerError().body("Failed to update bot status")
        }
    }
}

#[post("/api/admin/send_message/{id}")]
pub async fn send_message(
    path: web::Path<String>,
    payload: web::Json<SendMessagePayload>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id_path = path.into_inner();

    if user_id_path != payload.user_id {
        return HttpResponse::BadRequest().body("User ID in path and body do not match");
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO messages (user_id, content, sender, receiver, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        payload.user_id,
        payload.content,
        payload.sender,
        payload.receiver,
        payload.timestamp
    )
    .execute(db_pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Message sent"),
        Err(e) => {
            eprintln!("DB insert error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to insert message")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users);
    cfg.service(get_messages);
    cfg.service(update_bot_status);
    cfg.service(send_message);
}