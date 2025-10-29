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
        #[arg(long, default_value = "15001")]
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
        Commands::Cli => {
            // Run legacy CLI
            if let Err(e) = vectorizer::cli::run().await {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

async fn run_servers(project: PathBuf, daemon: bool, host: String, port: u16, mcp_port: u16) {
    // Validate project directory
    if !project.exists() || !project.is_dir() {
        eprintln!(
            "Error: Project directory '{}' does not exist",
            project.display()
        );
        std::process::exit(1);
    }

    println!("🚀 Starting Vectorizer Servers...");
    println!("==================================");
    println!("📁 Project Directory: {}", project.display());
    println!("🌐 REST API: {}:{}", host, port);
    println!("🔧 MCP Server: 127.0.0.1:{}", mcp_port);

    if daemon {
        println!("👻 Running as daemon...");
        run_as_daemon(project, host, port, mcp_port).await;
    } else {
        run_interactive(project, host, port, mcp_port).await;
    }
}

async fn run_interactive(project: PathBuf, host: String, port: u16, mcp_port: u16) {
    use tokio::signal;

    println!("Starting MCP server...");
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

    println!(
        "✅ MCP server started (PID: {})",
        mcp_child.id().unwrap_or(0)
    );

    // Wait a moment for MCP server to initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("Starting REST API server...");
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

    println!(
        "✅ REST API server started (PID: {})",
        rest_child.id().unwrap_or(0)
    );

    println!("\n🎉 Both servers are running!");
    println!("==================================");
    println!("📡 REST API: http://{}:{}", host, port);
    println!("🔧 MCP Server: http://127.0.0.1:{}/sse", mcp_port);
    println!("\n⚡ Press Ctrl+C to stop both servers\n");

    // Wait for shutdown signal
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    println!("\n🛑 Shutting down servers...");
    let _ = mcp_child.kill().await;
    let _ = rest_child.kill().await;
    println!("✅ Servers stopped.");
}

async fn run_as_daemon(project: PathBuf, _host: String, _port: u16, _mcp_port: u16) {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;

        println!("🐧 Setting up Linux daemon...");

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
                println!("✅ MCP daemon started (PID: {})", child.id());
                let _ = child.wait();
            }
            Err(e) => {
                eprintln!("❌ Failed to start daemon: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("🪟 Setting up Windows service...");
        // Windows service implementation would go here
        eprintln!("❌ Windows daemon mode not yet implemented");
        std::process::exit(1);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        eprintln!("❌ Daemon mode not supported on this platform");
        std::process::exit(1);
    }
}

async fn stop_servers() {
    println!("🛑 Stopping Vectorizer Servers...");

    let mcp_pids = find_processes("vectorizer-mcp-server");
    let rest_pids = find_processes("vectorizer-server");

    for pid in &mcp_pids {
        println!("Stopping MCP server (PID: {})", pid);
        let _ = Command::new("kill").arg(&pid.to_string()).status();
    }

    for pid in &rest_pids {
        println!("Stopping REST server (PID: {})", pid);
        let _ = Command::new("kill").arg(&pid.to_string()).status();
    }

    if mcp_pids.is_empty() && rest_pids.is_empty() {
        println!("ℹ️  No running servers found");
    } else {
        println!("✅ Servers stopped");
    }
}

async fn check_status() {
    println!("📊 Vectorizer Servers Status");
    println!("============================");

    let mcp_running = !find_processes("vectorizer-mcp-server").is_empty();
    let rest_running = !find_processes("vectorizer-server").is_empty();

    println!(
        "MCP Server: {}",
        if mcp_running {
            "✅ RUNNING"
        } else {
            "❌ NOT RUNNING"
        }
    );
    println!(
        "REST API Server: {}",
        if rest_running {
            "✅ RUNNING"
        } else {
            "❌ NOT RUNNING"
        }
    );

    if rest_running {
        // Try to check REST API health
        match reqwest::get("http://127.0.0.1:15001/health").await {
            Ok(resp) if resp.status().is_success() => println!("REST API Health: 🟢 OK"),
            _ => println!("REST API Health: 🟡 UNREACHABLE"),
        }
    }

    if mcp_running {
        // Try to check MCP server
        match reqwest::get("http://127.0.0.1:15002/sse").await {
            Ok(resp) if resp.status().is_success() => println!("MCP Server Health: 🟢 OK"),
            _ => println!("MCP Server Health: 🟡 UNREACHABLE"),
        }
    }
}

async fn install_service() {
    #[cfg(target_os = "linux")]
    {
        println!("🐧 Installing as Linux systemd service...");

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
                println!("✅ Service file created: {}", service_path);
                println!("📋 To enable: sudo systemctl enable vectorizer");
                println!("🚀 To start: sudo systemctl start vectorizer");
                println!("📊 To check status: sudo systemctl status vectorizer");
            }
            Err(e) => {
                eprintln!("❌ Failed to create service file: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("🪟 Installing as Windows service...");
        // Windows service installation would go here
        eprintln!("❌ Windows service installation not yet implemented");
        std::process::exit(1);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        eprintln!("❌ Service installation not supported on this platform");
        std::process::exit(1);
    }
}

async fn uninstall_service() {
    #[cfg(target_os = "linux")]
    {
        println!("🐧 Uninstalling Linux systemd service...");

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
                println!("✅ Service uninstalled successfully");
            }
            Err(e) => {
                eprintln!("❌ Failed to remove service file: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("🪟 Uninstalling Windows service...");
        // Windows service uninstallation would go here
        eprintln!("❌ Windows service uninstallation not yet implemented");
        std::process::exit(1);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        eprintln!("❌ Service uninstallation not supported on this platform");
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
