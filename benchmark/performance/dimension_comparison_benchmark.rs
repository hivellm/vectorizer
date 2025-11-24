//! Dimension Comparison Performance Benchmark
//!
//! Tests performance characteristics across different vector dimensions:
//! - Dimensions: 64, 128, 256, 512, 768, 1024, 1536
//! - Measures: build time, search latency, memory usage, quality
//! - Identifies optimal dimension for different use cases
//!
//! Usage:
//!   cargo run --release --bin dimension_comparison_benchmark

use std::collections::HashSet;
use tracing::{info, error, warn, debug};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use vectorizer::db::{OptimizedHnswConfig, OptimizedHnswIndex};
use vectorizer::embedding::bm25::{BM25Config, BM25Provider};
use vectorizer::embedding::{
    EmbeddingConfig, EmbeddingManager, EmbeddingProvider, EmbeddingProviderType,
};
use vectorizer::evaluation::{QueryResult, evaluate_search_quality};
use vectorizer::models::DistanceMetric;

/// Dimension benchmark result for a specific dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionBenchmarkResult {
    pub dimension: usize,
    pub dataset_size: usize,

    // Build metrics
    pub index_build_time_ms: f64,
    pub vectors_per_second: f64,

    // Memory metrics
    pub index_memory_mb: f64,
    pub bytes_per_vector: f64,
    pub memory_efficiency: f64, // vectors per MB

    // Search performance metrics
    pub avg_search_latency_us: f64,
    pub p95_search_latency_us: f64,
    pub p99_search_latency_us: f64,
    pub search_throughput_qps: f64,

    // Quality metrics
    pub map_score: f64,
    pub recall_at_10: f64,

    // Efficiency metrics
    pub quality_per_mb: f64, // MAP per MB
    pub speed_per_mb: f64,   // QPS per MB
    pub quality_per_us: f64, // MAP per microsecond
}

/// Complete dimension benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionBenchmarkReport {
    pub timestamp: String,
    pub dataset_size: usize,
    pub test_dimensions: Vec<usize>,
    pub results: Vec<DimensionBenchmarkResult>,
    pub recommendations: DimensionRecommendations,
}

/// Recommendations based on dimension analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionRecommendations {
    pub optimal_dimension: usize,
    pub memory_efficient_dimension: usize,
    pub speed_efficient_dimension: usize,
    pub quality_efficient_dimension: usize,
    pub balanced_dimension: usize,
    pub dimension_guidelines: Vec<DimensionGuideline>,
}

/// Guidelines for specific use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionGuideline {
    pub use_case: String,
    pub recommended_dimension: usize,
    pub reasoning: String,
    pub trade_offs: String,
}

/// Test dataset for dimension benchmarking
#[derive(Debug)]
struct TestDataset {
    documents: Vec<String>,
    queries: Vec<String>,
    ground_truth: Vec<HashSet<String>>,
}

