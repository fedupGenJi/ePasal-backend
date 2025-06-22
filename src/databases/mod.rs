use anyhow::{Context, Result};
use sqlx::{PgPool, Executor};
use std::{fs, path::Path};

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
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url).await.context("Failed to connect to database")?;

    let schema_dirs = [
        "databases/auth",
        "databases/temp",
        // Add other schema directories here as needed
    ];

    let combined_schema_sql = load_all_schemas(&schema_dirs)?;

    let required_tables = [
        "logininfo",
        "temp_users",
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
    Ok(pool)
}

async fn clear_temp_tables(pool: &PgPool) -> Result<()> {
    let temp_tables = ["temp_users"];

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