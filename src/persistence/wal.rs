use crate::persistence::types::{WALEntry, Operation, Transaction};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{info, warn, error, debug};
use serde_json;
use thiserror::Error;

/// WAL (Write-Ahead Log) implementation for atomic operations
pub struct WriteAheadLog {
    /// Path to WAL file
    file_path: PathBuf,
    /// Async file handle
    file: Arc<AsyncMutex<File>>,
    /// Current sequence number
    sequence: AtomicU64,
    /// Checkpoint threshold (number of operations)
    checkpoint_threshold: usize,
    /// Maximum WAL size in MB
    max_wal_size_mb: usize,
    /// Background checkpoint interval
    checkpoint_interval: Duration,
}

/// WAL errors
#[derive(Debug, Error)]
pub enum WALError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("WAL corruption detected at sequence {0}")]
    Corruption(u64),
    
    #[error("Checkpoint failed: {0}")]
    CheckpointFailed(String),
    
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
    
    #[error("WAL file not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid sequence number: expected {expected}, got {actual}")]
    InvalidSequence { expected: u64, actual: u64 },
}

/// WAL configuration
#[derive(Debug, Clone)]
pub struct WALConfig {
    /// Checkpoint threshold (number of operations)
    pub checkpoint_threshold: usize,
    /// Maximum WAL size in MB
    pub max_wal_size_mb: usize,
    /// Checkpoint interval
    pub checkpoint_interval: Duration,
    /// Enable compression
    pub compression: bool,
}

impl Default for WALConfig {
    fn default() -> Self {
        Self {
            checkpoint_threshold: 1000,
            max_wal_size_mb: 100,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
            compression: false,
        }
    }
}

impl WriteAheadLog {
    /// Create new WAL instance
    pub async fn new<P: AsRef<Path>>(file_path: P, config: WALConfig) -> Result<Self, WALError> {
        let file_path = file_path.as_ref().to_path_buf();
        
        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(WALError::IoError)?;
        }

        // Open or create WAL file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .write(true)
            .open(&file_path)
            .map_err(WALError::IoError)?;

        let mut wal = Self {
            file_path,
            file: Arc::new(AsyncMutex::new(file)),
            sequence: AtomicU64::new(0),
            checkpoint_threshold: config.checkpoint_threshold,
            max_wal_size_mb: config.max_wal_size_mb,
            checkpoint_interval: config.checkpoint_interval,
        };

        // Initialize sequence number from existing WAL
        wal.initialize_sequence().await?;
        
        info!("WAL initialized at {} with sequence {}", 
              wal.file_path.display(), 
              wal.sequence.load(Ordering::Relaxed));

