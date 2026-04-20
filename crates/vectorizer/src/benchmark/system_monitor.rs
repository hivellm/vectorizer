//! System Monitoring Utilities
//!
//! Provides utilities for monitoring system resources during benchmarks
//! including CPU usage, memory consumption, and disk I/O.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// System monitor for tracking resource usage during benchmarks
pub struct SystemMonitor {
    is_monitoring: Arc<AtomicBool>,
    cpu_usage: Arc<AtomicU64>,
    memory_usage: Arc<AtomicU64>,
    peak_memory: Arc<AtomicU64>,
    start_time: Instant,
    monitoring_thread: Option<thread::JoinHandle<()>>,
}

/// System resource snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: String,
    /// CPU usage percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Peak memory usage in MB
    pub peak_memory_mb: f64,
    /// Available memory in MB
    pub available_memory_mb: f64,
    /// Total memory in MB
    pub total_memory_mb: f64,
    /// Process memory usage in MB
    pub process_memory_mb: f64,
    /// Number of threads
    pub thread_count: usize,
    /// Load average (if available)
    pub load_average: Option<f64>,
}

impl SystemMonitor {
    /// Create new system monitor
    pub fn new() -> Self {
        Self {
            is_monitoring: Arc::new(AtomicBool::new(false)),
            cpu_usage: Arc::new(AtomicU64::new(0)),
            memory_usage: Arc::new(AtomicU64::new(0)),
            peak_memory: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            monitoring_thread: None,
        }
    }