impl TestDataset {
    fn generate_dimension_dataset(
        base_docs: &[String],
        base_queries: &[String],
        target_size: usize,
    ) -> Self {
        tracing::info!("🔧 Generating dataset with {target_size} vectors...");

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
        // Use tokio runtime for async operations
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Create embedding manager for semantic ground truth
            let config = EmbeddingConfig {
                provider: EmbeddingProviderType::BM25,
                dimension: 512,
                ..Default::default()
            };
            let manager = EmbeddingManager::new(config);

            // Create and add BM25 provider
            let bm25_config = BM25Config::default();
            let bm25 = BM25Provider::new(bm25_config);

            // Build vocabulary
            let _ = bm25.add_documents(docs).await;

            let mut ground_truth = Vec::new();

            for query in queries {
                let mut relevant = HashSet::new();

                // Get semantic similarity using BM25 embeddings
                if let Ok(query_emb) = bm25.embed(query).await {
                    // Calculate similarity to all documents
                    let mut similarities: Vec<(usize, f32)> = Vec::new();

                    for (idx, doc) in docs.iter().enumerate() {
                        if let Ok(doc_emb) = bm25.embed(doc).await {
                            // Cosine similarity
                            let dot_product: f32 = query_emb
                                .iter()
                                .zip(doc_emb.iter())
                                .map(|(a, b)| a * b)
                                .sum();
                            let norm_q: f32 = query_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                            let norm_d: f32 = doc_emb.iter().map(|x| x * x).sum::<f32>().sqrt();

                            if norm_q > 0.0 && norm_d > 0.0 {
                                let similarity = dot_product / (norm_q * norm_d);
                                similarities.push((idx, similarity));
                            }
                        }
                    }

                    // Sort by similarity (highest first)
                    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                    // Take top 10 most similar documents
                    for (idx, similarity) in similarities.into_iter().take(10) {
                        if similarity > 0.1 {
                            // Minimum similarity threshold
                            relevant.insert(format!("doc_0_{idx}"));
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

                ground_truth.push(relevant);
            }

            ground_truth
        })
    }
}

/// Benchmark a specific dimension
fn benchmark_dimension(
    dataset: &TestDataset,
    dimension: usize,
) -> Result<DimensionBenchmarkResult, Box<dyn std::error::Error>> {
    tracing::info!(
        "🚀 Benchmarking dimension: {}D with {} vectors",
        dimension,
        dataset.documents.len()
    );

    // Use tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Create BM25 provider
        let bm25_config = BM25Config::default();
        let bm25 = BM25Provider::new(bm25_config);

        // Build vocabulary
        let _ = bm25.add_documents(&dataset.documents).await;

        // Generate embeddings
        tracing::info!("  📊 Generating embeddings...");
        let embeddings_start = Instant::now();

        let mut embeddings: Vec<Vec<f32>> = Vec::new();
        for doc in &dataset.documents {
            if let Ok(emb) = bm25.embed(doc).await {
                embeddings.push(emb);
            }
        }

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
        let memory_efficiency = embeddings.len() as f64 / index_memory_mb;

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
            if let Ok(query_emb) = bm25.embed(&dataset.queries[0]).await {
                let _ = index.search(&query_emb, 10);
            }
        }

        // Actual benchmarking
        let search_start = Instant::now();

        for (query_idx, query) in dataset.queries.iter().enumerate() {
            if let Ok(query_emb) = bm25.embed(query).await {
                let query_start = Instant::now();
                if let Ok(results) = index.search(&query_emb, 10) {
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
            }
        }

        let total_search_time_ms = search_start.elapsed().as_millis() as f64;

        // Calculate search metrics
        let avg_search_latency_us = search_times.iter().sum::<f64>() / search_times.len() as f64;
        let search_throughput_qps = dataset.queries.len() as f64 / (total_search_time_ms / 1000.0);

        // Quality evaluation
        let eval_metrics = evaluate_search_quality(query_results, 10);

        let result = DimensionBenchmarkResult {
            dimension,
            dataset_size: dataset.documents.len(),
            index_build_time_ms,
            vectors_per_second,
            index_memory_mb,
            bytes_per_vector,
            memory_efficiency,
            avg_search_latency_us,
            p95_search_latency_us: percentile(&search_times, 95),
            p99_search_latency_us: percentile(&search_times, 99),
            search_throughput_qps,
            map_score: f64::from(eval_metrics.mean_average_precision),
            recall_at_10: f64::from(eval_metrics.precision_at_k.last().copied().unwrap_or(0.0)),
            quality_per_mb: f64::from(eval_metrics.mean_average_precision) / index_memory_mb,
            speed_per_mb: search_throughput_qps / index_memory_mb,
            quality_per_us: f64::from(eval_metrics.mean_average_precision) / avg_search_latency_us,
        };

        tracing::info!("  ✅ Search: {avg_search_latency_us:.0} μs avg, {search_throughput_qps:.1} QPS");
        tracing::info!(
            "  ✅ Quality: MAP={:.4}, Recall@10={:.3}",
            result.map_score, result.recall_at_10
        );

        Ok(result)
    })
}

