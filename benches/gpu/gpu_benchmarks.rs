//! GPU Performance Benchmarks
//!
//! Benchmarks comparing Metal GPU performance vs CPU performance
//! for various vector operations.
//!
//! Run with: cargo bench --features hive-gpu --bench gpu_benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vectorizer::{CollectionConfig, VectorStore};
use vectorizer::models::{DistanceMetric, HnswConfig, QuantizationConfig, CompressionConfig, Vector};

/// Create test vectors of specified dimension and count
fn create_test_vectors(dimension: usize, count: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| Vector {
            id: format!("vec_{}", i),
            data: vec![i as f32; dimension],
            payload: None,
        })
        .collect()
}

/// Create VectorStore with CPU mode
fn create_cpu_store() -> VectorStore {
    VectorStore::new()
}

/// Create VectorStore with auto GPU detection (Metal on macOS)
#[cfg(all(feature = "hive-gpu", target_os = "macos"))]
fn create_gpu_store() -> VectorStore {
    VectorStore::new_auto()
}

/// Benchmark: Single vector search
fn bench_single_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_search");
    
    let dimension = 512;
    let collection_size = 1000;
    let query = vec![0.5; dimension];
    
    // CPU benchmark
    {
        let store = create_cpu_store();
        let config = CollectionConfig {
            dimension,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::SQ { bits: 8 },
            compression: CompressionConfig::default(),
            normalization: None,
        };
        
        store.create_collection("bench_cpu", config).unwrap();
        let vectors = create_test_vectors(dimension, collection_size);
        store.insert("bench_cpu", vectors).unwrap();
        
        group.bench_function("cpu", |b| {
            b.iter(|| {
                store.search("bench_cpu", black_box(&query), 10).unwrap()
            });
        });
    }
    
    // GPU benchmark (Metal on macOS only)
    #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
    {
        let store = create_gpu_store();
        let config = CollectionConfig {
            dimension,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::SQ { bits: 8 },
            compression: CompressionConfig::default(),
            normalization: None,
        };
        
        store.create_collection("bench_gpu", config).unwrap();
        let vectors = create_test_vectors(dimension, collection_size);
        store.insert("bench_gpu", vectors).unwrap();
        
        group.bench_function("metal_gpu", |b| {
            b.iter(|| {
                store.search("bench_gpu", black_box(&query), 10).unwrap()
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Batch vector insertion
fn bench_batch_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_insert");
    
    let dimension = 512;
    
    for batch_size in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        
        let vectors = create_test_vectors(dimension, *batch_size);
        
        // CPU benchmark
        {
            let store = create_cpu_store();
            let config = CollectionConfig {
                dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: CompressionConfig::default(),
                normalization: None,
            };
            
            group.bench_with_input(
                BenchmarkId::new("cpu", batch_size),
                batch_size,
                |b, _| {
                    b.iter_with_setup(
                        || {
                            store.create_collection("bench_cpu_batch", config.clone()).ok();
                            vectors.clone()
                        },
                        |v| {
                            store.insert("bench_cpu_batch", black_box(v)).unwrap();
                            store.delete_collection("bench_cpu_batch").ok();
                        }
                    );
                }
            );
        }
        
        // GPU benchmark (Metal on macOS only)
        #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
        {
            let store = create_gpu_store();
            let config = CollectionConfig {
                dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: CompressionConfig::default(),
                normalization: None,
            };
            
            group.bench_with_input(
                BenchmarkId::new("metal_gpu", batch_size),
                batch_size,
                |b, _| {
                    b.iter_with_setup(
                        || {
                            store.create_collection("bench_gpu_batch", config.clone()).ok();
                            vectors.clone()
                        },
                        |v| {
                            store.insert("bench_gpu_batch", black_box(v)).unwrap();
                            store.delete_collection("bench_gpu_batch").ok();
                        }
                    );
                }
            );
        }
    }
    
    group.finish();
}

/// Benchmark: Batch search operations
fn bench_batch_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_search");
    
    let dimension = 512;
    let collection_size = 10000;
    
    // Prepare queries
    let query_counts = [10, 50, 100];
    
    for query_count in query_counts.iter() {
        group.throughput(Throughput::Elements(*query_count as u64));
        
        let queries: Vec<Vec<f32>> = (0..*query_count)
            .map(|i| vec![i as f32 * 0.1; dimension])
            .collect();
        
        // CPU benchmark
        {
            let store = create_cpu_store();
            let config = CollectionConfig {
                dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: CompressionConfig::default(),
                normalization: None,
            };
            
            store.create_collection("bench_cpu_search", config).unwrap();
            let vectors = create_test_vectors(dimension, collection_size);
            store.insert("bench_cpu_search", vectors).unwrap();
            
            group.bench_with_input(
                BenchmarkId::new("cpu", query_count),
                query_count,
                |b, _| {
                    b.iter(|| {
                        for query in &queries {
                            store.search("bench_cpu_search", black_box(query), 10).unwrap();
                        }
                    });
                }
            );
        }
        
        // GPU benchmark (Metal on macOS only)
        #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
        {
            let store = create_gpu_store();
            let config = CollectionConfig {
                dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: CompressionConfig::default(),
                normalization: None,
            };
            
            store.create_collection("bench_gpu_search", config).unwrap();
            let vectors = create_test_vectors(dimension, collection_size);
            store.insert("bench_gpu_search", vectors).unwrap();
            
            group.bench_with_input(
                BenchmarkId::new("metal_gpu", query_count),
                query_count,
                |b, _| {
                    b.iter(|| {
                        for query in &queries {
                            store.search("bench_gpu_search", black_box(query), 10).unwrap();
                        }
                    });
                }
            );
        }
    }
    
    group.finish();
}

/// Benchmark: Different vector dimensions
fn bench_by_dimension(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_by_dimension");
    
    let dimensions = [128, 256, 512, 1024];
    let collection_size = 1000;
    
    for dimension in dimensions.iter() {
        let query = vec![0.5; *dimension];
        
        // CPU benchmark
        {
            let store = create_cpu_store();
            let config = CollectionConfig {
                dimension: *dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: CompressionConfig::default(),
                normalization: None,
            };
            
            let collection_name = format!("bench_cpu_dim{}", dimension);
            store.create_collection(&collection_name, config).unwrap();
            let vectors = create_test_vectors(*dimension, collection_size);
            store.insert(&collection_name, vectors).unwrap();
            
            group.bench_with_input(
                BenchmarkId::new("cpu", dimension),
                dimension,
                |b, _| {
                    b.iter(|| {
                        store.search(&collection_name, black_box(&query), 10).unwrap()
                    });
                }
            );
        }
        
        // GPU benchmark (Metal on macOS only)
        #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
        {
            let store = create_gpu_store();
            let config = CollectionConfig {
                dimension: *dimension,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: CompressionConfig::default(),
                normalization: None,
            };
            
            let collection_name = format!("bench_gpu_dim{}", dimension);
            store.create_collection(&collection_name, config).unwrap();
            let vectors = create_test_vectors(*dimension, collection_size);
            store.insert(&collection_name, vectors).unwrap();
            
            group.bench_with_input(
                BenchmarkId::new("metal_gpu", dimension),
                dimension,
                |b, _| {
                    b.iter(|| {
                        store.search(&collection_name, black_box(&query), 10).unwrap()
                    });
                }
            );
        }
    }
    
    group.finish();
}

// Define benchmark groups
criterion_group!(
    benches,
    bench_single_search,
    bench_batch_insert,
    bench_batch_search,
    bench_by_dimension
);

criterion_main!(benches);

