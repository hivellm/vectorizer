//! Replication Performance Benchmarks
//!
//! Measures performance characteristics of the replication system:
//! - Replication log append throughput
//! - Snapshot creation/application speed
//! - Master-to-replica latency
//! - High-volume replication throughput

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::sync::Arc;
use std::time::Duration;
use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector};
use vectorizer::replication::{ReplicationLog, VectorOperation, CollectionConfigData};

// ============================================================================
// Replication Log Benchmarks
// ============================================================================

fn bench_replication_log_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("replication_log_append");
    
    for log_size in [1000, 10_000, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("circular_buffer", log_size),
            &log_size,
            |b, &size| {
                let log = Arc::new(ReplicationLog::new(size));
                let mut counter = 0;
                
                b.iter(|| {
                    let op = VectorOperation::InsertVector {
                        collection: "bench".to_string(),
                        id: format!("vec_{}", counter),
                        vector: vec![counter as f32; 128],
                        payload: None,
                    };
                    counter += 1;
                    log.append(op)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_replication_log_retrieve(c: &mut Criterion) {
    let mut group = c.benchmark_group("replication_log_retrieve");
    
    // Pre-populate log with operations
    let log = Arc::new(ReplicationLog::new(10_000));
    for i in 0..10_000 {
        let op = VectorOperation::InsertVector {
            collection: "bench".to_string(),
            id: format!("vec_{}", i),
            vector: vec![i as f32; 128],
            payload: None,
        };
        log.append(op);
    }
    
    for ops_to_retrieve in [10, 100, 1000, 5000] {
        group.throughput(Throughput::Elements(ops_to_retrieve));
        group.bench_with_input(
            BenchmarkId::new("get_operations", ops_to_retrieve),
            &ops_to_retrieve,
            |b, &count| {
                let from_offset = 10_000 - count as u64;
                b.iter(|| {
                    black_box(log.get_operations(from_offset))
                });
            },
        );
    }
    
    group.finish();
}

fn bench_replication_log_concurrent_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("replication_log_concurrent");
    group.measurement_time(Duration::from_secs(10));
    
    for num_threads in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(num_threads));
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &threads| {
                b.iter(|| {
                    let log = Arc::new(ReplicationLog::new(100_000));
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    
                    runtime.block_on(async {
                        let mut handles = vec![];
                        
                        for thread_id in 0..threads {
                            let log_clone = Arc::clone(&log);
                            let handle = tokio::spawn(async move {
                                for i in 0..1000 {
                                    let op = VectorOperation::InsertVector {
                                        collection: format!("col_{}", thread_id),
                                        id: format!("vec_{}", i),
                                        vector: vec![thread_id as f32; 64],
                                        payload: None,
                                    };
                                    log_clone.append(op);
                                }
                            });
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Snapshot Benchmarks
// ============================================================================

fn bench_snapshot_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_creation");
    group.measurement_time(Duration::from_secs(15));
    
    for num_vectors in [100, 1000, 10_000] {
        group.throughput(Throughput::Elements(num_vectors));
        group.bench_with_input(
            BenchmarkId::new("vectors", num_vectors),
            &num_vectors,
            |b, &count| {
                // Setup: create store with vectors
                let store = setup_store_with_vectors(count as usize, 128);
                
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        black_box(
                            vectorizer::replication::sync::create_snapshot(&store, 100)
                                .await
                                .unwrap()
                        )
                    });
            },
        );
    }
    
    group.finish();
}

fn bench_snapshot_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_application");
    group.measurement_time(Duration::from_secs(15));
    
    for num_vectors in [100, 1000, 10_000] {
        group.throughput(Throughput::Elements(num_vectors));
        group.bench_with_input(
            BenchmarkId::new("vectors", num_vectors),
            &num_vectors,
            |b, &count| {
                // Setup: create snapshot
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let store1 = setup_store_with_vectors(count as usize, 128);
                let snapshot = runtime
                    .block_on(vectorizer::replication::sync::create_snapshot(&store1, 100))
                    .unwrap();
                
                b.to_async(runtime)
                    .iter(|| async {
                        let store2 = VectorStore::new();
                        black_box(
                            vectorizer::replication::sync::apply_snapshot(&store2, &snapshot)
                                .await
                                .unwrap()
                        )
                    });
            },
        );
    }
    
    group.finish();
}

fn bench_snapshot_with_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_dimensions");
    group.measurement_time(Duration::from_secs(15));
    
    for dimension in [64, 128, 384, 768, 1536] {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("dim", dimension),
            &dimension,
            |b, &dim| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let store = setup_store_with_vectors(1000, dim);
                
                b.to_async(runtime)
                    .iter(|| async {
                        black_box(
                            vectorizer::replication::sync::create_snapshot(&store, 100)
                                .await
                                .unwrap()
                        )
                    });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Operation Serialization Benchmarks
// ============================================================================

fn bench_operation_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_serialization");
    
    let operations = vec![
        (
            "CreateCollection",
            VectorOperation::CreateCollection {
                name: "bench".to_string(),
                config: CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
            },
        ),
        (
            "InsertVector_128d",
            VectorOperation::InsertVector {
                collection: "bench".to_string(),
                id: "vec_1".to_string(),
                vector: vec![0.5; 128],
                payload: None,
            },
        ),
        (
            "InsertVector_1536d",
            VectorOperation::InsertVector {
                collection: "bench".to_string(),
                id: "vec_1".to_string(),
                vector: vec![0.5; 1536],
                payload: None,
            },
        ),
        (
            "DeleteVector",
            VectorOperation::DeleteVector {
                collection: "bench".to_string(),
                id: "vec_1".to_string(),
            },
        ),
    ];
    
    for (name, op) in operations {
        group.bench_function(name, |b| {
            b.iter(|| {
                let serialized = bincode::serialize(&op).unwrap();
                black_box(serialized)
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn setup_store_with_vectors(count: usize, dimension: usize) -> VectorStore {
    let store = VectorStore::new();
    
    let config = CollectionConfig {
        dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    
    store.create_collection("bench", config).unwrap();
    
    // Insert vectors in batches
    let batch_size = 100;
    for batch in 0..(count / batch_size) {
        let mut vectors = Vec::new();
        for i in 0..batch_size {
            let idx = batch * batch_size + i;
            let data: Vec<f32> = (0..dimension)
                .map(|j| ((idx + j) as f32) * 0.01)
                .collect();
            vectors.push(Vector {
                id: format!("vec_{}", idx),
                data,
                payload: None,
            });
        }
        store.insert("bench", vectors).unwrap();
    }
    
    store
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_replication_log_append,
    bench_replication_log_retrieve,
    bench_replication_log_concurrent_append,
    bench_snapshot_creation,
    bench_snapshot_application,
    bench_snapshot_with_dimensions,
    bench_operation_serialization,
);

criterion_main!(benches);