    /// Start monitoring system resources
    pub fn start_monitoring(&mut self, interval_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_monitoring.load(Ordering::Relaxed) {
            return Err("System monitor is already running".into());
        }

        self.is_monitoring.store(true, Ordering::Relaxed);
        self.start_time = Instant::now();

        let is_monitoring = Arc::clone(&self.is_monitoring);
        let cpu_usage = Arc::clone(&self.cpu_usage);
        let memory_usage = Arc::clone(&self.memory_usage);
        let peak_memory = Arc::clone(&self.peak_memory);

        let handle = thread::spawn(move || {
            let mut last_cpu_time = get_cpu_time();
            let mut last_check = Instant::now();

            while is_monitoring.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(interval_ms));

                // Update CPU usage
                let current_cpu_time = get_cpu_time();
                let current_time = Instant::now();
                let elapsed = current_time.duration_since(last_check).as_secs_f64();

                if elapsed > 0.0 {
                    let cpu_delta = current_cpu_time - last_cpu_time;
                    let cpu_percent = (cpu_delta as f64 / elapsed) * 100.0;
                    cpu_usage.store((cpu_percent * 1000.0) as u64, Ordering::Relaxed);

                    last_cpu_time = current_cpu_time;
                    last_check = current_time;
                }

                // Update memory usage
                let current_memory = get_memory_usage();
                memory_usage.store(current_memory, Ordering::Relaxed);

                // Update peak memory
                let current_peak = peak_memory.load(Ordering::Relaxed);
                if current_memory > current_peak {
                    peak_memory.store(current_memory, Ordering::Relaxed);
                }
            }
        });

        self.monitoring_thread = Some(handle);
        Ok(())
    }

    /// Stop monitoring system resources
    pub fn stop_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_monitoring.load(Ordering::Relaxed) {
            return Err("System monitor is not running".into());
        }

        self.is_monitoring.store(false, Ordering::Relaxed);

        if let Some(handle) = self.monitoring_thread.take() {
            handle
                .join()
                .map_err(|_| "Failed to join monitoring thread")?;
        }

        Ok(())
    }

    /// Get current system snapshot
    pub fn get_snapshot(&self) -> SystemSnapshot {
        let cpu_usage_raw = self.cpu_usage.load(Ordering::Relaxed);
        let memory_usage_raw = self.memory_usage.load(Ordering::Relaxed);
        let peak_memory_raw = self.peak_memory.load(Ordering::Relaxed);

        SystemSnapshot {
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            cpu_usage_percent: cpu_usage_raw as f64 / 1000.0,
            memory_usage_mb: memory_usage_raw as f64 / 1_048_576.0,
            peak_memory_mb: peak_memory_raw as f64 / 1_048_576.0,
            available_memory_mb: get_available_memory() as f64 / 1_048_576.0,
            total_memory_mb: get_total_memory() as f64 / 1_048_576.0,
            process_memory_mb: get_process_memory() as f64 / 1_048_576.0,
            thread_count: get_thread_count(),
            load_average: get_load_average(),
        }
    }

    /// Get current CPU usage percentage
    pub fn get_cpu_usage(&self) -> f64 {
        self.cpu_usage.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// Get current memory usage in MB
    pub fn get_memory_usage(&self) -> f64 {
        self.memory_usage.load(Ordering::Relaxed) as f64 / 1_048_576.0
    }

    /// Get peak memory usage in MB
    pub fn get_peak_memory(&self) -> f64 {
        self.peak_memory.load(Ordering::Relaxed) as f64 / 1_048_576.0
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.is_monitoring.load(Ordering::Relaxed)
    }

    /// Get monitoring duration
    pub fn get_duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Drop for SystemMonitor {
    fn drop(&mut self) {
        let _ = self.stop_monitoring();
    }
}

// Platform-specific system information functions

#[cfg(target_os = "linux")]
mod linux {
    use std::fs;

    pub fn get_cpu_time() -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            if let Some(line) = content.lines().next() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 8 {
                    let user: u64 = fields[1].parse().unwrap_or(0);
                    let nice: u64 = fields[2].parse().unwrap_or(0);
                    let system: u64 = fields[3].parse().unwrap_or(0);
                    let idle: u64 = fields[4].parse().unwrap_or(0);
                    let iowait: u64 = fields[5].parse().unwrap_or(0);
                    let irq: u64 = fields[6].parse().unwrap_or(0);
                    let softirq: u64 = fields[7].parse().unwrap_or(0);
                    return user + nice + system + idle + iowait + irq + softirq;
                }
            }
        }
        0
    }

    pub fn get_memory_usage() -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        return fields[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        0
    }

    pub fn get_available_memory() -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        return fields[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        0
    }

    pub fn get_total_memory() -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        return fields[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        0
    }

    pub fn get_process_memory() -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/self/status") {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        return fields[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        0
    }

    pub fn get_thread_count() -> usize {
        if let Ok(content) = fs::read_to_string("/proc/self/status") {
            for line in content.lines() {
                if line.starts_with("Threads:") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        return fields[1].parse().unwrap_or(0);
                    }
                }
            }
        }
        0
    }

    pub fn get_load_average() -> Option<f64> {
        if let Ok(content) = fs::read_to_string("/proc/loadavg") {
            if let Some(line) = content.lines().next() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if !fields.is_empty() {
                    return fields[0].parse().ok();
                }
            }
        }
        None
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::process::Command;

    pub fn get_cpu_time() -> u64 {
        // macOS doesn't provide easy access to CPU time via /proc
        // This is a simplified implementation
        0
    }

    pub fn get_memory_usage() -> u64 {
        if let Ok(output) = Command::new("vm_stat").output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.starts_with("Pages free:") {
                        let fields: Vec<&str> = line.split_whitespace().collect();
                        if fields.len() >= 3 {
                            if let Ok(pages) = fields[2].parse::<u64>() {
                                return pages * 4096; // Convert pages to bytes
                            }
                        }
                    }
                }
            }
        }
        0
    }

    pub fn get_available_memory() -> u64 {
        get_memory_usage()
    }

    pub fn get_total_memory() -> u64 {
        if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.memsize"]).output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.trim().parse().unwrap_or(0);
            }
        }
        0
    }

    pub fn get_process_memory() -> u64 {
        if let Ok(output) = Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Ok(kb) = output_str.trim().parse::<u64>() {
                    return kb * 1024; // Convert KB to bytes
                }
            }
        }
        0
    }

    pub fn get_thread_count() -> usize {
        if let Ok(output) = Command::new("ps")
            .args(&["-M", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.lines().count().saturating_sub(1); // Subtract header line
            }
        }
        0
    }

    pub fn get_load_average() -> Option<f64> {
        if let Ok(output) = Command::new("uptime").output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                // Parse load average from uptime output
                if let Some(load_avg_part) = output_str.split("load averages:").nth(1) {
                    if let Some(first_value) = load_avg_part.split_whitespace().next() {
                        return first_value.parse().ok();
                    }
                }
            }
        }
        None
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::process::Command;

    pub fn get_cpu_time() -> u64 {
        // Windows implementation would use WMI or Performance Counters
        // This is a placeholder
        0
    }

    pub fn get_memory_usage() -> u64 {
        if let Ok(output) = Command::new("wmic")
            .args(&["computersystem", "get", "TotalPhysicalMemory", "/value"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.starts_with("TotalPhysicalMemory=") {
                        if let Ok(bytes) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                            return bytes;
                        }
                    }
                }
            }
        }
        0
    }

    pub fn get_available_memory() -> u64 {
        // Windows implementation would use GlobalMemoryStatusEx
        0
    }

    pub fn get_total_memory() -> u64 {
        get_memory_usage()
    }

    pub fn get_process_memory() -> u64 {
        // Windows implementation would use GetProcessMemoryInfo
        0
    }

    pub fn get_thread_count() -> usize {
        // Windows implementation would use GetProcessHandleCount
        0
    }

    pub fn get_load_average() -> Option<f64> {
        None
    }
}

