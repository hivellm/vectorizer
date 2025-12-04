//! Request Signing Validation Module
//!
//! Provides HMAC-SHA256 based request signing validation for HiveHub integration.
//! This ensures request integrity and authenticity for sensitive operations.
//!
//! ## Signature Format
//!
//! Requests should include the following headers:
//! - `X-HiveHub-Signature`: HMAC-SHA256 signature of the canonical request
//! - `X-HiveHub-Timestamp`: Unix timestamp of the request (for replay protection)
//! - `X-HiveHub-Nonce`: Unique nonce for this request
//!
//! ## Canonical Request Format
//!
//! The canonical request string is constructed as:
//! ```text
//! METHOD\n
//! PATH\n
//! TIMESTAMP\n
//! NONCE\n
//! BODY_HASH
//! ```
//!
//! Where BODY_HASH is SHA256(request_body) in hex format, or empty string for no body.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use tracing::{debug, warn};

use crate::error::{Result, VectorizerError};

/// HMAC-SHA256 type alias
type HmacSha256 = Hmac<Sha256>;

/// Maximum clock skew allowed (5 minutes)
const MAX_CLOCK_SKEW_SECS: u64 = 300;

/// Nonce expiry time (10 minutes)
const NONCE_EXPIRY_SECS: u64 = 600;

/// Maximum nonce cache size
const MAX_NONCE_CACHE_SIZE: usize = 100_000;

/// Request signature header name
pub const HEADER_SIGNATURE: &str = "x-hivehub-signature";
/// Request timestamp header name
pub const HEADER_TIMESTAMP: &str = "x-hivehub-timestamp";
/// Request nonce header name
pub const HEADER_NONCE: &str = "x-hivehub-nonce";

/// Configuration for request signing validation
#[derive(Debug, Clone)]
pub struct SigningConfig {
    /// Whether request signing is enabled
    pub enabled: bool,
    /// Secret key for HMAC signing (base64 encoded)
    pub secret_key: Option<String>,
    /// Maximum allowed clock skew in seconds
    pub max_clock_skew_secs: u64,
    /// Paths that require signing (empty = all paths)
    pub required_paths: Vec<String>,
    /// Paths that are exempt from signing
    pub exempt_paths: Vec<String>,
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            secret_key: std::env::var("HIVEHUB_SIGNING_SECRET").ok(),
            max_clock_skew_secs: MAX_CLOCK_SKEW_SECS,
            required_paths: vec![],
            exempt_paths: vec!["/health".to_string(), "/metrics".to_string()],
        }
    }
}

/// Components of a signed request
#[derive(Debug, Clone)]
pub struct SignedRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path (without query string)
    pub path: String,
    /// Request timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Unique nonce
    pub nonce: String,
    /// SHA256 hash of the request body (hex)
    pub body_hash: String,
    /// The provided signature
    pub signature: String,
}

impl SignedRequest {
    /// Create canonical request string for signing
    pub fn canonical_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.method, self.path, self.timestamp, self.nonce, self.body_hash
        )
    }

    /// Parse from HTTP headers
    pub fn from_headers(
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> Result<Self> {
        let signature = headers
            .get(HEADER_SIGNATURE)
            .ok_or_else(|| {
                VectorizerError::AuthenticationError("Missing signature header".to_string())
            })?
            .clone();

        let timestamp_str = headers.get(HEADER_TIMESTAMP).ok_or_else(|| {
            VectorizerError::AuthenticationError("Missing timestamp header".to_string())
        })?;

        let timestamp: u64 = timestamp_str.parse().map_err(|_| {
            VectorizerError::AuthenticationError("Invalid timestamp format".to_string())
        })?;

        let nonce = headers
            .get(HEADER_NONCE)
            .ok_or_else(|| {
                VectorizerError::AuthenticationError("Missing nonce header".to_string())
            })?
            .clone();

        let body_hash = match body {
            Some(b) if !b.is_empty() => {
                let mut hasher = Sha256::new();
                hasher.update(b);
                format!("{:x}", hasher.finalize())
            }
            _ => String::new(),
        };

        Ok(Self {
            method: method.to_uppercase(),
            path: path.to_string(),
            timestamp,
            nonce,
            body_hash,
            signature,
        })
    }
}

/// Entry in the nonce cache for replay protection
#[derive(Debug)]
struct NonceEntry {
    /// When the nonce was first seen
    seen_at: SystemTime,
}

/// Request signing validator
#[derive(Debug)]
pub struct RequestSigningValidator {
    /// Configuration
    config: SigningConfig,
    /// Decoded secret key
    secret_key: Option<Vec<u8>>,
    /// Nonce cache for replay protection
    nonce_cache: Arc<RwLock<HashMap<String, NonceEntry>>>,
}

