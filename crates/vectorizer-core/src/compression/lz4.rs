//! LZ4 compression implementation
//!
//! LZ4 is a fast compression algorithm optimized for speed over compression ratio.
//! It provides very fast compression and decompression with reasonable compression ratios.

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::compression::traits::{CompressionMethod, Compressor, Decompressor};
use crate::compression::{CompressionError, CompressionResult};

/// LZ4 compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lz4Config {
    /// Compression level (1-9, higher = better compression)
    pub level: u8,
    /// Enable high compression mode
    pub high_compression: bool,
    /// Enable frame format (for streaming)
    pub frame_format: bool,
    /// Block size for compression
    pub block_size: usize,
}

impl Default for Lz4Config {
    fn default() -> Self {
        Self {
            level: 1,
            high_compression: false,
            frame_format: false,
            block_size: 64 * 1024, // 64KB
        }
    }
}

/// LZ4 compressor implementation
pub struct Lz4Compressor {
    config: Lz4Config,
}

impl Lz4Compressor {
    /// Create a new LZ4 compressor
    pub fn new(config: Lz4Config) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(Lz4Config::default())
    }

    /// Create with high compression
    pub fn high_compression() -> Self {
        Self::new(Lz4Config {
            level: 9,
            high_compression: true,
            frame_format: false,
            block_size: 64 * 1024,
        })
    }

    /// Create with fast compression
    pub fn fast() -> Self {
        Self::new(Lz4Config {
            level: 1,
            high_compression: false,
            frame_format: false,
            block_size: 64 * 1024,
        })
    }
}

impl Compressor for Lz4Compressor {
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        // Real LZ4 via the `lz4_flex` crate. The previous impl was a
        // hand-rolled toy with a broken ratio guard that rejected any
        // input where compression actually helped — surfaced when
        // phase4_split-vectorizer-workspace sub-phase 3 isolated the
        // tests in `vectorizer-core`. Switching to `lz4_flex` keeps
        // the same public API and adopts the real LZ4 frame format
        // that `lz4_flex::decompress_size_prepended` already expects
        // elsewhere in the crate (see `quantization::storage`).
        if data.is_empty() {
            return Ok(Vec::new());
        }
        Ok(lz4_flex::compress_prepend_size(data))
    }

    fn level(&self) -> u8 {
        self.config.level
    }

    fn algorithm(&self) -> &str {
        "lz4"
    }

    fn estimate_compressed_size(&self, original_size: usize) -> usize {
        // LZ4 typically achieves 2-4x compression on text data
        // Estimate conservative compression ratio
        original_size / 2
    }
}

impl Decompressor for Lz4Compressor {
    fn decompress(
        &self,
        compressed_data: &[u8],
        _original_size: Option<usize>,
    ) -> CompressionResult<Vec<u8>> {
        if compressed_data.is_empty() {
            return Ok(Vec::new());
        }
        lz4_flex::decompress_size_prepended(compressed_data)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))
    }

    fn algorithm(&self) -> &str {
        "lz4"
    }
}

impl CompressionMethod for Lz4Compressor {}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_compressor_creation() {
        let config = Lz4Config::default();
        let compressor = Lz4Compressor::new(config);

        assert_eq!(compressor.level(), 1);
        assert_eq!(Compressor::algorithm(&compressor), "lz4");
    }

    #[test]
    fn test_lz4_compression_decompression() {
        // Real LZ4 frames carry a 4-byte size header plus a per-block
        // overhead, so short payloads can compress LARGER than their
        // input. The contract this test pins is round-trip equality,
        // not size-shrinkage on every input — see test_lz4_repeated_data
        // for the size-shrinkage path on data that actually compresses.
        let compressor = Lz4Compressor::default();
        let data = b"Hello, world! This is a test string for compression.";

        let compressed = compressor.compress(data).unwrap();
        assert!(!compressed.is_empty());

        let decompressed = compressor
            .decompress(&compressed, Some(data.len()))
            .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_lz4_empty_data() {
        let compressor = Lz4Compressor::default();
        let data = b"";

        let compressed = compressor.compress(data).unwrap();
        assert!(compressed.is_empty());

        let decompressed = compressor.decompress(&compressed, Some(0)).unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_lz4_repeated_data() {
        // Use a payload long enough that the LZ4 frame overhead is
        // amortised by the actual compression savings (the tiny
        // 18-byte original input was below LZ4's break-even point).
        let compressor = Lz4Compressor::default();
        let data: Vec<u8> = b"AAAAAABBBBBBCCCCCC"
            .iter()
            .cycle()
            .take(2048)
            .copied()
            .collect();

        let compressed = compressor.compress(&data).unwrap();
        assert!(
            compressed.len() < data.len(),
            "LZ4 should shrink 2 KiB of repeated bytes; got {} → {}",
            data.len(),
            compressed.len()
        );

        let decompressed = compressor
            .decompress(&compressed, Some(data.len()))
            .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_lz4_estimate_compressed_size() {
        let compressor = Lz4Compressor::default();
        let original_size = 1000;
        let estimated = compressor.estimate_compressed_size(original_size);

        assert!(estimated <= original_size);
        assert!(estimated > 0);
    }

    #[test]
    fn test_lz4_high_compression() {
        let compressor = Lz4Compressor::high_compression();
        assert_eq!(compressor.level(), 9);
        assert_eq!(Compressor::algorithm(&compressor), "lz4");
    }

    #[test]
    fn test_lz4_fast() {
        let compressor = Lz4Compressor::fast();
        assert_eq!(compressor.level(), 1);
        assert_eq!(Compressor::algorithm(&compressor), "lz4");
    }
}
