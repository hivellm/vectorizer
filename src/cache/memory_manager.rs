//! Global cache memory manager for cluster mode
//!
//! Tracks and limits total cache memory usage across all caches in the system.
//! This is critical for cluster deployments where memory must be predictable.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use tracing::{debug, info, warn};

/// Configuration for the cache memory manager
#[derive(Debug, Clone)]
pub struct CacheMemoryManagerConfig {
    /// Maximum total cache memory in bytes
    pub max_memory_bytes: u64,
    /// Warning threshold (0-100 percent)
    pub warning_threshold_percent: u8,
    /// Enable strict enforcement (reject allocations over limit)
    pub strict_enforcement: bool,
}

impl Default for CacheMemoryManagerConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            warning_threshold_percent: 80,
            strict_enforcement: true,
        }
    }
}

/// Statistics for cache memory usage
#[derive(Debug, Clone, Default)]
pub struct CacheMemoryStats {
    /// Current total memory usage in bytes
    pub current_usage_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_usage_bytes: u64,
    /// Number of allocation requests
    pub allocation_count: u64,
    /// Number of deallocation requests
    pub deallocation_count: u64,
    /// Number of allocations rejected due to limit
    pub rejected_allocations: u64,
    /// Number of forced evictions
    pub forced_evictions: u64,
}

/// Result of a cache memory allocation attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationResult {
    /// Allocation succeeded
    Success,
    /// Allocation rejected due to memory limit
    Rejected { requested: u64, available: u64 },
    /// Allocation succeeded but triggered warning threshold
    SuccessWithWarning { current_usage: u64, max: u64 },
}

impl AllocationResult {
    /// Check if allocation was successful
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            AllocationResult::Success | AllocationResult::SuccessWithWarning { .. }
        )
    }
}

/// Global cache memory manager
///
/// Thread-safe singleton that tracks cache memory usage across the entire system.
/// In cluster mode, this ensures predictable memory usage.
pub struct CacheMemoryManager {
    /// Configuration
    config: CacheMemoryManagerConfig,
    /// Current total memory usage
    current_usage: AtomicU64,
    /// Peak memory usage
    peak_usage: AtomicU64,
    /// Statistics
    stats: RwLock<CacheMemoryStats>,
    /// Whether manager is enabled (only in cluster mode)
    enabled: bool,
}

impl CacheMemoryManager {
    /// Create a new cache memory manager
    pub fn new(config: CacheMemoryManagerConfig) -> Self {
        info!(
            "Initializing CacheMemoryManager: max_memory={} MB, warning_threshold={}%, strict={}",
            config.max_memory_bytes / (1024 * 1024),
            config.warning_threshold_percent,
            config.strict_enforcement
        );

        Self {
            config,
            current_usage: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            stats: RwLock::new(CacheMemoryStats::default()),
            enabled: true,
        }
    }

    /// Create a disabled manager (for non-cluster mode)
    pub fn disabled() -> Self {
        Self {
            config: CacheMemoryManagerConfig::default(),
            current_usage: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            stats: RwLock::new(CacheMemoryStats::default()),
            enabled: false,
        }
    }

    /// Check if the manager is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> u64 {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// Get available memory
    pub fn available(&self) -> u64 {
        let current = self.current_usage.load(Ordering::Relaxed);
        self.config.max_memory_bytes.saturating_sub(current)
    }

    /// Get usage percentage (0-100)
    pub fn usage_percent(&self) -> f64 {
        let current = self.current_usage.load(Ordering::Relaxed) as f64;
        let max = self.config.max_memory_bytes as f64;
        if max == 0.0 {
            return 0.0;
        }
        (current / max) * 100.0
    }

    /// Try to allocate memory from the cache budget
    ///
    /// Returns the allocation result indicating success, warning, or rejection.
    pub fn try_allocate(&self, bytes: u64) -> AllocationResult {
        if !self.enabled {
            return AllocationResult::Success;
        }

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.allocation_count += 1;
        }

        let current = self.current_usage.load(Ordering::Relaxed);
        let new_usage = current + bytes;

        // Check if allocation would exceed limit
        if new_usage > self.config.max_memory_bytes {
            if self.config.strict_enforcement {
                let mut stats = self.stats.write();
                stats.rejected_allocations += 1;
                warn!(
                    "Cache memory allocation rejected: requested={} bytes, available={} bytes",
                    bytes,
                    self.available()
                );
                return AllocationResult::Rejected {
                    requested: bytes,
                    available: self.available(),
                };
            }
            // Non-strict: allow but log warning
            warn!(
                "Cache memory limit exceeded (non-strict mode): usage={} bytes, limit={} bytes",
                new_usage, self.config.max_memory_bytes
            );
        }

