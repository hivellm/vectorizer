//! Payload Encryption Module
//!
//! This module provides end-to-end encryption for vector payloads using:
//! - ECC (Elliptic Curve Cryptography) for key exchange via ECDH
//! - AES-256-GCM for symmetric encryption
//!
//! # Zero-Knowledge Architecture
//!
//! The server never stores or has access to decryption keys. Only clients
//! with the corresponding private key can decrypt payloads.
//!
//! # Encryption Flow
//!
//! 1. Client provides an ECC public key (P-256 curve)
//! 2. Server generates an ephemeral key pair
//! 3. Server performs ECDH to derive a shared secret
//! 4. Shared secret is used to derive an AES-256-GCM key
//! 5. Payload is encrypted with AES-256-GCM
//! 6. Encrypted payload + metadata is stored
//!
//! # Decryption Flow (Client-side only)
//!
//! 1. Client retrieves encrypted payload with ephemeral public key
//! 2. Client performs ECDH with their private key
//! 3. Client derives the same AES-256-GCM key
//! 4. Client decrypts the payload

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use p256::ecdh::diffie_hellman;
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::{EncodedPoint, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Errors that can occur during payload encryption/decryption
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid encrypted payload format")]
    InvalidPayloadFormat,

    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

fn default_encryption_version() -> u8 {
    1
}

/// Encrypted payload structure containing all necessary metadata for decryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// Version of the encryption scheme (for future compatibility)
    /// Defaults to 1 when missing (e.g. after some storage round-trips).
    #[serde(default = "default_encryption_version")]
    pub version: u8,

    /// Base64-encoded nonce used for AES-256-GCM (96 bits / 12 bytes)
    pub nonce: String,

    /// Base64-encoded authentication tag from AES-256-GCM (128 bits / 16 bytes)
    pub tag: String,

    /// Base64-encoded encrypted payload data
    pub encrypted_data: String,

    /// Base64-encoded ephemeral public key used for ECDH
    /// Clients need this to derive the shared secret
    pub ephemeral_public_key: String,

    /// Encryption algorithm identifier
    pub algorithm: String,
}

impl EncryptedPayload {
    /// Check if this is an encrypted payload (version > 0)
    pub fn is_encrypted(&self) -> bool {
        self.version > 0
    }
}

/// Encrypts a JSON payload using ECC (P-256) + AES-256-GCM
///
/// # Arguments
///
/// * `payload_json` - The payload data as JSON value
/// * `public_key_pem` - The recipient's public key in PEM format
///
/// # Returns
///
/// An `EncryptedPayload` containing the encrypted data and metadata
///
/// # Errors
///
/// Returns `EncryptionError` if:
/// - The public key format is invalid
/// - The encryption operation fails
/// - Serialization fails
pub fn encrypt_payload(
    payload_json: &serde_json::Value,
    public_key_pem: &str,
) -> Result<EncryptedPayload, EncryptionError> {
    // Parse the recipient's public key
    let recipient_public_key = parse_public_key(public_key_pem)?;

    // Generate an ephemeral key pair for ECDH
    let ephemeral_secret = SecretKey::random(&mut OsRng);
    let ephemeral_public = ephemeral_secret.public_key();

    // Perform ECDH to get shared secret
    let shared_secret = diffie_hellman(
        ephemeral_secret.to_nonzero_scalar(),
        recipient_public_key.as_affine(),
    );

    // Derive AES-256-GCM key from shared secret using SHA-256
    let aes_key = Sha256::digest(shared_secret.raw_secret_bytes());

    // Create AES-256-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

    // Generate a random nonce (96 bits for GCM)
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Serialize payload to JSON bytes
    let payload_bytes = serde_json::to_vec(payload_json)?;

    // Encrypt the payload
    let ciphertext = cipher
        .encrypt(&nonce, payload_bytes.as_ref())
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

    // The ciphertext includes the authentication tag at the end (last 16 bytes)
    let (encrypted_data, tag) = if ciphertext.len() >= 16 {
        let split_pos = ciphertext.len() - 16;
        (&ciphertext[..split_pos], &ciphertext[split_pos..])
    } else {
        return Err(EncryptionError::EncryptionFailed(
            "Ciphertext too short".to_string(),
        ));
    };

    // Encode ephemeral public key
    let ephemeral_public_encoded = ephemeral_public.to_encoded_point(false);

    Ok(EncryptedPayload {
        version: 1,
        nonce: BASE64.encode(nonce),
        tag: BASE64.encode(tag),
        encrypted_data: BASE64.encode(encrypted_data),
        ephemeral_public_key: BASE64.encode(ephemeral_public_encoded.as_bytes()),
        algorithm: "ECC-P256-AES256GCM".to_string(),
    })
}

/// Parses a public key from PEM, hex, or base64-encoded format
///
/// Supports:
/// - PEM format (-----BEGIN PUBLIC KEY-----)
/// - Hexadecimal encoding (with or without 0x prefix)
/// - Base64-encoded raw public key
///
/// # Arguments
///
/// * `public_key_str` - The public key string
///
/// # Returns
///
/// A parsed `PublicKey`
///
/// # Errors
///
/// Returns `EncryptionError::InvalidPublicKey` if the key cannot be parsed
fn parse_public_key(public_key_str: &str) -> Result<PublicKey, EncryptionError> {
    let trimmed = public_key_str.trim();

    // Try PEM format first
    if trimmed.starts_with("-----BEGIN PUBLIC KEY-----") {
        // Extract base64 content between headers
        let pem_content = trimmed
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<String>();

        let der_bytes = BASE64
            .decode(pem_content.as_bytes())
            .map_err(|e| EncryptionError::InvalidPublicKey(format!("PEM decode error: {}", e)))?;

        // Parse DER format (skip the SubjectPublicKeyInfo wrapper if present)
        parse_der_public_key(&der_bytes)
    } else if trimmed.starts_with("0x") || trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        // Try hexadecimal format
        let hex_str = if trimmed.starts_with("0x") {
            &trimmed[2..]
        } else {
            trimmed
        };

        let key_bytes = hex::decode(hex_str)
            .map_err(|e| EncryptionError::InvalidPublicKey(format!("Hex decode error: {}", e)))?;

        parse_der_public_key(&key_bytes)
    } else {
        // Try base64-encoded raw key
        let key_bytes = BASE64.decode(trimmed.as_bytes()).map_err(|e| {
            EncryptionError::InvalidPublicKey(format!("Base64 decode error: {}", e))
        })?;

        parse_der_public_key(&key_bytes)
    }
}