impl RequestSigningValidator {
    /// Create a new validator with the given configuration
    pub fn new(config: SigningConfig) -> Result<Self> {
        let secret_key = if let Some(ref key_b64) = config.secret_key {
            Some(BASE64.decode(key_b64).map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid signing secret: {}", e))
            })?)
        } else {
            None
        };

        Ok(Self {
            config,
            secret_key,
            nonce_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Check if signing is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.secret_key.is_some()
    }

    /// Check if a path requires signing
    pub fn requires_signing(&self, path: &str) -> bool {
        if !self.is_enabled() {
            return false;
        }

        // Check exempt paths first
        for exempt in &self.config.exempt_paths {
            if path.starts_with(exempt) {
                return false;
            }
        }

        // If required_paths is empty, all non-exempt paths require signing
        if self.config.required_paths.is_empty() {
            return true;
        }

        // Check if path matches any required path
        for required in &self.config.required_paths {
            if path.starts_with(required) {
                return true;
            }
        }

        false
    }

    /// Validate a signed request
    pub fn validate(&self, request: &SignedRequest) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let secret = self
            .secret_key
            .as_ref()
            .ok_or_else(|| VectorizerError::ConfigurationError("Signing not configured".into()))?;

        // Validate timestamp (replay protection)
        self.validate_timestamp(request.timestamp)?;

        // Validate nonce (replay protection)
        self.validate_nonce(&request.nonce)?;

        // Compute expected signature
        let canonical = request.canonical_string();
        let expected_signature = self.compute_signature(secret, &canonical);

        // Constant-time comparison
        if !constant_time_compare(&request.signature, &expected_signature) {
            warn!(
                "Invalid request signature for {} {}",
                request.method, request.path
            );
            return Err(VectorizerError::AuthenticationError(
                "Invalid request signature".to_string(),
            ));
        }

        debug!(
            "Request signature validated for {} {}",
            request.method, request.path
        );
        Ok(())
    }

    /// Validate request timestamp
    fn validate_timestamp(&self, timestamp: u64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| VectorizerError::InternalError(format!("Time error: {}", e)))?
            .as_secs();

        let diff = if timestamp > now {
            timestamp - now
        } else {
            now - timestamp
        };

        if diff > self.config.max_clock_skew_secs {
            warn!(
                "Request timestamp too far from current time: {} vs {} (diff: {}s)",
                timestamp, now, diff
            );
            return Err(VectorizerError::AuthenticationError(
                "Request timestamp expired or too far in future".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate nonce (replay protection)
    fn validate_nonce(&self, nonce: &str) -> Result<()> {
        let now = SystemTime::now();

        // Check if nonce was already used
        {
            let cache = self.nonce_cache.read();
            if cache.contains_key(nonce) {
                warn!("Duplicate nonce detected: {}", nonce);
                return Err(VectorizerError::AuthenticationError(
                    "Duplicate request nonce (possible replay attack)".to_string(),
                ));
            }
        }

        // Add nonce to cache
        {
            let mut cache = self.nonce_cache.write();

            // Clean up expired nonces if cache is getting large
            if cache.len() >= MAX_NONCE_CACHE_SIZE {
                self.cleanup_nonces(&mut cache);
            }

            cache.insert(nonce.to_string(), NonceEntry { seen_at: now });
        }

        Ok(())
    }

    /// Clean up expired nonces
    fn cleanup_nonces(&self, cache: &mut HashMap<String, NonceEntry>) {
        let expiry = Duration::from_secs(NONCE_EXPIRY_SECS);
        let now = SystemTime::now();

        cache.retain(|_, entry| {
            now.duration_since(entry.seen_at)
                .map(|d| d < expiry)
                .unwrap_or(false)
        });

        debug!("Cleaned up nonce cache, {} entries remaining", cache.len());
    }

    /// Compute HMAC-SHA256 signature
    fn compute_signature(&self, secret: &[u8], message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    }

    /// Generate a signature for testing/client use
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        timestamp: u64,
        nonce: &str,
        body: Option<&[u8]>,
    ) -> Result<String> {
        let secret = self
            .secret_key
            .as_ref()
            .ok_or_else(|| VectorizerError::ConfigurationError("Signing not configured".into()))?;

        let body_hash = match body {
            Some(b) if !b.is_empty() => {
                let mut hasher = Sha256::new();
                hasher.update(b);
                format!("{:x}", hasher.finalize())
            }
            _ => String::new(),
        };

        let canonical = format!(
            "{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            path,
            timestamp,
            nonce,
            body_hash
        );

        Ok(self.compute_signature(secret, &canonical))
    }

    /// Get current Unix timestamp
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    /// Generate a unique nonce
    pub fn generate_nonce() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> usize {
        self.nonce_cache.read().len()
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

/// Helper to create signing headers for a request
pub fn create_signing_headers(
    validator: &RequestSigningValidator,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
) -> Result<HashMap<String, String>> {
    let timestamp = RequestSigningValidator::current_timestamp();
    let nonce = RequestSigningValidator::generate_nonce();
    let signature = validator.sign_request(method, path, timestamp, &nonce, body)?;

    let mut headers = HashMap::new();
    headers.insert(HEADER_SIGNATURE.to_string(), signature);
    headers.insert(HEADER_TIMESTAMP.to_string(), timestamp.to_string());
    headers.insert(HEADER_NONCE.to_string(), nonce);

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator() -> RequestSigningValidator {
        let config = SigningConfig {
            enabled: true,
            // Test secret: base64("test_secret_key_32bytes_long!!!")
            secret_key: Some("dGVzdF9zZWNyZXRfa2V5XzMyYnl0ZXNfbG9uZyEhIQ==".to_string()),
            max_clock_skew_secs: 300,
            required_paths: vec![],
            exempt_paths: vec!["/health".to_string()],
        };
        RequestSigningValidator::new(config).unwrap()
    }

    #[test]
    fn test_canonical_string() {
        let request = SignedRequest {
            method: "POST".to_string(),
            path: "/collections".to_string(),
            timestamp: 1234567890,
            nonce: "unique-nonce-123".to_string(),
            body_hash: "abc123".to_string(),
            signature: "sig".to_string(),
        };

        let canonical = request.canonical_string();
        assert_eq!(
            canonical,
            "POST\n/collections\n1234567890\nunique-nonce-123\nabc123"
        );
    }

    #[test]
    fn test_sign_and_validate() {
        let validator = create_test_validator();
        let timestamp = RequestSigningValidator::current_timestamp();
        let nonce = RequestSigningValidator::generate_nonce();
        let body = b"test body";

        // Sign the request
        let signature = validator
            .sign_request("POST", "/test", timestamp, &nonce, Some(body))
            .unwrap();

        // Create signed request
        let mut body_hasher = Sha256::new();
        body_hasher.update(body);
        let body_hash = format!("{:x}", body_hasher.finalize());

        let request = SignedRequest {
            method: "POST".to_string(),
            path: "/test".to_string(),
            timestamp,
            nonce,
            body_hash,
            signature,
        };

        // Validate should succeed
        assert!(validator.validate(&request).is_ok());
    }

    #[test]
    fn test_invalid_signature() {
        let validator = create_test_validator();
        let timestamp = RequestSigningValidator::current_timestamp();

        let request = SignedRequest {
            method: "POST".to_string(),
            path: "/test".to_string(),
            timestamp,
            nonce: "nonce1".to_string(),
            body_hash: String::new(),
            signature: "invalid_signature".to_string(),
        };

        let result = validator.validate(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_timestamp() {
        let validator = create_test_validator();
        let old_timestamp = RequestSigningValidator::current_timestamp() - 600; // 10 minutes ago

        let signature = validator
            .sign_request("GET", "/test", old_timestamp, "nonce2", None)
            .unwrap();

        let request = SignedRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            timestamp: old_timestamp,
            nonce: "nonce2".to_string(),
            body_hash: String::new(),
            signature,
        };

        let result = validator.validate(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_replay_protection() {
        let validator = create_test_validator();
        let timestamp = RequestSigningValidator::current_timestamp();
        let nonce = "same-nonce-123".to_string();

        let signature = validator
            .sign_request("GET", "/test", timestamp, &nonce, None)
            .unwrap();

        let request = SignedRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            timestamp,
            nonce: nonce.clone(),
            body_hash: String::new(),
            signature: signature.clone(),
        };

        // First request should succeed
        assert!(validator.validate(&request).is_ok());

        // Same nonce should fail (replay attack)
        let request2 = SignedRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            timestamp,
            nonce,
            body_hash: String::new(),
            signature,
        };
        assert!(validator.validate(&request2).is_err());
    }

    #[test]
    fn test_requires_signing() {
        let validator = create_test_validator();

        // Health endpoint is exempt
        assert!(!validator.requires_signing("/health"));
        assert!(!validator.requires_signing("/health/live"));

        // Other endpoints require signing
        assert!(validator.requires_signing("/collections"));
        assert!(validator.requires_signing("/api/v1/test"));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc", "abc"));
        assert!(!constant_time_compare("abc", "abd"));
        assert!(!constant_time_compare("abc", "ab"));
        assert!(!constant_time_compare("", "a"));
        assert!(constant_time_compare("", ""));
    }

    #[test]
    fn test_create_signing_headers() {
        let validator = create_test_validator();
        let headers = create_signing_headers(&validator, "POST", "/test", Some(b"body")).unwrap();

        assert!(headers.contains_key(HEADER_SIGNATURE));
        assert!(headers.contains_key(HEADER_TIMESTAMP));
        assert!(headers.contains_key(HEADER_NONCE));
    }
}
