//! Runtime Guardrails - System Protection and Stability
//!
//! This module implements runtime safety checks to prevent system crashes,
//! BSODs, and resource exhaustion. It monitors system resources and enforces
//! safe limits.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use sysinfo::System;
use tracing::{error, info, warn};

/// Maximum safe memory usage (percentage)
const MAX_MEMORY_PERCENT: f32 = 75.0;

/// Maximum safe CPU usage (percentage)
const MAX_CPU_PERCENT: f32 = 90.0;

/// Minimum free memory required (MB)
const MIN_FREE_MEMORY_MB: u64 = 512;

/// Maximum concurrent operations
const MAX_CONCURRENT_OPS: usize = 4;

/// Guardrail violation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    MemoryExhaustion,
    CpuOverload,
    DiskIoSaturation,
    TooManyConcurrentOps,
    GpuDriverIssue,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryExhaustion => write!(f, "Memory Exhaustion"),
            Self::CpuOverload => write!(f, "CPU Overload"),
            Self::DiskIoSaturation => write!(f, "Disk I/O Saturation"),
            Self::TooManyConcurrentOps => write!(f, "Too Many Concurrent Operations"),
            Self::GpuDriverIssue => write!(f, "GPU Driver Issue"),
        }
    }
}

/// System guardrails configuration
#[derive(Debug, Clone)]
pub struct GuardrailsConfig {
    /// Enable guardrails
    pub enabled: bool,

    /// Maximum memory usage percentage
    pub max_memory_percent: f32,

    /// Maximum CPU usage percentage
    pub max_cpu_percent: f32,

    /// Minimum free memory (MB)
    pub min_free_memory_mb: u64,

    /// Maximum concurrent operations
    pub max_concurrent_ops: usize,

    /// Enable automatic resource throttling
    pub auto_throttle: bool,

    /// Enable Windows-specific protections
    pub windows_protection: bool,
}

impl Default for GuardrailsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_memory_percent: MAX_MEMORY_PERCENT,
            max_cpu_percent: MAX_CPU_PERCENT,
            min_free_memory_mb: MIN_FREE_MEMORY_MB,
            max_concurrent_ops: MAX_CONCURRENT_OPS,
            auto_throttle: true,
            windows_protection: cfg!(target_os = "windows"),
        }
    }
}

/// Runtime guardrails system
pub struct Guardrails {
    config: GuardrailsConfig,
    system: Arc<parking_lot::Mutex<System>>,
    active_operations: Arc<AtomicUsize>,
    throttled: Arc<AtomicBool>,
    last_check: Arc<parking_lot::Mutex<Instant>>,
    violations: Arc<parking_lot::Mutex<Vec<(Instant, ViolationType)>>>,
}

