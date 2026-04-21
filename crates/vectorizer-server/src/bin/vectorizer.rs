//! Vectorizer Server - Unified MCP + REST API
//!
//! This is the unified server that provides MCP + REST API access
//! for all vector operations.

#![allow(clippy::uninlined_format_args)]

use clap::Parser;
use tracing::{error, info, warn};
use vectorizer::config::VectorizerConfig;
use vectorizer_server::server::{RootUserConfig, VectorizerServer};

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Vectorizer Server - MCP + REST API")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Server host (overrides config.yml)
    #[arg(long)]
    host: Option<String>,

    /// Server port (overrides config.yml)
    #[arg(long)]
    port: Option<u16>,

    /// Enable verbose logging (default: only warnings and errors)
    #[arg(long)]
    verbose: bool,

    /// Path to config file
    #[arg(long, default_value = "config.yml")]
    config: String,

    /// Root user username for dashboard authentication (default: "root")
    /// If no admin users exist, this user will be created on startup
    #[arg(long, env = "VECTORIZER_ADMIN_USERNAME")]
    root_user: Option<String>,

    /// Root user password for dashboard authentication
    /// If not provided, a secure random password will be generated
    #[arg(long, env = "VECTORIZER_ADMIN_PASSWORD")]
    root_password: Option<String>,

    /// Auto-generate and persist a JWT secret on first boot when
    /// `auth.jwt_secret` is empty. The key is written under the auth data
    /// directory (mode 0o600 on POSIX) and reused on every restart.
    /// Off by default — production deployments should set the secret
    /// explicitly via `VECTORIZER_JWT_SECRET` or config.yml.
    #[arg(long, env = "VECTORIZER_AUTO_GEN_JWT_SECRET")]
    auto_generate_jwt_secret: bool,
}

/// Load configuration from config.yml, creating with defaults if not exists
fn load_config(config_path: &str) -> VectorizerConfig {
    let path = std::path::Path::new(config_path);

    // If config doesn't exist, create it with defaults
    if !path.exists() {
        info!(
            "📝 Config file {} not found, creating with default values...",
            config_path
        );

        let default_config = VectorizerConfig::default();

        // Try to serialize and write the default config
        match serde_yaml::to_string(&default_config) {
            Ok(yaml_content) => {
                // Add helpful header comment
                let content = format!(
                    "# Vectorizer Configuration File\n\
                     # Generated automatically with default values\n\
                     # See config.example.yml for full documentation\n\n\
                     {}",
                    yaml_content
                );

                match std::fs::write(config_path, &content) {
                    Ok(_) => {
                        info!("✅ Created default config file: {}", config_path);
                    }
                    Err(e) => {
                        warn!("⚠️  Could not create config file {}: {}", config_path, e);
                        warn!("   Using in-memory defaults. Check write permissions.");
                    }
                }
            }
            Err(e) => {
                warn!("⚠️  Could not serialize default config: {}", e);
            }
        }

        return default_config;
    }

    // Config exists, try to load it
    match std::fs::read_to_string(config_path) {
        Ok(content) => match serde_yaml::from_str::<VectorizerConfig>(&content) {
            Ok(config) => {
                info!("✅ Loaded configuration from {}", config_path);
                config
            }
            Err(e) => {
                warn!("⚠️  Failed to parse {}: {}", config_path, e);
                warn!("   Using defaults. Please fix the config file syntax.");
                VectorizerConfig::default()
            }
        },
        Err(e) => {
            warn!("⚠️  Cannot read config file {}: {}", config_path, e);
            VectorizerConfig::default()
        }
    }
}