// Platform-specific function calls

#[cfg(target_os = "linux")]
fn get_cpu_time() -> u64 {
    linux::get_cpu_time()
}

#[cfg(target_os = "linux")]
fn get_memory_usage() -> u64 {
    linux::get_memory_usage()
}

#[cfg(target_os = "linux")]
fn get_available_memory() -> u64 {
    linux::get_available_memory()
}

#[cfg(target_os = "linux")]
fn get_total_memory() -> u64 {
    linux::get_total_memory()
}

#[cfg(target_os = "linux")]
fn get_process_memory() -> u64 {
    linux::get_process_memory()
}

#[cfg(target_os = "linux")]
fn get_thread_count() -> usize {
    linux::get_thread_count()
}

#[cfg(target_os = "linux")]
fn get_load_average() -> Option<f64> {
    linux::get_load_average()
}

#[cfg(target_os = "macos")]
fn get_cpu_time() -> u64 {
    macos::get_cpu_time()
}

#[cfg(target_os = "macos")]
fn get_memory_usage() -> u64 {
    macos::get_memory_usage()
}

#[cfg(target_os = "macos")]
fn get_available_memory() -> u64 {
    macos::get_available_memory()
}

#[cfg(target_os = "macos")]
fn get_total_memory() -> u64 {
    macos::get_total_memory()
}

#[cfg(target_os = "macos")]
fn get_process_memory() -> u64 {
    macos::get_process_memory()
}

#[cfg(target_os = "macos")]
fn get_thread_count() -> usize {
    macos::get_thread_count()
}

#[cfg(target_os = "macos")]
fn get_load_average() -> Option<f64> {
    macos::get_load_average()
}

#[cfg(target_os = "windows")]
fn get_cpu_time() -> u64 {
    windows::get_cpu_time()
}

#[cfg(target_os = "windows")]
fn get_memory_usage() -> u64 {
    windows::get_memory_usage()
}

#[cfg(target_os = "windows")]
fn get_available_memory() -> u64 {
    windows::get_available_memory()
}

#[cfg(target_os = "windows")]
fn get_total_memory() -> u64 {
    windows::get_total_memory()
}

#[cfg(target_os = "windows")]
fn get_process_memory() -> u64 {
    windows::get_process_memory()
}

#[cfg(target_os = "windows")]
fn get_thread_count() -> usize {
    windows::get_thread_count()
}

#[cfg(target_os = "windows")]
fn get_load_average() -> Option<f64> {
    windows::get_load_average()
}

// Fallback implementations for unsupported platforms
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_cpu_time() -> u64 {
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_memory_usage() -> u64 {
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_available_memory() -> u64 {
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_total_memory() -> u64 {
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_process_memory() -> u64 {
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_thread_count() -> usize {
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_load_average() -> Option<f64> {
    None
}
