use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::json;
use log::error;

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct TempUser {
    name: String,
    number: String,
    gmail: String,
    password: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct VerifyRequest {
    pub temp_id: String,
}

pub async fn verify(
    req: web::Json<VerifyRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let temp_id = &req.temp_id;

    let temp_user = match sqlx::query_as::<_, TempUser>(
    r#"
    SELECT name, number, gmail, password FROM temp_users WHERE temp_id = $1
    "#
)
.bind(temp_id)
.fetch_optional(db_pool.get_ref())
.await
{
    Ok(Some(user)) => user,
    Ok(None) => return HttpResponse::BadRequest().body("Invalid temp_id"),
    Err(e) => {
        error!("Error querying temp_users: {:?}", e);
        return HttpResponse::InternalServerError().body("DB query error");
    }
};

    let insert_res = sqlx::query(
    r#"
    INSERT INTO logininfo (name, phoneNumber, email, password, status)
    VALUES ($1, $2, $3, $4, 'user')
    "#
)
.bind(&temp_user.name)
.bind(&temp_user.number)
.bind(&temp_user.gmail)
.bind(&temp_user.password)
.execute(db_pool.get_ref())
.await;

    if let Err(e) = insert_res {
        error!("Error inserting into logininfo: {:?}", e);
        return HttpResponse::InternalServerError().body("DB insert error");
    }

    if let Err(e) = sqlx::query("DELETE FROM temp_users WHERE temp_id = $1")
    .bind(temp_id)
    .execute(db_pool.get_ref())
    .await
    {
        error!("Error deleting from temp_users: {:?}", e);
        return HttpResponse::InternalServerError().body("DB delete error");
    }

    HttpResponse::Ok().json(json!({
        "message": "User verified and registered successfully"
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/verify", web::post().to(verify));
}
