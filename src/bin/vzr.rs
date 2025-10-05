//! Vectorizer CLI - Unified command-line interface
//!
//! This binary provides a unified interface for running and managing Vectorizer servers,
//! including REST API, MCP server, and daemon/service management.

use chrono::Utc;
use clap::{Parser, Subcommand};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::json;
use std::fs::OpenOptions;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use tar::{Archive, Builder as TarBuilder};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use std::collections::HashMap;
use tokio::sync::Mutex;
// (fs already imported above)
use vectorizer::logging;

// Memory analysis available via /heap-analysis endpoint
use vectorizer::workspace::WorkspaceManager;
use vectorizer::{
    db::VectorStore,
    embedding::EmbeddingManager,
    grpc::server::start_grpc_server,
    grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient,
    config::{GrpcConfig, FileWatcherYamlConfig, VectorizerConfig},
    file_watcher::FileWatcherSystem,
    config::VectorizerConfig as FullVectorizerConfig,
    models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig},
};

/// Find executable path for a given binary name
fn find_executable(binary_name: &str) -> Option<PathBuf> {
    // Check current directory first
    let current_dir = std::env::current_dir().ok()?;
    let current_path = current_dir.join(binary_name);
    if current_path.exists() {
        return Some(current_path);
    }

    // Check with .exe extension on Windows
    #[cfg(target_os = "windows")]
    {
        let current_path_exe = current_dir.join(format!("{}.exe", binary_name));
        if current_path_exe.exists() {
            return Some(current_path_exe);
        }
    }

    // Check target/release directory
    let target_release = current_dir.join("target").join("release").join(binary_name);
    if target_release.exists() {
        return Some(target_release);
    }

    // Check with .exe extension in target/release on Windows
    #[cfg(target_os = "windows")]
    {
        let target_release_exe = current_dir.join("target").join("release").join(format!("{}.exe", binary_name));
        if target_release_exe.exists() {
            return Some(target_release_exe);
        }
    }

    None
}

/// Load CUDA configuration from config.yml
fn load_cuda_config() -> vectorizer::cuda::CudaConfig {
    use serde_yaml;

    // Try to load full config and extract CUDA section
    match std::fs::read_to_string("config.yml") {
        Ok(content) => {
            match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                Ok(yaml) => {
                    if let Some(cuda_section) = yaml.get("cuda") {
                        match serde_yaml::from_value::<vectorizer::cuda::CudaConfig>(cuda_section.clone()) {
                            Ok(mut config) => {
                                println!("✅ Loaded CUDA config from config.yml:");
                                println!("   - enabled: {}", config.enabled);
                                println!("   - device_id: {}", config.device_id);
                                println!("   - memory_limit_mb: {}", config.memory_limit_mb);
                                println!("   - max_threads_per_block: {}", config.max_threads_per_block);

                                // Override with defaults if not specified
                                if config.memory_limit_mb == 0 {
                                    config.memory_limit_mb = 4096; // 4GB default
                                }

                                config
                            }
                            Err(e) => {
                                println!("⚠️ Failed to parse CUDA config section: {}. Using CPU-only mode.", e);
                                let mut config = vectorizer::cuda::CudaConfig::default();
                                config.enabled = false;
                                config
                            }
                        }
                    } else {
                        println!("ℹ️ No CUDA section in config.yml, using CPU-only mode");
                        let mut config = vectorizer::cuda::CudaConfig::default();
                        config.enabled = false;
                        config
                    }
                }
                Err(e) => {
                    println!("⚠️ Failed to parse config.yml as YAML: {}. Using default CUDA config", e);
                    vectorizer::cuda::CudaConfig::default()
                }
            }
        }
        Err(_) => {
            println!("ℹ️ No config.yml found, using default CUDA config");
            vectorizer::cuda::CudaConfig::default()
        }
    }
}

/// Structured logging for workspace operations
struct WorkspaceLogger {
    log_file: PathBuf,
}

impl WorkspaceLogger {
    fn new() -> Self {
        let log_file = PathBuf::from("vectorizer-workspace.log");
        Self { log_file }
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_entry = format!("[{}] {}: {}\n", timestamp, level, message);

        // Write to log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        // Also write to stderr for ERROR and WARN
        if level == "ERROR" || level == "WARN" {
            eprintln!("{}", log_entry.trim());
        }
    }

    fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    fn warn(&self, message: &str) {
        self.log("WARN", message);
    }

    fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}

/// Check if any vectorizer processes are already running
fn check_existing_processes(logger: &WorkspaceLogger) -> bool {
    logger.info("Checking for existing vectorizer processes...");

    // Check for processes by name first
    if check_processes_by_name(logger) {
        return true;
    }

    // Then check for processes using our ports
    if check_processes_by_ports(logger) {
        return true;
    }

    logger.info("No existing vectorizer processes found");
    false
}

