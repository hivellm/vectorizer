//! Zstandard (Zstd) compression implementation
//!
//! Zstd is a fast compression algorithm that provides excellent compression ratios
//! with good compression and decompression speeds. It's particularly well-suited
//! for vector data compression.

use crate::compression::traits::{Compressor, Decompressor, CompressionMethod};
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
        let start = Instant::now();
        
        // Simple Zstd-like compression implementation
        // In a real implementation, you would use the zstd crate
        let compressed = self.simple_zstd_compress(data)?;
        
        let _compression_time = start.elapsed().as_micros() as u64;
        
        // Validate compression ratio
        let ratio = data.len() as f64 / compressed.len() as f64;
        if ratio < 1.0 {
            return Err(CompressionError::CompressionRatioTooLow {
                ratio,
                threshold: 1.0,
            });
        }
        
        Ok(compressed)
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
    fn decompress(&self, compressed_data: &[u8], _original_size: Option<usize>) -> CompressionResult<Vec<u8>> {
        let start = Instant::now();
        
        // Simple Zstd-like decompression implementation
        let decompressed = self.simple_zstd_decompress(compressed_data)?;
        
        let _decompression_time = start.elapsed().as_micros() as u64;
        
        Ok(decompressed)
    }
    
    fn algorithm(&self) -> &str {
        "zstd"
    }
}

impl CompressionMethod for ZstdCompressor {}

impl ZstdCompressor {
    /// Simple Zstd-like compression implementation
    /// This is a simplified version for demonstration purposes
    fn simple_zstd_compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut compressed = Vec::new();
        
        // Add frame header
        if self.config.frame_format {
            compressed.extend_from_slice(&[0x28, 0xB5, 0x2F, 0xFD]); // Zstd magic number
            compressed.push(self.config.level); // Compression level
        }
        
        // Simple LZ77 + Huffman-like compression
        let mut i = 0;
        while i < data.len() {
            let (match_length, match_distance) = self.find_longest_match(data, i);
            
            if match_length >= 3 && match_distance > 0 {
                // Encode match
                self.encode_match(&mut compressed, match_length, match_distance);
                i += match_length;
            } else {
                // Encode literal
                self.encode_literal(&mut compressed, data[i]);
                i += 1;
            }
        }
        
        // Add frame footer
        if self.config.frame_format {
            compressed.push(0x01); // End of frame marker
        }
        
        Ok(compressed)
    }
    
    /// Simple Zstd-like decompression implementation
    fn simple_zstd_decompress(&self, compressed_data: &[u8]) -> CompressionResult<Vec<u8>> {
        if compressed_data.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut decompressed = Vec::new();
        let mut i = 0;
        
        // Skip frame header
        if self.config.frame_format && compressed_data.len() >= 4 {
            if &compressed_data[0..4] == &[0x28, 0xB5, 0x2F, 0xFD] {
                i = 5; // Skip magic number + level
            }
        }
        
        while i < compressed_data.len() {
            if self.config.frame_format && compressed_data[i] == 0x01 {
                // End of frame marker
                break;
            }
            
            let token = compressed_data[i];
            i += 1;
            
            if (token & 0xF0) == 0x00 {
                // Literal
                decompressed.push(token);
            } else {
                // Match
                let match_length = ((token >> 4) & 0x0F) as usize + 3;
                if i < compressed_data.len() {
                    let match_distance = compressed_data[i] as usize + 1;
                    i += 1;
                    
                    // Copy match
                    let start = decompressed.len().saturating_sub(match_distance);
                    for j in 0..match_length {
                        if start + j < decompressed.len() {
                            decompressed.push(decompressed[start + j]);
                        }
                    }
                }
            }
        }
        
        Ok(decompressed)
    }
    
    /// Find the longest match in the data
    fn find_longest_match(&self, data: &[u8], pos: usize) -> (usize, usize) {
        let mut best_length = 0;
        let mut best_distance = 0;
        
        // Search in the previous 32KB
        let search_start = pos.saturating_sub(32 * 1024);
        
        for i in search_start..pos {
            let mut length = 0;
            while pos + length < data.len() 
                && i + length < pos 
                && data[i + length] == data[pos + length] 
                && length < 15 { // Max match length for our simple encoding
                length += 1;
            }
            
            if length > best_length {
                best_length = length;
                best_distance = pos - i;
            }
        }
        
        (best_length, best_distance)
    }
    
    /// Encode a literal byte
    fn encode_literal(&self, compressed: &mut Vec<u8>, byte: u8) {
        compressed.push(0x00); // Literal token
        compressed.push(byte);
    }
    
    /// Encode a match
    fn encode_match(&self, compressed: &mut Vec<u8>, length: usize, distance: usize) {
        let token = ((length - 3) << 4) as u8;
        compressed.push(token);
        compressed.push((distance - 1) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zstd_compressor_creation() {
        let config = ZstdConfig::default();
        let compressor = ZstdCompressor::new(config);
        
        assert_eq!(compressor.level(), 3);
        assert_eq!(compressor.algorithm(), "zstd");
    }
    
    #[test]
    fn test_zstd_compression_decompression() {
        let compressor = ZstdCompressor::default();
        let data = b"Hello, world! This is a test string for compression.";
        
        let compressed = compressor.compress(data).unwrap();
        assert!(!compressed.is_empty());
        assert!(compressed.len() <= data.len());
        
        let decompressed = compressor.decompress(&compressed, Some(data.len())).unwrap();
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
        let compressor = ZstdCompressor::default();
        let data = b"AAAAAABBBBBBCCCCCC";
        
        let compressed = compressor.compress(data).unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = compressor.decompress(&compressed, Some(data.len())).unwrap();
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
        assert_eq!(compressor.algorithm(), "zstd");
    }
    
    #[test]
    fn test_zstd_fast() {
        let compressor = ZstdCompressor::fast();
        assert_eq!(compressor.level(), 1);
        assert_eq!(compressor.algorithm(), "zstd");
    }
    
    #[test]
    fn test_zstd_balanced() {
        let compressor = ZstdCompressor::balanced();
        assert_eq!(compressor.level(), 3);
        assert_eq!(compressor.algorithm(), "zstd");
    }
}