        // Perform allocation
        self.current_usage.fetch_add(bytes, Ordering::Relaxed);

        // Update peak usage
        loop {
            let peak = self.peak_usage.load(Ordering::Relaxed);
            if new_usage <= peak {
                break;
            }
            if self
                .peak_usage
                .compare_exchange(peak, new_usage, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        // Update stats current usage
        {
            let mut stats = self.stats.write();
            stats.current_usage_bytes = new_usage;
            stats.peak_usage_bytes = self.peak_usage.load(Ordering::Relaxed);
        }

        // Check warning threshold
        let usage_percent = (new_usage as f64 / self.config.max_memory_bytes as f64) * 100.0;
        if usage_percent >= self.config.warning_threshold_percent as f64 {
            debug!(
                "Cache memory usage at {:.1}% ({} MB / {} MB)",
                usage_percent,
                new_usage / (1024 * 1024),
                self.config.max_memory_bytes / (1024 * 1024)
            );
            return AllocationResult::SuccessWithWarning {
                current_usage: new_usage,
                max: self.config.max_memory_bytes,
            };
        }

        AllocationResult::Success
    }

    /// Deallocate memory from the cache budget
    pub fn deallocate(&self, bytes: u64) {
        if !self.enabled {
            return;
        }

        let prev = self.current_usage.fetch_sub(bytes, Ordering::Relaxed);

        // Prevent underflow (shouldn't happen but be safe)
        if prev < bytes {
            warn!(
                "Cache memory deallocation underflow: tried to free {} bytes, had {} bytes",
                bytes, prev
            );
            self.current_usage.store(0, Ordering::Relaxed);
        }

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.deallocation_count += 1;
            stats.current_usage_bytes = self.current_usage.load(Ordering::Relaxed);
        }
    }

    /// Record a forced eviction
    pub fn record_eviction(&self) {
        if !self.enabled {
            return;
        }

        let mut stats = self.stats.write();
        stats.forced_evictions += 1;
    }

    /// Get current statistics
    pub fn stats(&self) -> CacheMemoryStats {
        let stats = self.stats.read();
        CacheMemoryStats {
            current_usage_bytes: self.current_usage.load(Ordering::Relaxed),
            peak_usage_bytes: self.peak_usage.load(Ordering::Relaxed),
            allocation_count: stats.allocation_count,
            deallocation_count: stats.deallocation_count,
            rejected_allocations: stats.rejected_allocations,
            forced_evictions: stats.forced_evictions,
        }
    }

    /// Reset statistics (but not current usage)
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        stats.allocation_count = 0;
        stats.deallocation_count = 0;
        stats.rejected_allocations = 0;
        stats.forced_evictions = 0;
        // Keep current_usage_bytes and peak_usage_bytes
    }

    /// Get configuration
    pub fn config(&self) -> &CacheMemoryManagerConfig {
        &self.config
    }

    /// Check if memory limit would be exceeded by an allocation
    pub fn would_exceed_limit(&self, bytes: u64) -> bool {
        if !self.enabled {
            return false;
        }
        let current = self.current_usage.load(Ordering::Relaxed);
        current + bytes > self.config.max_memory_bytes
    }

    /// Get recommended eviction size to make room for new allocation
    pub fn recommended_eviction_size(&self, requested_bytes: u64) -> Option<u64> {
        if !self.enabled {
            return None;
        }

        let current = self.current_usage.load(Ordering::Relaxed);
        let needed = current + requested_bytes;

        if needed <= self.config.max_memory_bytes {
            return None;
        }

        // Recommend evicting enough to get to 90% of limit
        let target = (self.config.max_memory_bytes as f64 * 0.9) as u64;
        Some(needed.saturating_sub(target))
    }
}

/// Global singleton for cache memory manager
static GLOBAL_CACHE_MEMORY_MANAGER: std::sync::OnceLock<Arc<CacheMemoryManager>> =
    std::sync::OnceLock::new();

/// Initialize the global cache memory manager
///
/// This should be called once at startup with the cluster configuration.
pub fn init_global_cache_memory_manager(
    config: CacheMemoryManagerConfig,
) -> Arc<CacheMemoryManager> {
    let manager = Arc::new(CacheMemoryManager::new(config));
    if GLOBAL_CACHE_MEMORY_MANAGER.set(manager.clone()).is_err() {
        warn!("Global cache memory manager already initialized");
    }
    manager
}