        Ok(wal)
    }

    /// Initialize sequence number from existing WAL file
    async fn initialize_sequence(&self) -> Result<(), WALError> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(0)).map_err(WALError::IoError)?;
        
        let reader = BufReader::new(&*file);
        let mut max_sequence = 0u64;
        
        for line in reader.lines() {
            let line = line.map_err(WALError::IoError)?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: WALEntry = serde_json::from_str(&line).map_err(WALError::SerializationError)?;
            max_sequence = max_sequence.max(entry.sequence);
        }
        
        self.sequence.store(max_sequence, Ordering::Relaxed);
        debug!("WAL sequence initialized to {}", max_sequence);
        
        Ok(())
    }

    /// Append entry to WAL
    pub async fn append(&self, collection_id: &str, operation: Operation) -> Result<u64, WALError> {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        
        let entry = WALEntry {
            sequence,
            timestamp: chrono::Utc::now(),
            operation,
            collection_id: collection_id.to_string(),
            transaction_id: None,
        };

        self.write_entry(&entry).await?;
        
        debug!("WAL entry {} appended for collection {}", sequence, collection_id);
        Ok(sequence)
    }

    /// Append transaction to WAL
    pub async fn append_transaction(&self, transaction: &Transaction) -> Result<u64, WALError> {
        let mut entries = Vec::new();
        let base_sequence = self.sequence.load(Ordering::Relaxed);
        
        for (i, operation) in transaction.operations.iter().enumerate() {
            let sequence = base_sequence + i as u64;
            let entry = WALEntry {
                sequence,
                timestamp: transaction.started_at,
                operation: operation.clone(),
                collection_id: transaction.collection_id.clone(),
                transaction_id: Some(transaction.id),
            };
            entries.push(entry);
        }
        
        // Write all entries atomically
        let mut file = self.file.lock().await;
        for entry in entries {
            self.write_entry_to_file(&mut *file, &entry)?;
        }
        file.flush().map_err(WALError::IoError)?;
        
        // Update sequence number
        self.sequence.fetch_add(transaction.operations.len() as u64, Ordering::Relaxed);
        
        info!("Transaction {} appended with {} operations", 
              transaction.id, transaction.operations.len());
        
        Ok(base_sequence)
    }

    /// Write single entry to WAL
    async fn write_entry(&self, entry: &WALEntry) -> Result<(), WALError> {
        let mut file = self.file.lock().await;
        self.write_entry_to_file(&mut *file, entry)?;
        file.flush().map_err(WALError::IoError)?;
        Ok(())
    }

    /// Write entry to file handle
    fn write_entry_to_file(&self, file: &mut File, entry: &WALEntry) -> Result<(), WALError> {
        let json = serde_json::to_string(entry).map_err(WALError::SerializationError)?;
        writeln!(file, "{}", json).map_err(WALError::IoError)?;
        Ok(())
    }

    /// Read entries from WAL starting from given sequence
    pub async fn read_from(&self, sequence: u64) -> Result<Vec<WALEntry>, WALError> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(0)).map_err(WALError::IoError)?;
        
        let reader = BufReader::new(&*file);
        let mut entries = Vec::new();
        
        for line in reader.lines() {
            let line = line.map_err(WALError::IoError)?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: WALEntry = serde_json::from_str(&line).map_err(WALError::SerializationError)?;
            
            if entry.sequence >= sequence {
                entries.push(entry);
            }
        }
        
        debug!("Read {} WAL entries from sequence {}", entries.len(), sequence);
        Ok(entries)
    }

    /// Read all entries for a specific collection
    pub async fn read_collection_entries(&self, collection_id: &str) -> Result<Vec<WALEntry>, WALError> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(0)).map_err(WALError::IoError)?;
        
        let reader = BufReader::new(&*file);
        let mut entries = Vec::new();
        
        for line in reader.lines() {
            let line = line.map_err(WALError::IoError)?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: WALEntry = serde_json::from_str(&line).map_err(WALError::SerializationError)?;
            
            if entry.collection_id == collection_id {
                entries.push(entry);
            }
        }
        
        debug!("Read {} WAL entries for collection {}", entries.len(), collection_id);
        Ok(entries)
    }

    /// Create checkpoint (truncate WAL)
    pub async fn checkpoint(&self) -> Result<u64, WALError> {
        let current_sequence = self.sequence.load(Ordering::Relaxed);
        
        // Truncate WAL file (no checkpoint marker needed)
        self.truncate().await?;
        
        info!("WAL checkpoint created at sequence {}", current_sequence);
        Ok(current_sequence)
    }

    /// Truncate WAL file
    async fn truncate(&self) -> Result<(), WALError> {
        // Create new WAL file
        let temp_path = self.file_path.with_extension("wal.tmp");
        let new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(WALError::IoError)?;

        // Replace old file with new one
        drop(new_file);
        fs::rename(&temp_path, &self.file_path).await.map_err(WALError::IoError)?;
        
        // Reopen file for appending
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .write(true)
            .open(&self.file_path)
            .map_err(WALError::IoError)?;

        *self.file.lock().await = file;
        
        debug!("WAL truncated successfully");
        Ok(())
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::Relaxed)
    }

    /// Get WAL file size in bytes
    pub async fn file_size(&self) -> Result<u64, WALError> {
        match fs::metadata(&self.file_path).await {
            Ok(metadata) => Ok(metadata.len()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0), // File doesn't exist yet
            Err(e) => Err(WALError::IoError(e)),
        }
    }

    /// Check if checkpoint is needed
    pub async fn should_checkpoint(&self) -> Result<bool, WALError> {
        let file_size = self.file_size().await?;
        let file_size_mb = file_size / (1024 * 1024);
        
        Ok(file_size_mb >= self.max_wal_size_mb as u64)
    }

    /// Validate WAL integrity
    pub async fn validate_integrity(&self) -> Result<(), WALError> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(0)).map_err(WALError::IoError)?;
        
        let reader = BufReader::new(&*file);
        let mut expected_sequence = 0u64;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(WALError::IoError)?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: WALEntry = serde_json::from_str(&line)
                .map_err(|e| WALError::SerializationError(e))?;
            
            if entry.sequence != expected_sequence {
                return Err(WALError::InvalidSequence {
                    expected: expected_sequence,
                    actual: entry.sequence,
                });
            }
            
            expected_sequence += 1;
        }
        
        info!("WAL integrity validation passed");
        Ok(())
    }

    /// Recover from WAL (replay operations)
    pub async fn recover(&self, collection_id: &str) -> Result<Vec<WALEntry>, WALError> {
        let entries = self.read_collection_entries(collection_id).await?;
        
        info!("Recovering {} WAL entries for collection {}", entries.len(), collection_id);
        
        // Validate entries are in correct sequence
        for (i, entry) in entries.iter().enumerate() {
            if entry.sequence != i as u64 {
                return Err(WALError::InvalidSequence {
                    expected: i as u64,
                    actual: entry.sequence,
                });
            }
        }
        
        Ok(entries)
    }

    /// Get WAL statistics
    pub async fn get_stats(&self) -> Result<WALStats, WALError> {
        let file_size = self.file_size().await?;
        let current_sequence = self.current_sequence();
        
        // Count entries
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(0)).map_err(WALError::IoError)?;
        
        let reader = BufReader::new(&*file);
        let mut entry_count = 0;
        
        for line in reader.lines() {
            let line = line.map_err(WALError::IoError)?;
            if !line.trim().is_empty() {
                entry_count += 1;
            }
        }
        
        Ok(WALStats {
            file_size_bytes: file_size,
            entry_count,
            current_sequence,
            checkpoint_threshold: self.checkpoint_threshold,
            max_size_mb: self.max_wal_size_mb,
        })
    }
}

