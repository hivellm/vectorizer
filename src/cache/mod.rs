//! Cache Management System for Vectorizer
//!
//! This module provides intelligent cache management and incremental indexing
//! capabilities to optimize startup times and resource usage.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;

pub mod incremental;
pub mod manager;
pub mod metadata;
pub mod validation;

pub use incremental::*;
pub use manager::*;
pub use metadata::*;
pub use validation::*;

#[cfg(test)]
mod tests;

/// Cache management error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("Cache validation failed: {0}")]
    Validation(String),

    #[error("Cache corruption detected: {0}")]
    Corruption(String),

    #[error("Cache version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("Cache operation timeout")]
    Timeout,

    #[error("Cache is locked by another process")]
    Locked,

    #[error("An unexpected error occurred: {0}")]
    Other(String),
}

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache directory path
    pub cache_path: PathBuf,

    /// Cache validation level
    pub validation_level: ValidationLevel,

    /// Cache cleanup settings
    pub cleanup: CleanupConfig,

    /// Cache compression settings
    pub compression: CompressionConfig,

    /// Maximum cache size in bytes
    pub max_size_bytes: u64,

    /// Cache TTL (time to live) in seconds
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_path: PathBuf::from(".vectorizer/cache"),
            validation_level: ValidationLevel::Basic,
            cleanup: CleanupConfig::default(),
            compression: CompressionConfig::default(),
            max_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            ttl_seconds: 30 * 24 * 60 * 60,          // 30 days
        }
    }
}

/// Cache validation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Skip validation, assume cache is valid
    None,
    /// Check file existence and basic metadata
    Basic,
    /// Validate all file hashes and content
    Full,
}

/// Cache cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Enable automatic cleanup
    pub enabled: bool,

    /// Maximum age for cache entries (in seconds)
    pub max_age_seconds: u64,

    /// Maximum cache size before cleanup (in bytes)
    pub max_size_bytes: u64,

    /// Cleanup interval (in seconds)
    pub interval_seconds: u64,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_age_seconds: 30 * 24 * 60 * 60,     // 30 days
            max_size_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            interval_seconds: 24 * 60 * 60,         // 24 hours
        }
    }
}

/// Cache compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,

    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,

    /// Compression level (1-9)
    pub level: u8,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Lz4,
            level: 6,
        }
    }
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// LZ4 compression (fast)
    Lz4,
    /// Gzip compression (balanced)
    Gzip,
    /// Brotli compression (high compression)
    Brotli,
}

// Re-export CacheStats from metadata module
pub use metadata::CacheStats;
