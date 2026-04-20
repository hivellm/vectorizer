//! Index Scale Performance Benchmark
//!
//! Tests performance degradation as index size increases:
//! - Dataset sizes: 1K, 5K, 10K, 25K, 50K, 100K, 250K, 500K vectors
//! - Measures: build time, search latency, throughput, memory, quality
//! - Identifies optimal collection size limits
//!
//! Usage:
//!   cargo run --release --bin scale_benchmark

use std::collections::HashSet;
use tracing::{info, error, warn, debug};
use std::fs;
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
// use vectorizer::VectorStore;
use vectorizer::db::{OptimizedHnswConfig, OptimizedHnswIndex};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::evaluation::{QueryResult, evaluate_search_quality};
use vectorizer::models::DistanceMetric;

/// Scale benchmark result for a specific dataset size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleBenchmarkResult {
    pub dataset_size: usize,
    pub dimension: usize,

    // Build metrics
    pub index_build_time_ms: f64,
    pub vectors_per_second: f64,

    // Memory metrics
    pub index_memory_mb: f64,
    pub bytes_per_vector: f64,

    // Search performance metrics
    pub avg_search_latency_us: f64,
    pub p95_search_latency_us: f64,
    pub p99_search_latency_us: f64,
    pub search_throughput_qps: f64,

    // Quality metrics
    pub map_score: f64,
    pub recall_at_10: f64,

    // Efficiency metrics
    pub memory_efficiency: f64, // MAP per MB
    pub speed_efficiency: f64,  // QPS per GB
}

/// Complete scale benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleBenchmarkReport {
    pub timestamp: String,
    pub dimension: usize,
    pub test_sizes: Vec<usize>,
    pub results: Vec<ScaleBenchmarkResult>,
    pub recommendations: ScaleRecommendations,
}

/// Recommendations based on scale analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleRecommendations {
    pub optimal_collection_size: usize,
    pub maximum_recommended_size: usize,
    pub performance_inflection_point: usize,
    pub memory_limit_gb: f64,
    pub quality_threshold_map: f64,
}

/// Test dataset for scale benchmarking
#[derive(Debug)]
struct TestDataset {
    documents: Vec<String>,
    queries: Vec<String>,
    ground_truth: Vec<HashSet<String>>,
}

impl TestDataset {
    fn generate_scaled_dataset(
        base_docs: &[String],
        base_queries: &[String],
        target_size: usize,
    ) -> Self {
        tracing::info!("🔧 Generating dataset with {target_size} vectors...");

        // Create embedding manager for semantic ground truth
        let mut manager = EmbeddingManager::new();
        let bm25 = Bm25Embedding::new(512);
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.set_default_provider("bm25").unwrap();

        // Build vocabulary from base documents
        if let Some(provider) = manager.get_provider_mut("bm25")
            && let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>()
        {
            bm25.build_vocabulary(base_docs);
        }

        // Generate documents by duplicating and slightly modifying base docs
        let mut documents = Vec::new();
        let mut vector_ids = Vec::new();

        let base_size = base_docs.len();
        let repetitions = target_size.div_ceil(base_size);

        for rep in 0..repetitions {
            for (i, base_doc) in base_docs.iter().enumerate() {
                if documents.len() >= target_size {
                    break;
                }

                // Create variation by adding prefix/suffix
                let doc = if rep == 0 {
                    base_doc.clone()
                } else {
                    format!("Version {}: {}", rep + 1, base_doc)
                };

                documents.push(doc);
                vector_ids.push(format!("doc_{rep}_{i}"));
            }
        }

        // Trim to exact target size
        documents.truncate(target_size);
        vector_ids.truncate(target_size);

        // Generate ground truth for queries
        let ground_truth = Self::generate_ground_truth(&documents, base_queries);

        Self {
            documents,
            queries: base_queries.to_vec(),
            ground_truth,
        }
    }