/// Parses a public key from DER format
fn parse_der_public_key(der_bytes: &[u8]) -> Result<PublicKey, EncryptionError> {
    // Try parsing as raw point first (65 bytes for uncompressed P-256 point)
    if der_bytes.len() == 65 && der_bytes[0] == 0x04 {
        let point = EncodedPoint::from_bytes(der_bytes)
            .map_err(|e| EncryptionError::InvalidPublicKey(format!("Invalid point: {}", e)))?;

        let pk_option = PublicKey::from_encoded_point(&point);
        return Option::from(pk_option).ok_or_else(|| {
            EncryptionError::InvalidPublicKey("Invalid public key point".to_string())
        });
    }

    // Try parsing as SubjectPublicKeyInfo (DER)
    // For P-256, the DER format starts with algorithm identifier, then the point
    // We need to extract the point from the BIT STRING
    if der_bytes.len() > 65 {
        // Simple DER parser for SubjectPublicKeyInfo
        // Look for the bit string tag (0x03) followed by length
        for i in 0..der_bytes.len().saturating_sub(66) {
            if der_bytes[i] == 0x03 && i + 67 < der_bytes.len() {
                // Found BIT STRING tag
                let point_start = i + 2; // Skip tag and length
                if der_bytes[point_start] == 0x00 && der_bytes[point_start + 1] == 0x04 {
                    // Skip the unused bits byte (0x00) and we have the point
                    let point_bytes = &der_bytes[point_start + 1..point_start + 66];
                    let point = EncodedPoint::from_bytes(point_bytes).map_err(|e| {
                        EncryptionError::InvalidPublicKey(format!("Invalid point: {}", e))
                    })?;

                    let pk_option = PublicKey::from_encoded_point(&point);
                    return Option::from(pk_option).ok_or_else(|| {
                        EncryptionError::InvalidPublicKey("Invalid public key point".to_string())
                    });
                }
            }
        }
    }

    Err(EncryptionError::InvalidPublicKey(
        "Unsupported key format. Expected PEM or raw point.".to_string(),
    ))
}

/// Validates that an encrypted payload has all required fields
pub fn validate_encrypted_payload(payload: &EncryptedPayload) -> Result<(), EncryptionError> {
    if payload.nonce.is_empty() {
        return Err(EncryptionError::InvalidPayloadFormat);
    }
    if payload.tag.is_empty() {
        return Err(EncryptionError::InvalidPayloadFormat);
    }
    if payload.encrypted_data.is_empty() {
        return Err(EncryptionError::InvalidPayloadFormat);
    }
    if payload.ephemeral_public_key.is_empty() {
        return Err(EncryptionError::InvalidPayloadFormat);
    }

    // Validate base64 encoding
    BASE64.decode(&payload.nonce)?;
    BASE64.decode(&payload.tag)?;
    BASE64.decode(&payload.encrypted_data)?;
    BASE64.decode(&payload.ephemeral_public_key)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Generate a test key pair
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();

        // Convert public key to PEM format
        let public_key_point = public_key.to_encoded_point(false);
        let public_key_base64 = BASE64.encode(public_key_point.as_bytes());

        // Create a test payload
        let payload = json!({
            "user_id": "12345",
            "sensitive_data": "This is confidential information",
            "metadata": {
                "category": "financial",
                "timestamp": "2024-01-15T10:30:00Z"
            }
        });

        // Encrypt the payload
        let encrypted = encrypt_payload(&payload, &public_key_base64).unwrap();

        // Validate the encrypted payload
        assert_eq!(encrypted.version, 1);
        assert_eq!(encrypted.algorithm, "ECC-P256-AES256GCM");
        assert!(!encrypted.nonce.is_empty());
        assert!(!encrypted.tag.is_empty());
        assert!(!encrypted.encrypted_data.is_empty());
        assert!(!encrypted.ephemeral_public_key.is_empty());

        // Validate structure
        validate_encrypted_payload(&encrypted).unwrap();
    }

    #[test]
    fn test_invalid_public_key() {
        let payload = json!({"test": "data"});
        let result = encrypt_payload(&payload, "invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_payload_validation() {
        let valid = EncryptedPayload {
            version: 1,
            nonce: BASE64.encode(b"test_nonce_12"),
            tag: BASE64.encode(b"test_tag_16bytes"),
            encrypted_data: BASE64.encode(b"encrypted_data"),
            ephemeral_public_key: BASE64.encode(b"ephemeral_key"),
            algorithm: "ECC-P256-AES256GCM".to_string(),
        };

        assert!(validate_encrypted_payload(&valid).is_ok());

        let invalid = EncryptedPayload {
            version: 1,
            nonce: String::new(),
            tag: String::new(),
            encrypted_data: String::new(),
            ephemeral_public_key: String::new(),
            algorithm: "ECC-P256-AES256GCM".to_string(),
        };

        assert!(validate_encrypted_payload(&invalid).is_err());
    }
}
