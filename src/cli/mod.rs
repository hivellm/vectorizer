//! Command-line interface for Vectorizer administration
//!
//! Provides CLI tools for managing the vector database, users, API keys, and system configuration

pub mod commands;
pub mod config;
pub mod setup;
pub mod utils;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
pub use commands::*;
use tracing::warn;
// Re-export CliConfig directly since it's defined in this module
pub use utils::*;

/// Vectorizer CLI - Administrative tools for the vector database
#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Administrative CLI for Vectorizer vector database")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.yml")]
    pub config: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Server management commands
    Server {
        #[command(subcommand)]
        action: ServerCommands,
    },
    /// User management commands
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
    /// API key management commands
    ApiKey {
        #[command(subcommand)]
        action: ApiKeyCommands,
    },
    /// Collection management commands
    Collection {
        #[command(subcommand)]
        action: CollectionCommands,
    },
    /// System status and monitoring
    Status {
        /// Show detailed status information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Database operations
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// Snapshot management commands
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },
    /// Storage management commands
    Storage {
        #[command(subcommand)]
        action: StorageCommands,
    },
}

/// Server management commands
#[derive(Subcommand)]
pub enum ServerCommands {
    /// Start the vector database server
    Start {
        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(long, default_value = "15002")]
        port: u16,
        /// Enable authentication
        #[arg(long)]
        auth: bool,
        /// Data directory path
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },
    /// Stop the server gracefully
    Stop {
        /// Server host
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Server port
        #[arg(long, default_value = "15002")]
        port: u16,
    },
    /// Restart the server
    Restart {
        /// Server host
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Server port
        #[arg(long, default_value = "15002")]
        port: u16,
    },
}

/// User management commands
#[derive(Subcommand)]
pub enum UserCommands {
    /// Create a new user
    Create {
        /// Username
        #[arg(short, long)]
        username: String,
        /// User roles (comma-separated)
        #[arg(short, long, default_value = "User")]
        roles: String,
        /// User description
        #[arg(long)]
        description: Option<String>,
    },
    /// List all users
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Delete a user
    Delete {
        /// Username to delete
        #[arg(short, long)]
        username: String,
    },
    /// Update user roles
    UpdateRoles {
        /// Username
        #[arg(short, long)]
        username: String,
        /// New roles (comma-separated)
        #[arg(short, long)]
        roles: String,
    },
}