/// Get the global cache memory manager
///
/// Returns a disabled manager if not initialized.
pub fn get_global_cache_memory_manager() -> Arc<CacheMemoryManager> {
    GLOBAL_CACHE_MEMORY_MANAGER
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(CacheMemoryManager::disabled()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_success() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024 * 1024, // 1MB
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        // Allocate 100KB
        let result = manager.try_allocate(100 * 1024);
        assert!(result.is_success());
        assert_eq!(manager.current_usage(), 100 * 1024);
    }

    #[test]
    fn test_allocation_rejected() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024 * 1024, // 1MB
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        // Try to allocate 2MB (exceeds limit)
        let result = manager.try_allocate(2 * 1024 * 1024);
        assert!(!result.is_success());
        assert!(matches!(result, AllocationResult::Rejected { .. }));
        assert_eq!(manager.current_usage(), 0);
    }

    #[test]
    fn test_allocation_with_warning() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024 * 1024, // 1MB
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        // Allocate 900KB (90%, above 80% threshold)
        let result = manager.try_allocate(900 * 1024);
        assert!(result.is_success());
        assert!(matches!(
            result,
            AllocationResult::SuccessWithWarning { .. }
        ));
    }

    #[test]
    fn test_deallocation() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024 * 1024,
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        // Allocate then deallocate
        manager.try_allocate(100 * 1024);
        assert_eq!(manager.current_usage(), 100 * 1024);

        manager.deallocate(50 * 1024);
        assert_eq!(manager.current_usage(), 50 * 1024);

        manager.deallocate(50 * 1024);
        assert_eq!(manager.current_usage(), 0);
    }

    #[test]
    fn test_peak_usage() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024 * 1024,
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        manager.try_allocate(100 * 1024);
        manager.try_allocate(200 * 1024);
        assert_eq!(manager.peak_usage(), 300 * 1024);

        manager.deallocate(150 * 1024);
        assert_eq!(manager.current_usage(), 150 * 1024);
        assert_eq!(manager.peak_usage(), 300 * 1024); // Peak unchanged
    }

    #[test]
    fn test_non_strict_mode() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024 * 1024,
            warning_threshold_percent: 80,
            strict_enforcement: false, // Non-strict
        };
        let manager = CacheMemoryManager::new(config);

        // Should succeed even exceeding limit in non-strict mode
        let result = manager.try_allocate(2 * 1024 * 1024);
        assert!(result.is_success());
        assert_eq!(manager.current_usage(), 2 * 1024 * 1024);
    }

    #[test]
    fn test_disabled_manager() {
        let manager = CacheMemoryManager::disabled();

        assert!(!manager.is_enabled());

        // All allocations should succeed
        let result = manager.try_allocate(100 * 1024 * 1024 * 1024);
        assert!(result.is_success());
        assert_eq!(manager.current_usage(), 0); // Not tracked
    }

    #[test]
    fn test_usage_percent() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1000,
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        manager.try_allocate(500);
        assert!((manager.usage_percent() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_would_exceed_limit() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024,
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        assert!(!manager.would_exceed_limit(512));
        manager.try_allocate(512);
        assert!(!manager.would_exceed_limit(512));
        assert!(manager.would_exceed_limit(513));
    }

    #[test]
    fn test_recommended_eviction_size() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1000,
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        // No eviction needed
        assert!(manager.recommended_eviction_size(500).is_none());

        // Fill up and check eviction recommendation
        manager.try_allocate(900);
        let eviction = manager.recommended_eviction_size(200);
        assert!(eviction.is_some());
    }

    #[test]
    fn test_stats() {
        let config = CacheMemoryManagerConfig {
            max_memory_bytes: 1024,
            warning_threshold_percent: 80,
            strict_enforcement: true,
        };
        let manager = CacheMemoryManager::new(config);

        manager.try_allocate(100);
        manager.try_allocate(100);
        manager.deallocate(50);
        manager.try_allocate(10000); // Will be rejected
        manager.record_eviction();

        let stats = manager.stats();
        assert_eq!(stats.allocation_count, 3);
        assert_eq!(stats.deallocation_count, 1);
        assert_eq!(stats.rejected_allocations, 1);
        assert_eq!(stats.forced_evictions, 1);
    }
}
