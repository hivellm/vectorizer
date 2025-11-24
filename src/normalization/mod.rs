//! Text Normalization Module
//!
//! This module provides intelligent text normalization to reduce storage footprint
//! and improve embedding consistency. It includes:
//! - Content type detection (code, markdown, plain text, etc.)
//! - Content-aware text normalization (conservative, moderate, aggressive)
//! - Content hashing for deduplication (BLAKE3)
//!
//! # Architecture
//!
//! ```text
//! Raw Text → Type Detection → Normalization → Content Hash → Storage
//! ```
//!
//! # Example
//!
//! ```no_run
//! use vectorizer::normalization::{TextNormalizer, NormalizationPolicy, NormalizationLevel};
//!
//! let policy = NormalizationPolicy::default();
//! let normalizer = TextNormalizer::new(policy);
//!
//! let raw = "Hello   World\r\n\r\n  With extra   spaces  ";
//! let normalized = normalizer.normalize(raw, None);
//!
//! println!("Original: {} bytes", raw.len());
//! println!("Normalized: {} bytes", normalized.text.len());
//! println!("Content hash: {:?}", normalized.content_hash);
//! ```

pub mod cache;
pub mod config;
pub mod detector;
pub mod hasher;
pub mod integration;
pub mod normalizer;

pub use cache::{CacheConfig, CacheManager, CacheStats};
pub use config::NormalizationConfig;
pub use detector::{ContentType, ContentTypeDetector, TableFormat};
pub use hasher::{ContentHash, ContentHashCalculator, VectorKey};
pub use integration::{NormalizationPipeline, ProcessedDocument};
pub use normalizer::{
    NormalizationLevel, NormalizationMetadata, NormalizationPolicy, NormalizedContent,
    TextNormalizer,
};

/// Version of the normalization implementation
pub const NORMALIZATION_VERSION: u32 = 1;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod quick_test;
