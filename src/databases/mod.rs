use anyhow::{anyhow, Context, Result};
use sqlx::{PgPool, Executor};
use std::{fs, path::Path};

async fn is_ollama_running() -> bool {
    let client = reqwest::Client::new();
    client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .map(|res| res.status().is_success())
        .unwrap_or(false)
}

fn load_all_schemas(schema_dirs: &[&str]) -> Result<String> {
    let mut combined_sql = String::new();

    for dir in schema_dirs {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir).join("schema.sql");
        let sql = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema file: {:?}", schema_path))?;
        combined_sql.push_str(&sql);
        combined_sql.push('\n');
    }

    Ok(combined_sql)
}

async fn check_tables_exist(pool: &PgPool, tables: &[&str]) -> Result<bool> {
    for &table in tables {
        // Query PostgreSQL system catalog to check if table exists
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_schema = 'public' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(pool)
        .await?;

        if !exists.0 {
            println!("Table '{}' does NOT exist.", table);
            return Ok(false);
        }
    }
    Ok(true)
}

pub async fn setup_backend() -> Result<PgPool> {
    if !is_ollama_running().await {
        println!("⚠️  Ollama is not running at http://127.0.0.1:11434");
        println!("Please start it with: `ollama serve` in another terminal.");
        return Err(anyhow!("Ollama server not running"));
    }

    println!("✅ Ollama is running!");

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url).await.context("Failed to connect to database")?;

    let schema_dirs = [
        "databases/auth",
        "databases/temp",
        "databases/products",
        "databases/conversation",
        "databases/khalti",
        // Add other schema directories here as needed
    ];

    let combined_schema_sql = load_all_schemas(&schema_dirs)?;

    let required_tables = [
        "logininfo",
        "temp_users",
        "laptop_details",
        "laptop_side_images",
        "messages",
        "user_bot_settings",
        "khalti_temp_payments",
        // Add other expected table names here
    ];

    let tables_exist = check_tables_exist(&pool, &required_tables).await?;

    if !tables_exist {
        println!("Some tables missing. Running schema SQL to create tables...");
        pool.execute(combined_schema_sql.as_str())
            .await
            .context("Failed to execute schema SQL")?;
        println!("Schema SQL executed successfully.");
    } else {
        println!("All required tables exist.");
    }

    clear_temp_tables(&pool).await?;
    ensure_admin_user(&pool).await?;
    Ok(pool)
}

async fn clear_temp_tables(pool: &PgPool) -> Result<()> {
    let temp_tables = ["temp_users", "khalti_temp_payments"];

    for &table in &temp_tables {
        let query = format!("DELETE FROM {}", table);
        pool.execute(query.as_str())
            .await
            .with_context(|| format!("Failed to clear temp table '{}'", table))?;
    }

    println!("Temporary tables cleared.");
    Ok(())
}

pub mod auth;

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncSmtpTransport;
use lettre::Tokio1Executor;
use lettre::AsyncTransport;
use rand::{distributions::Alphanumeric, Rng};

pub async fn ensure_admin_user(pool: &PgPool) -> Result<()> {
    let admin_email = std::env::var("ADMIN_EMAIL").context("ADMIN_EMAIL must be set in .env")?;
    let admin_phone = std::env::var("ADMIN_PHONE").context("ADMIN_PHONE must be set in .env")?;

    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS (
            SELECT 1 FROM logininfo WHERE email = $1
        )",
    )
    .bind(&admin_email)
    .fetch_one(pool)
    .await
    .context("Failed to query admin existence")?;

    if exists.0 {
        println!("Admin user already exists.");
        return Ok(());
    }

    let raw_password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
    .hash_password(raw_password.as_bytes(), &salt)
    .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

    sqlx::query(
        "INSERT INTO logininfo (name, email, phoneNumber, password, status)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind("Admin")
    .bind(&admin_email)
    .bind(&admin_phone)
    .bind(hashed_password.to_string())
    .bind("admin")
    .execute(pool)
    .await
    .context("Failed to insert admin user")?;

    println!("Admin user created.");
    println!("Generated password for admin: {}", raw_password);

    send_admin_password_email(&admin_email, &raw_password).await?;

    Ok(())
}

async fn send_admin_password_email(recipient: &str, password: &str) -> Result<()> {
    let smtp_email = std::env::var("SMTP_EMAIL").context("SMTP_EMAIL must be set")?;
    let smtp_password = std::env::var("SMTP_PASSWORD").context("SMTP_PASSWORD must be set")?;
    let smtp_server = std::env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .context("Invalid SMTP_PORT")?;

    let email = Message::builder()
        .from(Mailbox::new(None, smtp_email.parse()?))
        .to(Mailbox::new(None, recipient.parse()?))
        .subject("Your Admin Account Has Been Created")
        .body(format!(
            "Hello Admin,\n\nYour admin account has been created.\n\nLogin Email: {}\nPassword: {}",
            recipient, password
        ))?;

    let creds = Credentials::new(smtp_email, smtp_password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_server)?
        .port(smtp_port)
        .credentials(creds)
        .build();

    mailer.send(email).await.context("Failed to send admin email")?;

    println!("Admin credentials sent to email: {}", recipient);

    Ok(())
}