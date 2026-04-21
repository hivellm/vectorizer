//! Zstandard (Zstd) compression implementation
//!
//! Zstd is a fast compression algorithm that provides excellent compression ratios
//! with good compression and decompression speeds. It's particularly well-suited
//! for vector data compression.

use crate::compression::traits::{CompressionMethod, Compressor, Decompressor};
use crate::compression::{CompressionError, CompressionResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Zstd compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZstdConfig {
    /// Compression level (1-22, higher = better compression)
    pub level: u8,
    /// Enable dictionary compression
    pub use_dictionary: bool,
    /// Dictionary size for compression
    pub dictionary_size: usize,
    /// Enable checksum validation
    pub enable_checksum: bool,
    /// Enable frame format
    pub frame_format: bool,
}

impl Default for ZstdConfig {
    fn default() -> Self {
        Self {
            level: 3,
            use_dictionary: false,
            dictionary_size: 0,
            enable_checksum: true,
            frame_format: true,
        }
    }
}

/// Zstd compressor implementation
pub struct ZstdCompressor {
    config: ZstdConfig,
}

impl ZstdCompressor {
    /// Create a new Zstd compressor
    pub fn new(config: ZstdConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(ZstdConfig::default())
    }

    /// Create with high compression
    pub fn high_compression() -> Self {
        Self::new(ZstdConfig {
            level: 22,
            use_dictionary: false,
            dictionary_size: 0,
            enable_checksum: true,
            frame_format: true,
        })
    }

    /// Create with fast compression
    pub fn fast() -> Self {
        Self::new(ZstdConfig {
            level: 1,
            use_dictionary: false,
            dictionary_size: 0,
            enable_checksum: false,
            frame_format: true,
        })
    }

    /// Create with balanced compression
    pub fn balanced() -> Self {
        Self::new(ZstdConfig {
            level: 3,
            use_dictionary: false,
            dictionary_size: 0,
            enable_checksum: true,
            frame_format: true,
        })
    }
}

impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        // Real Zstd via the `zstd` crate. The previous impl was a
        // hand-rolled toy that produced output larger than input on
        // short payloads and then tripped a broken ratio guard —
        // surfaced when phase4_split-vectorizer-workspace sub-phase
        // 3 isolated the tests in `vectorizer-core`. The dep is
        // already pulled in for `compression::ZstdCompressor`'s
        // sister sites in the persistence layer.
        if data.is_empty() {
            return Ok(Vec::new());
        }
        zstd::stream::encode_all(data, i32::from(self.config.level))
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }

    fn level(&self) -> u8 {
        self.config.level
    }

    fn algorithm(&self) -> &str {
        "zstd"
    }

    fn estimate_compressed_size(&self, original_size: usize) -> usize {
        // Zstd typically achieves 3-6x compression on text data
        // Estimate conservative compression ratio
        original_size / 3
    }
}

impl Decompressor for ZstdCompressor {
    fn decompress(
        &self,
        compressed_data: &[u8],
        _original_size: Option<usize>,
    ) -> CompressionResult<Vec<u8>> {
        if compressed_data.is_empty() {
            return Ok(Vec::new());
        }
        zstd::stream::decode_all(compressed_data)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))
    }

    fn algorithm(&self) -> &str {
        "zstd"
    }
}

impl CompressionMethod for ZstdCompressor {}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_compressor_creation() {
        let config = ZstdConfig::default();
        let compressor = ZstdCompressor::new(config);

        assert_eq!(compressor.level(), 3);
        assert_eq!(Compressor::algorithm(&compressor), "zstd");
    }

    #[test]
    fn test_zstd_compression_decompression() {
        // Real Zstd frames carry per-frame headers, so short payloads
        // can compress LARGER than their input. The contract this
        // test pins is round-trip equality, not size-shrinkage on
        // every input — see test_zstd_repeated_data for the
        // size-shrinkage path on data that actually compresses.
        let compressor = ZstdCompressor::default();
        let data = b"Hello, world! This is a test string for compression.";

        let compressed = compressor.compress(data).unwrap();
        assert!(!compressed.is_empty());

        let decompressed = compressor
            .decompress(&compressed, Some(data.len()))
            .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_zstd_empty_data() {
        let compressor = ZstdCompressor::default();
        let data = b"";

        let compressed = compressor.compress(data).unwrap();
        assert!(compressed.is_empty());

        let decompressed = compressor.decompress(&compressed, Some(0)).unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_zstd_repeated_data() {
        // Use a payload long enough that the Zstd frame overhead is
        // amortised by the actual compression savings (the tiny
        // 18-byte original input was below Zstd's break-even point).
        let compressor = ZstdCompressor::default();
        let data: Vec<u8> = b"AAAAAABBBBBBCCCCCC"
            .iter()
            .cycle()
            .take(2048)
            .copied()
            .collect();

        let compressed = compressor.compress(&data).unwrap();
        assert!(
            compressed.len() < data.len(),
            "Zstd should shrink 2 KiB of repeated bytes; got {} → {}",
            data.len(),
            compressed.len()
        );

        let decompressed = compressor
            .decompress(&compressed, Some(data.len()))
            .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_zstd_estimate_compressed_size() {
        let compressor = ZstdCompressor::default();
        let original_size = 1000;
        let estimated = compressor.estimate_compressed_size(original_size);

        assert!(estimated <= original_size);
        assert!(estimated > 0);
    }

    #[test]
    fn test_zstd_high_compression() {
        let compressor = ZstdCompressor::high_compression();
        assert_eq!(compressor.level(), 22);
        assert_eq!(Compressor::algorithm(&compressor), "zstd");
    }

    #[test]
    fn test_zstd_fast() {
        let compressor = ZstdCompressor::fast();
        assert_eq!(compressor.level(), 1);
        assert_eq!(Compressor::algorithm(&compressor), "zstd");
    }

    #[test]
    fn test_zstd_balanced() {
        let compressor = ZstdCompressor::balanced();
        assert_eq!(compressor.level(), 3);
        assert_eq!(Compressor::algorithm(&compressor), "zstd");
    }
}
