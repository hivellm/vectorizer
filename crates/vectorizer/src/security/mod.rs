//! Security Module
//!
//! This module provides advanced security features including:
//! - Rate limiting (per API key and global)
//! - TLS/mTLS support
//! - Audit logging
//! - Role-based access control (RBAC)
//! - Payload encryption (ECC + AES-256-GCM)
//!
//! # Features
//!
//! - **Rate Limiting**: Prevent abuse with configurable limits per API key
//! - **TLS**: Encrypted communication with rustls
//! - **Audit Logging**: Track all API calls for compliance
//! - **RBAC**: Fine-grained permissions (Viewer, Editor, Admin)
//! - **Payload Encryption**: End-to-end encryption for sensitive payload data

pub mod audit;
pub mod payload_encryption;
pub mod rate_limit;
pub mod rbac;
pub mod tls;

pub use audit::AuditLogger;
pub use payload_encryption::{EncryptedPayload, EncryptionError, encrypt_payload};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use rbac::{Permission, Role};
