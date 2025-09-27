//! Process Manager - Shared utilities for managing duplicate processes
//!
//! This module provides utilities to check for and kill existing vectorizer processes
//! to prevent conflicts when starting new instances.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use std::io::Write;

/// Structured logging for process management operations
pub struct ProcessLogger {
    log_file: PathBuf,
}

impl ProcessLogger {
    pub fn new(server_type: &str) -> Self {
        let log_file = PathBuf::from(format!("vectorizer-{}-process.log", server_type));
        Self { log_file }
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_entry = format!("[{}] {}: {}\n", timestamp, level, message);

        // Write to log file
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        // Also print to stderr for immediate feedback
        eprintln!("[{}] {}: {}", timestamp, level, message);
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn warn(&self, message: &str) {
        self.log("WARN", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}

/// PID file management for process tracking
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn new(server_type: &str) -> Self {
        let pid_file = format!("vectorizer-{}-{}.pid", server_type, std::process::id());
        Self {
            path: PathBuf::from(pid_file),
        }
    }

    pub fn create(&self, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        let content = format!("{}\n", pid);
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn read_pid(&self) -> Result<Option<u32>, Box<dyn std::error::Error>> {
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

    pub fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Check if a process is actually running by PID
pub fn is_process_running(pid: u32) -> bool {
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

/// Check for processes by executable name
pub fn check_processes_by_name(logger: &ProcessLogger, server_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use tasklist command
        let output = Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}.exe", server_name), "/FI", "STATUS eq RUNNING"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // tasklist returns header lines, so we check if there are actual process entries
                let lines: Vec<&str> = stdout.lines().collect();
                if lines.len() > 3 { // Header + separator + at least one process
                    logger.warn(&format!("Found existing {}.exe processes", server_name));
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
                    if line.contains(server_name) && !line.contains("grep") {
                        logger.warn(&format!("Found existing {} processes", server_name));
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check for processes using specific ports
pub fn check_processes_by_ports(logger: &ProcessLogger, ports: &[u16]) -> bool {
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
                    for &port in ports {
                        if line.contains(&format!(":{}", port)) {
                            logger.warn(&format!("Found processes listening on port {}", port));
                            return true;
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems: Use lsof
        let port_args: Vec<String> = ports.iter()
            .map(|port| format!(":{}", port))
            .collect();
        
        let mut args = vec!["-i"];
        args.extend(port_args.iter().map(|s| s.as_str()));

        let output = Command::new("lsof")
            .args(&args)
            .output();

        if let Ok(output) = output {
            if output.status.success() && !output.stdout.is_empty() {
                for &port in ports {
                    logger.warn(&format!("Found processes listening on port {}", port));
                }
                return true;
            }
        }
    }

    false
}

/// Kill processes by executable name
pub fn kill_processes_by_name(logger: &ProcessLogger, server_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Use taskkill command
        let output = Command::new("taskkill")
            .args(&["/F", "/IM", &format!("{}.exe", server_name)])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                logger.info(&format!("Killed {}.exe processes", server_name));
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
            .args(&["-f", server_name])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                logger.info(&format!("Killed {} processes using pkill", server_name));
            } else {
                // pkill returns non-zero if no processes found, which is fine
                logger.info(&format!("No {} processes found to kill", server_name));
            }
        }

        // Fallback: use killall if pkill is not available
        let killall_output = Command::new("killall")
            .args(&["-9", server_name])
            .output();

        if let Ok(killall_output) = killall_output {
            if killall_output.status.success() {
                logger.info(&format!("Killed {} processes using killall", server_name));
            }
        }
    }

    Ok(())
}

/// Kill processes using specific ports
pub fn kill_processes_by_ports(logger: &ProcessLogger, ports: &[u16]) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Find PIDs using netstat and kill with taskkill
        let netstat_output = Command::new("netstat")
            .args(&["-ano"])
            .output();

        if let Ok(netstat_output) = netstat_output {
            if netstat_output.status.success() {
                let stdout = String::from_utf8_lossy(&netstat_output.stdout);
                let mut pids_to_kill = HashSet::new();

                for line in stdout.lines() {
                    for &port in ports {
                        if line.contains(&format!(":{}", port)) {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 5 {
                                if let Ok(pid) = parts[4].parse::<u32>() {
                                    pids_to_kill.insert(pid);
                                }
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
        let port_args: Vec<String> = ports.iter()
            .map(|port| format!(":{}", port))
            .collect();
        
        let mut args = vec!["-ti"];
        args.extend(port_args.iter().map(|s| s.as_str()));

        let lsof_output = Command::new("lsof")
            .args(&args)
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

/// Enhanced process checking with PID file support
pub fn check_existing_processes_enhanced(
    logger: &ProcessLogger, 
    server_name: &str, 
    ports: &[u16]
) -> bool {
    logger.info(&format!("Enhanced check for existing {} processes...", server_name));

    // Check PID files first
    let pid_file = PidFile::new(server_name);

    // Check PID file
    if let Ok(Some(pid)) = pid_file.read_pid() {
        if is_process_running(pid) {
            logger.warn(&format!("Found running {} with PID {}", server_name, pid));
            return true;
        } else {
            logger.info(&format!("Found stale {} PID file, cleaning up...", server_name));
            let _ = pid_file.cleanup();
        }
    }

    // Check for processes by name
    if check_processes_by_name(logger, server_name) {
        return true;
    }

    // Check for processes using our ports
    if check_processes_by_ports(logger, ports) {
        return true;
    }

    logger.info(&format!("No existing {} processes found", server_name));
    false
}

/// Kill existing processes
pub fn kill_existing_processes(
    logger: &ProcessLogger, 
    server_name: &str, 
    ports: &[u16]
) -> Result<(), Box<dyn std::error::Error>> {
    logger.info(&format!("Killing existing {} processes...", server_name));

    // First, try to kill processes by name
    kill_processes_by_name(logger, server_name)?;

    // Then kill processes using our ports
    kill_processes_by_ports(logger, ports)?;

    // Wait a moment for processes to terminate
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Verify that processes are actually terminated
    if check_existing_processes_enhanced(logger, server_name, ports) {
        logger.warn("Some processes may still be running after kill attempt");
        return Err("Failed to completely terminate existing processes".into());
    }

    logger.info(&format!("Existing {} processes terminated successfully", server_name));
    Ok(())
}

/// Initialize process management for a server
pub fn initialize_process_management(server_name: &str, ports: &[u16]) -> Result<ProcessLogger, Box<dyn std::error::Error>> {
    let logger = ProcessLogger::new(server_name);
    
    logger.info(&format!("Initializing process management for {}", server_name));
    
    // Check for existing processes and kill them if found
    if check_existing_processes_enhanced(&logger, server_name, ports) {
        logger.warn(&format!("Found existing {} processes. Killing them to prevent conflicts...", server_name));
        if let Err(e) = kill_existing_processes(&logger, server_name, ports) {
            logger.error(&format!("Failed to kill existing processes: {}", e));
            eprintln!("❌ Failed to kill existing {} processes: {}", server_name, e);
            eprintln!("💡 Please manually kill existing processes");
            std::process::exit(1);
        }
    }

    // Create PID file for current process
    let pid_file = PidFile::new(server_name);
    if let Err(e) = pid_file.create(std::process::id()) {
        logger.warn(&format!("Failed to create PID file: {}", e));
    }

    logger.info(&format!("Process management initialized for {}", server_name));
    Ok(logger)
}

/// Cleanup process management on exit
pub fn cleanup_process_management(server_name: &str) {
    let logger = ProcessLogger::new(server_name);
    let pid_file = PidFile::new(server_name);
    
    if let Err(e) = pid_file.cleanup() {
        logger.warn(&format!("Failed to cleanup PID file: {}", e));
    }
    
    logger.info(&format!("Process management cleanup completed for {}", server_name));
}