/// Generate comprehensive dimension analysis
fn analyze_dimension_performance(results: &[DimensionBenchmarkResult]) -> DimensionRecommendations {
    // Find optimal dimension for different criteria

    // Memory efficiency: highest vectors per MB
    let memory_efficient = results
        .iter()
        .max_by(|a, b| {
            a.memory_efficiency
                .partial_cmp(&b.memory_efficiency)
                .unwrap()
        })
        .unwrap()
        .dimension;

    // Speed efficiency: highest QPS per MB
    let speed_efficient = results
        .iter()
        .max_by(|a, b| a.speed_per_mb.partial_cmp(&b.speed_per_mb).unwrap())
        .unwrap()
        .dimension;

    // Quality efficiency: highest MAP per MB
    let quality_efficient = results
        .iter()
        .max_by(|a, b| a.quality_per_mb.partial_cmp(&b.quality_per_mb).unwrap())
        .unwrap()
        .dimension;

    // Balanced: best overall score
    let mut balanced = results[0].dimension;
    let mut best_score = 0.0;

    for result in results {
        // Score = Quality * Speed * (1 / normalized_memory)
        let quality_score = result.map_score;
        let speed_score = result.search_throughput_qps / 1000.0; // Normalize QPS
        let memory_penalty = 1.0 / (result.index_memory_mb / 100.0); // Normalize memory
        let score = quality_score * speed_score * memory_penalty;

        if score > best_score {
            best_score = score;
            balanced = result.dimension;
        }
    }

    // Overall optimal: best MAP score
    let optimal = results
        .iter()
        .max_by(|a, b| a.map_score.partial_cmp(&b.map_score).unwrap())
        .unwrap()
        .dimension;

    // Generate guidelines
    let mut guidelines = Vec::new();

    // Low latency applications
    if let Some(low_lat) = results.iter().min_by(|a, b| {
        a.avg_search_latency_us
            .partial_cmp(&b.avg_search_latency_us)
            .unwrap()
    }) {
        guidelines.push(DimensionGuideline {
            use_case: "Low Latency Applications".to_string(),
            recommended_dimension: low_lat.dimension,
            reasoning: format!(
                "Lowest search latency: {:.0}μs",
                low_lat.avg_search_latency_us
            ),
            trade_offs: "May sacrifice some quality for speed".to_string(),
        });
    }

    // High quality applications
    if let Some(high_qual) = results
        .iter()
        .max_by(|a, b| a.map_score.partial_cmp(&b.map_score).unwrap())
    {
        guidelines.push(DimensionGuideline {
            use_case: "High Quality Applications".to_string(),
            recommended_dimension: high_qual.dimension,
            reasoning: format!("Highest MAP score: {:.4}", high_qual.map_score),
            trade_offs: "May require more memory and computation".to_string(),
        });
    }

    // Memory constrained applications
    if let Some(mem_eff) = results.iter().max_by(|a, b| {
        a.memory_efficiency
            .partial_cmp(&b.memory_efficiency)
            .unwrap()
    }) {
        guidelines.push(DimensionGuideline {
            use_case: "Memory Constrained Applications".to_string(),
            recommended_dimension: mem_eff.dimension,
            reasoning: format!(
                "Highest memory efficiency: {:.0} vectors/MB",
                mem_eff.memory_efficiency
            ),
            trade_offs: "May sacrifice quality for memory efficiency".to_string(),
        });
    }

    // High throughput applications
    if let Some(high_tp) = results.iter().max_by(|a, b| {
        a.search_throughput_qps
            .partial_cmp(&b.search_throughput_qps)
            .unwrap()
    }) {
        guidelines.push(DimensionGuideline {
            use_case: "High Throughput Applications".to_string(),
            recommended_dimension: high_tp.dimension,
            reasoning: format!(
                "Highest throughput: {:.0} QPS",
                high_tp.search_throughput_qps
            ),
            trade_offs: "May require more memory for optimal performance".to_string(),
        });
    }

    DimensionRecommendations {
        optimal_dimension: optimal,
        memory_efficient_dimension: memory_efficient,
        speed_efficient_dimension: speed_efficient,
        quality_efficient_dimension: quality_efficient,
        balanced_dimension: balanced,
        dimension_guidelines: guidelines,
    }
}

