//! MCP Connection Manager
//!
//! This module provides connection pooling and management for MCP operations.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub active_connections: usize,
    pub total_connections: u64,
    pub average_connection_duration_ms: f64,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// MCP Connection Manager
pub struct MCPConnectionManager {
    active_connections: Arc<RwLock<HashMap<String, Instant>>>,
    connection_stats: Arc<RwLock<ConnectionStats>>,
    max_connections: usize,
    connection_timeout: Duration,
}

impl MCPConnectionManager {
    /// Create a new connection manager
    pub fn new(max_connections: usize, connection_timeout: Duration) -> Self {
        Self {
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            connection_stats: Arc::new(RwLock::new(ConnectionStats {
                active_connections: 0,
                total_connections: 0,
                average_connection_duration_ms: 0.0,
                last_activity: None,
            })),
            max_connections,
            connection_timeout,
        }
    }

    /// Register a new connection
    pub async fn register_connection(&self, connection_id: String) -> Result<(), String> {
        let mut connections = self.active_connections.write().await;
        let mut stats = self.connection_stats.write().await;

        if connections.len() >= self.max_connections {
            return Err("Maximum connections reached".to_string());
        }

        connections.insert(connection_id.clone(), Instant::now());
        stats.active_connections = connections.len();
        stats.total_connections += 1;
        stats.last_activity = Some(chrono::Utc::now());

        Ok(())
    }

    /// Unregister a connection
    pub async fn unregister_connection(&self, connection_id: &str) {
        let mut connections = self.active_connections.write().await;
        let mut stats = self.connection_stats.write().await;

        if let Some(start_time) = connections.remove(connection_id) {
            let duration = start_time.elapsed();
            stats.active_connections = connections.len();

            // Update average connection duration
            if stats.total_connections > 0 {
                let total_duration =
                    stats.average_connection_duration_ms * (stats.total_connections - 1) as f64;
                stats.average_connection_duration_ms =
                    (total_duration + duration.as_millis() as f64) / stats.total_connections as f64;
            }
        }
    }

    /// Clean up expired connections
    pub async fn cleanup_expired_connections(&self) {
        let mut connections = self.active_connections.write().await;
        let mut stats = self.connection_stats.write().await;

        let now = Instant::now();
        let expired_connections: Vec<String> = connections
            .iter()
            .filter(|(_, start_time)| now.duration_since(**start_time) > self.connection_timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for connection_id in expired_connections {
            connections.remove(&connection_id);
        }

        stats.active_connections = connections.len();
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        self.connection_stats.read().await.clone()
    }

    /// Check if we can accept new connections
    pub async fn can_accept_connection(&self) -> bool {
        let connections = self.active_connections.read().await;
        connections.len() < self.max_connections
    }

    /// Get current active connection count
    pub async fn active_connection_count(&self) -> usize {
        let connections = self.active_connections.read().await;
        connections.len()
    }
}

impl Default for MCPConnectionManager {
    fn default() -> Self {
        Self::new(100, Duration::from_secs(300)) // 100 max connections, 5 minute timeout
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let manager = MCPConnectionManager::new(10, Duration::from_secs(60));
        assert_eq!(manager.active_connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_registration() {
        let manager = MCPConnectionManager::new(10, Duration::from_secs(60));

        let result = manager
            .register_connection("test-connection".to_string())
            .await;
        assert!(result.is_ok());
        assert_eq!(manager.active_connection_count().await, 1);

        manager.unregister_connection("test-connection").await;
        assert_eq!(manager.active_connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_max_connections() {
        let manager = MCPConnectionManager::new(2, Duration::from_secs(60));

        assert!(
            manager
                .register_connection("conn1".to_string())
                .await
                .is_ok()
        );
        assert!(
            manager
                .register_connection("conn2".to_string())
                .await
                .is_ok()
        );
        assert!(
            manager
                .register_connection("conn3".to_string())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_connection_stats() {
        let manager = MCPConnectionManager::new(10, Duration::from_secs(60));

        manager
            .register_connection("test".to_string())
            .await
            .unwrap();
        let stats = manager.get_stats().await;

        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.total_connections, 1);
        assert!(stats.last_activity.is_some());
    }
}
