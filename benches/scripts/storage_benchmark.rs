//! Storage format benchmark - Compare legacy vs .vecdb performance

#![allow(clippy::uninlined_format_args)]

use std::fs::{self, File};
use tracing::{info, error, warn, debug};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use vectorizer::models::{Payload, Vector};
use vectorizer::storage::{StorageReader, StorageWriter};

/// Benchmark result for a single test
#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    load_time_ms: u128,
    save_time_ms: u128,
    memory_mb: f64,
    storage_size_mb: f64,
    compression_ratio: f64,
}

/// Run all storage benchmarks
fn main() {
    tracing::info!("🚀 Storage Format Benchmark");
    tracing::info!("=====================================\n");

    let results = run_benchmarks();

    tracing::info!("\n📊 Results Summary:");
    tracing::info!("=====================================");
    print_results_table(&results);

    tracing::info!("\n📈 Analysis:");
    analyze_results(&results);
}

/// Run all benchmark scenarios
fn run_benchmarks() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();

    // Test with different dataset sizes
    let test_sizes = vec![
        (100, "Small (100 vectors)"),
        (1_000, "Medium (1K vectors)"),
        (10_000, "Large (10K vectors)"),
    ];

    for (size, label) in test_sizes {
        tracing::info!("\n📦 Testing {} dataset:", label);

        // Legacy format
        let legacy_result = benchmark_legacy_format(size);
        results.push(legacy_result);

        // Compact format
        let compact_result = benchmark_compact_format(size);
        results.push(compact_result);
    }

    results
}

/// Benchmark legacy file format
fn benchmark_legacy_format(vector_count: usize) -> BenchmarkResult {
    let temp_base = std::env::temp_dir().join(format!("vectorizer_bench_{}", std::process::id()));
    fs::create_dir_all(&temp_base).unwrap();
    let data_dir = temp_base.join("data");
    let collection_dir = data_dir.join("test_collection");
    fs::create_dir_all(&collection_dir).unwrap();

    // Create test data
    let vectors = create_test_vectors(vector_count);

    // Measure save time
    let save_start = Instant::now();
    save_vectors_legacy(&collection_dir, &vectors);
    let save_time = save_start.elapsed();

    // Get storage size
    let storage_size = get_directory_size(&data_dir);

    // Measure load time
    let load_start = Instant::now();
    let _loaded = load_vectors_legacy(&collection_dir);
    let load_time = load_start.elapsed();

    BenchmarkResult {
        name: format!("Legacy ({} vectors)", vector_count),
        load_time_ms: load_time.as_millis(),
        save_time_ms: save_time.as_millis(),
        memory_mb: estimate_memory_usage(&vectors),
        storage_size_mb: storage_size as f64 / 1_048_576.0,
        compression_ratio: 1.0, // No compression
    }
}

/// Benchmark compact .vecdb format
fn benchmark_compact_format(vector_count: usize) -> BenchmarkResult {
    let temp_base =
        std::env::temp_dir().join(format!("vectorizer_bench_compact_{}", std::process::id()));
    fs::create_dir_all(&temp_base).unwrap();
    let data_dir = temp_base.join("data");
    let collections_dir = data_dir.join("collections");
    let collection_dir = collections_dir.join("test_collection");
    fs::create_dir_all(&collection_dir).unwrap();

    // Create test data
    let vectors = create_test_vectors(vector_count);

    // Save in legacy format first
    save_vectors_legacy(&collection_dir, &vectors);

    // Measure compaction time
    let save_start = Instant::now();
    let writer = StorageWriter::new(&data_dir, 3);
    let index = writer.write_archive(&collections_dir).unwrap();
    let save_time = save_start.elapsed();

    // Get storage size
    let vecdb_path = data_dir.join("vectorizer.vecdb");
    let storage_size = fs::metadata(&vecdb_path).unwrap().len();

    // Measure load time
    let load_start = Instant::now();
    let reader = StorageReader::new(&data_dir).unwrap();
    let _files = reader.read_collection_files("test_collection").unwrap();
    let load_time = load_start.elapsed();

    BenchmarkResult {
        name: format!("Compact ({} vectors)", vector_count),
        load_time_ms: load_time.as_millis(),
        save_time_ms: save_time.as_millis(),
        memory_mb: estimate_memory_usage(&vectors),
        storage_size_mb: storage_size as f64 / 1_048_576.0,
        compression_ratio: index.compression_ratio,
    }
}

/// Create test vectors
fn create_test_vectors(count: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..384)
                .map(|j| (i as f32 * 0.01) + (j as f32 * 0.001))
                .collect();

            Vector::with_payload(
                format!("vec_{}", i),
                data,
                Payload::new(serde_json::json!({
                    "index": i,
                    "category": "test",
                })),
            )
        })
        .collect()
}

