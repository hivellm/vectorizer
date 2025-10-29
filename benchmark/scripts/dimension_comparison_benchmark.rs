//! Embedding Dimension Comparison Benchmark
//!
//! Tests multiple embedding dimensions to find optimal balance between:
//! - Search quality (MAP, Recall, Precision)
//! - Memory usage
//! - Search performance (latency, throughput)
//! - Index build time
//!
//! Dimensions tested: 64, 128, 256, 384, 512, 768, 1024, 1536
//!
//! Usage:
//!   cargo run --release --bin dimension_comparison_benchmark

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing_subscriber;
use vectorizer::VectorStore;
use vectorizer::db::{OptimizedHnswConfig, OptimizedHnswIndex};
use vectorizer::document_loader::{DocumentLoader, LoaderConfig};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager, EmbeddingProvider};
use vectorizer::evaluation::{EvaluationMetrics, QueryResult, evaluate_search_quality};

/// Results for a specific dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionBenchmark {
    pub dimension: usize,
    pub memory_mb: f64,
    pub memory_per_vector_bytes: f64,
    pub index_build_time_ms: f64,
    pub avg_search_latency_us: f64,
    pub p95_search_latency_us: f64,
    pub search_throughput_qps: f64,
    pub quality_map: f64,
    pub quality_mrr: f64,
    pub quality_precision_at_5: f64,
    pub quality_recall_at_5: f64,
    pub quality_precision_at_10: f64,
    pub quality_recall_at_10: f64,
}

/// Test dataset with multiple dimensions
struct MultiDimensionDataset {
    documents: Vec<String>,
    queries: Vec<String>,
    ground_truth: Vec<HashSet<String>>,
}

impl MultiDimensionDataset {
    fn load_from_workspace(max_documents: usize) -> Result<Self, Box<dyn std::error::Error>> {
        println!("📂 Loading dataset from workspace...");

        let test_paths = vec!["../gov", "../vectorizer/docs", "../task-queue/docs"];

        let mut all_documents = Vec::new();
        let temp_store = VectorStore::new();

        for test_path in &test_paths {
            if !Path::new(test_path).exists() {
                continue;
            }

            let config = LoaderConfig {
                collection_name: "dimension_benchmark".to_string(),
                max_chunk_size: 1000,
                chunk_overlap: 200,
                include_patterns: vec!["**/*.md".to_string(), "**/*.json".to_string()],
                exclude_patterns: vec![],
                embedding_dimension: 512, // Will be overridden
                embedding_type: "bm25".to_string(),
                allowed_extensions: vec![".md".to_string(), ".json".to_string()],
                max_file_size: 1024 * 1024,
            };

            let mut loader = DocumentLoader::new(config);

            match loader.load_project(test_path, &temp_store) {
                Ok(_) => {
                    let docs = loader.get_processed_documents();
                    all_documents.extend(docs);
                }
                Err(_) => {}
            }
        }

        if all_documents.len() > max_documents {
            all_documents.truncate(max_documents);
        }

        println!("  ✅ Loaded {} documents", all_documents.len());

        // Create diverse queries
        let queries = vec![
            "governance voting consensus mechanism implementation".to_string(),
            "vector database semantic search HNSW algorithm".to_string(),
            "BIP proposal workflow approval process".to_string(),
            "authentication security access control".to_string(),
            "performance optimization memory management".to_string(),
            "API REST endpoint documentation specification".to_string(),
            "testing unit integration coverage strategy".to_string(),
            "error handling logging monitoring".to_string(),
            "database schema migration persistence".to_string(),
            "configuration environment variables setup".to_string(),
            "deployment infrastructure Docker Kubernetes".to_string(),
            "TypeScript interface type definition generic".to_string(),
            "Rust async tokio concurrent parallel".to_string(),
            "Python dependency requirements virtual environment".to_string(),
            "documentation technical specification architecture".to_string(),
        ];

        // Generate ground truth
        let ground_truth = Self::generate_ground_truth(&all_documents, &queries);

        Ok(Self {
            documents: all_documents,
            queries,
            ground_truth,
        })
    }

