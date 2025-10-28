//! Vectorizer Server - Unified MCP + REST API
//!
//! This is the unified server that provides MCP + REST API access
//! for all vector operations.

#![allow(clippy::uninlined_format_args)]

use clap::Parser;
use tracing::error;
use vectorizer::server::VectorizerServer;

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Vectorizer Server - MCP + REST API")]
struct Cli {
    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Server port
    #[arg(long, default_value = "15002")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging - log errors but don't fail
    if let Err(e) = vectorizer::logging::init_logging("vectorizer") {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    let cli = Cli::parse();

    println!("🚀 Starting Vectorizer Server");
    println!("🌐 Server: {}:{}", cli.host, cli.port);

    // Check for legacy data and offer migration BEFORE creating the server
    let data_dir = std::path::Path::new("./data");
    if data_dir.exists() {
        use vectorizer::storage::{StorageFormat, StorageMigrator, detect_format};

        let format = detect_format(data_dir);
        if format == StorageFormat::Legacy {
            println!("\n🔄 Legacy data format detected - migrating automatically to .vecdb format...");

            let migrator = StorageMigrator::new(data_dir, 6);
            match migrator.migrate() {
                Ok(result) => {
                    println!("✅ Migration completed successfully!");
                    println!("   Collections migrated: {}", result.collections_migrated);
                    println!("   Legacy files removed from data directory");
                    if let Some(backup) = result.backup_path {
                        println!("   Backup saved to: {}", backup.display());
                        println!(
                            "   You can safely delete the backup after verifying the migration"
                        );
                    }
                    println!();
                }
                Err(e) => {
                    eprintln!("❌ Migration failed: {}", e);
                    eprintln!("   The vectorizer will continue using the legacy format.");
                }
            }
        }
    }

    // Create and start the server
    let server = VectorizerServer::new().await?;

    // Start the server (this will block)
    if let Err(e) = server.start(&cli.host, cli.port).await {
        error!("❌ Server failed: {}", e);
        eprintln!("❌ Server failed: {}", e);
        std::process::exit(1);
    }

    println!("✅ Server completed successfully");

    Ok(())
}