/// Save vectors in legacy format
fn save_vectors_legacy(dir: &Path, vectors: &[Vector]) {
    for (i, vector) in vectors.iter().enumerate() {
        let file_path = dir.join(format!("vector_{}.bin", i));
        let data = bincode::serialize(&vector.data).unwrap();

        let mut file = File::create(&file_path).unwrap();
        file.write_all(&data).unwrap();
    }
}

/// Load vectors from legacy format
fn load_vectors_legacy(dir: &PathBuf) -> Vec<Vec<f32>> {
    let mut vectors = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|e| e.to_str()) == Some("bin") {
            let data = fs::read(entry.path()).unwrap();
            let vector: Vec<f32> = bincode::deserialize(&data).unwrap_or_default();
            vectors.push(vector);
        }
    }

    vectors
}

/// Get total directory size recursively
fn get_directory_size(dir: &PathBuf) -> u64 {
    let mut total = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    total += metadata.len();
                }
            } else if path.is_dir() {
                total += get_directory_size(&path);
            }
        }
    }

    total
}

/// Estimate memory usage for vectors
fn estimate_memory_usage(vectors: &[Vector]) -> f64 {
    let size_per_vector = std::mem::size_of::<f32>() * 384; // 384 dimensions
    let total_bytes = size_per_vector * vectors.len();
    total_bytes as f64 / 1_048_576.0
}

/// Print results as a formatted table
fn print_results_table(results: &[BenchmarkResult]) {
    tracing::info!(
        "\n{:<25} {:>12} {:>12} {:>12} {:>12} {:>10}",
        "Test", "Load (ms)", "Save (ms)", "Size (MB)", "Memory (MB)", "Compress"
    );
    tracing::info!("{}", "-".repeat(95));

    for result in results {
        tracing::info!(
            "{:<25} {:>12} {:>12} {:>12.2} {:>12.2} {:>9.1}%",
            result.name,
            result.load_time_ms,
            result.save_time_ms,
            result.storage_size_mb,
            result.memory_mb,
            result.compression_ratio * 100.0
        );
    }
}

/// Analyze and compare results
fn analyze_results(results: &[BenchmarkResult]) {
    let legacy_results: Vec<_> = results
        .iter()
        .filter(|r| r.name.contains("Legacy"))
        .collect();
    let compact_results: Vec<_> = results
        .iter()
        .filter(|r| r.name.contains("Compact"))
        .collect();

    if legacy_results.len() != compact_results.len() {
        return;
    }

    tracing::info!("\n📊 Performance Comparison:");
    tracing::info!("=====================================");

    for (legacy, compact) in legacy_results.iter().zip(compact_results.iter()) {
        let load_improvement = ((legacy.load_time_ms as f64 - compact.load_time_ms as f64)
            / legacy.load_time_ms as f64)
            * 100.0;

        let save_improvement = ((legacy.save_time_ms as f64 - compact.save_time_ms as f64)
            / legacy.save_time_ms as f64)
            * 100.0;

        let size_reduction =
            ((legacy.storage_size_mb - compact.storage_size_mb) / legacy.storage_size_mb) * 100.0;

        tracing::info!("\n{}", legacy.name.split(" ").next().unwrap_or("Test"));
        tracing::info!(
            "  Load time: {}{:.1}%",
            if load_improvement > 0.0 {
                "✅ "
            } else {
                "⚠️ "
            },
            load_improvement.abs()
        );
        tracing::info!(
            "  Save time: {}{:.1}%",
            if save_improvement > 0.0 {
                "✅ "
            } else {
                "⚠️ "
            },
            save_improvement.abs()
        );
        tracing::info!("  Storage size: ✅ {:.1}% reduction", size_reduction);
        tracing::info!(
            "  Compression ratio: {:.1}%",
            compact.compression_ratio * 100.0
        );
    }

    // Overall recommendation
    tracing::info!("\n💡 Recommendation:");
    let avg_size_reduction: f64 = legacy_results
        .iter()
        .zip(compact_results.iter())
        .map(|(l, c)| ((l.storage_size_mb - c.storage_size_mb) / l.storage_size_mb) * 100.0)
        .sum::<f64>()
        / legacy_results.len() as f64;

    if avg_size_reduction > 30.0 {
        tracing::info!(
            "  ✅ .vecdb format provides significant space savings ({:.1}% reduction)",
            avg_size_reduction
        );
        tracing::info!("  ✅ Recommended for production use");
    } else if avg_size_reduction > 10.0 {
        tracing::info!(
            "  ⚠️  .vecdb format provides moderate space savings ({:.1}% reduction)",
            avg_size_reduction
        );
        tracing::info!("  ℹ️  Consider for large datasets");
    } else {
        tracing::info!(
            "  ℹ️  .vecdb format provides minimal space savings ({:.1}% reduction)",
            avg_size_reduction
        );
        tracing::info!("  ℹ️  Use based on other benefits (snapshots, portability)");
    }
}
