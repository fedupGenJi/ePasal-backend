use sqlx::{FromRow, PgPool};
use serde::Deserialize;

#[derive(Debug, Deserialize, FromRow)]
pub struct LoginUser {
    pub id: i32,
    //pub email: String,
    pub hashed_password: String,
    pub status: String,
}

pub async fn get_user_by_gmail(pool: &PgPool, gmail: &str) -> Result<Option<LoginUser>, sqlx::Error> {
    let result = sqlx::query_as::<_, LoginUser>(
        r#"
        SELECT id, password AS hashed_password, status
        FROM logininfo
        WHERE email = $1
        "#
    )
    .bind(gmail)
    .fetch_optional(pool)
    .await;

    result
}