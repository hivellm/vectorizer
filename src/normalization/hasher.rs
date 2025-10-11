//! Content Hashing
//!
//! Provides fast, collision-resistant content hashing using BLAKE3.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Content hash (BLAKE3, 32 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", &self.to_hex()[..16])
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Content hash calculator using BLAKE3
pub struct ContentHashCalculator {
    // BLAKE3 hasher is stateless, we create new instances per hash
}

impl ContentHashCalculator {
    /// Create a new hash calculator
    pub fn new() -> Self {
        Self {}
    }

    /// Hash content to produce deterministic content hash
    pub fn hash(&self, content: &str) -> ContentHash {
        let hash = blake3::hash(content.as_bytes());
        ContentHash::from_bytes(*hash.as_bytes())
    }

    /// Hash binary content
    pub fn hash_bytes(&self, content: &[u8]) -> ContentHash {
        let hash = blake3::hash(content);
        ContentHash::from_bytes(*hash.as_bytes())
    }

    /// Verify content matches hash
    pub fn verify(&self, content: &str, expected: &ContentHash) -> bool {
        let actual = self.hash(content);
        actual == *expected
    }
}

impl Default for ContentHashCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector key for deduplication (content hash + embedding config)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VectorKey {
    /// Content hash
    pub content_hash: ContentHash,
    /// Embedding model identifier
    pub embedding_model: String,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Quantization version (0 = none)
    pub quant_version: u32,
}

impl VectorKey {
    /// Create a new vector key
    pub fn new(
        content_hash: ContentHash,
        embedding_model: String,
        embedding_dim: usize,
        quant_version: u32,
    ) -> Self {
        Self {
            content_hash,
            embedding_model,
            embedding_dim,
            quant_version,
        }
    }

    /// Serialize to bytes for storage key
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("VectorKey serialization failed")
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

// Add hex crate to dependencies
mod hex {
    use std::fmt;

    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    #[derive(Debug)]
    pub enum FromHexError {
        InvalidHexCharacter { c: char, index: usize },
        InvalidStringLength,
        OddLength,
    }

    impl fmt::Display for FromHexError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidHexCharacter { c, index } => {
                    write!(f, "Invalid hex character '{}' at position {}", c, index)
                }
                Self::InvalidStringLength => write!(f, "Invalid string length"),
                Self::OddLength => write!(f, "Odd number of hex digits"),
            }
        }
    }

    impl std::error::Error for FromHexError {}

    pub fn decode(s: &str) -> Result<Vec<u8>, FromHexError> {
        if s.len() % 2 != 0 {
            return Err(FromHexError::OddLength);
        }

        let mut result = Vec::with_capacity(s.len() / 2);
        let chars: Vec<char> = s.chars().collect();

        for i in (0..chars.len()).step_by(2) {
            let high = chars[i]
                .to_digit(16)
                .ok_or(FromHexError::InvalidHexCharacter {
                    c: chars[i],
                    index: i,
                })?;
            let low = chars[i + 1]
                .to_digit(16)
                .ok_or(FromHexError::InvalidHexCharacter {
                    c: chars[i + 1],
                    index: i + 1,
                })?;

            result.push(((high << 4) | low) as u8);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash() {
        let calculator = ContentHashCalculator::new();

        // Test determinism
        let content = "Hello, world!";
        let hash1 = calculator.hash(content);
        let hash2 = calculator.hash(content);
        assert_eq!(hash1, hash2);

        // Test different content produces different hash
        let hash3 = calculator.hash("Different content");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_verification() {
        let calculator = ContentHashCalculator::new();

        let content = "Test content";
        let hash = calculator.hash(content);

        assert!(calculator.verify(content, &hash));
        assert!(!calculator.verify("Different content", &hash));
    }

    #[test]
    fn test_hash_hex_conversion() {
        let calculator = ContentHashCalculator::new();

        let content = "Test";
        let hash = calculator.hash(content);

        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars

        let parsed = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_vector_key() {
        let hash = ContentHash::from_bytes([0u8; 32]);
        let key = VectorKey::new(
            hash,
            "all-MiniLM-L6-v2".to_string(),
            384,
            1,
        );

        // Test serialization
        let bytes = key.to_bytes();
        let deserialized = VectorKey::from_bytes(&bytes).unwrap();
        assert_eq!(key, deserialized);
    }

    #[test]
    fn test_hash_bytes() {
        let calculator = ContentHashCalculator::new();

        let data = b"Binary data";
        let hash1 = calculator.hash_bytes(data);
        let hash2 = calculator.hash_bytes(data);
        assert_eq!(hash1, hash2);

        let different_data = b"Different binary data";
        let hash3 = calculator.hash_bytes(different_data);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_collision_resistance() {
        let calculator = ContentHashCalculator::new();

        // Test that similar content produces different hashes
        let hashes: Vec<_> = (0..1000)
            .map(|i| calculator.hash(&format!("Content {}", i)))
            .collect();

        // Check all hashes are unique
        let unique_count = hashes.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 1000);
    }
}