/// Generate comprehensive report
fn generate_dimension_report(report: &DimensionBenchmarkReport) -> String {
    let mut md = String::new();

    md.push_str("# Dimension Comparison Performance Benchmark\n\n");
    md.push_str(&format!("**Generated**: {}\n\n", report.timestamp));

    md.push_str("## Test Configuration\n\n");
    md.push_str(&format!(
        "- **Dataset Size**: {} vectors\n",
        report.dataset_size
    ));
    md.push_str(&format!(
        "- **Test Dimensions**: {:?}D\n",
        report.test_dimensions
    ));
    md.push_str("- **HNSW Config**: M=16, ef_construction=200\n");
    md.push_str("- **Distance**: Cosine\n\n");

    md.push_str("## Performance Results\n\n");

    md.push_str("| Dimension | Build Time | Memory | Search Latency | QPS | MAP | Recall@10 | Memory Eff |\n");
    md.push_str("|-----------|-----------|--------|----------------|-----|-----|-----------|------------|\n");

    for result in &report.results {
        md.push_str(&format!(
            "| {}D | {:.1}s | {:.1}MB | {:.0}μs | {:.0} | {:.3} | {:.3} | {:.0}/MB |\n",
            result.dimension,
            result.index_build_time_ms / 1000.0,
            result.index_memory_mb,
            result.avg_search_latency_us,
            result.search_throughput_qps,
            result.map_score,
            result.recall_at_10,
            result.memory_efficiency
        ));
    }

    md.push_str("\n## Dimension Analysis\n\n");

    // Performance vs dimension
    md.push_str("### Performance vs Dimension\n\n");
    md.push_str("| Dimension | Latency | QPS | Memory | Quality |\n");
    md.push_str("|-----------|---------|-----|--------|----------|\n");

    if let Some(first) = report.results.first() {
        let baseline_latency = first.avg_search_latency_us;
        let baseline_qps = first.search_throughput_qps;
        let baseline_map = first.map_score;

        for result in &report.results {
            let latency_ratio = result.avg_search_latency_us / baseline_latency;
            let qps_ratio = result.search_throughput_qps / baseline_qps;
            let quality_ratio = result.map_score / baseline_map;

            md.push_str(&format!(
                "| {}D | {:.1}x | {:.1}x | {:.1}MB | {:.1}x |\n",
                result.dimension, latency_ratio, qps_ratio, result.index_memory_mb, quality_ratio
            ));
        }
    }

    md.push_str("\n## Recommendations\n\n");

    let rec = &report.recommendations;
    md.push_str(&format!(
        "### Optimal Dimension: **{}D**\n\n",
        rec.optimal_dimension
    ));
    md.push_str(&format!(
        "### Memory Efficient: **{}D**\n\n",
        rec.memory_efficient_dimension
    ));
    md.push_str(&format!(
        "### Speed Efficient: **{}D**\n\n",
        rec.speed_efficient_dimension
    ));
    md.push_str(&format!(
        "### Quality Efficient: **{}D**\n\n",
        rec.quality_efficient_dimension
    ));
    md.push_str(&format!(
        "### Balanced: **{}D**\n\n",
        rec.balanced_dimension
    ));

    md.push_str("## Use Case Guidelines\n\n");

    for guideline in &rec.dimension_guidelines {
        md.push_str(&format!("### {}\n\n", guideline.use_case));
        md.push_str(&format!(
            "**Recommended**: {}D\n\n",
            guideline.recommended_dimension
        ));
        md.push_str(&format!("**Reasoning**: {}\n\n", guideline.reasoning));
        md.push_str(&format!("**Trade-offs**: {}\n\n", guideline.trade_offs));
    }

    md.push_str("## Implementation Guidelines\n\n");

    md.push_str("### Dimension Selection Strategy\n\n");
    md.push_str("1. **Low Latency** (< 1ms): Use 64D-128D\n");
    md.push_str("2. **Balanced Performance**: Use 256D-512D\n");
    md.push_str("3. **High Quality**: Use 768D-1024D\n");
    md.push_str("4. **Memory Constrained**: Use 64D-256D\n");
    md.push_str("5. **High Throughput**: Use 128D-512D\n\n");

    md.push_str("### Monitoring Thresholds\n\n");
    md.push_str("- **Latency Alert**: > 5ms average search\n");
    md.push_str("- **Quality Alert**: < 0.8 MAP score\n");
    md.push_str("- **Memory Alert**: > 1GB per collection\n");
    md.push_str("- **Throughput Alert**: < 100 QPS\n\n");

    md.push_str("---\n\n");
    md.push_str("*Dimension comparison benchmark report generated automatically*\n");

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

    tracing::info!("📈 Dimension Comparison Performance Benchmark");
    tracing::info!("============================================\n");

    let dataset_size = 10_000;
    let test_dimensions = vec![64, 128, 256, 512, 768, 1024, 1536];

    tracing::info!("📊 Configuration:");
    tracing::info!("  - Dataset size: {dataset_size} vectors");
    tracing::info!("  - Test dimensions: {test_dimensions:?}D");
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

    let dataset = TestDataset::generate_dimension_dataset(&base_docs, &base_queries, dataset_size);

    let mut results = Vec::new();

    // Test each dimension
    for &dimension in &test_dimensions {
        tracing::info!("\n{}", "=".repeat(60));
        tracing::info!("🧪 TESTING DIMENSION: {dimension}D");

        match benchmark_dimension(&dataset, dimension) {
            Ok(result) => {
                results.push(result);
                tracing::info!("✅ Dimension {dimension}D completed successfully");
            }
            Err(e) => {
                tracing::info!("❌ Failed to benchmark dimension {dimension}D: {e}");
                // Continue with other dimensions
            }
        }
    }

    // Generate analysis and recommendations
    let recommendations = analyze_dimension_performance(&results);

    let report = DimensionBenchmarkReport {
        timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        dataset_size,
        test_dimensions: test_dimensions.clone(),
        results,
        recommendations,
    };

    // Generate and save report
    tracing::info!("\n📊 Generating comprehensive report...");
    let md_report = generate_dimension_report(&report);

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");

    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }

    let report_path = report_dir.join(format!("dimension_benchmark_{timestamp}.md"));
    fs::write(&report_path, &md_report)?;

    let json_path = report_dir.join(format!("dimension_benchmark_{timestamp}.json"));
    let json_data = serde_json::to_string_pretty(&report)?;
    fs::write(&json_path, json_data)?;

    // Print final recommendations
    tracing::info!("\n🎯 FINAL RECOMMENDATIONS");
    tracing::info!("{}", "=".repeat(40));

    let rec = &report.recommendations;
    tracing::info!("📏 Optimal Dimension: {}D", rec.optimal_dimension);
    tracing::info!("📏 Memory Efficient: {}D", rec.memory_efficient_dimension);
    tracing::info!("📏 Speed Efficient: {}D", rec.speed_efficient_dimension);
    tracing::info!("📏 Quality Efficient: {}D", rec.quality_efficient_dimension);
    tracing::info!("📏 Balanced: {}D", rec.balanced_dimension);

    tracing::info!("\n📄 Full report: {}", report_path.display());
    tracing::info!("📊 JSON data: {}", json_path.display());

    tracing::info!("\n✅ Dimension comparison benchmark completed successfully!");

    Ok(())
}