/// WAL statistics
#[derive(Debug, Clone)]
pub struct WALStats {
    pub file_size_bytes: u64,
    pub entry_count: usize,
    pub current_sequence: u64,
    pub checkpoint_threshold: usize,
    pub max_size_mb: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_wal_creation_and_append() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();
        
        let operation = Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        let sequence = wal.append("collection1", operation).await.unwrap();
        assert_eq!(sequence, 0);
        
        let stats = wal.get_stats().await.unwrap();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.current_sequence, 1);
    }

    #[tokio::test]
    async fn test_wal_transaction() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();
        
        let mut transaction = Transaction::new(1, "collection1".to_string());
        transaction.add_operation(Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0],
            metadata: HashMap::new(),
        });
        transaction.add_operation(Operation::InsertVector {
            vector_id: "vec2".to_string(),
            data: vec![3.0, 4.0],
            metadata: HashMap::new(),
        });
        
        let sequence = wal.append_transaction(&transaction).await.unwrap();
        assert_eq!(sequence, 0);
        
        let stats = wal.get_stats().await.unwrap();
        assert_eq!(stats.entry_count, 2);
        assert_eq!(stats.current_sequence, 2);
    }

    #[tokio::test]
    async fn test_wal_read_entries() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();
        
        // Add some entries
        wal.append("collection1", Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0],
            metadata: HashMap::new(),
        }).await.unwrap();
        
        wal.append("collection1", Operation::InsertVector {
            vector_id: "vec2".to_string(),
            data: vec![2.0],
            metadata: HashMap::new(),
        }).await.unwrap();
        
        // Read entries
        let entries = wal.read_from(0).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 0);
        assert_eq!(entries[1].sequence, 1);
        
        // Read from specific sequence
        let entries = wal.read_from(1).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].sequence, 1);
    }

    #[tokio::test]
    async fn test_wal_checkpoint() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();
        
        // Add some entries
        wal.append("collection1", Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0],
            metadata: HashMap::new(),
        }).await.unwrap();
        
        wal.append("collection1", Operation::InsertVector {
            vector_id: "vec2".to_string(),
            data: vec![2.0],
            metadata: HashMap::new(),
        }).await.unwrap();
        
        // Create checkpoint
        let checkpoint_sequence = wal.checkpoint().await.unwrap();
        assert_eq!(checkpoint_sequence, 2);
        
        // Verify WAL is truncated (empty after checkpoint)
        let stats = wal.get_stats().await.unwrap();
        assert_eq!(stats.entry_count, 0); // Empty after checkpoint
    }

    #[tokio::test]
    async fn test_wal_integrity_validation() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();
        
        // Add some entries
        wal.append("collection1", Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0],
            metadata: HashMap::new(),
        }).await.unwrap();
        
        wal.append("collection1", Operation::InsertVector {
            vector_id: "vec2".to_string(),
            data: vec![2.0],
            metadata: HashMap::new(),
        }).await.unwrap();
        
        // Validate integrity
        wal.validate_integrity().await.unwrap();
    }
}
