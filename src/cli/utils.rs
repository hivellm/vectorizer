//! CLI utility functions
//! 
//! Common utilities for CLI operations

use crate::error::{Result, VectorizerError};
use std::path::PathBuf;
use tracing::{info, warn};

/// Utility functions for CLI operations
pub struct CliUtils;

impl CliUtils {
    /// Ensure directory exists, create if it doesn't
    pub fn ensure_directory(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            info!("Creating directory: {:?}", path);
            std::fs::create_dir_all(path)
                .map_err(|e| VectorizerError::IoError(e))?;
        } else if !path.is_dir() {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("Path exists but is not a directory: {:?}", path),
            });
        }
        Ok(())
    }

    /// Check if file exists and is readable
    pub fn check_file_readable(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(VectorizerError::NotFound(format!("File not found: {:?}", path)));
        }

        if !path.is_file() {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("Path is not a file: {:?}", path),
            });
        }

        // Try to read the file to check permissions
        std::fs::read(path)
            .map_err(|e| VectorizerError::IoError(e))?;

        Ok(())
    }

    /// Check if file exists and is writable
    pub fn check_file_writable(path: &PathBuf) -> Result<()> {
        if path.exists() {
            if !path.is_file() {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!("Path exists but is not a file: {:?}", path),
                });
            }

            // Try to open file for writing to check permissions
            std::fs::OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(|e| VectorizerError::IoError(e))?;
        } else {
            // Check if parent directory is writable
            if let Some(parent) = path.parent() {
                Self::ensure_directory(&parent.to_path_buf())?;
            }
        }

        Ok(())
    }

    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: u64 = 1024;

        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD as f64;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format duration as human-readable string
    pub fn format_duration(seconds: u64) -> String {
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else if seconds < 86400 {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        } else {
            format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
        }
    }

    /// Prompt user for confirmation
    pub fn confirm_action(message: &str, default: bool) -> Result<bool> {
        let default_text = if default { "Y/n" } else { "y/N" };
        print!("{} [{}]: ", message, default_text);
        
        std::io::Write::flush(&mut std::io::stdout())
            .map_err(|e| VectorizerError::IoError(e))?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| VectorizerError::IoError(e))?;

        let input = input.trim().to_lowercase();
        
        match input.as_str() {
            "" => Ok(default),
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => {
                warn!("Invalid input: {}. Using default: {}", input, default);
                Ok(default)
            }
        }
    }

    /// Get system information
    pub fn get_system_info() -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            vectorizer_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Check system requirements
    pub fn check_system_requirements() -> Result<()> {
        let info = Self::get_system_info();
        
        info!("System Information:");
        info!("  OS: {}", info.os);
        info!("  Architecture: {}", info.arch);
        info!("  Rust Version: {}", info.rust_version);
        info!("  Vectorizer Version: {}", info.vectorizer_version);

        // Check available memory (basic check)
        if let Ok(mem_info) = sys_info::mem_info() {
            let total_gb = mem_info.total / (1024 * 1024 * 1024);
            if total_gb < 1 {
                warn!("System has less than 1GB RAM. Performance may be limited.");
            }
            info!("  Total Memory: {} GB", total_gb);
        }

        // Check disk space
        if let Ok(disk_info) = sys_info::disk_info() {
            let free_gb = disk_info.free / (1024 * 1024 * 1024);
            if free_gb < 1 {
                warn!("Less than 1GB disk space available. Consider freeing up space.");
            }
            info!("  Free Disk Space: {} GB", free_gb);
        }

        Ok(())
    }

    /// Validate port number
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Port cannot be 0".to_string(),
            });
        }

        if port < 1024 && cfg!(unix) {
            warn!("Port {} is below 1024. You may need root privileges to bind to this port.", port);
        }

        Ok(())
    }

    /// Validate host address
    pub fn validate_host(host: &str) -> Result<()> {
        if host.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Host cannot be empty".to_string(),
            });
        }

        // Basic validation for IP addresses and hostnames
        if host == "0.0.0.0" {
            warn!("Binding to 0.0.0.0 will make the server accessible from all network interfaces.");
        }

        Ok(())
    }

    /// Generate secure random string
    pub fn generate_secure_string(length: usize) -> Result<String> {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        
        let mut rng = rand::thread_rng();
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Ok(password)
    }

    /// Check if running as root/admin
    pub fn is_elevated() -> bool {
        #[cfg(unix)]
        {
            // Simple check - in production, you'd want to use a proper crate
            std::env::var("USER").unwrap_or_default() == "root"
        }
        #[cfg(windows)]
        {
            // Windows elevation check would go here
            false
        }
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }

    /// Get current working directory
    pub fn get_current_dir() -> Result<PathBuf> {
        std::env::current_dir()
            .map_err(|e| VectorizerError::IoError(e))
    }

    /// Set working directory
    pub fn set_current_dir(path: &PathBuf) -> Result<()> {
        std::env::set_current_dir(path)
            .map_err(|e| VectorizerError::IoError(e))
    }
}

