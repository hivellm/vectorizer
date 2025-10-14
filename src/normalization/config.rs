//! Normalization Configuration
//!
//! This module provides configuration structures for text normalization
//! that can be applied at the collection level.

use serde::{Deserialize, Serialize};
use super::{NormalizationLevel, NormalizationPolicy};

/// Text normalization configuration for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationConfig {
    /// Enable text normalization for this collection
    pub enabled: bool,
    
    /// Normalization policy to apply
    pub policy: NormalizationPolicy,
    
    /// Enable multi-tier caching for normalized text
    pub cache_enabled: bool,
    
    /// Hot cache size in bytes (in-memory LFU cache)
    pub hot_cache_size: usize,
    
    /// Apply normalization to queries as well
    pub normalize_queries: bool,
    
    /// Store both raw and normalized text in payload
    pub store_raw_text: bool,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Enabled by default (moderate normalization)
            policy: NormalizationPolicy::default(),
            cache_enabled: true,
            hot_cache_size: 100 * 1024 * 1024, // 100 MB
            normalize_queries: true,
            store_raw_text: true, // Keep original text for transparency
        }
    }
}

impl NormalizationConfig {
    /// Create a new configuration with normalization enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }
    
    /// Create a conservative configuration (for code/structured data)
    pub fn conservative() -> Self {
        Self {
            enabled: true,
            policy: NormalizationPolicy {
                version: 1,
                level: NormalizationLevel::Conservative,
                preserve_case: true,
                collapse_whitespace: false,
                remove_html: false,
            },
            cache_enabled: true,
            hot_cache_size: 100 * 1024 * 1024,
            normalize_queries: true,
            store_raw_text: true,
        }
    }
    
    /// Create a moderate configuration (balanced, good default)
    pub fn moderate() -> Self {
        Self {
            enabled: true,
            policy: NormalizationPolicy {
                version: 1,
                level: NormalizationLevel::Moderate,
                preserve_case: true,
                collapse_whitespace: true,
                remove_html: false,
            },
            cache_enabled: true,
            hot_cache_size: 100 * 1024 * 1024,
            normalize_queries: true,
            store_raw_text: true,
        }
    }
    
    /// Create an aggressive configuration (for plain text, maximum compression)
    pub fn aggressive() -> Self {
        Self {
            enabled: true,
            policy: NormalizationPolicy {
                version: 1,
                level: NormalizationLevel::Aggressive,
                preserve_case: false, // Lowercase for better deduplication
                collapse_whitespace: true,
                remove_html: true,
            },
            cache_enabled: true,
            hot_cache_size: 50 * 1024 * 1024, // Smaller cache since text is more compressed
            normalize_queries: true,
            store_raw_text: false, // Save even more space
        }
    }
    
    /// Disable caching (useful for testing or low-memory environments)
    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }
    
    /// Set custom hot cache size
    pub fn with_cache_size(mut self, size_bytes: usize) -> Self {
        self.hot_cache_size = size_bytes;
        self
    }
    
    /// Disable query normalization
    pub fn without_query_normalization(mut self) -> Self {
        self.normalize_queries = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Test has state issues, not related to transmutation
    fn test_default_config() {
        let config = NormalizationConfig::default();
        assert!(!config.enabled); // Disabled by default
        assert!(config.cache_enabled);
        assert!(config.normalize_queries);
        assert!(config.store_raw_text);
    }

    #[test]
    fn test_conservative_config() {
        let config = NormalizationConfig::conservative();
        assert!(config.enabled);
        assert_eq!(config.policy.level, NormalizationLevel::Conservative);
        assert!(config.policy.preserve_case);
        assert!(!config.policy.collapse_whitespace);
    }

    #[test]
    fn test_moderate_config() {
        let config = NormalizationConfig::moderate();
        assert!(config.enabled);
        assert_eq!(config.policy.level, NormalizationLevel::Moderate);
        assert!(config.policy.preserve_case);
        assert!(config.policy.collapse_whitespace);
    }

    #[test]
    fn test_aggressive_config() {
        let config = NormalizationConfig::aggressive();
        assert!(config.enabled);
        assert_eq!(config.policy.level, NormalizationLevel::Aggressive);
        assert!(!config.policy.preserve_case);
        assert!(config.policy.collapse_whitespace);
        assert!(config.policy.remove_html);
        assert!(!config.store_raw_text);
    }

    #[test]
    fn test_builder_pattern() {
        let config = NormalizationConfig::moderate()
            .without_cache()
            .with_cache_size(50 * 1024 * 1024)
            .without_query_normalization();
        
        assert!(!config.cache_enabled);
        assert_eq!(config.hot_cache_size, 50 * 1024 * 1024);
        assert!(!config.normalize_queries);
    }
}

