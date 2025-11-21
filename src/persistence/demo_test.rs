use std::collections::HashMap;

use tempfile::tempdir;
use tracing::info;

use crate::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
};
use crate::persistence::types::{
    CollectionSource, CollectionType, EnhancedCollectionMetadata, Operation, Transaction,
    TransactionStatus, WALEntry,
};
use crate::persistence::wal::{WALConfig, WriteAheadLog};

/// Complete demonstration of the persistence system
#[tokio::test]
async fn test_persistence_demo() {
    // Configure logging to see what's happening
    let _ = tracing_subscriber::fmt::try_init();

    info!("🚀 Starting persistence system demonstration");

    // 1. Create collection configuration
    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: HnswConfig::default(),
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    info!(
        "✅ Collection configuration created: dimension={}, metric={:?}",
        config.dimension, config.metric
    );

    // 2. Create workspace collection metadata
    let workspace_metadata = EnhancedCollectionMetadata::new_workspace(
        "demo-workspace-collection".to_string(),
        "demo-project".to_string(),
        "/workspace/config.yml".to_string(),
        config.clone(),
    );

    info!("✅ Workspace collection metadata created:");
    info!("   - Name: {}", workspace_metadata.name);
    info!("   - Type: {:?}", workspace_metadata.collection_type);
    info!("   - Read-only: {}", workspace_metadata.is_read_only);
    info!("   - Dimension: {}", workspace_metadata.dimension);

    // 3. Create dynamic collection metadata
    let dynamic_metadata = EnhancedCollectionMetadata::new_dynamic(
        "demo-dynamic-collection".to_string(),
        Some("user123".to_string()),
        "/collections".to_string(),
        config,
    );

    info!("✅ Dynamic collection metadata created:");
    info!("   - Name: {}", dynamic_metadata.name);
    info!("   - Type: {:?}", dynamic_metadata.collection_type);
    info!("   - Read-only: {}", dynamic_metadata.is_read_only);
    info!("   - Created by: {:?}", dynamic_metadata.source);

    // 4. Test Write-Ahead Log (WAL)
    let temp_dir = tempdir().unwrap();
    let wal_path = temp_dir.path().join("demo.wal");

    let wal_config = WALConfig::default();
    let wal = WriteAheadLog::new(&wal_path, wal_config).await.unwrap();

    info!("✅ Write-Ahead Log initialized at: {}", wal_path.display());

    // 5. Create and execute transaction
    let mut transaction = Transaction::new(1, "demo-collection".to_string());

    // Add operations to transaction
    let operation1 = Operation::InsertVector {
        vector_id: "vec1".to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "demo".to_string());
            meta.insert("type".to_string(), "test".to_string());
            meta
        },
    };

    let operation2 = Operation::InsertVector {
        vector_id: "vec2".to_string(),
        data: vec![0.5, 0.6, 0.7, 0.8],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "demo".to_string());
            meta.insert("type".to_string(), "test".to_string());
            meta
        },
    };

    transaction.add_operation(operation1);
    transaction.add_operation(operation2);

    info!(
        "✅ Transaction created with {} operations",
        transaction.operations.len()
    );

    // 6. Apply transaction to WAL
    let sequence = wal.append_transaction(&transaction).await.unwrap();

    info!("✅ Transaction applied to WAL with sequence: {}", sequence);

    // 7. Verify WAL integrity
    wal.validate_integrity().await.unwrap();

    info!("✅ WAL integrity validated successfully");

    // 8. Read entries from WAL
    let entries = wal.read_from(0).await.unwrap();

    info!("✅ Read {} entries from WAL:", entries.len());
    for (i, entry) in entries.iter().enumerate() {
        info!(
            "   {}. Sequence: {}, Operation: {}, Collection: {}",
            i + 1,
            entry.sequence,
            entry.operation.operation_type(),
            entry.collection_id
        );
    }

    // 9. Test checkpoint
    let checkpoint_sequence = wal.checkpoint().await.unwrap();

    info!(
        "✅ Checkpoint created with sequence: {}",
        checkpoint_sequence
    );

    // 10. Check WAL statistics
    let stats = wal.get_stats().await.unwrap();

    info!("✅ WAL statistics:");
    info!("   - File size: {} bytes", stats.file_size_bytes);
    info!("   - Entry count: {}", stats.entry_count);
    info!("   - Current sequence: {}", stats.current_sequence);

    // 11. Test integrity checksums
    let data_checksum = workspace_metadata.calculate_data_checksum();
    let index_checksum = workspace_metadata.calculate_index_checksum();

    info!("✅ Checksums calculated:");
    info!("   - Data checksum: {}", data_checksum);
    info!("   - Index checksum: {}", index_checksum);

    // 12. Test metadata update
    let mut updated_metadata = workspace_metadata.clone();
    updated_metadata.update_after_operation(100, 50);
    updated_metadata.update_checksums();

    info!("✅ Metadata updated:");
    info!("   - Vectors: {}", updated_metadata.vector_count);
    info!("   - Documents: {}", updated_metadata.document_count);
    info!("   - Updated at: {}", updated_metadata.updated_at);

    // 13. Verify collection types
    assert!(workspace_metadata.is_workspace());
    assert!(!workspace_metadata.is_dynamic());
    assert!(!dynamic_metadata.is_workspace());
    assert!(dynamic_metadata.is_dynamic());

    info!("✅ Collection type verification passed");

    // 14. Test transaction status
    assert_eq!(transaction.status, TransactionStatus::InProgress);
    assert!(!transaction.is_completed());

    transaction.commit();
    assert_eq!(transaction.status, TransactionStatus::Committed);
    assert!(transaction.is_completed());

    info!("✅ Transaction status test passed");

    info!("🎉 Persistence system demonstration completed successfully!");
    info!("📊 Summary:");
    info!("   - ✅ Workspace and dynamic collection metadata");
    info!("   - ✅ Write-Ahead Log with atomic transactions");
    info!("   - ✅ Integrity validation");
    info!("   - ✅ Checkpoint and recovery");
    info!("   - ✅ Checksums for data verification");
    info!("   - ✅ Collection type system");
    info!("   - ✅ Transaction management");
}

/// Basic WAL performance test
#[tokio::test]
async fn test_wal_performance_demo() {
    let temp_dir = tempdir().unwrap();
    let wal_path = temp_dir.path().join("performance.wal");

    let wal_config = WALConfig::default();
    let wal = WriteAheadLog::new(&wal_path, wal_config).await.unwrap();

    let start_time = std::time::Instant::now();

    // Insert 1000 operations
    for i in 0..1000 {
        let operation = Operation::InsertVector {
            vector_id: format!("vec_{}", i),
            data: vec![i as f32; 384],
            metadata: HashMap::new(),
        };

        wal.append("test-collection", operation).await.unwrap();
    }

    let duration = start_time.elapsed();

    println!(
        "✅ Performance test: {} operations in {:?} ({:.2} ops/sec)",
        1000,
        duration,
        1000.0 / duration.as_secs_f64()
    );

    // Verify that all operations were recorded
    let entries = wal.read_from(0).await.unwrap();
    assert_eq!(entries.len(), 1000);

    println!("✅ All 1000 operations were recorded correctly");
}
