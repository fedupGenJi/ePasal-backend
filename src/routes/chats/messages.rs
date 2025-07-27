use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i32,
    pub user_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub receiver: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewMessage {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub sender: String,
}