    fn generate_ground_truth(documents: &[String], queries: &[String]) -> Vec<HashSet<String>> {
        let mut ground_truth = Vec::new();

        for query in queries {
            let mut relevant = HashSet::new();
            let query_lower = query.to_lowercase();
            let keywords: Vec<&str> = query_lower.split_whitespace().collect();

            for (idx, doc) in documents.iter().enumerate() {
                let doc_lower = doc.to_lowercase();

                // Document is relevant if contains at least 40% of keywords
                let matching = keywords.iter().filter(|kw| doc_lower.contains(*kw)).count();

                let relevance_ratio = matching as f64 / keywords.len() as f64;

                if relevance_ratio >= 0.4 || matching >= 3 {
                    relevant.insert(format!("doc_{}", idx));
                }
            }

            // Ensure minimum relevant docs
            if relevant.len() < 5 {
                for i in 0..5.min(documents.len()) {
                    relevant.insert(format!("doc_{}", i));
                }
            }

            ground_truth.push(relevant);
        }

        ground_truth
    }
}

/// Benchmark a specific dimension
fn benchmark_dimension(
    dataset: &MultiDimensionDataset,
    dimension: usize,
) -> Result<DimensionBenchmark, Box<dyn std::error::Error>> {
    println!("\n📐 Benchmarking dimension = {}...", dimension);

    // Create embedding manager
    let mut manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(dimension);
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25")?;

    // Build vocabulary
    if let Some(provider) = manager.get_provider_mut("bm25") {
        if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
            bm25.build_vocabulary(&dataset.documents);
        }
    }

    // Generate embeddings
    println!("  Generating embeddings...");
    let embed_start = Instant::now();
    let mut vectors = Vec::new();
    let mut vector_ids = Vec::new();

    for (idx, doc) in dataset.documents.iter().enumerate() {
        let embedding = manager.embed(doc)?;
        vectors.push(embedding);
        vector_ids.push(format!("doc_{}", idx));
    }

    let embed_time = embed_start.elapsed();
    println!(
        "    Generated {} embeddings in {:.2}s",
        vectors.len(),
        embed_time.as_secs_f64()
    );

    // Build index
    println!("  Building HNSW index...");
    let build_start = Instant::now();

    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: vectorizer::models::DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    let batch_vectors: Vec<(String, Vec<f32>)> = vector_ids
        .iter()
        .zip(vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let build_time_ms = build_start.elapsed().as_millis() as f64;
    println!("    Index built in {:.2}ms", build_time_ms);

    // Measure memory
    let memory_stats = index.memory_stats();
    let memory_mb = memory_stats.total_memory_bytes as f64 / 1_048_576.0;
    let memory_per_vector = memory_stats.total_memory_bytes as f64 / vectors.len() as f64;

    println!(
        "    Memory: {:.2} MB ({:.0} bytes/vector)",
        memory_mb, memory_per_vector
    );

    // Benchmark search performance
    println!("  Benchmarking search...");
    let mut search_latencies = Vec::new();
    let search_start = Instant::now();
    let num_queries = 100; // Multiple runs per query

    // Warmup
    for _ in 0..5 {
        let query_emb = manager.embed(&dataset.queries[0])?;
        let _ = index.search(&query_emb, 10)?;
    }

    // Actual benchmark
    for i in 0..num_queries {
        let query_idx = i % dataset.queries.len();
        let query_emb = manager.embed(&dataset.queries[query_idx])?;

        let start = Instant::now();
        let _ = index.search(&query_emb, 10)?;
        let elapsed_us = start.elapsed().as_micros() as f64;
        search_latencies.push(elapsed_us);
    }

    let total_search_time = search_start.elapsed().as_secs_f64();
    let avg_search_us = search_latencies.iter().sum::<f64>() / search_latencies.len() as f64;
    let p95_search_us = percentile(&search_latencies, 95);
    let qps = num_queries as f64 / total_search_time;

    println!("    Search: {:.0} μs avg, {:.2} QPS", avg_search_us, qps);

    // Measure quality
    println!("  Evaluating search quality...");
    let mut query_results = Vec::new();

    for (query_idx, query) in dataset.queries.iter().enumerate() {
        let query_emb = manager.embed(query)?;
        let results = index.search(&query_emb, 10)?;

        let query_result: Vec<QueryResult> = results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance,
            })
            .collect();

        query_results.push((query_result, dataset.ground_truth[query_idx].clone()));
    }

    let eval_metrics = evaluate_search_quality(query_results, 10);

    println!(
        "    Quality: MAP={:.4}, Recall@10={:.4}",
        eval_metrics.mean_average_precision,
        eval_metrics.recall_at_k.get(9).copied().unwrap_or(0.0)
    );

    Ok(DimensionBenchmark {
        dimension,
        memory_mb,
        memory_per_vector_bytes: memory_per_vector,
        index_build_time_ms: build_time_ms,
        avg_search_latency_us: avg_search_us,
        p95_search_latency_us: p95_search_us,
        search_throughput_qps: qps,
        quality_map: eval_metrics.mean_average_precision as f64,
        quality_mrr: eval_metrics.mean_reciprocal_rank as f64,
        quality_precision_at_5: eval_metrics.precision_at_k.get(4).copied().unwrap_or(0.0) as f64,
        quality_recall_at_5: eval_metrics.recall_at_k.get(4).copied().unwrap_or(0.0) as f64,
        quality_precision_at_10: eval_metrics.precision_at_k.get(9).copied().unwrap_or(0.0) as f64,
        quality_recall_at_10: eval_metrics.recall_at_k.get(9).copied().unwrap_or(0.0) as f64,
    })
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

