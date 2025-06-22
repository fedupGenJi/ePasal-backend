use sqlx::PgPool;
use crate::databases::auth::temp_user::{SignupData, TempUser};
use uuid::Uuid;
use rand::Rng;

pub async fn user_exists(pool: &PgPool, email: &str, number: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
    "SELECT 1 FROM logininfo WHERE email = $1 OR phoneNumber = $2"
)
.bind(email)
.bind(number)
.fetch_optional(pool)
.await?;

    Ok(result.is_some())
}

pub async fn insert_temp_user(pool: &PgPool, data: SignupData) -> Result<TempUser, sqlx::Error> {
    let temp_id = format!("signup{}", Uuid::new_v4());
    let code = format!("{:05}", rand::thread_rng().gen_range(10000..99999));

    sqlx::query(
    "INSERT INTO temp_users (temp_id, name, number, gmail, password, code)
     VALUES ($1, $2, $3, $4, $5, $6)"
)
.bind(&temp_id)
.bind(&data.name)
.bind(&data.number)
.bind(&data.email)
.bind(&data.password)
.bind(&code)
.execute(pool)
.await?;
    Ok(TempUser {
        temp_id,
        name: data.name,
        number: data.number,
        email: data.email,
        password: data.password,
        code,
    })
}
