//! LZ4 compression implementation
//!
//! LZ4 is a fast compression algorithm optimized for speed over compression ratio.
//! It provides very fast compression and decompression with reasonable compression ratios.

use crate::compression::traits::{Compressor, Decompressor, CompressionMethod};
use crate::compression::{CompressionError, CompressionResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

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
        let start = Instant::now();
        
        // Simple LZ4-like compression implementation
        // In a real implementation, you would use the lz4 crate
        let compressed = self.simple_lz4_compress(data)?;
        
        let compression_time = start.elapsed().as_micros() as u64;
        
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
        "lz4"
    }
    
    fn estimate_compressed_size(&self, original_size: usize) -> usize {
        // LZ4 typically achieves 2-4x compression on text data
        // Estimate conservative compression ratio
        original_size / 2
    }
}

impl Decompressor for Lz4Compressor {
    fn decompress(&self, compressed_data: &[u8], original_size: Option<usize>) -> CompressionResult<Vec<u8>> {
        let start = Instant::now();
        
        // Simple LZ4-like decompression implementation
        let decompressed = self.simple_lz4_decompress(compressed_data, original_size)?;
        
        let _decompression_time = start.elapsed().as_micros() as u64;
        
        Ok(decompressed)
    }
    
    fn algorithm(&self) -> &str {
        "lz4"
    }
}

impl CompressionMethod for Lz4Compressor {}

impl Lz4Compressor {
    /// Simple LZ4-like compression implementation
    /// This is a simplified version for demonstration purposes
    fn simple_lz4_compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            // Find the longest match
            let (match_length, match_distance) = self.find_longest_match(data, i);
            
            if match_length >= 4 && match_distance > 0 {
                // Encode literal + match
                let literal_length = i - (i - match_distance);
                if literal_length > 0 {
                    compressed.push(literal_length as u8);
                    compressed.extend_from_slice(&data[i - literal_length..i]);
                }
                
                // Encode match
                compressed.push((match_length << 4) as u8);
                compressed.push(match_distance as u8);
                i += match_length;
            } else {
                // Encode literal
                compressed.push(0);
                compressed.push(data[i]);
                i += 1;
            }
        }
        
        Ok(compressed)
    }
    
    /// Simple LZ4-like decompression implementation
    fn simple_lz4_decompress(&self, compressed_data: &[u8], _original_size: Option<usize>) -> CompressionResult<Vec<u8>> {
        if compressed_data.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut decompressed = Vec::new();
        let mut i = 0;
        
        while i < compressed_data.len() {
            let token = compressed_data[i];
            i += 1;
            
            if token == 0 {
                // Literal
                if i < compressed_data.len() {
                    decompressed.push(compressed_data[i]);
                    i += 1;
                }
            } else {
                // Match
                let match_length = (token >> 4) as usize;
                if i < compressed_data.len() {
                    let match_distance = compressed_data[i] as usize;
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
        
        // Search in the previous 64KB
        let search_start = pos.saturating_sub(64 * 1024);
        
        for i in search_start..pos {
            let mut length = 0;
            while pos + length < data.len() 
                && i + length < pos 
                && data[i + length] == data[pos + length] 
                && length < 255 {
                length += 1;
            }
            
            if length > best_length {
                best_length = length;
                best_distance = pos - i;
            }
        }
        
        (best_length, best_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lz4_compressor_creation() {
        let config = Lz4Config::default();
        let compressor = Lz4Compressor::new(config);
        
        assert_eq!(compressor.level(), 1);
        assert_eq!(compressor.algorithm(), "lz4");
    }
    
    #[test]
    fn test_lz4_compression_decompression() {
        let compressor = Lz4Compressor::default();
        let data = b"Hello, world! This is a test string for compression.";
        
        let compressed = compressor.compress(data).unwrap();
        assert!(!compressed.is_empty());
        assert!(compressed.len() <= data.len());
        
        let decompressed = compressor.decompress(&compressed, Some(data.len())).unwrap();
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
        let compressor = Lz4Compressor::default();
        let data = b"AAAAAABBBBBBCCCCCC";
        
        let compressed = compressor.compress(data).unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = compressor.decompress(&compressed, Some(data.len())).unwrap();
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
        assert_eq!(compressor.algorithm(), "lz4");
    }
    
    #[test]
    fn test_lz4_fast() {
        let compressor = Lz4Compressor::fast();
        assert_eq!(compressor.level(), 1);
        assert_eq!(compressor.algorithm(), "lz4");
    }
}
