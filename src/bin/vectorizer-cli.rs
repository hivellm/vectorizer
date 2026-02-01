//! Vectorizer CLI - Unified command-line interface
//!
//! This binary provides a unified interface for running and managing Vectorizer servers,
//! including REST API, MCP server, and daemon/service management.

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_borrows_for_generic_args)]

use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::{Parser, Subcommand};
#[cfg(target_os = "linux")]
use libc::setsid;
use tokio::process::Command as TokioCommand;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Unified Vectorizer CLI for running servers and managing services")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start both REST API and MCP servers
    Start {
        /// Project directory to index
        #[arg(short, long, default_value = "../gov")]
        project: PathBuf,

        /// Run as daemon/service (background)
        #[arg(long)]
        daemon: bool,

        /// Host for REST API server
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port for REST API server
        #[arg(long, default_value = "15002")]
        port: u16,

        /// Port for MCP server
        #[arg(long, default_value = "15002")]
        mcp_port: u16,
    },
    /// Stop running servers
    Stop,
    /// Check status of servers
    Status,
    /// Install as system service (Linux) or Windows service
    Install,
    /// Uninstall system service
    Uninstall,
    /// Run legacy CLI commands
    Cli,
    /// Run interactive setup wizard to configure workspace
    Setup {
        /// Path to project directory to analyze
        #[arg(short, long, default_value = ".")]
        path: std::path::PathBuf,

        /// Open web-based setup wizard in browser instead of CLI
        #[arg(long)]
        wizard: bool,
    },
    /// Open API documentation in browser
    Docs {
        /// Open API sandbox instead of documentation
        #[arg(long)]
        sandbox: bool,
    },
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=info")
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Start {
            project,
            daemon,
            host,
            port,
            mcp_port,
        } => {
            run_servers(project, daemon, host, port, mcp_port).await;
        }
        Commands::Stop => {
            stop_servers().await;
        }
        Commands::Status => {
            check_status().await;
        }
        Commands::Install => {
            install_service().await;
        }
        Commands::Uninstall => {
            uninstall_service().await;
        }
        Commands::Setup { path, wizard } => {
            if wizard {
                if let Err(e) = vectorizer::cli::setup::run_wizard().await {
                    error!("Setup wizard failed: {e}");
                    std::process::exit(1);
                }
            } else if let Err(e) = vectorizer::cli::setup::run(path).await {
                error!("Setup failed: {e}");
                std::process::exit(1);
            }
        }
        Commands::Docs { sandbox } => {
            if let Err(e) = vectorizer::cli::setup::run_docs(sandbox).await {
                error!("Failed to open docs: {e}");
                std::process::exit(1);
            }
        }
        Commands::Cli => {
            // Run legacy CLI
            if let Err(e) = vectorizer::cli::run().await {
                error!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

async fn run_servers(project: PathBuf, daemon: bool, host: String, port: u16, mcp_port: u16) {
    // Validate project directory
    if !project.exists() || !project.is_dir() {
        error!(
            "Error: Project directory '{}' does not exist",
            project.display()
        );
        std::process::exit(1);
    }

    info!("🚀 Starting Vectorizer Servers...");
    info!("==================================");
    info!("📁 Project Directory: {}", project.display());
    info!("🌐 REST API: {}:{}", host, port);
    info!("🔧 MCP Server: 127.0.0.1:{}", mcp_port);

    if daemon {
        info!("👻 Running as daemon...");
        run_as_daemon(project, host, port, mcp_port).await;
    } else {
        run_interactive(project, host, port, mcp_port).await;
    }
}

async fn run_interactive(project: PathBuf, host: String, port: u16, mcp_port: u16) {
    use tokio::signal;

    info!("Starting MCP server...");
    let mut mcp_child = TokioCommand::new("cargo")
        .args(&[
            "run",
            "--bin",
            "vectorizer-mcp-server",
            "--",
            &project.to_string_lossy(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start MCP server");

    info!(
        "✅ MCP server started (PID: {})",
        mcp_child.id().unwrap_or(0)
    );

    // Wait a moment for MCP server to initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    info!("Starting REST API server...");
    let mut rest_child = TokioCommand::new("cargo")
        .args(&[
            "run",
            "--bin",
            "vectorizer-server",
            "--",
            "--host",
            &host,
            "--port",
            &port.to_string(),
            "--project",
            &project.to_string_lossy(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start REST API server");

    info!(
        "✅ REST API server started (PID: {})",
        rest_child.id().unwrap_or(0)
    );

    info!("\n🎉 Both servers are running!");
    info!("==================================");
    info!("📡 REST API: http://{}:{}", host, port);
    info!("🔧 MCP Server: http://127.0.0.1:{}/sse", mcp_port);
    info!("\n⚡ Press Ctrl+C to stop both servers\n");

    // Wait for shutdown signal
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    info!("\n🛑 Shutting down servers...");
    let _ = mcp_child.kill().await;
    let _ = rest_child.kill().await;
    info!("✅ Servers stopped.");
}

async fn run_as_daemon(
    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))] project: PathBuf,
    _host: String,
    _port: u16,
    _mcp_port: u16,
) {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;

        info!("🐧 Setting up Linux daemon...");

        // Daemonize the process
        let result = unsafe {
            Command::new("cargo")
                .args(&[
                    "run",
                    "--bin",
                    "vectorizer-mcp-server",
                    "--",
                    &project.to_string_lossy(),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .pre_exec(|| {
                    // Detach from controlling terminal
                    if setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                })
                .spawn()
        };

        match result {
            Ok(mut child) => {
                info!("✅ MCP daemon started (PID: {})", child.id());
                let _ = child.wait();
            }
            Err(e) => {
                error!("❌ Failed to start daemon: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        info!("🪟 Setting up Windows service...");
        // Windows service implementation would go here
        error!("❌ Windows daemon mode not yet implemented");
        std::process::exit(1);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        error!("❌ Daemon mode not supported on this platform");
        std::process::exit(1);
    }
}

async fn stop_servers() {
    info!("🛑 Stopping Vectorizer Servers...");

    let mcp_pids = find_processes("vectorizer-mcp-server");
    let rest_pids = find_processes("vectorizer-server");

    for pid in &mcp_pids {
        info!("Stopping MCP server (PID: {})", pid);
        let _ = Command::new("kill").arg(&pid.to_string()).status();
    }

    for pid in &rest_pids {
        info!("Stopping REST server (PID: {})", pid);
        let _ = Command::new("kill").arg(&pid.to_string()).status();
    }

    if mcp_pids.is_empty() && rest_pids.is_empty() {
        info!("ℹ️  No running servers found");
    } else {
        info!("✅ Servers stopped");
    }
}

async fn check_status() {
    info!("📊 Vectorizer Servers Status");
    info!("============================");

    let mcp_running = !find_processes("vectorizer-mcp-server").is_empty();
    let rest_running = !find_processes("vectorizer-server").is_empty();

    info!(
        "MCP Server: {}",
        if mcp_running {
            "✅ RUNNING"
        } else {
            "❌ NOT RUNNING"
        }
    );
    info!(
        "REST API Server: {}",
        if rest_running {
            "✅ RUNNING"
        } else {
            "❌ NOT RUNNING"
        }
    );

    if rest_running {
        // Try to check REST API health
        match reqwest::get("http://127.0.0.1:15002/health").await {
            Ok(resp) if resp.status().is_success() => info!("REST API Health: 🟢 OK"),
            _ => warn!("REST API Health: 🟡 UNREACHABLE"),
        }
    }

    if mcp_running {
        // Try to check MCP server
        match reqwest::get("http://127.0.0.1:15002/sse").await {
            Ok(resp) if resp.status().is_success() => info!("MCP Server Health: 🟢 OK"),
            _ => warn!("MCP Server Health: 🟡 UNREACHABLE"),
        }
    }
}

async fn install_service() {
    #[cfg(target_os = "linux")]
    {
        info!("🐧 Installing as Linux systemd service...");

        let username = whoami::username();
        let exe_path = std::env::current_exe().unwrap().display().to_string();
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/var/lib/vectorizer"))
            .display()
            .to_string();

        let service_content = format!(
            r"[Unit]
Description=Vectorizer Server
After=network.target

[Service]
Type=simple
User={username}
WorkingDirectory={current_dir}
ExecStart={exe_path} start --daemon
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"
        );

        let service_path = "/etc/systemd/system/vectorizer.service";
        match std::fs::write(service_path, service_content) {
            Ok(_) => {
                info!("✅ Service file created: {}", service_path);
                info!("📋 To enable: sudo systemctl enable vectorizer");
                info!("🚀 To start: sudo systemctl start vectorizer");
                info!("📊 To check status: sudo systemctl status vectorizer");
            }
            Err(e) => {
                error!("❌ Failed to create service file: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        info!("🪟 Installing as Windows service...");
        // Windows service installation would go here
        error!("❌ Windows service installation not yet implemented");
        std::process::exit(1);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        error!("❌ Service installation not supported on this platform");
        std::process::exit(1);
    }
}

async fn uninstall_service() {
    #[cfg(target_os = "linux")]
    {
        info!("🐧 Uninstalling Linux systemd service...");

        let service_path = "/etc/systemd/system/vectorizer.service";

        // Stop and disable service first
        let _ = Command::new("sudo")
            .args(&["systemctl", "stop", "vectorizer"])
            .status();
        let _ = Command::new("sudo")
            .args(&["systemctl", "disable", "vectorizer"])
            .status();

        match std::fs::remove_file(service_path) {
            Ok(_) => {
                info!("✅ Service uninstalled successfully");
            }
            Err(e) => {
                error!("❌ Failed to remove service file: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        info!("🪟 Uninstalling Windows service...");
        // Windows service uninstallation would go here
        error!("❌ Windows service uninstallation not yet implemented");
        std::process::exit(1);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        error!("❌ Service uninstallation not supported on this platform");
        std::process::exit(1);
    }
}

fn find_processes(name: &str) -> Vec<u32> {
    let output = Command::new("pgrep")
        .args(["-f", name])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        });

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect()
    } else {
        vec![]
    }
}