/// System information structure
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: String,
    pub vectorizer_version: String,
}

/// Progress bar for long operations
pub struct ProgressBar {
    total: u64,
    current: u64,
    width: usize,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: 0,
            width: 50,
        }
    }

    /// Update progress
    pub fn update(&mut self, current: u64) {
        self.current = current;
        self.display();
    }

    /// Increment progress
    pub fn increment(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.display();
    }

    /// Display the progress bar
    fn display(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };

        let filled = (percentage / 100.0 * self.width as f64) as usize;
        let empty = self.width - filled;

        print!("\r[");
        for _ in 0..filled {
            print!("=");
        }
        for _ in 0..empty {
            print!("-");
        }
        print!("] {:.1}% ({}/{})", percentage, self.current, self.total);
        
        if self.current >= self.total {
            println!();
        } else {
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }

    /// Finish the progress bar
    pub fn finish(&mut self) {
        self.current = self.total;
        self.display();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(CliUtils::format_bytes(0), "0 B");
        assert_eq!(CliUtils::format_bytes(1024), "1.0 KB");
        assert_eq!(CliUtils::format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(CliUtils::format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(CliUtils::format_duration(30), "30s");
        assert_eq!(CliUtils::format_duration(90), "1m 30s");
        assert_eq!(CliUtils::format_duration(3661), "1h 1m");
        assert_eq!(CliUtils::format_duration(90061), "1d 1h");
    }

    #[test]
    fn test_ensure_directory() {
        let temp_dir = tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_directory");
        
        // Directory doesn't exist initially
        assert!(!new_dir.exists());
        
        // Create directory
        CliUtils::ensure_directory(&new_dir).unwrap();
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
        
        // Creating again should not fail
        CliUtils::ensure_directory(&new_dir).unwrap();
    }

    #[test]
    fn test_validate_port() {
        assert!(CliUtils::validate_port(8080).is_ok());
        assert!(CliUtils::validate_port(15001).is_ok());
        assert!(CliUtils::validate_port(0).is_err());
    }

    #[test]
    fn test_validate_host() {
        assert!(CliUtils::validate_host("127.0.0.1").is_ok());
        assert!(CliUtils::validate_host("localhost").is_ok());
        assert!(CliUtils::validate_host("0.0.0.0").is_ok());
        assert!(CliUtils::validate_host("").is_err());
    }

    #[test]
    fn test_generate_secure_string() {
        let password = CliUtils::generate_secure_string(32).unwrap();
        assert_eq!(password.len(), 32);
        
        // Should contain only alphanumeric characters
        for ch in password.chars() {
            assert!(ch.is_ascii_alphanumeric());
        }
    }

    #[test]
    fn test_progress_bar() {
        let mut pb = ProgressBar::new(100);
        
        pb.update(50);
        pb.increment();
        pb.finish();
        
        // Test is mainly for compilation - actual output testing would be complex
        assert_eq!(pb.current, 100);
        assert_eq!(pb.total, 100);
    }
}
