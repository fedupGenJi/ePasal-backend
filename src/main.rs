mod routes;

mod databases;
mod services {
    pub mod email;
}

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use local_ip_address::local_ip;
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = "0.0.0.0";
    let port = 8080;

    let pool: PgPool = match databases::setup_backend().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Backend setup failed: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Server running on:");
    println!("  -> http://localhost:{}", port);
    if let Ok(local_ip) = local_ip() {
        println!("  -> http://{}:{}", local_ip, port);
    }

    let local_ip = local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
    let frontend_origin = format!("http://{}:5173", local_ip);
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin(&frontend_origin)
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
            )
            .configure(routes::auth::init)
            .configure(routes::verify::init)
    })
    .bind((host, port))?
    .run()
    .await
}