    fn generate_ground_truth(docs: &[String], queries: &[String]) -> Vec<HashSet<String>> {
        // Create embedding manager for semantic ground truth
        let mut manager = EmbeddingManager::new();
        let bm25 = Bm25Embedding::new(512);
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.set_default_provider("bm25").unwrap();

        // Build vocabulary
        if let Some(provider) = manager.get_provider_mut("bm25")
            && let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>()
        {
            bm25.build_vocabulary(docs);
        }

        queries
            .iter()
            .map(|query| {
                let mut relevant = HashSet::new();

                // Get semantic similarity using BM25 embeddings
                if let Ok(query_emb) = manager.embed(query) {
                    // Calculate similarity to all documents
                    let mut similarities: Vec<(usize, f32)> = docs
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, doc)| {
                            if let Ok(doc_emb) = manager.embed(doc) {
                                // Cosine similarity
                                let dot_product: f32 = query_emb
                                    .iter()
                                    .zip(doc_emb.iter())
                                    .map(|(a, b)| a * b)
                                    .sum();
                                let norm_q: f32 =
                                    query_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                                let norm_d: f32 = doc_emb.iter().map(|x| x * x).sum::<f32>().sqrt();

                                if norm_q > 0.0 && norm_d > 0.0 {
                                    let similarity = dot_product / (norm_q * norm_d);
                                    Some((idx, similarity))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Sort by similarity (highest first)
                    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                    // Take top 10 most similar documents
                    for (idx, similarity) in similarities.into_iter().take(10) {
                        if similarity > 0.1 {
                            // Minimum similarity threshold
                            relevant.insert(format!("doc_0_{idx}")); // Use consistent ID format
                        }
                    }
                }

                // Fallback: lexical matching if semantic fails
                if relevant.is_empty() {
                    let query_lower = query.to_lowercase();
                    let keywords: Vec<&str> = query_lower.split_whitespace().collect();

                    for (idx, doc) in docs.iter().enumerate() {
                        let doc_lower = doc.to_lowercase();
                        let matching = keywords.iter().filter(|kw| doc_lower.contains(*kw)).count();

                        if matching >= 1 {
                            relevant.insert(format!("doc_0_{idx}"));
                        }
                    }
                }

                // Ensure at least 3 relevant documents per query
                if relevant.len() < 3 {
                    for i in 0..3.min(docs.len()) {
                        relevant.insert(format!("doc_0_{i}"));
                    }
                }

                relevant
            })
            .collect()
    }
}

/// Benchmark a specific dataset size
fn benchmark_dataset_size(
    dataset: &TestDataset,
    dimension: usize,
) -> Result<ScaleBenchmarkResult, Box<dyn std::error::Error>> {
    tracing::info!(
        "🚀 Benchmarking dataset size: {} vectors",
        dataset.documents.len()
    );

    // Create embedding manager
    let mut manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(dimension);
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25")?;

    // Build vocabulary
    if let Some(provider) = manager.get_provider_mut("bm25")
        && let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>()
    {
        bm25.build_vocabulary(&dataset.documents);
    }

    // Generate embeddings
    tracing::info!("  📊 Generating embeddings...");
    let embeddings_start = Instant::now();

    let embeddings: Vec<Vec<f32>> = dataset
        .documents
        .iter()
        .filter_map(|doc| manager.embed(doc).ok())
        .collect();

    let embeddings_time = embeddings_start.elapsed().as_millis() as f64;

    tracing::info!(
        "  ✅ Generated {} embeddings in {:.1}s",
        embeddings.len(),
        embeddings_time / 1000.0
    );

    // Build HNSW index
    tracing::info!("  🏗️  Building HNSW index...");
    let index_build_start = Instant::now();

    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: embeddings.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    let batch_vectors: Vec<(String, Vec<f32>)> = (0..embeddings.len())
        .map(|i| (format!("doc_0_{i}"), embeddings[i].clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let index_build_time_ms = index_build_start.elapsed().as_millis() as f64;
    let vectors_per_second = embeddings.len() as f64 / (index_build_time_ms / 1000.0);

    // Memory measurement
    let index_memory_mb = (embeddings.len() * dimension * 4) as f64 / 1_048_576.0; // 4 bytes per f32
    let bytes_per_vector = (dimension * 4) as f64;

    tracing::info!(
        "  ✅ Index built in {:.1}s ({:.0} vectors/sec)",
        index_build_time_ms / 1000.0,
        vectors_per_second
    );
    tracing::info!("  ✅ Memory usage: {index_memory_mb:.2} MB ({bytes_per_vector:.0} bytes/vector)");

    // Search performance benchmarking
    tracing::info!("  🔍 Benchmarking search performance...");

    let mut search_times = Vec::new();
    let mut query_results = Vec::new();

    // Warmup
    for _ in 0..3 {
        let query_emb = manager.embed(&dataset.queries[0])?;
        let _ = index.search(&query_emb, 10)?;
    }

    // Actual benchmarking
    let search_start = Instant::now();

    for (query_idx, query) in dataset.queries.iter().enumerate() {
        let query_emb = manager.embed(query)?;

        let query_start = Instant::now();
        let results = index.search(&query_emb, 10)?;
        let elapsed_us = query_start.elapsed().as_micros() as f64;
        search_times.push(elapsed_us);

        // Convert results for quality evaluation
        let query_result: Vec<QueryResult> = results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance,
            })
            .collect();

        query_results.push((query_result, dataset.ground_truth[query_idx].clone()));
    }

    let total_search_time_ms = search_start.elapsed().as_millis() as f64;

    // Calculate search metrics
    let avg_search_latency_us = search_times.iter().sum::<f64>() / search_times.len() as f64;
    let search_throughput_qps = dataset.queries.len() as f64 / (total_search_time_ms / 1000.0);

    // Quality evaluation
    let eval_metrics = evaluate_search_quality(query_results, 10);

    let result = ScaleBenchmarkResult {
        dataset_size: dataset.documents.len(),
        dimension,
        index_build_time_ms,
        vectors_per_second,
        index_memory_mb,
        bytes_per_vector,
        avg_search_latency_us,
        p95_search_latency_us: percentile(&search_times, 95),
        p99_search_latency_us: percentile(&search_times, 99),
        search_throughput_qps,
        map_score: f64::from(eval_metrics.mean_average_precision),
        recall_at_10: eval_metrics.precision_at_k.last().copied().unwrap_or(0.0) as f64,
        memory_efficiency: eval_metrics.mean_average_precision as f64 / index_memory_mb,
        speed_efficiency: search_throughput_qps / (index_memory_mb / 1024.0), // QPS per GB
    };

    tracing::info!(
        "  ✅ Search: {:.0} μs avg, {:.1} QPS",
        avg_search_latency_us, search_throughput_qps
    );
    tracing::info!(
        "  ✅ Quality: MAP={:.4}, Recall@10={:.3}",
        result.map_score, result.recall_at_10
    );

    Ok(result)
}

/// Generate comprehensive scale analysis
fn analyze_scale_performance(results: &[ScaleBenchmarkResult]) -> ScaleRecommendations {
    // Find inflection points

    // Quality degradation: find where MAP drops significantly
    let mut quality_inflection = results[0].dataset_size;
    let baseline_map = results[0].map_score;

    for result in results.iter().skip(1) {
        if baseline_map - result.map_score > 0.1 {
            // 10% quality drop
            quality_inflection = result.dataset_size;
            break;
        }
    }

    // Performance degradation: find where latency doubles
    let mut performance_inflection = results[0].dataset_size;
    let baseline_latency = results[0].avg_search_latency_us;

    for result in results.iter().skip(1) {
        if result.avg_search_latency_us > baseline_latency * 2.0 {
            performance_inflection = result.dataset_size;
            break;
        }
    }

    // Memory limit: find where memory usage becomes problematic (> 8GB)
    let mut memory_limit = results.last().unwrap().dataset_size;
    for result in results.iter() {
        if result.index_memory_mb > 8192.0 {
            // 8GB
            memory_limit = result.dataset_size;
            break;
        }
    }

    // Optimal size: balance between quality and performance
    let mut optimal_size = results[0].dataset_size;
    let mut best_score = 0.0;

    for result in results.iter() {
        // Score = Quality * Efficiency * (1 / normalized_latency)
        let quality_score = result.map_score;
        let efficiency_score = result.memory_efficiency;
        let latency_penalty = results[0].avg_search_latency_us / result.avg_search_latency_us;
        let score = quality_score * efficiency_score * latency_penalty;

        if score > best_score {
            best_score = score;
            optimal_size = result.dataset_size;
        }
    }

    ScaleRecommendations {
        optimal_collection_size: optimal_size,
        maximum_recommended_size: performance_inflection.min(quality_inflection),
        performance_inflection_point: performance_inflection,
        memory_limit_gb: memory_limit as f64 / 1024.0,
        quality_threshold_map: baseline_map * 0.9, // 90% of baseline quality
    }
}

/// Generate comprehensive report
fn generate_scale_report(report: &ScaleBenchmarkReport) -> String {
    let mut md = String::new();

    md.push_str("# Index Scale Performance Benchmark\n\n");
    md.push_str(&format!("**Generated**: {}\n\n", report.timestamp));

    md.push_str("## Test Configuration\n\n");
    md.push_str(&format!("- **Dimension**: {}D\n", report.dimension));
    md.push_str(&format!("- **Test Sizes**: {:?}\n", report.test_sizes));
    md.push_str("- **HNSW Config**: M=16, ef_construction=200\n");
    md.push_str("- **Distance**: Cosine\n\n");

    md.push_str("## Performance Results\n\n");

    md.push_str("| Size | Build Time | Memory | Search Latency | QPS | MAP | Recall@10 |\n");
    md.push_str("|------|-----------|--------|----------------|-----|-----|-----------|\n");

    for result in &report.results {
        let size_str = if result.dataset_size >= 1000 {
            format!("{}K", result.dataset_size / 1000)
        } else {
            format!("{}K", result.dataset_size)
        };

        md.push_str(&format!(
            "| {} | {:.1}s | {:.1}MB | {:.0}μs | {:.0} | {:.3} | {:.3} |\n",
            size_str,
            result.index_build_time_ms / 1000.0,
            result.index_memory_mb,
            result.avg_search_latency_us,
            result.search_throughput_qps,
            result.map_score,
            result.recall_at_10
        ));
    }

    md.push_str("\n## Scale Analysis\n\n");

    // Performance degradation
    md.push_str("### Performance Degradation\n\n");
    md.push_str("| Size | Latency | QPS | Memory | Quality |\n");
    md.push_str("|------|---------|-----|--------|---------|\n");

    if let Some(first) = report.results.first() {
        let baseline_latency = first.avg_search_latency_us;
        let baseline_qps = first.search_throughput_qps;
        let baseline_map = first.map_score;

        for result in &report.results {
            let latency_ratio = result.avg_search_latency_us / baseline_latency;
            let qps_ratio = result.search_throughput_qps / baseline_qps;
            let quality_ratio = result.map_score / baseline_map;

            let size_str = if result.dataset_size >= 1000 {
                format!("{}K", result.dataset_size / 1000)
            } else {
                format!("{}K", result.dataset_size)
            };

            md.push_str(&format!(
                "| {} | {:.1}x | {:.1}x | {:.1}MB | {:.1}x |\n",
                size_str, latency_ratio, qps_ratio, result.index_memory_mb, quality_ratio
            ));
        }
    }

    md.push_str("\n## Recommendations\n\n");

    let rec = &report.recommendations;
    md.push_str(&format!(
        "### Optimal Collection Size: **{}K vectors**\n\n",
        rec.optimal_collection_size / 1000
    ));
    md.push_str(&format!(
        "### Maximum Recommended Size: **{}K vectors**\n\n",
        rec.maximum_recommended_size / 1000
    ));
    md.push_str(&format!(
        "### Performance Inflection Point: **{}K vectors**\n\n",
        rec.performance_inflection_point / 1000
    ));
    md.push_str(&format!(
        "### Memory Limit: **{:.1} GB**\n\n",
        rec.memory_limit_gb
    ));
    md.push_str(&format!(
        "### Quality Threshold: **MAP ≥ {:.3}**\n\n",
        rec.quality_threshold_map
    ));

    md.push_str("## Implementation Guidelines\n\n");
    md.push_str("### Collection Size Strategy\n\n");
    md.push_str("1. **Small Collections** (< 10K): No sharding needed\n");
    md.push_str("2. **Medium Collections** (10K-50K): Consider sharding by domain\n");
    md.push_str("3. **Large Collections** (50K+): Implement collection sharding\n\n");

    md.push_str("### Monitoring Thresholds\n\n");
    md.push_str("- **Latency Alert**: > 5ms average search\n");
    md.push_str("- **Quality Alert**: < 90% of baseline MAP\n");
    md.push_str("- **Memory Alert**: > 4GB per collection\n\n");

    md.push_str("---\n\n");
    md.push_str("*Scale benchmark report generated automatically*\n");

    md
}

fn percentile(values: &[f64], p: usize) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let idx = ((p as f64 / 100.0) * sorted.len() as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    tracing::info!("📈 Index Scale Performance Benchmark");
    tracing::info!("====================================\n");

    let dimension = 512;
    let test_sizes = vec![
        1_000, 5_000, 10_000, 25_000, 50_000, 100_000, 250_000, 500_000,
    ];

    tracing::info!("📊 Configuration:");
    tracing::info!("  - Dimension: {}D", dimension);
    tracing::info!("  - Test sizes: {:?} vectors", test_sizes);
    tracing::info!("  - HNSW: M=16, ef_construction=200");
    tracing::info!();

    // Load base dataset
    tracing::info!("📂 Loading base dataset...");
    let base_docs = vec![
        "Rust is a systems programming language focused on safety and performance.".to_string(),
        "Machine learning models require large amounts of training data.".to_string(),
        "Vector databases enable efficient similarity search at scale.".to_string(),
        "HNSW algorithm provides fast approximate nearest neighbor search.".to_string(),
        "Embeddings capture semantic meaning of text documents.".to_string(),
        "Quantization reduces memory usage while maintaining search quality.".to_string(),
        "Collections in vector databases group related vectors together.".to_string(),
        "Performance benchmarks help identify optimal configurations.".to_string(),
        "Memory efficiency is crucial for large-scale vector search.".to_string(),
        "Search latency directly impacts user experience.".to_string(),
    ];

    let base_queries = vec![
        "programming language safety".to_string(),
        "machine learning training".to_string(),
        "vector database similarity search".to_string(),
        "approximate nearest neighbor".to_string(),
        "semantic text embeddings".to_string(),
        "memory quantization techniques".to_string(),
        "vector collection management".to_string(),
        "performance optimization".to_string(),
        "large scale search".to_string(),
        "user experience latency".to_string(),
    ];

    let mut results = Vec::new();

    // Test each scale
    for &size in &test_sizes {
        tracing::info!("\n{}", "=".repeat(60));
        tracing::info!("🧪 TESTING SCALE: {} vectors", size);

        let dataset = TestDataset::generate_scaled_dataset(&base_docs, &base_queries, size);

        match benchmark_dataset_size(&dataset, dimension) {
            Ok(result) => {
                results.push(result);
                tracing::info!("✅ Scale {} completed successfully", size);
            }
            Err(e) => {
                tracing::info!("❌ Failed to benchmark size {}: {}", size, e);
                // Continue with other sizes
            }
        }
    }

    // Generate analysis and recommendations
    let recommendations = analyze_scale_performance(&results);

    let report = ScaleBenchmarkReport {
        timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        dimension,
        test_sizes: test_sizes.clone(),
        results,
        recommendations,
    };

    // Generate and save report
    tracing::info!("\n📊 Generating comprehensive report...");
    let md_report = generate_scale_report(&report);

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");

    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }

    let report_path = report_dir.join(format!("scale_benchmark_{}.md", timestamp));
    fs::write(&report_path, &md_report)?;

    let json_path = report_dir.join(format!("scale_benchmark_{}.json", timestamp));
    let json_data = serde_json::to_string_pretty(&report)?;
    fs::write(&json_path, json_data)?;

    // Print final recommendations
    tracing::info!("\n🎯 FINAL RECOMMENDATIONS");
    tracing::info!("{}", "=".repeat(40));

    let rec = &report.recommendations;
    tracing::info!(
        "📏 Optimal Collection Size: {}K vectors",
        rec.optimal_collection_size / 1000
    );
    tracing::info!(
        "📏 Maximum Recommended Size: {}K vectors",
        rec.maximum_recommended_size / 1000
    );
    tracing::info!(
        "📏 Performance Inflection Point: {}K vectors",
        rec.performance_inflection_point / 1000
    );
    tracing::info!("💾 Memory Limit: {:.1} GB", rec.memory_limit_gb);
    tracing::info!(
        "🎯 Quality Threshold: MAP ≥ {:.3}",
        rec.quality_threshold_map
    );

    tracing::info!("\n📄 Full report: {}", report_path.display());
    tracing::info!("📊 JSON data: {}", json_path.display());

    tracing::info!("\n✅ Scale benchmark completed successfully!");

    Ok(())
}
