//! Security Module
//!
//! This module provides advanced security features including:
//! - Rate limiting (per API key and global)
//! - TLS/mTLS support
//! - Audit logging
//! - Role-based access control (RBAC)
<<<<<<< HEAD
=======
//! - Enhanced security with MFA, threat detection, and encryption
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
//!
//! # Features
//!
//! - **Rate Limiting**: Prevent abuse with configurable limits per API key
//! - **TLS**: Encrypted communication with rustls
//! - **Audit Logging**: Track all API calls for compliance
//! - **RBAC**: Fine-grained permissions (Viewer, Editor, Admin)
<<<<<<< HEAD

pub mod audit;
=======
//! - **MFA**: Multi-factor authentication support
//! - **Threat Detection**: Advanced threat detection and response
//! - **Encryption**: End-to-end encryption for data at rest and in transit

pub mod audit;
pub mod enhanced_security;
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
pub mod rate_limit;
pub mod rbac;
pub mod tls;

pub use audit::AuditLogger;
<<<<<<< HEAD
=======
pub use enhanced_security::*;
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use rbac::{Permission, Role};