/// Generate comprehensive report
fn generate_report(results: &[DimensionBenchmark], dataset_size: usize) -> String {
    let mut md = String::new();

    md.push_str("# Embedding Dimension Comparison Benchmark\n\n");
    md.push_str(&format!(
        "**Generated**: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    md.push_str("## Test Configuration\n\n");
    md.push_str(&format!("- **Dataset Size**: {} documents\n", dataset_size));
    md.push_str("- **Dimensions Tested**: ");
    for (i, r) in results.iter().enumerate() {
        if i > 0 {
            md.push_str(", ");
        }
        md.push_str(&format!("{}", r.dimension));
    }
    md.push_str("\n");
    md.push_str("- **Embedding Model**: BM25\n");
    md.push_str("- **HNSW Config**: M=16, ef_construction=200\n");
    md.push_str("- **Distance Metric**: Cosine\n\n");

    md.push_str("## Executive Summary\n\n");
    md.push_str("| Dimension | Memory (MB) | Bytes/Vec | Build Time (ms) | Search (μs) | QPS | MAP | Recall@10 |\n");
    md.push_str("|-----------|-------------|-----------|-----------------|-------------|-----|-----|----------|\n");

    for result in results {
        md.push_str(&format!(
            "| {} | {:.2} | {:.0} | {:.0} | {:.0} | {:.0} | {:.4} | {:.4} |\n",
            result.dimension,
            result.memory_mb,
            result.memory_per_vector_bytes,
            result.index_build_time_ms,
            result.avg_search_latency_us,
            result.search_throughput_qps,
            result.quality_map,
            result.quality_recall_at_10,
        ));
    }

    md.push_str("\n## Detailed Analysis\n\n");

    // Find reference (512 is current default)
    let reference = results
        .iter()
        .find(|r| r.dimension == 512)
        .or_else(|| results.first())
        .unwrap();

    md.push_str("### Quality vs Dimension\n\n");
    md.push_str("| Dimension | MAP | vs 512D | Recall@10 | vs 512D | Precision@10 |\n");
    md.push_str("|-----------|-----|---------|-----------|---------|-------------|\n");

    for result in results {
        let map_diff = ((result.quality_map / reference.quality_map) - 1.0) * 100.0;
        let recall_diff =
            ((result.quality_recall_at_10 / reference.quality_recall_at_10) - 1.0) * 100.0;

        md.push_str(&format!(
            "| {} | {:.4} | {:+.1}% | {:.4} | {:+.1}% | {:.4} |\n",
            result.dimension,
            result.quality_map,
            map_diff,
            result.quality_recall_at_10,
            recall_diff,
            result.quality_precision_at_10,
        ));
    }

    md.push_str("\n### Memory vs Dimension\n\n");
    md.push_str("| Dimension | Total Memory | Memory/Vector | vs 512D |\n");
    md.push_str("|-----------|--------------|---------------|----------|\n");

    for result in results {
        let mem_ratio = result.memory_mb / reference.memory_mb;

        md.push_str(&format!(
            "| {} | {:.2} MB | {:.0} bytes | {:.2}x |\n",
            result.dimension, result.memory_mb, result.memory_per_vector_bytes, mem_ratio,
        ));
    }

    md.push_str("\n### Performance vs Dimension\n\n");
    md.push_str("| Dimension | Search Latency | P95 | QPS | vs 512D |\n");
    md.push_str("|-----------|----------------|-----|-----|----------|\n");

    for result in results {
        let speed_ratio = result.avg_search_latency_us / reference.avg_search_latency_us;

        md.push_str(&format!(
            "| {} | {:.0} μs | {:.0} μs | {:.0} | {:.2}x |\n",
            result.dimension,
            result.avg_search_latency_us,
            result.p95_search_latency_us,
            result.search_throughput_qps,
            speed_ratio,
        ));
    }

    md.push_str("\n## Key Insights\n\n");

    // Find sweet spots
    let best_quality = results
        .iter()
        .max_by(|a, b| a.quality_map.partial_cmp(&b.quality_map).unwrap())
        .unwrap();

    let best_speed = results
        .iter()
        .min_by(|a, b| {
            a.avg_search_latency_us
                .partial_cmp(&b.avg_search_latency_us)
                .unwrap()
        })
        .unwrap();

    let best_memory = results
        .iter()
        .min_by(|a, b| a.memory_mb.partial_cmp(&b.memory_mb).unwrap())
        .unwrap();

    md.push_str(&format!("### Best Quality: {}D\n", best_quality.dimension));
    md.push_str(&format!("- MAP: {:.4}\n", best_quality.quality_map));
    md.push_str(&format!(
        "- Recall@10: {:.4}\n",
        best_quality.quality_recall_at_10
    ));
    md.push_str(&format!(
        "- Memory cost: {:.2} MB\n\n",
        best_quality.memory_mb
    ));

    md.push_str(&format!("### Fastest Search: {}D\n", best_speed.dimension));
    md.push_str(&format!(
        "- Latency: {:.0} μs\n",
        best_speed.avg_search_latency_us
    ));
    md.push_str(&format!("- QPS: {:.0}\n", best_speed.search_throughput_qps));
    md.push_str(&format!(
        "- Quality (MAP): {:.4}\n\n",
        best_speed.quality_map
    ));

    md.push_str(&format!(
        "### Most Memory Efficient: {}D\n",
        best_memory.dimension
    ));
    md.push_str(&format!("- Memory: {:.2} MB\n", best_memory.memory_mb));
    md.push_str(&format!(
        "- Bytes/vector: {:.0}\n",
        best_memory.memory_per_vector_bytes
    ));
    md.push_str(&format!(
        "- Quality (MAP): {:.4}\n\n",
        best_memory.quality_map
    ));

    md.push_str("### Quality vs Size Trade-offs\n\n");

    for result in results {
        let quality_per_mb = result.quality_map / result.memory_mb;
        let quality_retention = (result.quality_map / best_quality.quality_map) * 100.0;

        md.push_str(&format!(
            "- **{}D**: {:.1}% quality retention, {:.4} MAP/MB efficiency\n",
            result.dimension, quality_retention, quality_per_mb
        ));
    }

    md.push_str("\n### Diminishing Returns Analysis\n\n");

    // Calculate quality gain per dimension increase
    for i in 1..results.len() {
        let prev = &results[i - 1];
        let curr = &results[i];

        let dim_increase = curr.dimension - prev.dimension;
        let quality_gain = ((curr.quality_map / prev.quality_map) - 1.0) * 100.0;
        let memory_increase = ((curr.memory_mb / prev.memory_mb) - 1.0) * 100.0;

        md.push_str(&format!(
            "- **{}D → {}D** (+{} dims): {:+.2}% quality, {:+.1}% memory\n",
            prev.dimension, curr.dimension, dim_increase, quality_gain, memory_increase
        ));
    }

    md.push_str("\n## Recommendations\n\n");

    // Find optimal dimension based on quality/memory/speed
    let mut scored_results: Vec<_> = results
        .iter()
        .map(|r| {
            // Score: balance quality, speed, and memory
            let quality_score = r.quality_map / best_quality.quality_map;
            let speed_score = best_speed.avg_search_latency_us / r.avg_search_latency_us;
            let memory_score = best_memory.memory_mb / r.memory_mb;

            let total_score = quality_score * 0.5 + speed_score * 0.3 + memory_score * 0.2;

            (r, total_score)
        })
        .collect();

    scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    md.push_str("### Overall Best Balance\n\n");
    md.push_str(&format!(
        "**Recommended: {}D** (score: {:.3})\n\n",
        scored_results[0].0.dimension, scored_results[0].1
    ));

    md.push_str("Reasoning:\n");
    md.push_str(&format!(
        "- Quality: {:.4} MAP ({:.1}% of best)\n",
        scored_results[0].0.quality_map,
        (scored_results[0].0.quality_map / best_quality.quality_map) * 100.0
    ));
    md.push_str(&format!(
        "- Speed: {:.0} μs ({:.1}x vs fastest)\n",
        scored_results[0].0.avg_search_latency_us,
        scored_results[0].0.avg_search_latency_us / best_speed.avg_search_latency_us
    ));
    md.push_str(&format!(
        "- Memory: {:.2} MB ({:.1}x vs smallest)\n\n",
        scored_results[0].0.memory_mb,
        scored_results[0].0.memory_mb / best_memory.memory_mb
    ));

    md.push_str("### Use Cases\n\n");
    md.push_str("1. **Maximum Quality**: Use ");
    md.push_str(&format!("{}D ", best_quality.dimension));
    md.push_str("when accuracy is critical\n");

    md.push_str("2. **Maximum Speed**: Use ");
    md.push_str(&format!("{}D ", best_speed.dimension));
    md.push_str("when low latency is priority\n");

    md.push_str("3. **Memory Constrained**: Use ");
    md.push_str(&format!("{}D ", best_memory.dimension));
    md.push_str("when storage is limited\n");

    md.push_str("4. **Production Balanced**: Use ");
    md.push_str(&format!("{}D ", scored_results[0].0.dimension));
    md.push_str("for best overall balance\n\n");

    md.push_str("### Scaling Projections\n\n");
    md.push_str("Estimated requirements for 1M vectors:\n\n");

    for result in results {
        let mem_1m = (result.memory_per_vector_bytes * 1_000_000.0) / 1_048_576.0;
        let build_time_1m = (result.index_build_time_ms / dataset_size as f64) * 1_000_000.0;

        md.push_str(&format!(
            "- **{}D**: {:.2} GB memory, ~{:.0}s build time\n",
            result.dimension,
            mem_1m / 1024.0,
            build_time_1m / 1000.0
        ));
    }

    md.push_str("\n---\n\n");
    md.push_str("*Report generated by Vectorizer Dimension Comparison Benchmark*\n");

    md
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("🚀 Vectorizer Embedding Dimension Benchmark");
    println!("===========================================\n");

    // Load dataset
    let max_docs = 5000; // Reasonable size for multiple dimension tests
    let dataset = MultiDimensionDataset::load_from_workspace(max_docs)?;

    println!(
        "📊 Dataset: {} documents, {} queries\n",
        dataset.documents.len(),
        dataset.queries.len()
    );

    // Test dimensions
    let dimensions = vec![64, 128, 256, 384, 512, 768, 1024, 1536];

    println!("🧪 Testing dimensions: {:?}\n", dimensions);

    let mut results = Vec::new();

    for &dimension in &dimensions {
        match benchmark_dimension(&dataset, dimension) {
            Ok(result) => {
                println!("  ✅ Completed: {}D", dimension);
                results.push(result);
            }
            Err(e) => {
                println!("  ❌ Error for {}D: {}", dimension, e);
            }
        }
    }

    // Generate report
    println!("\n📊 Generating comprehensive report...");
    let md_report = generate_report(&results, dataset.documents.len());

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");

    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }

    let report_path = report_dir.join(format!("dimension_comparison_{}.md", timestamp));
    fs::write(&report_path, &md_report)?;

    println!("✅ Markdown report saved to: {}", report_path.display());

    // Save JSON
    let json_path = report_dir.join(format!("dimension_comparison_{}.json", timestamp));
    let json_data = serde_json::to_string_pretty(&results)?;
    fs::write(&json_path, json_data)?;

    println!("✅ JSON data saved to: {}", json_path.display());

    // Find reference and best performers
    let reference = results
        .iter()
        .find(|r| r.dimension == 512)
        .or_else(|| results.first())
        .unwrap();

    let best_speed = results
        .iter()
        .min_by(|a, b| {
            a.avg_search_latency_us
                .partial_cmp(&b.avg_search_latency_us)
                .unwrap()
        })
        .unwrap();

    let best_quality = results
        .iter()
        .max_by(|a, b| a.quality_map.partial_cmp(&b.quality_map).unwrap())
        .unwrap();

    // Print summary
    println!("\n📈 DIMENSION COMPARISON SUMMARY");
    println!("==============================");
    println!(
        "{:<10} {:<12} {:<12} {:<12} {:<10} {:<10}",
        "Dimension", "Memory", "Search", "QPS", "MAP", "Recall@10"
    );
    println!("{}", "-".repeat(70));

    for result in &results {
        let quality_symbol = if result.quality_map >= reference.quality_map * 0.95 {
            "✅"
        } else if result.quality_map >= reference.quality_map * 0.85 {
            "⚠️"
        } else {
            "❌"
        };

        println!(
            "{:<10} {:<12} {:<12} {:<10} {:<10} {:<10}",
            format!("{}D", result.dimension),
            format!("{:.1}MB", result.memory_mb),
            format!("{:.0}μs", result.avg_search_latency_us),
            format!("{:.0}", result.search_throughput_qps),
            format!("{:.4} {}", result.quality_map, quality_symbol),
            format!("{:.4}", result.quality_recall_at_10),
        );
    }

    println!("\n💡 Key Findings:");

    println!(
        "  🏆 Best Quality: {}D (MAP: {:.4})",
        best_quality.dimension, best_quality.quality_map
    );

    // Find most efficient (quality/memory)
    let most_efficient = results
        .iter()
        .max_by(|a, b| {
            let a_eff = a.quality_map / a.memory_mb;
            let b_eff = b.quality_map / b.memory_mb;
            a_eff.partial_cmp(&b_eff).unwrap()
        })
        .unwrap();

    println!(
        "  ⚡ Most Efficient: {}D ({:.4} MAP/MB)",
        most_efficient.dimension,
        most_efficient.quality_map / most_efficient.memory_mb
    );

    // Quality retention analysis
    println!("\n  📊 Quality Retention vs 512D:");
    for result in &results {
        let retention = (result.quality_map / reference.quality_map) * 100.0;
        let memory_saving =
            ((reference.memory_mb - result.memory_mb) / reference.memory_mb) * 100.0;

        if result.dimension < 512 {
            println!(
                "     {}D: {:.1}% quality, {:.1}% less memory",
                result.dimension, retention, memory_saving
            );
        } else if result.dimension > 512 {
            let memory_cost =
                ((result.memory_mb - reference.memory_mb) / reference.memory_mb) * 100.0;
            println!(
                "     {}D: {:.1}% quality, {:+.1}% more memory",
                result.dimension, retention, memory_cost
            );
        }
    }

    println!("\n  💡 Recommendation:");

    // Score-based recommendation
    let mut scored: Vec<_> = results
        .iter()
        .map(|r| {
            let quality_score = r.quality_map / best_quality.quality_map;
            let speed_score = best_speed.avg_search_latency_us / r.avg_search_latency_us;
            let memory_score = results
                .iter()
                .map(|x| x.memory_mb)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                / r.memory_mb;

            let total = quality_score * 0.5 + speed_score * 0.3 + memory_score * 0.2;
            (r, total)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("     Use {}D for optimal balance", scored[0].0.dimension);
    println!(
        "     - Quality: {:.1}% of best",
        (scored[0].0.quality_map / best_quality.quality_map) * 100.0
    );
    println!(
        "     - Speed: {:.0} μs avg",
        scored[0].0.avg_search_latency_us
    );
    println!("     - Memory: {:.2} MB", scored[0].0.memory_mb);

    println!("\n✅ Benchmark completed successfully!");
    println!("📄 Full report: {}", report_path.display());
    println!("📊 JSON data: {}", json_path.display());

    Ok(())
}
