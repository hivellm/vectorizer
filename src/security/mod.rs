//! Security Module
//!
//! This module provides advanced security features including:
//! - Rate limiting (per API key and global)
//! - TLS/mTLS support
//! - Audit logging
//! - Role-based access control (RBAC)
//! - Enhanced security with MFA, threat detection, and encryption
//!
//! # Features
//!
//! - **Rate Limiting**: Prevent abuse with configurable limits per API key
//! - **TLS**: Encrypted communication with rustls
//! - **Audit Logging**: Track all API calls for compliance
//! - **RBAC**: Fine-grained permissions (Viewer, Editor, Admin)
//! - **MFA**: Multi-factor authentication support
//! - **Threat Detection**: Advanced threat detection and response
//! - **Encryption**: End-to-end encryption for data at rest and in transit

pub mod audit;
pub mod enhanced_security;
pub mod rate_limit;
pub mod rbac;
pub mod tls;

pub use audit::AuditLogger;
pub use enhanced_security::*;
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use rbac::{Permission, Role};