/// Check for processes by executable name
fn check_processes_by_name(logger: &WorkspaceLogger) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use tasklist command
        let output = Command::new("tasklist")
            .args(&["/FI", "IMAGENAME eq vectorizer.exe", "/FI", "STATUS eq RUNNING"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // tasklist returns header lines, so we check if there are actual process entries
                let lines: Vec<&str> = stdout.lines().collect();
                if lines.len() > 3 { // Header + separator + at least one process
                    logger.warn("Found existing vectorizer.exe processes");
                    return true;
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems: Use ps command
        let output = Command::new("ps")
            .args(&["aux"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("vectorizer") && !line.contains("grep") {
                        logger.warn("Found existing vectorizer processes");
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check for processes using our specific ports
fn check_processes_by_ports(logger: &WorkspaceLogger) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use netstat to find processes using our ports
        let output = Command::new("netstat")
            .args(&["-ano"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains(":15001") || line.contains(":15002") {
                        logger.warn("Found processes listening on vectorizer ports (15001/15002)");
                        return true;
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems: Use lsof
        let output = Command::new("lsof")
            .args(&["-i", ":15001", "-i", ":15002"])
            .output();

        if let Ok(output) = output {
            if output.status.success() && !output.stdout.is_empty() {
                logger.warn("Found processes listening on vectorizer ports (15001/15002)");
                return true;
            }
        }
    }

    false
}

/// Kill existing vectorizer processes
fn kill_existing_processes(logger: &WorkspaceLogger) -> Result<(), Box<dyn std::error::Error>> {
    logger.info("Killing existing vectorizer processes...");

    // First, try to kill processes by name
    kill_processes_by_name(logger)?;

    // Then kill processes using our ports
    kill_processes_by_ports(logger)?;

    // Wait a moment for processes to terminate
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Verify that processes are actually terminated
    if check_existing_processes(logger) {
        logger.warn("Some processes may still be running after kill attempt");
        return Err("Failed to completely terminate existing processes".into());
    }

    logger.info("Existing processes terminated successfully");
    Ok(())
}

/// Kill processes by executable name
fn kill_processes_by_name(logger: &WorkspaceLogger) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use taskkill command
        let output = Command::new("taskkill")
            .args(&["/F", "/IM", "vectorizer.exe"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                logger.info("Killed vectorizer.exe processes");
            } else {
                // Check if it's because no processes were found
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("not found") {
                    logger.warn(&format!("taskkill warning: {}", stderr));
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems: Use pkill
        let output = Command::new("pkill")
            .args(&["-f", "vectorizer"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                logger.info("Killed vectorizer processes using pkill");
            } else {
                // pkill returns non-zero if no processes found, which is fine
                logger.info("No vectorizer processes found to kill");
            }
        }

        // Fallback: use killall if pkill is not available
        let killall_output = Command::new("killall")
            .args(&["-9", "vectorizer"])
            .output();

        if let Ok(killall_output) = killall_output {
            if killall_output.status.success() {
                logger.info("Killed vectorizer processes using killall");
            }
        }
    }

    Ok(())
}

/// Kill processes using our specific ports
fn kill_processes_by_ports(logger: &WorkspaceLogger) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Find PIDs using netstat and kill with taskkill
        let netstat_output = Command::new("netstat")
            .args(&["-ano"])
            .output();

        if let Ok(netstat_output) = netstat_output {
            if netstat_output.status.success() {
                let stdout = String::from_utf8_lossy(&netstat_output.stdout);
                let mut pids_to_kill = std::collections::HashSet::new();

                for line in stdout.lines() {
                    if line.contains(":15001") || line.contains(":15002") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 5 {
                            if let Ok(pid) = parts[4].parse::<u32>() {
                                pids_to_kill.insert(pid);
                            }
                        }
                    }
                }

                for pid in pids_to_kill {
                    logger.info(&format!("Killing process {} using vectorizer ports", pid));
                    let _ = Command::new("taskkill")
                        .args(&["/F", "/PID", &pid.to_string()])
                        .output();
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems: Use lsof to find PIDs and kill them
        let lsof_output = Command::new("lsof")
            .args(&["-ti", ":15001", ":15002"])
            .output();

        if let Ok(lsof_output) = lsof_output {
            if lsof_output.status.success() && !lsof_output.stdout.is_empty() {
                let stdout_str = String::from_utf8_lossy(&lsof_output.stdout);
                let pids: Vec<String> = stdout_str
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| line.trim().to_string())
                    .collect();

                for pid in pids {
                    logger.info(&format!("Killing process {} using vectorizer ports", pid));
                    let _ = Command::new("kill").args(&["-9", &pid]).output();
                }
            }
        }
    }

    Ok(())
}

/// PID file management for process tracking
struct PidFile {
    path: PathBuf,
}

impl PidFile {
    fn new(server_type: &str) -> Self {
        let pid_file = format!("vectorizer-{}-{}.pid", server_type, std::process::id());
        Self {
            path: PathBuf::from(pid_file),
        }
    }

    fn create(&self, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        let content = format!("{}\n", pid);
        fs::write(&self.path, content)?;
        Ok(())
    }

    fn read_pid(&self) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.path)?;
        let pid_str = content.trim();
        match pid_str.parse::<u32>() {
            Ok(pid) => Ok(Some(pid)),
            Err(_) => Ok(None),
        }
    }

    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Check if a process is actually running by PID
fn is_process_running(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Check if the PID appears in the output (excluding header)
                let lines: Vec<&str> = stdout.lines().collect();
                lines.len() > 1 // More than just the header line
            } else {
                false
            }
        } else {
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("ps")
            .args(&["-p", &pid.to_string()])
            .output();

        if let Ok(output) = output {
            output.status.success()
        } else {
            false
        }
    }
}

/// Enhanced process checking with PID file support
fn check_existing_processes_enhanced(logger: &WorkspaceLogger) -> bool {
    logger.info("Enhanced check for existing vectorizer processes...");

    // Check PID files first
    let mcp_pid_file = PidFile::new("mcp-server");
    let rest_pid_file = PidFile::new("rest-server");

    // Check MCP server PID file
    if let Ok(Some(pid)) = mcp_pid_file.read_pid() {
        if is_process_running(pid) {
            logger.warn(&format!("Found running MCP server with PID {}", pid));
            return true;
        } else {
            logger.info("Found stale MCP server PID file, cleaning up...");
            let _ = mcp_pid_file.cleanup();
        }
    }

    // Check REST server PID file
    if let Ok(Some(pid)) = rest_pid_file.read_pid() {
        if is_process_running(pid) {
            logger.warn(&format!("Found running REST server with PID {}", pid));
            return true;
        } else {
            logger.info("Found stale REST server PID file, cleaning up...");
            let _ = rest_pid_file.cleanup();
        }
    }

    // Fall back to the original check methods
    check_existing_processes(logger)
}

#[cfg(target_os = "linux")]
use libc::setsid;

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Unified Vectorizer CLI for running servers and managing services")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Start both REST API and MCP servers
    Start {
        /// Project directory to index (legacy, use workspace instead)
        #[arg(short, long)]
        project: Option<PathBuf>,

        /// Workspace configuration file
        #[arg(short, long)]
        workspace: Option<PathBuf>,

        /// Configuration file
        #[arg(short, long, default_value = "config.yml")]
        config: PathBuf,

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

        /// GPU backend to use (auto, metal, vulkan, dx12, cuda, cpu)
        #[arg(long, default_value = "auto")]
        gpu_backend: String,
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
    /// Workspace management commands
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    /// Create a compressed backup from the data directory
    Backup {
        /// Data directory to archive (default: data)
        #[arg(long, default_value = "data")]
        data_dir: PathBuf,
        /// Output archive path (default: backups/vectorizer_data_<timestamp>.tar.gz)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Restore a compressed backup archive into the data directory
    Restore {
        /// Archive file to restore (tar.gz)
        #[arg(long)]
        archive: PathBuf,
        /// Destination data directory (default: data)
        #[arg(long, default_value = "data")]
        data_dir: PathBuf,
        /// If set, clears destination before restoring
        #[arg(long, default_value_t = false)]
        clean: bool,
    },
}

#[derive(Subcommand, Clone)]
enum WorkspaceCommands {
    /// Initialize a new workspace
    Init {
        /// Workspace directory
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,

        /// Workspace name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Validate workspace configuration
    Validate {
        /// Workspace configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Show workspace status
    Status {
        /// Workspace configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// List projects in workspace
    List {
        /// Workspace configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}


#[tokio::main]
async fn main() {
    eprintln!("🚀 VZR MAIN: Started");


    // Initialize centralized logging
    if let Err(e) = logging::init_logging("vzr") {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    let args = Args::parse();

    // Skip process checks for non-server commands (backup/restore)
    let skip_process_check = matches!(&args.command, Commands::Backup { .. } | Commands::Restore { .. });

    // Check for duplicate processes before starting for server-related commands
    let logger = WorkspaceLogger::new();
    if !skip_process_check {
        if check_existing_processes_enhanced(&logger) {
            logger.warn("Found existing vectorizer processes. Killing them to prevent conflicts...");
            if let Err(e) = kill_existing_processes(&logger) {
                logger.error(&format!("Failed to kill existing processes: {}", e));
                eprintln!("❌ Failed to kill existing vectorizer processes: {}", e);
                eprintln!("💡 Please manually kill existing processes with: pkill -f vectorizer");
                std::process::exit(1);
            }
        }
    }

    match args.command {
        Commands::Start {
            project,
            workspace,
            config,
            daemon,
            host,
            port,
            mcp_port,
            gpu_backend,
        } => {
            run_servers(project, workspace, config, daemon, host, port, mcp_port, gpu_backend).await;
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
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Workspace { command } => {
            handle_workspace_command(command).await;
        }
        Commands::Backup { data_dir, output } => {
            if let Err(e) = run_backup_command(&data_dir, output.as_ref()) {
                eprintln!("❌ Backup failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Restore { archive, data_dir, clean } => {
            if let Err(e) = run_restore_command(&archive, &data_dir, clean) {
                eprintln!("❌ Restore failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}

async fn run_servers(
    project: Option<PathBuf>,
    workspace: Option<PathBuf>,
    config: PathBuf,
    daemon: bool,
    host: String,
    port: u16,
    mcp_port: u16,
    gpu_backend: String,
) {
    let logger = WorkspaceLogger::new();

    logger.info("Starting Vectorizer Servers");
    logger.info(&format!("Config: {}", config.display()));
    logger.info(&format!("Workspace mode: {}", workspace.is_some()));
    logger.info(&format!("GPU Backend: {}", gpu_backend));

    println!("🚀 Starting Vectorizer Servers...");
    println!("🎨 GPU Backend: {}", gpu_backend);

    // Determine if using workspace or legacy project mode
    if let Some(workspace_path) = workspace {
        logger.info(&format!(
            "Loading workspace from: {}",
            workspace_path.display()
        ));

        // Load and validate workspace
        let workspace_manager = match WorkspaceManager::load_from_file(&workspace_path) {
            Ok(manager) => {
                logger.info("Workspace configuration loaded successfully");
                manager
            }
            Err(e) => {
                logger.error(&format!("Failed to load workspace configuration: {}", e));
                eprintln!("❌ Error: Failed to load workspace configuration: {}", e);
                std::process::exit(1);
            }
        };

        let status = workspace_manager.get_status();
        logger.info(&format!("Loaded {} projects", status.enabled_projects));
        println!(
            "📊 {} - {} projects",
            status.workspace_name, status.enabled_projects
        );

        if daemon {
            println!("👻 Running as daemon...");
            run_as_daemon_workspace(workspace_manager, config, host, port, mcp_port).await;
        } else {
            run_interactive_workspace(workspace_manager, config, host, port, mcp_port).await;
        }
    } else if let Some(project_path) = project {
        // Legacy project mode
        println!("📁 Project Directory: {}", project_path.display());

        // Validate project directory
        if !project_path.exists() || !project_path.is_dir() {
            eprintln!(
                "Error: Project directory '{}' does not exist",
                project_path.display()
            );
            std::process::exit(1);
        }

        if daemon {
            println!("👻 Running as daemon...");
            run_as_daemon(project_path, config, host, port, mcp_port).await;
        } else {
            run_interactive(project_path, config, host, port, mcp_port).await;
        }
    } else {
        eprintln!("Error: Either --project or --workspace must be specified");
        std::process::exit(1);
    }
}

/// Create workspace info JSON for servers
fn create_project_workspace_info(
    workspace_manager: &WorkspaceManager,
    project: &vectorizer::workspace::config::ProjectConfig,
) -> String {
    use serde_json::json;

    let project_path = workspace_manager.get_project_path(&project.name).unwrap();
    let project_info = json!({
        "workspace": {
            "name": workspace_manager.config().workspace.name,
            "version": workspace_manager.config().workspace.version,
            "description": workspace_manager.config().workspace.description
        },
        "projects": [{
            "name": project.name,
            "description": project.description,
            "path": project_path.to_string_lossy(),
            "enabled": project.enabled,
            "collections": project.collections.iter().map(|collection| {
                json!({
                    "name": collection.name,
                    "description": collection.description,
                    "dimension": collection.dimension,
                    "metric": match collection.metric {
                        vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                        vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                        vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                    },
                    "embedding": {
                        "model": match collection.embedding.model {
                            vectorizer::workspace::config::EmbeddingModel::TfIdf => "tfidf",
                            vectorizer::workspace::config::EmbeddingModel::Bm25 => "bm25",
                            vectorizer::workspace::config::EmbeddingModel::Svd => "svd",
                            vectorizer::workspace::config::EmbeddingModel::Bert => "bert",
                            vectorizer::workspace::config::EmbeddingModel::MiniLm => "minilm",
                            vectorizer::workspace::config::EmbeddingModel::BagOfWords => "bagofwords",
                            vectorizer::workspace::config::EmbeddingModel::CharNGram => "charngram",
                            vectorizer::workspace::config::EmbeddingModel::RealModel => "real_model",
                            vectorizer::workspace::config::EmbeddingModel::OnnxModel => "onnx_model",
                        },
                        "dimension": collection.dimension,
                        "parameters": {
                            "k1": 1.5,
                            "b": 0.75
                        }
                    },
                    "indexing": {
                        "index_type": collection.indexing.index_type,
                        "parameters": collection.indexing.parameters
                    },
                    "processing": {
                        "chunk_size": collection.processing.chunk_size,
                        "chunk_overlap": collection.processing.chunk_overlap,
                        "include_patterns": collection.processing.include_patterns,
                        "exclude_patterns": collection.processing.exclude_patterns
                    }
                })
            }).collect::<Vec<_>>()
        }],
        "total_projects": 1,
        "total_collections": project.collections.len()
    });

    serde_json::to_string_pretty(&project_info).expect("Failed to serialize project workspace info")
}

fn create_workspace_info(
    workspace_manager: &WorkspaceManager,
    enabled_projects: &[&vectorizer::workspace::config::ProjectConfig],
) -> String {
    use serde_json::json;

    let projects_info: Vec<serde_json::Value> = enabled_projects.iter().map(|project| {
        let project_path = workspace_manager.get_project_path(&project.name).unwrap();
        json!({
            "name": project.name,
            "description": project.description,
            "path": project_path.to_string_lossy(),
            "enabled": project.enabled,
            "collections": project.collections.iter().map(|collection| {
                json!({
                    "name": collection.name,
                    "description": collection.description,
                    "dimension": collection.dimension,
                    "metric": match collection.metric {
                        vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                        vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                        vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                    },
                    "embedding": {
                        "model": match collection.embedding.model {
                            vectorizer::workspace::config::EmbeddingModel::TfIdf => "tfidf",
                            vectorizer::workspace::config::EmbeddingModel::Bm25 => "bm25",
                            vectorizer::workspace::config::EmbeddingModel::Svd => "svd",
                            vectorizer::workspace::config::EmbeddingModel::Bert => "bert",
                            vectorizer::workspace::config::EmbeddingModel::MiniLm => "minilm",
                            vectorizer::workspace::config::EmbeddingModel::BagOfWords => "bagofwords",
                            vectorizer::workspace::config::EmbeddingModel::CharNGram => "charngram",
                            vectorizer::workspace::config::EmbeddingModel::RealModel => "real_model",
                            vectorizer::workspace::config::EmbeddingModel::OnnxModel => "onnx_model",
                        },
                        "dimension": collection.embedding.dimension,
                        "parameters": collection.embedding.parameters
                    },
                    "indexing": {
                        "index_type": collection.indexing.index_type,
                        "parameters": collection.indexing.parameters
                    },
                    "processing": {
                        "chunk_size": collection.processing.chunk_size,
                        "chunk_overlap": collection.processing.chunk_overlap,
                        "include_patterns": collection.processing.include_patterns,
                        "exclude_patterns": collection.processing.exclude_patterns
                    }
                })
            }).collect::<Vec<_>>()
        })
    }).collect();

    let workspace_info = json!({
        "workspace": {
            "name": workspace_manager.config().workspace.name,
            "version": workspace_manager.config().workspace.version,
            "description": workspace_manager.config().workspace.description
        },
        "projects": projects_info,
        "total_projects": enabled_projects.len(),
        "total_collections": enabled_projects.iter().map(|p| p.collections.len()).sum::<usize>()
    });

    serde_json::to_string_pretty(&workspace_info).expect("Failed to serialize workspace info")
}

/// Handle workspace commands
async fn handle_workspace_command(command: WorkspaceCommands) {
    match command {
        WorkspaceCommands::Init { directory, name } => {
            init_workspace(directory, name).await;
        }
        WorkspaceCommands::Validate { config } => {
            validate_workspace(config).await;
        }
        WorkspaceCommands::Status { config } => {
            show_workspace_status(config).await;
        }
        WorkspaceCommands::List { config } => {
            list_workspace_projects(config).await;
        }
    }
}

fn default_backup_path() -> io::Result<PathBuf> {
    let backups = PathBuf::from("backups");
    if !backups.exists() {
        fs::create_dir_all(&backups)?;
    }
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    Ok(backups.join(format!("vectorizer_data_{}.tar.gz", ts)))
}

fn run_backup_command(data_dir: &PathBuf, output: Option<&PathBuf>) -> io::Result<()> {
    let data = data_dir;
    if !data.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Data dir not found: {}", data.display())));
    }

    let out_path = match output { Some(p) => p.clone(), None => default_backup_path()? };
    if let Some(parent) = out_path.parent() { fs::create_dir_all(parent)?; }

    let tar_gz = fs::File::create(&out_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = TarBuilder::new(enc);
    tar.append_dir_all(".", data)?;
    tar.into_inner()?.finish()?;

    println!("✅ Backup criado: {}", out_path.display());
    Ok(())
}

fn run_restore_command(archive: &PathBuf, data_dir: &PathBuf, clean: bool) -> io::Result<()> {
    if !archive.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Arquivo não encontrado: {}", archive.display())));
    }
    if clean && data_dir.exists() {
        fs::remove_dir_all(&data_dir)?;
    }
    fs::create_dir_all(&data_dir)?;

    let file = fs::File::open(archive)?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut ar = Archive::new(dec);
    ar.unpack(&data_dir)?;

    println!("✅ Restore concluído em: {}", data_dir.display());
    Ok(())
}

/// Initialize a new workspace
async fn init_workspace(directory: PathBuf, name: Option<String>) {
    println!("🚀 Initializing Vectorizer Workspace...");
    println!("=====================================");
    println!("📁 Directory: {}", directory.display());

    // Create workspace manager
    let workspace_manager = match WorkspaceManager::create_default(&directory) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error: Failed to create workspace: {}", e);
            std::process::exit(1);
        }
    };

    // Update workspace name if provided
    if let Some(workspace_name) = name {
        println!("📝 Setting workspace name: {}", workspace_name);
        // Note: This would require modifying the WorkspaceManager to allow updating the name
        // For now, we'll just show the created workspace
    }

    let status = workspace_manager.get_status();
    println!("✅ Workspace created successfully!");
    println!("📊 Name: {}", status.workspace_name);
    println!("📊 Version: {}", status.workspace_version);
    println!(
        "📁 Config file: {}",
        workspace_manager.config_path().display()
    );
    println!("\n💡 Next steps:");
    println!(
        "   1. Edit {} to configure your projects",
        workspace_manager.config_path().display()
    );
    println!("   2. Run 'vectorizer workspace validate' to check configuration");
    println!(
        "   3. Run 'vectorizer start --workspace {}' to start servers",
        workspace_manager.config_path().display()
    );
}

/// Validate workspace configuration
async fn validate_workspace(config: Option<PathBuf>) {
    println!("🔍 Validating Workspace Configuration...");
    println!("==========================================");

    let config_path = match config {
        Some(path) => path,
        None => {
            // Try to find workspace config in current directory
            match vectorizer::workspace::parser::find_workspace_config(".") {
                Ok(Some(path)) => path,
                Ok(None) => {
                    eprintln!("Error: No workspace configuration found");
                    eprintln!("💡 Run 'vectorizer workspace init' to create a new workspace");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: Failed to find workspace configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    println!("📁 Config file: {}", config_path.display());

    // Load and validate workspace
    let workspace_manager = match WorkspaceManager::load_from_file(&config_path) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error: Failed to load workspace configuration: {}", e);
            std::process::exit(1);
        }
    };

    let validation_result = workspace_manager.validate();

    if validation_result.is_valid() {
        println!("✅ Workspace configuration is valid!");

        if !validation_result.warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in &validation_result.warnings {
                println!("   - {}", warning);
            }
        }

        let status = workspace_manager.get_status();
        println!("\n📊 Workspace Status:");
        println!("   Name: {}", status.workspace_name);
        println!("   Version: {}", status.workspace_version);
        println!(
            "   Projects: {} enabled of {} total",
            status.enabled_projects, status.total_projects
        );
        println!("   Collections: {} total", status.total_collections);
    } else {
        println!("❌ Workspace configuration has errors:");
        for error in &validation_result.errors {
            println!("   - {}", error);
        }

        if !validation_result.warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in &validation_result.warnings {
                println!("   - {}", warning);
            }
        }

        std::process::exit(1);
    }
}

/// Show workspace status
async fn show_workspace_status(config: Option<PathBuf>) {
    println!("📊 Workspace Status");
    println!("===================");

    let config_path = match config {
        Some(path) => path,
        None => {
            // Try to find workspace config in current directory
            match vectorizer::workspace::parser::find_workspace_config(".") {
                Ok(Some(path)) => path,
                Ok(None) => {
                    eprintln!("Error: No workspace configuration found");
                    eprintln!("💡 Run 'vectorizer workspace init' to create a new workspace");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: Failed to find workspace configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    println!("📁 Config file: {}", config_path.display());

    // Load workspace
    let workspace_manager = match WorkspaceManager::load_from_file(&config_path) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error: Failed to load workspace configuration: {}", e);
            std::process::exit(1);
        }
    };

    let status = workspace_manager.get_status();

    println!("\n📋 Workspace Information:");
    println!("   Name: {}", status.workspace_name);
    println!("   Version: {}", status.workspace_version);
    println!("   Last Updated: {}", status.last_updated);

    println!("\n📂 Projects:");
    println!("   Total: {}", status.total_projects);
    println!("   Enabled: {}", status.enabled_projects);
    println!(
        "   Disabled: {}",
        status.total_projects - status.enabled_projects
    );

    println!("\n🗂️  Collections:");
    println!("   Total: {}", status.total_collections);

    // Show project details
    let enabled_projects = workspace_manager.enabled_projects();
    if !enabled_projects.is_empty() {
        println!("\n📋 Enabled Projects:");
        for project in enabled_projects {
            println!("   📁 {} - {}", project.name, project.description);
            println!("      Path: {}", project.path.display());
            println!("      Collections: {}", project.collections.len());

            for collection in &project.collections {
                println!(
                    "         🗂️  {} ({}D, {})",
                    collection.name,
                    collection.dimension,
                    match collection.metric {
                        vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                        vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                        vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                    }
                );
            }
        }
    }

    // Try to connect to the GRPC server to get actual collections
    println!("\n📊 Actual Collections in Vector Store:");
    match check_actual_collections().await {
        Ok(collections) => {
            println!("   Total: {}", collections.len());
            for (name, info) in collections {
                println!("   🗂️  {} ({}D, {} vectors)", name, info.dimension, info.vector_count);
            }
        }
        Err(e) => {
            println!("   ⚠️  Could not connect to server: {}", e);
            println!("   💡 Start the server with './scripts/start.sh' to see actual collections");
        }
    }
}

/// List workspace projects
async fn list_workspace_projects(config: Option<PathBuf>) {
    println!("📂 Workspace Projects");
    println!("=====================");

    let config_path = match config {
        Some(path) => path,
        None => {
            // Try to find workspace config in current directory
            match vectorizer::workspace::parser::find_workspace_config(".") {
                Ok(Some(path)) => path,
                Ok(None) => {
                    eprintln!("Error: No workspace configuration found");
                    eprintln!("💡 Run 'vectorizer workspace init' to create a new workspace");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: Failed to find workspace configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    println!("📁 Config file: {}", config_path.display());

    // Load workspace
    let workspace_manager = match WorkspaceManager::load_from_file(&config_path) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error: Failed to load workspace configuration: {}", e);
            std::process::exit(1);
        }
    };

    let config = workspace_manager.config();

    if config.projects.is_empty() {
        println!("\n📭 No projects configured");
        println!("💡 Add projects to your workspace configuration file");
        return;
    }

    println!("\n📋 All Projects:");
    for (index, project) in config.projects.iter().enumerate() {
        let status = if project.enabled { "✅" } else { "❌" };
        println!(
            "   {} {} {} - {}",
            index + 1,
            status,
            project.name,
            project.description
        );
        println!("      Path: {}", project.path.display());
        println!("      Collections: {}", project.collections.len());

        if !project.collections.is_empty() {
            for collection in &project.collections {
                println!(
                    "         🗂️  {} ({}D, {})",
                    collection.name,
                    collection.dimension,
                    match collection.metric {
                        vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                        vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                        vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                    }
                );
            }
        }
        println!();
    }
}

async fn run_interactive(
    project: PathBuf,
    config: PathBuf,
    host: String,
    port: u16,
    mcp_port: u16,
) {
    use tokio::signal;

    // Find MCP server executable
    let mcp_executable = find_executable("vectorizer-mcp-server")
        .expect("vectorizer-mcp-server executable not found. Please build the project first.");

    println!("Starting MCP server...");
    let mut mcp_child = TokioCommand::new(&mcp_executable)
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

    // Find REST API server executable
    let rest_executable = find_executable("vectorizer-server")
        .expect("vectorizer-server executable not found. Please build the project first.");

    println!("Starting REST API server...");
    let mut rest_child = TokioCommand::new(&rest_executable)
        .args(&[
            "--host",
            &host,
            "--port",
            &port.to_string(),
            "--project",
            &project.to_string_lossy(),
            "--config",
            &config.to_string_lossy(),
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

    // Initialize File Watcher System for legacy mode with GRPC connection (universal GPU detection)
    #[cfg(feature = "wgpu-gpu")]
    let file_watcher_vector_store = Arc::new({
        let mut store = VectorStore::new_auto();
        // Load dynamic collections after workspace initialization
        if let Err(e) = store.load_dynamic_collections() {
            eprintln!("⚠️ Failed to load dynamic collections for file watcher: {}", e);
        }
        store
    });

    #[cfg(not(feature = "wgpu-gpu"))]
    let file_watcher_vector_store = Arc::new({
        let mut store = VectorStore::new_auto();
        // Load dynamic collections after workspace initialization
        if let Err(e) = store.load_dynamic_collections() {
            eprintln!("⚠️ Failed to load dynamic collections for file watcher: {}", e);
        }
        store
    });
    let file_watcher_embedding_manager = Arc::new(Mutex::new({
        let mut manager = EmbeddingManager::new();
        
        // Register default providers
        use vectorizer::embedding::{TfIdfEmbedding, Bm25Embedding};
        let tfidf = TfIdfEmbedding::new(128);
        let bm25 = Bm25Embedding::new(128);
        manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.set_default_provider("tfidf").unwrap();
        
        manager
    }));
    
    // Wait for REST API server to be ready before starting File Watcher and indexing
    let project_path_clone = project.clone();
    tokio::spawn(async move {
        // Wait for server to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Start background indexing for the project
        let indexing_vector_store = Arc::clone(&file_watcher_vector_store);
        let indexing_embedding_manager = Arc::clone(&file_watcher_embedding_manager);
        
        // Create a simple indexing progress tracker
        let indexing_progress = Arc::new(Mutex::new(std::collections::HashMap::new()));
        
        // Start indexing in background
        tokio::spawn(async move {
            start_background_indexing_for_project(
                project_path_clone.to_string_lossy().to_string(),
                indexing_vector_store,
                indexing_embedding_manager,
                indexing_progress,
            ).await;
        });
        
        // Start file watcher
        if let Err(e) = start_file_watcher_system_with_grpc(
            file_watcher_vector_store,
            file_watcher_embedding_manager,
            format!("http://{}:{}", host, port),
            Arc::new(Mutex::new(None::<vectorizer::file_watcher::FileWatcherSystem>)),
        ).await {
            eprintln!("❌ Failed to start File Watcher System: {}", e);
        }
    });

    // Wait for shutdown signal
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    println!("\n🛑 Shutting down servers...");
    
    // Kill child processes
    if let Err(e) = mcp_child.kill().await {
        eprintln!("Warning: Failed to kill MCP server: {}", e);
    }
    if let Err(e) = rest_child.kill().await {
        eprintln!("Warning: Failed to kill REST server: {}", e);
    }
    
    // Wait for processes to terminate
    let _ = mcp_child.wait().await;
    let _ = rest_child.wait().await;
    
    println!("✅ Servers stopped.");
}

/// Run servers interactively with workspace
async fn run_interactive_workspace(
    workspace_manager: WorkspaceManager,
    config: PathBuf,
    host: String,
    port: u16,
    _mcp_port: u16,
) {
    use tokio::signal;

    // Check if we have enabled projects
    let enabled_projects_count = workspace_manager.enabled_projects().len();
    if enabled_projects_count == 0 {
        eprintln!("Error: No enabled projects found in workspace");
        std::process::exit(1);
    }

    println!("📊 Loading {} projects from workspace:", enabled_projects_count);
    
    // Load projects for display
    let enabled_projects = workspace_manager.enabled_projects();
    let _enabled_projects_clone = enabled_projects.clone();
    for project in &enabled_projects {
        println!("   📁 {} - {}", project.name, project.description);
        println!(
            "      Path: {}",
            workspace_manager
                .get_project_path(&project.name)
                .unwrap()
                .display()
        );
        println!("      Collections: {}", project.collections.len());
        for collection in &project.collections {
            println!(
                "         🗂️  {} ({}D, {})",
                collection.name,
                collection.dimension,
                match collection.metric {
                    vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                    vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                    vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                }
            );
        }
    }

    let logger = WorkspaceLogger::new();

    // Check for existing processes and kill them if found
    if check_existing_processes(&logger) {
        logger.warn("Found existing vectorizer processes. Killing them to prevent conflicts...");
        if let Err(e) = kill_existing_processes(&logger) {
            logger.error(&format!("Failed to kill existing processes: {}", e));
            eprintln!("❌ Failed to kill existing vectorizer processes: {}", e);
            eprintln!("💡 Please manually kill existing processes with: pkill -f vectorizer");
            std::process::exit(1);
        }
    }

    logger.info(&format!(
        "Starting unified server for workspace with {} projects",
        enabled_projects.len()
    ));

    // Create unified workspace info file with ALL projects
    let workspace_info_path = std::env::temp_dir().join("vectorizer-workspace-full.json");

    // Create unified workspace info containing ALL projects directly
    let projects_array: Vec<serde_json::Value> = enabled_projects.iter().map(|project| {
        let project_path = workspace_manager.get_project_path(&project.name).unwrap();
        json!({
            "name": project.name,
            "description": project.description,
            "path": project_path.to_string_lossy(),
            "enabled": project.enabled,
            "collections": project.collections.iter().map(|collection| {
                json!({
                    "name": collection.name,
                    "description": collection.description,
                    "dimension": collection.dimension,
                    "metric": match collection.metric {
                        vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                        vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                        vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                    },
                    "embedding": {
                        "model": match collection.embedding.model {
                            vectorizer::workspace::config::EmbeddingModel::TfIdf => "tfidf",
                            vectorizer::workspace::config::EmbeddingModel::Bm25 => "bm25",
                            vectorizer::workspace::config::EmbeddingModel::Svd => "svd",
                            vectorizer::workspace::config::EmbeddingModel::Bert => "bert",
                            vectorizer::workspace::config::EmbeddingModel::MiniLm => "minilm",
                            vectorizer::workspace::config::EmbeddingModel::BagOfWords => "bagofwords",
                            vectorizer::workspace::config::EmbeddingModel::CharNGram => "charngram",
                            vectorizer::workspace::config::EmbeddingModel::RealModel => "real_model",
                            vectorizer::workspace::config::EmbeddingModel::OnnxModel => "onnx_model",
                        },
                        "dimension": collection.embedding.dimension,
                        "parameters": {
                            "k1": 1.5,
                            "b": 0.75
                        }
                    },
                    "indexing": {
                        "index_type": collection.indexing.index_type,
                        "parameters": collection.indexing.parameters
                    },
                    "processing": {
                        "chunk_size": collection.processing.chunk_size,
                        "chunk_overlap": collection.processing.chunk_overlap,
                        "include_patterns": collection.processing.include_patterns,
                        "exclude_patterns": collection.processing.exclude_patterns
                    }
                })
            }).collect::<Vec<_>>()
        })
    }).collect();

    let workspace_info = json!({
        "workspace_name": workspace_manager.config().workspace.name,
        "workspace_version": workspace_manager.config().workspace.version,
        "projects": projects_array
    });

    let workspace_info = serde_json::to_string(&workspace_info).unwrap();
    if let Err(e) = std::fs::write(&workspace_info_path, workspace_info) {
        logger.error(&format!("Failed to create unified workspace config: {}", e));
        std::process::exit(1);
    }

    logger.info(&format!(
        "Created unified workspace config at: {}",
        workspace_info_path.display()
    ));

    // Find MCP server executable
    let mcp_executable = find_executable("vectorizer-mcp-server")
        .expect("vectorizer-mcp-server executable not found. Please build the project first.");

    // Start SINGLE MCP server for all projects
    let mut mcp_child = match TokioCommand::new(&mcp_executable)
        .env("VECTORIZER_WORKSPACE_INFO", &workspace_info_path)
        .env("VECTORIZER_SERVER_PORT", "15002")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            logger.info(&format!(
                "Started unified MCP server (PID:{}, Port:15002)",
                child.id().unwrap_or(0)
            ));
            child
        }
        Err(e) => {
            logger.error(&format!("Failed to start unified MCP server: {}", e));
            std::process::exit(1);
        }
    };

    // Find REST API server executable
    let rest_executable = find_executable("vectorizer-server")
        .expect("vectorizer-server executable not found. Please build the project first.");

    // Start SINGLE REST API server for all projects
    let api_child = match TokioCommand::new(&rest_executable)
        .args(&[
            "--host",
            &host,
            "--port",
            "15001",
            "--workspace",
            &workspace_info_path.to_string_lossy(),
            "--config",
            &config.to_string_lossy(),
        ])
        .env("VECTORIZER_WORKSPACE_INFO", &workspace_info_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            logger.info(&format!(
                "Started unified REST API server (PID:{}, Port:15001)",
                child.id().unwrap_or(0)
            ));
            child
        }
        Err(e) => {
            logger.error(&format!("Failed to start unified REST API server: {}", e));
            let _ = mcp_child.kill().await;
            std::process::exit(1);
        }
    };

    let server_handles = vec![(
        "unified".to_string(),
        mcp_child,
        api_child,
        15001u16,
        15002u16,
    )];

    // In unified mode, we either start both servers or fail completely
    logger.info("Unified servers started successfully");

    println!("✅ Unified server running");
    println!("  REST API: http://{}:15001", host);
    println!("  MCP Server: http://127.0.0.1:15002/sse");
    println!("  GRPC Server: http://127.0.0.1:15003");
    println!(
        "  Collections: {} from {} projects",
        enabled_projects
            .iter()
            .map(|p| p.collections.len())
            .sum::<usize>(),
        enabled_projects_count
    );
    println!("\n⚡ Ctrl+C to stop | 📄 vectorizer-workspace.log\n");

    // Initialize GRPC server components (universal GPU detection)
    #[cfg(feature = "wgpu-gpu")]
    let grpc_vector_store = Arc::new({
        let mut store = VectorStore::new_auto();
        // Load dynamic collections after workspace initialization
        if let Err(e) = store.load_dynamic_collections() {
            eprintln!("⚠️ Failed to load dynamic collections for GRPC server: {}", e);
        }
        store
    });

    #[cfg(not(feature = "wgpu-gpu"))]
    let grpc_vector_store = Arc::new({
        let mut store = VectorStore::new_auto();
        // Load dynamic collections after workspace initialization
        if let Err(e) = store.load_dynamic_collections() {
            eprintln!("⚠️ Failed to load dynamic collections for GRPC server: {}", e);
        }
        store
    });
    let grpc_embedding_manager = Arc::new(Mutex::new({
        let mut manager = EmbeddingManager::new();
        
        // Register all embedding providers
        use vectorizer::embedding::{
            TfIdfEmbedding, Bm25Embedding, SvdEmbedding, 
            BertEmbedding, MiniLmEmbedding, BagOfWordsEmbedding, CharNGramEmbedding
        };
        
        let tfidf = TfIdfEmbedding::new(512);
        let bm25 = Bm25Embedding::new(512);
        let svd = SvdEmbedding::new(512, 512);
        let bert = BertEmbedding::new(512);
        let minilm = MiniLmEmbedding::new(512);
        let bow = BagOfWordsEmbedding::new(512);
        let char_ngram = CharNGramEmbedding::new(512, 3);
        
        manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.register_provider("svd".to_string(), Box::new(svd));
        manager.register_provider("bert".to_string(), Box::new(bert));
        manager.register_provider("minilm".to_string(), Box::new(minilm));
        manager.register_provider("bagofwords".to_string(), Box::new(bow));
        manager.register_provider("charngram".to_string(), Box::new(char_ngram));
        
        manager
    }));
    let grpc_indexing_progress = Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    // Load full configuration including summarization
    let full_config = FullVectorizerConfig::from_yaml_file(&std::path::PathBuf::from("config.yml"))
        .unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to load config.yml: {}, using defaults", e);
            FullVectorizerConfig::default()
        });

    println!("🔧 Summarization config loaded: enabled={}", full_config.summarization.enabled);

    let summarization_config = if full_config.summarization.enabled {
        println!("✅ Summarization ENABLED - creating SummarizationManager");
        Some(full_config.summarization)
    } else {
        println!("❌ Summarization DISABLED - skipping SummarizationManager creation");
        None
    };
    
    // Start GRPC server
    let grpc_config = GrpcConfig::from_env();
    let grpc_vector_store_clone = Arc::clone(&grpc_vector_store);
    let grpc_embedding_manager_clone = Arc::clone(&grpc_embedding_manager);
    let grpc_indexing_progress_clone = Arc::clone(&grpc_indexing_progress);
    
    tokio::spawn(async move {
        if let Err(e) = start_grpc_server(
            grpc_config.server,
            grpc_vector_store_clone,
            grpc_embedding_manager_clone,
            grpc_indexing_progress_clone,
            summarization_config,
        ).await {
            eprintln!("❌ Failed to start GRPC server: {}", e);
        }
    });

    // Initialize and start File Watcher System with shared reference
    let file_watcher_vector_store = Arc::clone(&grpc_vector_store);
    let file_watcher_embedding_manager = Arc::clone(&grpc_embedding_manager);
    let file_watcher_system = Arc::new(Mutex::new(None::<vectorizer::file_watcher::FileWatcherSystem>));

    // Start background indexing and status replication
    let indexing_vector_store = Arc::clone(&grpc_vector_store);
    let indexing_embedding_manager = Arc::clone(&grpc_embedding_manager);
    let indexing_progress = Arc::clone(&grpc_indexing_progress);
    let indexing_file_watcher = Arc::clone(&file_watcher_system);

    tokio::spawn(async move {
        start_background_indexing_with_config(
            indexing_vector_store,
            indexing_embedding_manager,
            indexing_progress,
            indexing_file_watcher,
        ).await;
    });
    
    // Start file watcher system
    let file_watcher_system_clone = Arc::clone(&file_watcher_system);
    tokio::spawn(async move {
        if let Err(e) = start_file_watcher_system_with_grpc(
            file_watcher_vector_store,
            file_watcher_embedding_manager,
            format!("http://{}:{}", host, port),
            file_watcher_system_clone,
        ).await {
            eprintln!("❌ Failed to start File Watcher System: {}", e);
        }
    });

    // Wait for shutdown signal
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    println!("\n🛑 Stopping unified server...");
    for (_, mut mcp_child, mut api_child, _, _) in server_handles {
        let _ = mcp_child.kill().await;
        let _ = api_child.kill().await;
    }
    println!("✅ Stopped");
}

/// Start background indexing process with configuration loading
async fn start_background_indexing_with_config(
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    indexing_progress: Arc<Mutex<std::collections::HashMap<String, vectorizer::grpc::vectorizer::IndexingStatus>>>,
    file_watcher_system: Arc<Mutex<Option<vectorizer::file_watcher::FileWatcherSystem>>>,
) {
    println!("🔄 [BACKGROUND INDEXING] Starting background indexing process...");

    // Wait for servers to be ready with health check
    println!("⏳ [BACKGROUND INDEXING] Waiting for servers to be ready...");
    wait_for_server_ready().await;
    println!("✅ [BACKGROUND INDEXING] Servers are ready");

    // Load workspace configuration
    println!("📂 [BACKGROUND INDEXING] Loading workspace configuration...");
    let workspace_manager = match vectorizer::workspace::manager::WorkspaceManager::load_from_file("vectorize-workspace.yml") {
        Ok(manager) => {
            println!("✅ [BACKGROUND INDEXING] Workspace configuration loaded successfully");
            manager
        },
        Err(e) => {
            println!("❌ [BACKGROUND INDEXING] Failed to load workspace configuration: {}", e);
            return;
        }
    };

    let enabled_projects = workspace_manager.enabled_projects();
    let total_projects = enabled_projects.len();
    println!("📊 [BACKGROUND INDEXING] Found {} enabled projects", total_projects);

    // Process projects in parallel (up to 4 concurrent)
    let max_concurrent_projects = std::cmp::min(4, total_projects);
    println!("🚀 [BACKGROUND INDEXING] Starting parallel processing of {} projects (max {} concurrent)", total_projects, max_concurrent_projects);
    
    use tokio::sync::Semaphore;
    let semaphore = Arc::new(Semaphore::new(max_concurrent_projects));
    let mut project_tasks = Vec::new();

    for (i, project_ref) in enabled_projects.iter().enumerate() {
        let semaphore_clone = Arc::clone(&semaphore);
        let vector_store_clone = Arc::clone(&vector_store);
        let embedding_manager_clone = Arc::clone(&embedding_manager);
        let file_watcher_system_clone = Arc::clone(&file_watcher_system);

        // Clone project data to avoid lifetime issues
        let project_name = project_ref.name.clone();
        let project_path = project_ref.path.clone();
        let collections = project_ref.collections.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            // Process all collections for this project sequentially
            for collection in &collections {
                //println!("🗂️ Processing collection: {} from {}", collection.name, project_path.display());

                // Update status to processing
                update_collection_status(&collection.name, "processing", 0.0, None).await;

                // Try real indexing
                match do_real_indexing(
                    &collection.name,
                    &project_path.to_string_lossy(),
                    Arc::clone(&vector_store_clone),
                    Arc::clone(&embedding_manager_clone),
                ).await {
                    Ok(count) => {
                        update_collection_status(&collection.name, "completed", 100.0, Some(count)).await;
                        
                        // Apply quantization to all vectors if enabled
                        if let Ok(coll) = vector_store_clone.get_collection(&collection.name) {
                            let config = coll.config();
                            if matches!(config.quantization, QuantizationConfig::SQ { bits: 8 }) {
                                if let Err(e) = coll.requantize_existing_vectors() {
                                    eprintln!("⚠️ Failed to quantize vectors in collection '{}': {}", collection.name, e);
                                } else {
                                    println!("✅ Successfully quantized {} vectors in collection '{}'", count, collection.name);
                                }
                            } 
                        } else {
                            println!("🔍 DEBUG: Could not get collection '{}'", collection.name);
                        }

                        // Create summary collections for this collection
                        create_summary_collections_for_project(
                            &collection.name,
                            &project_path.to_string_lossy(),
                            Arc::clone(&vector_store_clone),
                            Arc::clone(&embedding_manager_clone),
                        ).await;

                        // Update file watcher with newly indexed collection
                        let mut system = file_watcher_system_clone.lock().await;
                        if let Some(ref mut watcher) = *system {
                            if let Err(e) = watcher.update_with_collection(&collection.name).await {
                                eprintln!("⚠️ Failed to update file watcher with collection '{}': {}", collection.name, e);
                            }
                        }
                    }
                    Err(e) => {
                        update_collection_status(&collection.name, "failed", 0.0, None).await;
                        println!("❌ Collection '{}' indexing failed: {}", collection.name, e);
                    }
                }
            }

            // Return project info
            (project_name, true)
        });

        project_tasks.push(task);
    }

    // Wait for all project tasks to complete
    let mut completed_projects = 0;
    for task in project_tasks {
        match task.await {
            Ok((project_name, success)) => {
                if success {
                    completed_projects += 1;
                    println!("✅ Project '{}' completed successfully", project_name);
                } else {
                    println!("⚠️ Project '{}' completed with issues", project_name);
                }
            }
            Err(e) => {
                println!("❌ Project task failed: {}", e);
            }
        }
    }

    println!("✅ Parallel processing completed: {}/{} projects processed successfully", completed_projects, total_projects);
    
    println!("✅ Background indexing completed");
}

/// Start File Watcher System for real-time file monitoring with GRPC connection
async fn start_file_watcher_system_with_grpc(
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    grpc_endpoint: String,
    file_watcher_system: Arc<Mutex<Option<vectorizer::file_watcher::FileWatcherSystem>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("👁️ Starting File Watcher System...");
    
    // Load workspace configuration first
    let workspace_config = match vectorizer::workspace::WorkspaceManager::load_from_file("vectorize-workspace.yml") {
        Ok(manager) => {
            println!("✅ Workspace config loaded successfully");
            manager.config().clone()
        }
        Err(e) => {
            vectorizer::workspace::config::WorkspaceConfig::default()
        }
    };

    // Get file watcher configuration from workspace or use defaults
    let file_watcher_config = workspace_config.file_watcher.unwrap_or_else(|| {
        println!("ℹ️ No file watcher config in workspace, using defaults");
        FileWatcherYamlConfig::default()
    });

    // Check if file watcher is enabled
    if !file_watcher_config.enabled {
        println!("ℹ️ File Watcher is disabled in configuration");
        return Ok(());
    }

    // Convert to FileWatcherConfig and set GRPC endpoint
    let mut watcher_config = file_watcher_config.to_file_watcher_config();
    watcher_config.grpc_endpoint = Some(grpc_endpoint.clone());
    
    // Convert Mutex to RwLock for compatibility
    // For now, create a new EmbeddingManager with default providers
    let mut new_manager = EmbeddingManager::new();
    
    // Register default providers
    use vectorizer::embedding::{TfIdfEmbedding, Bm25Embedding};
    let tfidf = TfIdfEmbedding::new(128);
    let bm25 = Bm25Embedding::new(128);
    new_manager.register_provider("tfidf".to_string(), Box::new(tfidf));
    new_manager.register_provider("bm25".to_string(), Box::new(bm25));
    new_manager.set_default_provider("tfidf").unwrap();
    
    let embedding_manager_rwlock = Arc::new(tokio::sync::RwLock::new(new_manager));
    
    // Create GRPC client for communication with vectorizer-server
    let grpc_client = match create_grpc_client(&grpc_endpoint).await {
        Ok(client) => {
            println!("✅ GRPC client connected to {}", grpc_endpoint);
            Some(Arc::new(client))
        }
        Err(e) => {
            println!("⚠️ Failed to create GRPC client: {}. File Watcher will use local operations.", e);
            None
        }
    };
    
    // Create and start File Watcher System
    let file_watcher = FileWatcherSystem::new(
        watcher_config,
        vector_store,
        embedding_manager_rwlock,
        grpc_client,
    );

    println!("👁️ File Watcher System initialized");
    println!("📝 Collection: {}", file_watcher_config.collection_name.unwrap_or_else(|| "default_collection".to_string()));

    // Store the file watcher system for incremental updates
    {
        let mut system = file_watcher_system.lock().await;
        *system = Some(file_watcher);
    }

    println!("✅ File Watcher System started successfully");
    
    // Keep the file watcher running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        // File watcher runs in background, this is just to keep the task alive
    }
}

/// Load file watcher configuration from YAML file
async fn load_file_watcher_config(config_path: &std::path::Path) -> Result<FileWatcherYamlConfig, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(config_path).await?;
    
    // Parse the YAML and extract file_watcher section
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    if let Some(file_watcher_section) = yaml.get("file_watcher") {
        let config: FileWatcherYamlConfig = serde_yaml::from_value(file_watcher_section.clone())?;
        Ok(config)
    } else {
        // Return default config if no file_watcher section found
        Ok(FileWatcherYamlConfig::default())
    }
}


/// Wait for server to be ready with health check
async fn wait_for_server_ready() {
    println!("⏳ Waiting for vectorizer-server to be ready...");
    
    let client = reqwest::Client::new();
    let mut attempts = 0;
    let max_attempts = 30; // 30 seconds max
    
    loop {
        match client.get("http://127.0.0.1:15001/api/v1/collections")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await 
        {
            Ok(response) if response.status().is_success() => {
                println!("✅ vectorizer-server is ready!");
                break;
            }
            Ok(_) => {
            }
            Err(_) => {
            }
        }
        
        attempts += 1;
        if attempts >= max_attempts {
            println!("⚠️ Server not ready after {} seconds, proceeding anyway...", max_attempts);
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

/// Update collection status via API with retry logic
async fn update_collection_status(collection_name: &str, status: &str, progress: f32, vector_count: Option<usize>) {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "collection": collection_name,
        "status": status,
        "progress": progress,
        "total_documents": vector_count.unwrap_or(0),
        "processed_documents": if status == "completed" { vector_count.unwrap_or(0) } else { (progress as usize) },
        "vector_count": vector_count.unwrap_or(0)
    });
    
    // Retry logic for API calls
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        match client.post("http://127.0.0.1:15001/api/v1/indexing/progress")
            .timeout(std::time::Duration::from_secs(5))
            .json(&payload)
            .send()
            .await 
        {
            Ok(response) if response.status().is_success() => {
                // Status updated successfully
                return;
            }
            Ok(response) => {
                println!("⚠️ Server returned status {} for {}", response.status(), collection_name);
            }
            Err(e) => {
                println!("❌ Failed to update status for {} (attempt {}/{}): {}", 
                    collection_name, attempts + 1, max_attempts, e);
            }
        }
        
        attempts += 1;
        if attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}

/// Do real indexing of a collection using DocumentLoader
async fn do_real_indexing(
    collection_name: &str,
    project_path: &str,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
) -> Result<usize, String> {
    println!("📁 Indexing collection '{}' from path: {}", collection_name, project_path);
    
    // Load workspace to get collection-specific patterns
    let workspace_manager = match vectorizer::workspace::WorkspaceManager::load_from_file("vectorize-workspace.yml") {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to load workspace: {}", e);
            return Err(format!("Failed to load workspace: {}", e));
        }
    };
    
    // Find the specific collection configuration
    let collection_config = workspace_manager.enabled_projects()
        .iter()
        .flat_map(|p| &p.collections)
        .find(|c| c.name == collection_name)
        .ok_or_else(|| format!("Collection '{}' not found in workspace", collection_name))?;
    
    // Create loader config
    let loader_config = vectorizer::document_loader::LoaderConfig {
        collection_name: collection_name.to_string(),
        max_chunk_size: collection_config.processing.chunk_size,
        chunk_overlap: collection_config.processing.chunk_overlap,
        allowed_extensions: vec![
            "md".to_string(), "txt".to_string(), "rs".to_string(), "py".to_string(), 
            "js".to_string(), "ts".to_string(), "json".to_string(), "yaml".to_string(),
            "yml".to_string(), "toml".to_string(), "html".to_string(), "css".to_string()
        ],
        include_patterns: collection_config.processing.include_patterns.clone(),
        exclude_patterns: {
            let mut exclude = collection_config.processing.exclude_patterns.clone();
            // Add common exclusions
            exclude.extend(vec![
                "**/target/**".to_string(), 
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/*.png".to_string(),
                "**/*.jpg".to_string(),
                "**/*.jpeg".to_string(),
                "**/*.gif".to_string(),
                "**/*.bmp".to_string(),
                "**/*.webp".to_string(),
                "**/*.svg".to_string(),
                "**/*.ico".to_string(),
                "**/*.mp4".to_string(),
                "**/*.avi".to_string(),
                "**/*.mov".to_string(),
                "**/*.wmv".to_string(),
                "**/*.flv".to_string(),
                "**/*.webm".to_string(),
                "**/*.mp3".to_string(),
                "**/*.wav".to_string(),
                "**/*.flac".to_string(),
                "**/*.aac".to_string(),
                "**/*.ogg".to_string(),
                "**/*.db".to_string(),
                "**/*.sqlite".to_string(),
                "**/*.sqlite3".to_string(),
                "**/*.bin".to_string(),
                "**/*.exe".to_string(),
                "**/*.dll".to_string(),
                "**/*.so".to_string(),
                "**/*.dylib".to_string(),
                "**/*.zip".to_string(),
                "**/*.tar".to_string(),
                "**/*.gz".to_string(),
                "**/*.rar".to_string(),
                "**/*.7z".to_string(),
                "**/*.pdf".to_string(),
                "**/*.doc".to_string(),
                "**/*.docx".to_string(),
                "**/*.xls".to_string(),
                "**/*.xlsx".to_string(),
                "**/*.ppt".to_string(),
                "**/*.pptx".to_string(),
            ]);
            exclude
        },
        embedding_type: format!("{:?}", collection_config.embedding.model).to_lowercase(),
        embedding_dimension: collection_config.embedding.dimension,
        max_file_size: 10 * 1024 * 1024, // 10MB
    };
    
    // Load full configuration including summarization for document loader
    let full_config_loader = FullVectorizerConfig::from_yaml_file(&std::path::PathBuf::from("config.yml"))
        .unwrap_or_else(|e| {
            FullVectorizerConfig::default()
        });

    let summarization_config = Some(full_config_loader.summarization);
    let mut loader = vectorizer::document_loader::DocumentLoader::new_with_summarization(loader_config, summarization_config);
    
    // Load project with real indexing (async version) using shared vector store
    match loader.load_project_with_cache(project_path, &vector_store).await {
        Ok((count, _cached)) => {
            println!("✅ Indexed {} documents for collection '{}'", count, collection_name);
            Ok(count)
        }
        Err(e) => {
            let error_msg = format!("Failed to index collection '{}': {}", collection_name, e);
            println!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}


/// Run servers as daemon with workspace
async fn run_as_daemon_workspace(
    workspace_manager: WorkspaceManager,
    config: PathBuf,
    host: String,
    port: u16,
    mcp_port: u16,
) {
    let logger = WorkspaceLogger::new();
    logger.info("Starting daemon mode with workspace");

    // Load all enabled projects from workspace
    let enabled_projects = workspace_manager.enabled_projects();
    if enabled_projects.is_empty() {
        eprintln!("Error: No enabled projects found in workspace");
        std::process::exit(1);
    }

    // Check for existing processes and kill them if found
    if check_existing_processes(&logger) {
        logger.warn("Found existing vectorizer processes. Killing them to prevent conflicts...");
        if let Err(e) = kill_existing_processes(&logger) {
            logger.error(&format!("Failed to kill existing processes: {}", e));
            eprintln!("❌ Failed to kill existing vectorizer processes: {}", e);
            std::process::exit(1);
        }
    }

    logger.info(&format!(
        "Starting unified server for workspace with {} projects",
        enabled_projects.len()
    ));

    // Create unified workspace info file with ALL projects
    let workspace_info_path = std::env::temp_dir().join("vectorizer-workspace-full.json");

    // Create unified workspace info containing ALL projects directly
    let projects_array: Vec<serde_json::Value> = enabled_projects.iter().map(|project| {
        let project_path = workspace_manager.get_project_path(&project.name).unwrap();
        json!({
            "name": project.name,
            "description": project.description,
            "path": project_path.to_string_lossy(),
            "enabled": project.enabled,
            "collections": project.collections.iter().map(|collection| {
                json!({
                    "name": collection.name,
                    "description": collection.description,
                    "dimension": collection.dimension,
                    "metric": match collection.metric {
                        vectorizer::workspace::config::DistanceMetric::Cosine => "cosine",
                        vectorizer::workspace::config::DistanceMetric::Euclidean => "euclidean",
                        vectorizer::workspace::config::DistanceMetric::DotProduct => "dot_product",
                    },
                    "embedding": {
                        "model": match collection.embedding.model {
                            vectorizer::workspace::config::EmbeddingModel::TfIdf => "tfidf",
                            vectorizer::workspace::config::EmbeddingModel::Bm25 => "bm25",
                            vectorizer::workspace::config::EmbeddingModel::Svd => "svd",
                            vectorizer::workspace::config::EmbeddingModel::Bert => "bert",
                            vectorizer::workspace::config::EmbeddingModel::MiniLm => "minilm",
                            vectorizer::workspace::config::EmbeddingModel::BagOfWords => "bagofwords",
                            vectorizer::workspace::config::EmbeddingModel::CharNGram => "charngram",
                            vectorizer::workspace::config::EmbeddingModel::RealModel => "real_model",
                            vectorizer::workspace::config::EmbeddingModel::OnnxModel => "onnx_model",
                        },
                        "dimension": collection.embedding.dimension,
                        "parameters": collection.embedding.parameters
                    },
                    "indexing": {
                        "index_type": collection.indexing.index_type,
                        "parameters": collection.indexing.parameters
                    },
                    "processing": {
                        "chunk_size": collection.processing.chunk_size,
                        "chunk_overlap": collection.processing.chunk_overlap,
                        "include_patterns": collection.processing.include_patterns,
                        "exclude_patterns": collection.processing.exclude_patterns
                    }
                })
            }).collect::<Vec<_>>()
        })
    }).collect();

    let workspace_info = json!({
        "workspace_name": workspace_manager.config().workspace.name,
        "workspace_version": workspace_manager.config().workspace.version,
        "projects": projects_array
    });

    let workspace_info_json = serde_json::to_string(&workspace_info).unwrap();
    if let Err(e) = std::fs::write(&workspace_info_path, workspace_info_json) {
        logger.error(&format!("Failed to create unified workspace config: {}", e));
        std::process::exit(1);
    }

    logger.info(&format!(
        "Created unified workspace config at: {}",
        workspace_info_path.display()
    ));

    // Find MCP server executable
    let mcp_executable = find_executable("vectorizer-mcp-server")
        .expect("vectorizer-mcp-server executable not found. Please build the project first.");

    // Start SINGLE MCP server for all projects
    let mut mcp_child = match TokioCommand::new(&mcp_executable)
        .env("VECTORIZER_WORKSPACE_INFO", &workspace_info_path)
        .env("VECTORIZER_SERVER_PORT", "15002")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            logger.info(&format!(
                "Started unified MCP server (PID:{}, Port:15002)",
                child.id().unwrap_or(0)
            ));
            child
        }
        Err(e) => {
            logger.error(&format!("Failed to start unified MCP server: {}", e));
            std::process::exit(1);
        }
    };

    // Find REST API server executable
    let rest_executable = find_executable("vectorizer-server")
        .expect("vectorizer-server executable not found. Please build the project first.");

    // Start SINGLE REST API server for all projects
    let api_child = match TokioCommand::new(&rest_executable)
        .args(&[
            "--host",
            &host,
            "--port",
            "15001",
            "--workspace",
            &workspace_info_path.to_string_lossy(),
            "--config",
            &config.to_string_lossy(),
        ])
        .env("VECTORIZER_WORKSPACE_INFO", &workspace_info_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            logger.info(&format!(
                "Started unified REST API server (PID:{}, Port:15001)",
                child.id().unwrap_or(0)
            ));
            child
        }
        Err(e) => {
            logger.error(&format!("Failed to start unified REST API server: {}", e));
            let _ = mcp_child.kill().await;
            std::process::exit(1);
        }
    };

    // In unified mode, we either start both servers or fail completely
    logger.info("Unified servers started successfully");

    // Print minimal success message
    println!("✅ Daemon services started successfully!");
    println!("📡 REST API: http://{}:15001", host);
    println!("🔧 MCP Server: http://127.0.0.1:15002/sse");
    println!("🔧 GRPC Server: http://127.0.0.1:15003");
    println!(
        "  Collections: {} from {} projects",
        enabled_projects
            .iter()
            .map(|p| p.collections.len())
            .sum::<usize>(),
        enabled_projects.len()
    );
    println!("📄 Logs: vectorizer-workspace.log");
    println!("🛑 Use 'vectorizer stop' to stop all services");

    // Initialize GRPC server components with universal GPU detection
    #[cfg(feature = "wgpu-gpu")]
    let grpc_vector_store = Arc::new(VectorStore::new_auto());
    
    #[cfg(not(feature = "wgpu-gpu"))]
    let grpc_vector_store = Arc::new({
        let mut store = VectorStore::new_auto();
        // Load dynamic collections after workspace initialization
        if let Err(e) = store.load_dynamic_collections() {
            eprintln!("⚠️ Failed to load dynamic collections for GRPC server: {}", e);
        }
        store
    });
    let grpc_embedding_manager = Arc::new(Mutex::new({
        let mut manager = EmbeddingManager::new();
        
        // Register all embedding providers
        use vectorizer::embedding::{
            TfIdfEmbedding, Bm25Embedding, SvdEmbedding, 
            BertEmbedding, MiniLmEmbedding, BagOfWordsEmbedding, CharNGramEmbedding
        };
        
        let tfidf = TfIdfEmbedding::new(512);
        let bm25 = Bm25Embedding::new(512);
        let svd = SvdEmbedding::new(512, 512);
        let bert = BertEmbedding::new(512);
        let minilm = MiniLmEmbedding::new(512);
        let bow = BagOfWordsEmbedding::new(512);
        let char_ngram = CharNGramEmbedding::new(512, 3);
        
        manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.register_provider("svd".to_string(), Box::new(svd));
        manager.register_provider("bert".to_string(), Box::new(bert));
        manager.register_provider("minilm".to_string(), Box::new(minilm));
        manager.register_provider("bagofwords".to_string(), Box::new(bow));
        manager.register_provider("charngram".to_string(), Box::new(char_ngram));
        
        manager
    }));
    let grpc_indexing_progress = Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    // Load full configuration including summarization
    let full_config = FullVectorizerConfig::from_yaml_file(&std::path::PathBuf::from("config.yml"))
        .unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to load config.yml: {}, using defaults", e);
            FullVectorizerConfig::default()
        });

    println!("🔧 Summarization config loaded: enabled={}", full_config.summarization.enabled);

    let summarization_config = if full_config.summarization.enabled {
        println!("✅ Summarization ENABLED - creating SummarizationManager");
        Some(full_config.summarization)
    } else {
        println!("❌ Summarization DISABLED - skipping SummarizationManager creation");
        None
    };
    
    // Start GRPC server (same as interactive mode)
    let grpc_config = GrpcConfig::from_env();
    let grpc_vector_store_clone = Arc::clone(&grpc_vector_store);
    let grpc_embedding_manager_clone = Arc::clone(&grpc_embedding_manager);
    let grpc_indexing_progress_clone = Arc::clone(&grpc_indexing_progress);
    
    tokio::spawn(async move {
        if let Err(e) = start_grpc_server(
            grpc_config.server,
            grpc_vector_store_clone,
            grpc_embedding_manager_clone,
            grpc_indexing_progress_clone,
            summarization_config,
        ).await {
            eprintln!("❌ Failed to start GRPC server: {}", e);
        }
    });

    // Initialize and start File Watcher System with shared reference
    let file_watcher_vector_store = Arc::clone(&grpc_vector_store);
    let file_watcher_embedding_manager = Arc::clone(&grpc_embedding_manager);
    let file_watcher_system = Arc::new(Mutex::new(None::<vectorizer::file_watcher::FileWatcherSystem>));

    // Start background indexing and status replication
    let indexing_vector_store = Arc::clone(&grpc_vector_store);
    let indexing_embedding_manager = Arc::clone(&grpc_embedding_manager);
    let indexing_progress = Arc::clone(&grpc_indexing_progress);
    let indexing_file_watcher = Arc::clone(&file_watcher_system);

    tokio::spawn(async move {
        start_background_indexing_with_config(
            indexing_vector_store,
            indexing_embedding_manager,
            indexing_progress,
            indexing_file_watcher,
        ).await;
    });
    
    // Start file watcher system
    let file_watcher_system_clone = Arc::clone(&file_watcher_system);
    tokio::spawn(async move {
        if let Err(e) = start_file_watcher_system_with_grpc(
            file_watcher_vector_store,
            file_watcher_embedding_manager,
            format!("http://{}:{}", host, port),
            file_watcher_system_clone,
        ).await {
            eprintln!("❌ Failed to start File Watcher System: {}", e);
        }
    });

    // In daemon mode, the vzr process continues running to manage GRPC server
    // but we don't wait for shutdown signals - it runs indefinitely
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn run_as_daemon(
    project: PathBuf,
    config: PathBuf,
    host: String,
    port: u16,
    mcp_port: u16,
) {
    let logger = WorkspaceLogger::new();
    logger.info("Starting daemon mode with project");

    // Find MCP server executable
    let mcp_executable = find_executable("vectorizer-mcp-server")
        .expect("vectorizer-mcp-server executable not found. Please build the project first.");

    // Start MCP server as background process
    let mut mcp_child = match TokioCommand::new(&mcp_executable)
        .env("VECTORIZER_SERVER_PORT", &mcp_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            logger.info(&format!("MCP daemon started with PID: {}", child.id().unwrap_or(0)));
            child
        }
        Err(e) => {
            eprintln!("❌ Failed to start MCP daemon: {}", e);
            logger.error(&format!("Failed to start MCP daemon: {}", e));
            std::process::exit(1);
        }
    };

    // Find REST API server executable
    let rest_executable = find_executable("vectorizer-server")
        .expect("vectorizer-server executable not found. Please build the project first.");

    // Start REST API server as background process
    let api_child = match TokioCommand::new(&rest_executable)
        .args(&[
            "--host",
            &host,
            "--port",
            &port.to_string(),
            "--project",
            &project.to_string_lossy(),
            "--config",
            &config.to_string_lossy(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            logger.info(&format!("REST API daemon started with PID: {}", child.id().unwrap_or(0)));
            child
        }
        Err(e) => {
            eprintln!("❌ Failed to start REST API daemon: {}", e);
            logger.error(&format!("Failed to start REST API daemon: {}", e));
            let _ = mcp_child.kill().await;
            std::process::exit(1);
        }
    };

    // Print minimal success message
    println!("✅ Daemon services started successfully!");
    println!("📡 REST API: http://{}:{}", host, port);
    println!("🔧 MCP Server: http://127.0.0.1:{}/sse", mcp_port);
    println!("🔧 GRPC Server: http://127.0.0.1:15003");
    println!("📄 Logs: vectorizer-workspace.log");
    println!("🛑 Use 'vectorizer stop' to stop all services");

    // Initialize GRPC server components with universal GPU detection
    #[cfg(feature = "wgpu-gpu")]
    let grpc_vector_store = Arc::new(VectorStore::new_auto());
    
    #[cfg(not(feature = "wgpu-gpu"))]
    let grpc_vector_store = Arc::new({
        let mut store = VectorStore::new_auto();
        // Load dynamic collections after workspace initialization
        if let Err(e) = store.load_dynamic_collections() {
            eprintln!("⚠️ Failed to load dynamic collections for GRPC server: {}", e);
        }
        store
    });
    let grpc_embedding_manager = Arc::new(Mutex::new({
        let mut manager = EmbeddingManager::new();
        
        // Register all embedding providers
        use vectorizer::embedding::{
            TfIdfEmbedding, Bm25Embedding, SvdEmbedding, 
            BertEmbedding, MiniLmEmbedding, BagOfWordsEmbedding, CharNGramEmbedding
        };
        
        let tfidf = TfIdfEmbedding::new(512);
        let bm25 = Bm25Embedding::new(512);
        let svd = SvdEmbedding::new(512, 512);
        let bert = BertEmbedding::new(512);
        let minilm = MiniLmEmbedding::new(512);
        let bow = BagOfWordsEmbedding::new(512);
        let char_ngram = CharNGramEmbedding::new(512, 3);
        
        manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.register_provider("svd".to_string(), Box::new(svd));
        manager.register_provider("bert".to_string(), Box::new(bert));
        manager.register_provider("minilm".to_string(), Box::new(minilm));
        manager.register_provider("bagofwords".to_string(), Box::new(bow));
        manager.register_provider("charngram".to_string(), Box::new(char_ngram));
        
        manager
    }));
    let grpc_indexing_progress = Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    // Load full configuration including summarization
    let full_config = FullVectorizerConfig::from_yaml_file(&std::path::PathBuf::from("config.yml"))
        .unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to load config.yml: {}, using defaults", e);
            FullVectorizerConfig::default()
        });

    println!("🔧 Summarization config loaded: enabled={}", full_config.summarization.enabled);

    let summarization_config = if full_config.summarization.enabled {
        println!("✅ Summarization ENABLED - creating SummarizationManager");
        Some(full_config.summarization)
    } else {
        println!("❌ Summarization DISABLED - skipping SummarizationManager creation");
        None
    };
    
    // Start GRPC server (same as interactive mode)
    let grpc_config = GrpcConfig::from_env();
    let grpc_vector_store_clone = Arc::clone(&grpc_vector_store);
    let grpc_embedding_manager_clone = Arc::clone(&grpc_embedding_manager);
    let grpc_indexing_progress_clone = Arc::clone(&grpc_indexing_progress);
    
    tokio::spawn(async move {
        if let Err(e) = start_grpc_server(
            grpc_config.server,
            grpc_vector_store_clone,
            grpc_embedding_manager_clone,
            grpc_indexing_progress_clone,
            summarization_config,
        ).await {
            eprintln!("❌ Failed to start GRPC server: {}", e);
        }
    });

    // Initialize and start File Watcher System with shared reference
    let file_watcher_vector_store = Arc::clone(&grpc_vector_store);
    let file_watcher_embedding_manager = Arc::clone(&grpc_embedding_manager);
    let file_watcher_system = Arc::new(Mutex::new(None::<vectorizer::file_watcher::FileWatcherSystem>));

    // Start background indexing and status replication
    let indexing_vector_store = Arc::clone(&grpc_vector_store);
    let indexing_embedding_manager = Arc::clone(&grpc_embedding_manager);
    let indexing_progress = Arc::clone(&grpc_indexing_progress);
    let indexing_file_watcher = Arc::clone(&file_watcher_system);

    tokio::spawn(async move {
        start_background_indexing_with_config(
            indexing_vector_store,
            indexing_embedding_manager,
            indexing_progress,
            indexing_file_watcher,
        ).await;
    });
    
    // Start file watcher system
    let file_watcher_system_clone = Arc::clone(&file_watcher_system);
    tokio::spawn(async move {
        if let Err(e) = start_file_watcher_system_with_grpc(
            file_watcher_vector_store,
            file_watcher_embedding_manager,
            format!("http://{}:{}", host, port),
            file_watcher_system_clone,
        ).await {
            eprintln!("❌ Failed to start File Watcher System: {}", e);
        }
    });

    // In daemon mode, the vzr process continues running to manage GRPC server
    // but we don't wait for shutdown signals - it runs indefinitely
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn stop_servers() {
    println!("🛑 Stopping Vectorizer Servers...");

    let mcp_pids = find_processes("vectorizer-mcp-server");
    let rest_pids = find_processes("vectorizer-server");

    for &pid in &mcp_pids {
        println!("Stopping MCP server (PID: {})", pid);
        let _ = Command::new("kill").arg(&pid.to_string()).status();
    }

    for &pid in &rest_pids {
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

        let service_content = format!(
            r#"[Unit]
Description=Vectorizer Server
After=network.target

[Service]
Type=simple
User={}
ExecStart={} --project ../gov --config config.yml --daemon
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#,
            whoami::username(),
            std::env::current_exe().unwrap().display()
        );

        let service_path = "/etc/systemd/system/vectorizer.service";
        match fs::write(service_path, service_content) {
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
        .args(&["-f", name])
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

/// Create GRPC client for communication with vectorizer-server
async fn create_grpc_client(grpc_endpoint: &str) -> Result<VectorizerServiceClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
    // Convert HTTP endpoint to GRPC endpoint
    let grpc_url = grpc_endpoint.replace("http://", "").replace("https://", "");
    let grpc_endpoint = format!("http://{}", grpc_url);
    
    // Add retry logic and better error handling
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        match VectorizerServiceClient::connect(grpc_endpoint.clone()).await {
            Ok(client) => return Ok(client),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(Box::new(e));
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
    
    Err("Failed to connect to GRPC server after multiple attempts".into())
}

/// Start background indexing for a single project
async fn start_background_indexing_for_project(
    project_path: String,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    _indexing_progress: Arc<Mutex<std::collections::HashMap<String, vectorizer::grpc::vectorizer::IndexingStatus>>>,
) {
    // Wait for servers to be ready with health check
    wait_for_server_ready().await;
    
    // Create a default collection for the project
    let collection_name = "workspace-files";
    
    // Update status to processing
    update_collection_status(&collection_name, "processing", 0.0, None).await;
    
    // Try real indexing
    match do_real_indexing(
        &collection_name, 
        &project_path,
        Arc::clone(&vector_store),
        Arc::clone(&embedding_manager),
    ).await {
        Ok(count) => {
            update_collection_status(&collection_name, "completed", 100.0, Some(count)).await;
            println!("✅ Project '{}' indexed successfully: {} vectors", project_path, count);
        }
        Err(e) => {
            update_collection_status(&collection_name, "failed", 0.0, None).await;
            println!("❌ Project '{}' indexing failed: {}", project_path, e);
        }
    }
}

/// Verify summary collections for a project collection
async fn create_summary_collections_for_project(
    collection_name: &str,
    _project_path: &str,
    vector_store: Arc<VectorStore>,
    _embedding_manager: Arc<Mutex<EmbeddingManager>>,
) {
    //println!("📄 Verifying summary collections for '{}'", collection_name);

    // Summary collection names
    let file_summary_collection = format!("{}_summaries", collection_name);
    let chunk_summary_collection = format!("{}_chunk_summaries", collection_name);

    // Check if summary collections exist
    let file_collection_exists = vector_store.get_collection(&file_summary_collection).is_ok();
    let chunk_collection_exists = vector_store.get_collection(&chunk_summary_collection).is_ok();

    if file_collection_exists {
        if let Ok(collection) = vector_store.get_collection(&file_summary_collection) {
            let count = collection.vector_count();
            //println!("   📊 Contains {} summaries", count);
        }
    } else {
        println!("⚠️ File summary collection not found: {}", file_summary_collection);
    }

    if chunk_collection_exists {
        if let Ok(collection) = vector_store.get_collection(&chunk_summary_collection) {
            let count = collection.vector_count();
        }
    } else {
        println!("⚠️ Chunk summary collection not found: {}", chunk_summary_collection);
    }

    // Note: The actual summary generation happens in DocumentLoader during indexing
    // This function only verifies the collections were created
}

/// Create a summary collection in the vector store
async fn create_summary_collection(
    collection_name: &str,
    vector_store: Arc<VectorStore>,
) -> Result<(), String> {
    use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig};

    let config = CollectionConfig {
        dimension: 512, // Same as main collections
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 64,
            seed: Some(42),
        },
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: Default::default(),
    };

    vector_store.create_collection_with_quantization(collection_name, config)
        .map_err(|e| format!("Failed to create collection '{}': {}", collection_name, e))
}

#[derive(Debug)]
struct CollectionInfo {
    dimension: usize,
    vector_count: usize,
}

/// Check actual collections in the vector store via GRPC
async fn check_actual_collections() -> Result<std::collections::HashMap<String, CollectionInfo>, Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use tonic::transport::Channel;
    use vectorizer::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;
    use vectorizer::grpc::vectorizer::Empty;
    
    // Connect to GRPC server
    let channel = Channel::from_static("http://127.0.0.1:15003")
        .connect()
        .await?;
    
    let mut client = VectorizerServiceClient::new(channel);
    
    // List collections - this returns full CollectionInfo for each collection
    let request = tonic::Request::new(Empty {});
    let response = client.list_collections(request).await?;
    let collections_list = response.into_inner();
    
    let mut collection_map = HashMap::new();
    
    // The list_collections already returns CollectionInfo with all metadata
    for collection_info in collections_list.collections {
        collection_map.insert(
            collection_info.name.clone(),
            CollectionInfo {
                dimension: collection_info.dimension as usize,
                vector_count: collection_info.vector_count as usize,
            }
        );
    }
    
    Ok(collection_map)
}
