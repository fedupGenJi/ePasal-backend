use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::process::Command;
use std::thread;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use reqwest::blocking::Client;

fn get_local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().map(|addr| addr.ip()).ok()
}

fn wait_for_react(react_url: &str) {
    let client = Client::new();

    for _ in 0..30 {
        if let Ok(resp) = client.get(react_url).send() {
            if resp.status().is_success() {
                println!("✅ React dev server is ready at: {}", react_url);
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
    // Get local LAN IP
    let local_ip = get_local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let react_url = format!("http://{}:5173", local_ip);
    let backend_url = format!("http://{}:8080", local_ip);

    // Spawn React dev server
    thread::spawn(|| {
        let react_path = "../react-frontend"; // Set path accordingly
        let result = Command::new("cmd")
            .args(&["/C", "npm", "run", "dev"])
            .current_dir(react_path)
            .spawn();

        match result {
            Ok(_) => println!("🚀 React dev server started"),
            Err(e) => eprintln!("❌ Failed to start React dev server: {}", e),
        }
    });

    // Wait and open in browser
    let react_url_clone = react_url.clone();
    thread::spawn(move || {
        wait_for_react(&react_url_clone);
        if let Err(e) = open::that(&react_url_clone) {
            eprintln!("❌ Failed to open browser: {}", e);
        }
    });

    println!("🌐 Backend available at:  {}", backend_url);
    println!("🌐 React available at:    {}", react_url);

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}