/// Validate write permissions for data directory and config
fn validate_permissions(_config: &VectorizerConfig, config_path: &str) -> Result<(), String> {
    let mut errors = Vec::new();

    // 1. Check data directory — resolved by vectorizer_core::paths
    // so the binary writes to the OS-canonical user-data location
    // (XDG on Linux, Application Support on macOS, AppData on
    // Windows). Override with VECTORIZER_DATA_DIR.
    let data_dir = vectorizer_core::paths::data_dir();
    if !data_dir.exists() {
        // Try to create it
        match std::fs::create_dir_all(&data_dir) {
            Ok(_) => {
                info!("📁 Created data directory: {}", data_dir.display());
            }
            Err(e) => {
                errors.push(format!(
                    "Cannot create data directory {}: {}",
                    data_dir.display(),
                    e
                ));
            }
        }
    } else {
        // Check write permissions by creating a test file
        let test_file = data_dir.join(".write_test");
        match std::fs::write(&test_file, "test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
            }
            Err(e) => {
                errors.push(format!(
                    "No write permission in {}: {}",
                    data_dir.display(),
                    e
                ));
            }
        }
    }

    // 2. Check snapshots directory if enabled
    let snapshots_dir = data_dir.join("snapshots");
    if !snapshots_dir.exists() {
        match std::fs::create_dir_all(&snapshots_dir) {
            Ok(_) => {
                info!(
                    "📁 Created snapshots directory: {}",
                    snapshots_dir.display()
                );
            }
            Err(e) => {
                warn!("⚠️  Could not create snapshots directory: {}", e);
                // Not critical
            }
        }
    }

    // 3. Check config file is writable (for updates)
    let config_path = std::path::Path::new(config_path);
    if config_path.exists() {
        match std::fs::OpenOptions::new().write(true).open(config_path) {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "⚠️  Config file {} is not writable: {}",
                    config_path.display(),
                    e
                );
                // This is a warning, not an error - we can still run
            }
        }
    }

    // 4. Check workspace.yml parent directory
    let workspace_dir = std::path::Path::new(".");
    let test_workspace = workspace_dir.join(".workspace_write_test");
    match std::fs::write(&test_workspace, "test") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_workspace);
        }
        Err(e) => {
            errors.push(format!(
                "No write permission for workspace.yml in current directory: {}",
                e
            ));
        }
    }

    // 5. Check logs directory — same OS-canonical resolution as
    // `data_dir`, see vectorizer_core::paths.
    let logs_dir = vectorizer_core::paths::logs_dir();
    if !logs_dir.exists() {
        match std::fs::create_dir_all(&logs_dir) {
            Ok(_) => {
                info!("📁 Created logs directory: {}", logs_dir.display());
            }
            Err(e) => {
                warn!(
                    "⚠️  Cannot create logs directory {}: {} (logging to console only)",
                    logs_dir.display(),
                    e
                );
                // Not critical, just warn
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install panic handler to log panics before aborting
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        error!("❌ PANIC: {} at {}", message, location);

        // Log to file if possible
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(".logs/panic.log")
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] PANIC: {} at {}",
                chrono::Utc::now().to_rfc3339(),
                message,
                location
            );
        }
    }));

    let cli = Cli::parse();

    // Initialize logging with verbose flag (do this early for config loading messages)
    let log_level = if cli.verbose { "debug" } else { "warn" };
    let _ = vectorizer_server::logging::init_logging_with_level("vectorizer", log_level);

    // Load configuration from config.yml first
    let config = load_config(&cli.config);

    // Validate write permissions for data directories
    info!("🔍 Validating directory permissions...");
    match validate_permissions(&config, &cli.config) {
        Ok(_) => {
            info!("✅ All directory permissions validated");
        }
        Err(errors) => {
            error!("❌ PERMISSION ERRORS:");
            for line in errors.lines() {
                error!("   • {}", line);
            }
            error!("");
            error!("💡 To fix permission issues:");
            error!("   • Linux/macOS: sudo chown -R $(whoami) ./data ./.logs");
            error!("   • Windows: Run as Administrator or check folder permissions");
            error!("   • Docker: Ensure volume mounts have correct permissions");
            std::process::exit(1);
        }
    }

    // CLI arguments override config.yml values
    let host = cli.host.clone().unwrap_or(config.server.host.clone());
    let port = cli.port.unwrap_or(config.server.port);

    info!(
        "🚀 Starting Vectorizer Server v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("🌐 Server: {}:{}", host, port);
    info!("📁 Data directory: ./data");
    info!("📄 Config file: {}", cli.config);

    // Create root user configuration from CLI arguments (config path so server uses same file we loaded).
    let root_config = RootUserConfig {
        root_user: cli.root_user,
        root_password: cli.root_password,
        config_path: Some(cli.config.clone()),
        auto_generate_jwt_secret: cli.auto_generate_jwt_secret,
    };

    // Create and start the server with root user configuration
    let server = VectorizerServer::new_with_root_config(root_config).await?;

    // Start the server (this will block)
    if let Err(e) = server.start(&host, port).await {
        error!("❌ Server failed: {}", e);
        std::process::exit(1);
    }

    info!("✅ Server completed successfully");

    // Force exit to ensure process terminates
    // This prevents hanging if any background tasks are still running
    std::process::exit(0);
}