/// API key management commands
#[derive(Subcommand)]
pub enum ApiKeyCommands {
    /// Create a new API key
    Create {
        /// User ID for the API key
        #[arg(short, long)]
        user_id: String,
        /// API key name/description
        #[arg(short, long)]
        name: String,
        /// Permissions (comma-separated)
        #[arg(short, long, default_value = "Read,Write")]
        permissions: String,
        /// Expiration time in hours (0 = never expires)
        #[arg(short, long, default_value = "0")]
        expires_in_hours: u64,
    },
    /// List API keys
    List {
        /// Filter by user ID
        #[arg(short, long)]
        user_id: Option<String>,
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Revoke an API key
    Revoke {
        /// API key ID to revoke
        #[arg(short, long)]
        key_id: String,
    },
    /// Test an API key
    Test {
        /// API key to test
        #[arg(short, long)]
        api_key: String,
    },
}

/// Collection management commands
#[derive(Subcommand)]
pub enum CollectionCommands {
    /// Create a new collection
    Create {
        /// Collection name
        #[arg(short, long)]
        name: String,
        /// Vector dimension
        #[arg(short, long)]
        dimension: usize,
        /// Distance metric (euclidean, cosine, dot_product)
        #[arg(short, long, default_value = "cosine")]
        metric: String,
    },
    /// List all collections
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Delete a collection
    Delete {
        /// Collection name
        #[arg(short, long)]
        name: String,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Get collection statistics
    Stats {
        /// Collection name
        #[arg(short, long)]
        name: String,
    },
}

/// Database operations
#[derive(Subcommand)]
pub enum DbCommands {
    /// Backup the database
    Backup {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        /// Include collections
        #[arg(long)]
        collections: bool,
    },
    /// Restore from backup
    Restore {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
        /// Overwrite existing data
        #[arg(long)]
        force: bool,
    },
    /// Optimize database
    Optimize {
        /// Rebuild indexes
        #[arg(long)]
        rebuild_indexes: bool,
        /// Clean up expired data
        #[arg(long)]
        cleanup: bool,
    },
}

/// Configuration management commands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Show sensitive values (be careful!)
        #[arg(long)]
        show_secrets: bool,
    },
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Generate default configuration
    Generate {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// Snapshot management commands
#[derive(Subcommand)]
pub enum SnapshotCommands {
    /// List all available snapshots
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Create a new snapshot
    Create {
        /// Optional snapshot description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Restore from a snapshot
    Restore {
        /// Snapshot ID to restore from
        #[arg(short, long)]
        id: String,
        /// Force restore without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Delete a snapshot
    Delete {
        /// Snapshot ID to delete
        #[arg(short, long)]
        id: String,
    },
    /// Clean up old snapshots
    Cleanup {
        /// Dry run (show what would be deleted)
        #[arg(long)]
        dry_run: bool,
    },
}

/// Storage management commands
#[derive(Subcommand)]
pub enum StorageCommands {
    /// Show storage information and statistics
    Info {
        /// Show detailed statistics
        #[arg(short, long)]
        detailed: bool,
    },
    /// Migrate from legacy format to .vecdb
    Migrate {
        /// Force migration even if already migrated
        #[arg(short, long)]
        force: bool,
        /// Compression level (1-22)
        #[arg(long, default_value = "3")]
        level: i32,
    },
    /// Verify storage integrity
    Verify {
        /// Fix issues if possible
        #[arg(long)]
        fix: bool,
    },
    /// Compact storage manually
    Compact {
        /// Force compaction
        #[arg(short, long)]
        force: bool,
    },
}

/// CLI configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CliConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Authentication configuration
    pub auth: crate::auth::AuthConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Storage configuration
    #[serde(default)]
    pub storage: crate::storage::StorageConfig,
}

/// Server configuration for CLI
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerConfig {
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Data directory
    pub data_dir: PathBuf,
    /// Enable authentication
    pub auth_enabled: bool,
}

/// Database configuration for CLI
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DatabaseConfig {
    /// Persistence path
    pub persistence_path: PathBuf,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Compression threshold
    pub compression_threshold: usize,
}

/// Logging configuration for CLI
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log to file
    pub log_to_file: bool,
    /// Log file path
    pub log_file: Option<PathBuf>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 15002,
                data_dir: PathBuf::from("./data"),
                auth_enabled: true,
            },
            auth: crate::auth::AuthConfig::default(),
            database: DatabaseConfig {
                persistence_path: PathBuf::from("./data"),
                compression_enabled: true,
                compression_threshold: 1024,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                log_to_file: false,
                log_file: None,
            },
            storage: crate::storage::StorageConfig::default(),
        }
    }
}

/// Main CLI entry point
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose)?;

    // Load configuration
    let config = load_config(&cli.config)?;

    // Execute command
    match cli.command {
        Commands::Server { action } => {
            handle_server_command(action, &config).await?;
        }
        Commands::User { action } => {
            handle_user_command(action, &config).await?;
        }
        Commands::ApiKey { action } => {
            handle_api_key_command(action, &config).await?;
        }
        Commands::Collection { action } => {
            handle_collection_command(action, &config).await?;
        }
        Commands::Status { detailed } => {
            handle_status_command(detailed, &config).await?;
        }
        Commands::Db { action } => {
            handle_db_command(action, &config).await?;
        }
        Commands::Config { action } => {
            handle_config_command(action, &config).await?;
        }
        Commands::Snapshot { action } => {
            commands::handle_snapshot_command(action, &config).await?;
        }
        Commands::Storage { action } => {
            commands::handle_storage_command(action, &config).await?;
        }
    }

    Ok(())
}

/// Initialize logging based on CLI options
fn init_logging(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let level = if verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(format!("vectorizer={}", level))
        .init();

    Ok(())
}

/// Load configuration from file
fn load_config(path: &PathBuf) -> Result<CliConfig, Box<dyn std::error::Error>> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        // Try to parse, but fall back to default if it fails
        match serde_yaml::from_str::<CliConfig>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                warn!("Failed to parse config file, using defaults: {}", e);
                Ok(CliConfig::default())
            }
        }
    } else {
        // Return default configuration
        Ok(CliConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = vec![
            "vectorizer",
            "server",
            "start",
            "--host",
            "0.0.0.0",
            "--port",
            "8080",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Server { action } => match action {
                ServerCommands::Start { host, port, .. } => {
                    assert_eq!(host, "0.0.0.0");
                    assert_eq!(port, 8080);
                }
                _ => panic!("Expected Start command"),
            },
            _ => panic!("Expected Server command"),
        }
    }

    #[test]
    fn test_cli_config_default() {
        let config = CliConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 15002);
        assert!(config.auth.enabled);
    }
}
