use std::process::Command;
use std::thread;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{App, HttpServer, Responder};
use serde::Serialize;
use reqwest::blocking::Client;

fn wait_for_react() {
    let client = Client::new();
    let url = "http://localhost:5173";

    for _ in 0..30 {
        if let Ok(resp) = client.get(url).send() {
            if resp.status().is_success() {
                println!("✅ React dev server is ready!");
                return;
            }
        }
        println!("⏳ Waiting for React dev server...");
        thread::sleep(Duration::from_secs(1));
    }
    eprintln!("⚠️ React dev server did not become ready in time.");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Spawn React server
    thread::spawn(|| {
        let react_path = "../react-frontend"; //set path accordingly
        let result = Command::new("cmd")
            .args(&["/C", "npm", "run", "dev"])
            .current_dir(react_path)
            .spawn();

        match result {
            Ok(_) => println!("🚀 React dev server started"),
            Err(e) => eprintln!("Failed to start React dev server: {}", e),
        }
    });

    // Wait for React server and then open browser
    thread::spawn(|| {
        wait_for_react();
        if let Err(e) = open::that("http://localhost:5173") {
            eprintln!("Failed to open browser: {}", e);
        }
    });

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
