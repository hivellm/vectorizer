//! Audit Logging
//!
//! This module provides audit logging for compliance and security monitoring.
//! All API calls, authentication attempts, and security events are logged.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// User or API key that performed the action
    pub principal: String,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Endpoint path
    pub endpoint: String,
    /// HTTP status code
    pub status_code: u16,
    /// Request duration in milliseconds
    pub duration_ms: u64,
    /// Client IP address
    pub client_ip: Option<String>,
    /// Correlation ID for request tracking
    pub correlation_id: Option<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Audit logger
pub struct AuditLogger {
    entries: Arc<RwLock<Vec<AuditLogEntry>>>,
    max_entries: usize,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Log an audit entry
    pub async fn log(&self, entry: AuditLogEntry) {
        let mut entries = self.entries.write().await;

        // Add new entry
        entries.push(entry.clone());

        // Trim if exceeds max
        if entries.len() > self.max_entries {
            entries.remove(0);
        }

        // Also log to tracing for aggregation
        info!(
            target: "audit",
            principal = %entry.principal,
            method = %entry.method,
            endpoint = %entry.endpoint,
            status = entry.status_code,
            duration_ms = entry.duration_ms,
            correlation_id = ?entry.correlation_id,
            "Audit log entry"
        );
    }

    /// Log an authentication attempt
    pub async fn log_auth_attempt(&self, principal: &str, success: bool, reason: Option<&str>) {
        let entry = AuditLogEntry {
            timestamp: Utc::now(),
            principal: principal.to_string(),
            method: "AUTH".to_string(),
            endpoint: "/auth".to_string(),
            status_code: if success { 200 } else { 401 },
            duration_ms: 0,
            client_ip: None,
            correlation_id: None,
            metadata: reason.map(|r| {
                serde_json::json!({
                    "success": success,
                    "reason": r
                })
            }),
        };

        self.log(entry).await;

        if !success {
            warn!(
                target: "audit.auth",
                principal = %principal,
                reason = ?reason,
                "Failed authentication attempt"
            );
        }
    }

    /// Get recent audit entries
    pub async fn get_entries(&self, limit: usize) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let start = entries.len().saturating_sub(limit);
        entries[start..].to_vec()
    }

    /// Get entry count
    pub async fn count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Clear all entries (for testing)
    #[cfg(test)]
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000) // Keep last 10k entries by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let logger = AuditLogger::new(100);
        assert_eq!(logger.count().await, 0);
    }

    #[tokio::test]
    async fn test_log_entry() {
        let logger = AuditLogger::default();

        let entry = AuditLogEntry {
            timestamp: Utc::now(),
            principal: "user@example.com".to_string(),
            method: "POST".to_string(),
            endpoint: "/collections".to_string(),
            status_code: 200,
            duration_ms: 15,
            client_ip: Some("192.168.1.1".to_string()),
            correlation_id: Some("test-id-123".to_string()),
            metadata: None,
        };

        logger.log(entry.clone()).await;
        assert_eq!(logger.count().await, 1);

        let entries = logger.get_entries(10).await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].principal, "user@example.com");
    }

    #[tokio::test]
    async fn test_log_auth_attempt() {
        let logger = AuditLogger::default();

        // Successful auth
        logger
            .log_auth_attempt("admin@example.com", true, None)
            .await;
        assert_eq!(logger.count().await, 1);

        // Failed auth
        logger
            .log_auth_attempt("hacker@evil.com", false, Some("Invalid credentials"))
            .await;
        assert_eq!(logger.count().await, 2);

        let entries = logger.get_entries(10).await;
        assert_eq!(entries[0].status_code, 200);
        assert_eq!(entries[1].status_code, 401);
    }

    #[tokio::test]
    async fn test_max_entries_limit() {
        let logger = AuditLogger::new(5);

        // Add 10 entries
        for i in 0..10 {
            let entry = AuditLogEntry {
                timestamp: Utc::now(),
                principal: format!("user{}", i),
                method: "GET".to_string(),
                endpoint: "/test".to_string(),
                status_code: 200,
                duration_ms: 10,
                client_ip: None,
                correlation_id: None,
                metadata: None,
            };
            logger.log(entry).await;
        }

        // Should only keep last 5
        assert_eq!(logger.count().await, 5);

        let entries = logger.get_entries(10).await;
        assert_eq!(entries[0].principal, "user5");
        assert_eq!(entries[4].principal, "user9");
    }

    #[tokio::test]
    async fn test_get_entries_limit() {
        let logger = AuditLogger::default();

        // Add 10 entries
        for i in 0..10 {
            let entry = AuditLogEntry {
                timestamp: Utc::now(),
                principal: format!("user{}", i),
                method: "GET".to_string(),
                endpoint: "/test".to_string(),
                status_code: 200,
                duration_ms: 10,
                client_ip: None,
                correlation_id: None,
                metadata: None,
            };
            logger.log(entry).await;
        }

        // Request only last 3
        let entries = logger.get_entries(3).await;
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].principal, "user7");
        assert_eq!(entries[2].principal, "user9");
    }
}
