//! Centralized logging system for Vectorizer
//! 
//! This module provides a unified logging system that:
//! - Stores all logs in the `.logs` directory
//! - Includes date in log file names for better organization
//! - Automatically cleans up logs older than 1 day
//! - Provides consistent formatting across all services

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the centralized logging system
pub fn init_logging(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    let logs_dir = PathBuf::from(".logs");
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)?;
        info!("Created logs directory: {:?}", logs_dir);
    }

    // Clean up old logs before initializing
    cleanup_old_logs(&logs_dir)?;

    // Generate log filename with date using the standard format
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let log_filename = format!("{}-{}.log", service_name, date_str);
    let log_path = logs_dir.join(log_filename);

    // Create log file
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Initialize tracing with both console and file output
    let result = tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}={}", service_name, "info").into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(move || log_file.try_clone().expect("Failed to clone log file"))
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true),
        )
        .try_init();
    
    if let Err(e) = result {
        eprintln!("Failed to initialize tracing: {}", e);
        return Err(format!("Failed to initialize tracing: {}", e).into());
    }

    info!("Logging initialized for {} - Log file: {:?}", service_name, log_path);
    Ok(())
}


/// Clean up log files older than 1 day
fn cleanup_old_logs(logs_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cutoff_time = SystemTime::now() - Duration::from_secs(24 * 60 * 60); // 1 day ago
    
    if !logs_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(logs_dir)?;
    let mut cleaned_count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Only process .log files
        if path.extension().map_or(false, |ext| ext == "log") {
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        if let Err(e) = fs::remove_file(&path) {
                            error!("Failed to remove old log file {:?}: {}", path, e);
                        } else {
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }
    }

    if cleaned_count > 0 {
        info!("Cleaned up {} old log files", cleaned_count);
    }

    Ok(())
}

/// Clean up old logs manually (can be called periodically)
pub fn cleanup_old_logs_manual() -> Result<(), Box<dyn std::error::Error>> {
    let logs_dir = PathBuf::from(".logs");
    cleanup_old_logs(&logs_dir)
}

/// Get the current log directory path
pub fn get_logs_dir() -> PathBuf {
    PathBuf::from(".logs")
}

/// Get the log file path for a specific service and date
pub fn get_log_file_path(service_name: &str, date: Option<DateTime<Local>>) -> PathBuf {
    let logs_dir = get_logs_dir();
    let date_str = match date {
        Some(dt) => dt.format("%Y-%m-%d").to_string(),
        None => Local::now().format("%Y-%m-%d").to_string(),
    };
    let filename = format!("{}-{}.log", service_name, date_str);
    logs_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;


    #[test]
    fn test_get_log_file_path() {
        let path = get_log_file_path("test-service", None);
        assert!(path.to_string_lossy().contains("test-service"));
        assert!(path.to_string_lossy().contains(".log"));
        assert!(path.to_string_lossy().contains(".logs"));
    }

    #[test]
    fn test_cleanup_old_logs() {
        // This test creates a temporary log file and verifies cleanup
        let logs_dir = get_logs_dir();
        fs::create_dir_all(&logs_dir).unwrap();
        
        // Create a fake old log file
        let old_log = logs_dir.join("old-test-2020-01-01.log");
        fs::write(&old_log, "old log content").unwrap();
        
        // Run cleanup
        cleanup_old_logs(&logs_dir).unwrap();
        
        // The old file should be removed
        assert!(!old_log.exists());
        
        // Clean up
        let _ = fs::remove_file(old_log);
    }
}
