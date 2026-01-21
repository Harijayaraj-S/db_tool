use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use sqlx::{Connection, Executor, PgConnection};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "db-ops")]
#[command(about = "A CLI tool for database orchestration", long_about = None)]
struct Cli {
    /// The Database connection URL.
    #[arg(short, long, env = "DATABASE_URL")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pings the database to ensure connection is valid
    Status,
    /// Recreates the database using the shell script
    Recreate,
    /// Seeds the database using the SQL file
    Seed {
        /// Path to the SQL file
        #[arg(short, long, default_value = "insert_data.sql")]
        file: PathBuf,
    },
    /// Runs Recreate and then Seed
    Reset {
        #[arg(short, long, default_value = "insert_data.sql")]
        file: PathBuf,
    },
}

async fn check_status(db_url: &str) -> Result<()> {
    println!("🔌 Pinging database...");
    let mut conn = PgConnection::connect(db_url)
        .await
        .context("Failed to establish connection to the database. Check your URL.")?;

    conn.execute("SELECT 1")
        .await
        .context("Failed to execute test query (SELECT 1)")?;

    println!("✅ Connection Successful! Database is ready.");
    Ok(())
}

async fn run_recreate_script(db_url: &str) -> Result<()> {
    println!("♻️  Recreating database via script...");

    // Check if script exists and is executable
    if !std::path::Path::new("./db_recreate.sh").exists() {
        anyhow::bail!("Error: ./db_recreate.sh not found in current directory.");
    }

    let status = Command::new("./db_recreate.sh")
        .arg(db_url)
        .status()
        .context("Failed to execute db_recreate.sh")?;

    if !status.success() {
        anyhow::bail!("db_recreate.sh exited with error code: {:?}", status.code());
    }

    println!("✅ Database recreated successfully.");
    Ok(())
}

async fn run_seed_sql(db_url: &str, file_path: &PathBuf) -> Result<()> {
    println!("🌱 Seeding database from {:?}...", file_path);

    let sql_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read SQL file: {:?}", file_path))?;

    let mut conn = PgConnection::connect(db_url)
        .await
        .context("Failed to connect to database for seeding")?;

    conn.execute(sql_content.as_str())
        .await
        .context("Failed to execute SQL seed query")?;

    println!("✅ Seeding completed.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load env but don't fail if missing (allows explicit flags)
    dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Status => {
            check_status(&cli.url).await?;
        }
        Commands::Recreate => {
            run_recreate_script(&cli.url).await?;
        }
        Commands::Seed { file } => {
            run_seed_sql(&cli.url, file).await?;
        }
        Commands::Reset { file } => {
            run_recreate_script(&cli.url).await?;
            run_seed_sql(&cli.url, file).await?;
        }
    }

    Ok(())
}