impl Guardrails {
    /// Create new guardrails system
    pub fn new(config: GuardrailsConfig) -> Self {
        info!("🛡️  Initializing System Guardrails...");

        if !config.enabled {
            warn!("⚠️  Guardrails are DISABLED - system protection unavailable");
        }

        if cfg!(target_os = "windows") && config.windows_protection {
            info!("🪟 Windows-specific protections ENABLED");
        }

        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config,
            system: Arc::new(parking_lot::Mutex::new(system)),
            active_operations: Arc::new(AtomicUsize::new(0)),
            throttled: Arc::new(AtomicBool::new(false)),
            last_check: Arc::new(parking_lot::Mutex::new(Instant::now())),
            violations: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    /// Check if operation is safe to proceed
    pub fn check_safe(&self) -> Result<(), ViolationType> {
        if !self.config.enabled {
            return Ok(());
        }

        // Rate limit checks (max once per second)
        {
            let mut last_check = self.last_check.lock();
            if last_check.elapsed() < Duration::from_secs(1) {
                // Use cached throttle state
                if self.throttled.load(Ordering::Relaxed) {
                    return Err(ViolationType::CpuOverload);
                }
                return Ok(());
            }
            *last_check = Instant::now();
        }

        // Refresh system info
        let mut system = self.system.lock();
        system.refresh_memory();
        system.refresh_cpu_all();

        // Check concurrent operations limit
        let active_ops = self.active_operations.load(Ordering::Relaxed);
        if active_ops >= self.config.max_concurrent_ops {
            self.record_violation(ViolationType::TooManyConcurrentOps);
            return Err(ViolationType::TooManyConcurrentOps);
        }

        // Check memory usage
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_percent = (used_memory as f32 / total_memory as f32) * 100.0;

        if memory_percent > self.config.max_memory_percent {
            warn!(
                "⚠️  High memory usage: {:.1}% (limit: {:.1}%)",
                memory_percent, self.config.max_memory_percent
            );
            self.record_violation(ViolationType::MemoryExhaustion);

            if self.config.auto_throttle {
                self.throttled.store(true, Ordering::Relaxed);
            }

            return Err(ViolationType::MemoryExhaustion);
        }

        // Check minimum free memory
        let available_memory_mb = system.available_memory() / (1024 * 1024);
        if available_memory_mb < self.config.min_free_memory_mb {
            warn!(
                "⚠️  Low free memory: {} MB (minimum: {} MB)",
                available_memory_mb, self.config.min_free_memory_mb
            );
            self.record_violation(ViolationType::MemoryExhaustion);
            return Err(ViolationType::MemoryExhaustion);
        }

        // Check CPU usage
        let cpu_usage = system.global_cpu_usage();
        if cpu_usage > self.config.max_cpu_percent {
            warn!(
                "⚠️  High CPU usage: {:.1}% (limit: {:.1}%)",
                cpu_usage, self.config.max_cpu_percent
            );
            self.record_violation(ViolationType::CpuOverload);

            if self.config.auto_throttle {
                self.throttled.store(true, Ordering::Relaxed);
            }

            return Err(ViolationType::CpuOverload);
        }

        // All checks passed - clear throttle
        self.throttled.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Acquire operation permit (increases active operation count)
    pub fn acquire_permit(&self) -> Result<OperationPermit, ViolationType> {
        self.check_safe()?;

        let count = self.active_operations.fetch_add(1, Ordering::SeqCst);

        if count >= self.config.max_concurrent_ops {
            self.active_operations.fetch_sub(1, Ordering::SeqCst);
            return Err(ViolationType::TooManyConcurrentOps);
        }

        Ok(OperationPermit {
            guardrails: self.active_operations.clone(),
        })
    }

    /// Record violation for monitoring
    fn record_violation(&self, violation: ViolationType) {
        let mut violations = self.violations.lock();
        violations.push((Instant::now(), violation));
        
        // Keep only last 100 violations
        if violations.len() > 100 {
            let excess = violations.len() - 100;
            violations.drain(0..excess);
        }
    }

    /// Get violation statistics
    pub fn get_violations(&self, since: Duration) -> Vec<(Instant, ViolationType)> {
        let violations = self.violations.lock();
        let cutoff = Instant::now() - since;

        violations
            .iter()
            .filter(|(time, _)| *time > cutoff)
            .cloned()
            .collect()
    }

    /// Get current system status
    pub fn get_status(&self) -> SystemStatus {
        let mut system = self.system.lock();
        system.refresh_memory();
        system.refresh_cpu_all();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();

        SystemStatus {
            memory_percent: (used_memory as f32 / total_memory as f32) * 100.0,
            available_memory_mb: available_memory / (1024 * 1024),
            cpu_percent: system.global_cpu_usage(),
            active_operations: self.active_operations.load(Ordering::Relaxed),
            throttled: self.throttled.load(Ordering::Relaxed),
            is_safe: self.check_safe().is_ok(),
        }
    }

    /// Wait for system to stabilize
    pub async fn wait_for_stability(&self, timeout: Duration) -> Result<(), ViolationType> {
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                error!("⏱️  Timeout waiting for system stabilization");
                return Err(ViolationType::CpuOverload);
            }

            match self.check_safe() {
                Ok(_) => {
                    info!("✅ System stabilized");
                    return Ok(());
                }
                Err(violation) => {
                    warn!("⏳ Waiting for system to stabilize ({})...", violation);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
}

/// Operation permit - automatically decrements counter on drop
pub struct OperationPermit {
    guardrails: Arc<AtomicUsize>,
}

impl Drop for OperationPermit {
    fn drop(&mut self) {
        self.guardrails.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Current system status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub memory_percent: f32,
    pub available_memory_mb: u64,
    pub cpu_percent: f32,
    pub active_operations: usize,
    pub throttled: bool,
    pub is_safe: bool,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory: {:.1}% ({} MB free) | CPU: {:.1}% | Ops: {} | Throttled: {} | Safe: {}",
            self.memory_percent,
            self.available_memory_mb,
            self.cpu_percent,
            self.active_operations,
            self.throttled,
            self.is_safe
        )
    }
}

/// Windows-specific checks
#[cfg(target_os = "windows")]
pub mod windows {
    use super::*;

    /// Check for known problematic GPU drivers
    pub fn check_gpu_drivers() -> Result<(), String> {
        warn!("🪟 Checking Windows GPU drivers...");

        // This is a placeholder - would need Windows-specific APIs
        // to actually check driver versions

        if cfg!(feature = "hive-gpu") {
            warn!("⚠️  GPU features enabled on Windows - BSOD risk!");
            warn!("⚠️  Ensure you have latest GPU drivers installed");
            return Err("GPU features enabled - high BSOD risk".to_string());
        }

        Ok(())
    }

    /// Check Windows version and updates
    pub fn check_windows_version() -> Result<(), String> {
        info!("🪟 Windows platform detected");
        // Would use winapi to check actual Windows version
        Ok(())
    }

    /// Apply Windows-specific resource limits
    pub fn apply_windows_limits() -> GuardrailsConfig {
        warn!("🪟 Applying Windows-specific resource limits");

        GuardrailsConfig {
            enabled: true,
            max_memory_percent: 60.0, // More conservative on Windows
            max_cpu_percent: 70.0,    // Lower CPU limit
            min_free_memory_mb: 1024, // Higher minimum free memory
            max_concurrent_ops: 2,    // Fewer concurrent ops
            auto_throttle: true,
            windows_protection: true,
        }
    }
}

/// Initialize guardrails with platform-specific settings
pub fn init() -> Guardrails {
    #[cfg(target_os = "windows")]
    {
        warn!("🪟 Windows platform detected - applying strict resource limits");

        if let Err(e) = windows::check_gpu_drivers() {
            error!("❌ GPU driver check failed: {}", e);
            error!("❌ Build with --no-default-features to avoid GPU features");
        }

        let _ = windows::check_windows_version();
        let config = windows::apply_windows_limits();

        Guardrails::new(config)
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("🐧 Unix platform detected - using standard limits");
        Guardrails::new(GuardrailsConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrails_creation() {
        let config = GuardrailsConfig::default();
        let guardrails = Guardrails::new(config);

        let status = guardrails.get_status();
        assert!(status.memory_percent >= 0.0);
        assert!(status.cpu_percent >= 0.0);
    }

    #[test]
    fn test_operation_permit() {
        let config = GuardrailsConfig {
            enabled: true,
            max_concurrent_ops: 2,
            ..Default::default()
        };

        let guardrails = Guardrails::new(config);

        let permit1 = guardrails.acquire_permit().unwrap();
        let permit2 = guardrails.acquire_permit().unwrap();

        // Third permit should fail
        assert!(guardrails.acquire_permit().is_err());

        // Drop permits
        drop(permit1);
        drop(permit2);

        // Should work again
        assert!(guardrails.acquire_permit().is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_stability() {
        let guardrails = Guardrails::new(GuardrailsConfig::default());

        // Should complete immediately if system is stable
        let result = guardrails.wait_for_stability(Duration::from_secs(5)).await;

        // May fail if system is actually under load, but shouldn't panic
        let _ = result;
    }
}
