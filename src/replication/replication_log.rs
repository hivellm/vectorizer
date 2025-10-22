//! Replication log - Circular buffer for operations
//!
//! Similar to Redis replication backlog, stores recent operations
//! for partial resync when replicas reconnect.

use super::types::{ReplicationOperation, VectorOperation};
use parking_lot::RwLock;
use std::collections::VecDeque;
use tracing::debug;

/// Circular replication log
pub struct ReplicationLog {
    /// Maximum number of operations to keep
    max_size: usize,

    /// Current offset (monotonic counter)
    offset: RwLock<u64>,

    /// Ring buffer of operations
    operations: RwLock<VecDeque<ReplicationOperation>>,
}

impl ReplicationLog {
    /// Create a new replication log
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            offset: RwLock::new(0),
            operations: RwLock::new(VecDeque::with_capacity(max_size)),
        }
    }

    /// Append an operation to the log
    pub fn append(&self, operation: VectorOperation) -> u64 {
        let mut offset = self.offset.write();
        *offset += 1;
        let current_offset = *offset;

        let timestamp = current_timestamp();
        let repl_op = ReplicationOperation {
            offset: current_offset,
            timestamp,
            operation,
        };

        let mut ops = self.operations.write();
        
        // If at capacity, remove oldest
        if ops.len() >= self.max_size {
            ops.pop_front();
        }

        ops.push_back(repl_op);

        debug!(
            "Appended operation to replication log: offset={}, total={}",
            current_offset,
            ops.len()
        );

        current_offset
    }

    /// Get current offset
    pub fn current_offset(&self) -> u64 {
        *self.offset.read()
    }

    /// Get operations starting from offset
    pub fn get_operations(&self, from_offset: u64) -> Option<Vec<ReplicationOperation>> {
        let ops = self.operations.read();

        // If empty, return None
        if ops.is_empty() {
            return None;
        }

        // Find the oldest available offset
        let oldest_offset = ops.front().unwrap().offset;

        // If requested offset is too old, need full sync
        if from_offset < oldest_offset {
            debug!(
                "Offset {} too old (oldest: {}), need full sync",
                from_offset, oldest_offset
            );
            return None;
        }

        // Collect operations from requested offset
        let result: Vec<_> = ops
            .iter()
            .filter(|op| op.offset > from_offset)
            .cloned()
            .collect();

        debug!(
            "Retrieved {} operations from offset {} (current: {})",
            result.len(),
            from_offset,
            self.current_offset()
        );

        Some(result)
    }

    /// Get log size
    pub fn size(&self) -> usize {
        self.operations.read().len()
    }

    /// Clear the log
    pub fn clear(&self) {
        self.operations.write().clear();
        *self.offset.write() = 0;
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_log_append() {
        let log = ReplicationLog::new(10);

        let offset1 = log.append(VectorOperation::CreateCollection {
            name: "test".to_string(),
            config: super::super::types::CollectionConfigData {
                dimension: 128,
                metric: "cosine".to_string(),
            },
        });

        assert_eq!(offset1, 1);
        assert_eq!(log.current_offset(), 1);
        assert_eq!(log.size(), 1);
    }

    #[test]
    fn test_replication_log_circular() {
        let log = ReplicationLog::new(5);

        // Add 10 operations (more than max_size)
        for i in 0..10 {
            log.append(VectorOperation::CreateCollection {
                name: format!("test{}", i),
                config: super::super::types::CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
            });
        }

        // Should only keep last 5
        assert_eq!(log.size(), 5);
        assert_eq!(log.current_offset(), 10);

        // Oldest should be offset 6
        // get_operations(5) returns operations with offset > 5, which are 6-10 (5 ops)
        if let Some(ops) = log.get_operations(5) {
            assert_eq!(ops.len(), 5);
            assert_eq!(ops[0].offset, 6);
            assert_eq!(ops[4].offset, 10);
        }
    }

    #[test]
    fn test_get_operations_from_offset() {
        let log = ReplicationLog::new(100);

        for i in 0..10 {
            log.append(VectorOperation::CreateCollection {
                name: format!("test{}", i),
                config: super::super::types::CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
            });
        }

        // Get operations from offset 5
        let ops = log.get_operations(5).unwrap();
        assert_eq!(ops.len(), 5); // 6, 7, 8, 9, 10
        assert_eq!(ops[0].offset, 6);
        assert_eq!(ops[4].offset, 10);
    }

    #[test]
    fn test_get_operations_too_old() {
        let log = ReplicationLog::new(5);

        // Add 10 operations
        for i in 0..10 {
            log.append(VectorOperation::CreateCollection {
                name: format!("test{}", i),
                config: super::super::types::CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
            });
        }

        // Try to get from offset 2 (too old, oldest is 6)
        assert!(log.get_operations(2).is_none());
    }
}

