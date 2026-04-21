//! Combined Dimension + Quantization Optimization Benchmark
//!
//! Tests all combinations of:
//! - Embedding dimensions: 256, 384, 512, 768, 1024
//! - Quantization methods: None, SQ-8bit, PQ 8x256, Binary
//!
//! To find the optimal configuration that maximizes:
//! - Search quality (MAP, Recall)
//! - Memory efficiency
//! - Search performance
//!
//! Usage:
//!   cargo run --release --bin combined_optimization_benchmark

use std::collections::HashSet;
use tracing::{info, error, warn, debug};
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

/// Configuration combination result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBenchmark {
    pub dimension: usize,
    pub quantization: String,
    pub quantization_config: String,

    // Memory metrics
    pub memory_original_mb: f64,
    pub memory_quantized_mb: f64,
    pub compression_ratio: f64,
    pub bytes_per_vector: f64,

    // Performance metrics
    pub index_build_time_ms: f64,
    pub avg_search_latency_us: f64,
    pub p95_search_latency_us: f64,
    pub p99_search_latency_us: f64,
    pub search_throughput_qps: f64,

    // Quality metrics
    pub map: f64,
    pub mrr: f64,
    pub precision_at_5: f64,
    pub recall_at_5: f64,
    pub precision_at_10: f64,
    pub recall_at_10: f64,

    // Composite scores
    pub quality_score: f64,    // 0-1
    pub efficiency_score: f64, // quality / memory
    pub overall_score: f64,    // weighted combination
}

/// Product Quantizer (from previous benchmark)
pub struct ProductQuantizer {
    n_subquantizers: usize,
    n_centroids: usize,
    codebooks: Vec<Vec<Vec<f32>>>,
    dimension: usize,
}

impl ProductQuantizer {
    pub fn new(dimension: usize, n_subquantizers: usize, n_centroids: usize) -> Self {
        Self {
            n_subquantizers,
            n_centroids,
            codebooks: Vec::new(),
            dimension,
        }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        let subvector_dim = self.dimension / self.n_subquantizers;
        self.codebooks.clear();

        for sq_idx in 0..self.n_subquantizers {
            let start_dim = sq_idx * subvector_dim;
            let end_dim = start_dim + subvector_dim;

            let subvectors: Vec<Vec<f32>> = vectors
                .iter()
                .map(|v| v[start_dim..end_dim].to_vec())
                .collect();

            let centroids = self.kmeans(&subvectors, self.n_centroids);
            self.codebooks.push(centroids);
        }
    }

    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let subvector_dim = self.dimension / self.n_subquantizers;
        let mut codes = Vec::with_capacity(self.n_subquantizers);

        for sq_idx in 0..self.n_subquantizers {
            let start_dim = sq_idx * subvector_dim;
            let end_dim = start_dim + subvector_dim;
            let subvector = &vector[start_dim..end_dim];

            let code = self.find_nearest_centroid(subvector, &self.codebooks[sq_idx]);
            codes.push(code as u8);
        }

        codes
    }

    pub fn decode(&self, codes: &[u8]) -> Vec<f32> {
        let mut reconstructed = Vec::with_capacity(self.dimension);

        for (sq_idx, &code) in codes.iter().enumerate() {
            let centroid = &self.codebooks[sq_idx][code as usize];
            reconstructed.extend_from_slice(centroid);
        }

        reconstructed
    }

    fn kmeans(&self, vectors: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        if vectors.is_empty() {
            return Vec::new();
        }

        let dim = vectors[0].len();
        let k = k.min(vectors.len());

        let mut centroids: Vec<Vec<f32>> =
            (0..k).map(|i| vectors[i % vectors.len()].clone()).collect();

        for _ in 0..5 {
            let mut assignments: Vec<Vec<Vec<f32>>> = vec![Vec::new(); k];

            for vector in vectors {
                let nearest = self.find_nearest_centroid(vector, &centroids);
                assignments[nearest].push(vector.clone());
            }

            for (i, cluster) in assignments.iter().enumerate() {
                if !cluster.is_empty() {
                    centroids[i] = Self::compute_mean(cluster, dim);
                }
            }
        }

        centroids
    }

    fn find_nearest_centroid(&self, vector: &[f32], centroids: &[Vec<f32>]) -> usize {
        centroids
            .iter()
            .enumerate()
            .map(|(idx, centroid)| {
                let dist = self.euclidean_distance(vector, centroid);
                (idx, dist)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    fn compute_mean(vectors: &[Vec<f32>], dim: usize) -> Vec<f32> {
        let mut mean = vec![0.0; dim];
        let count = vectors.len() as f32;

        for vector in vectors {
            for (i, &val) in vector.iter().enumerate() {
                mean[i] += val / count;
            }
        }

        mean
    }
}

/// Scalar Quantizer
pub struct ScalarQuantizer {
    bits: usize,
    min_val: f32,
    max_val: f32,
}

impl ScalarQuantizer {
    pub fn new(bits: usize) -> Self {
        Self {
            bits,
            min_val: f32::MAX,
            max_val: f32::MIN,
        }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        for vector in vectors {
            for &val in vector {
                self.min_val = self.min_val.min(val);
                self.max_val = self.max_val.max(val);
            }
        }
    }

    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let levels = (1 << self.bits) - 1;
        let range = self.max_val - self.min_val;

        vector
            .iter()
            .map(|&val| {
                let normalized = (val - self.min_val) / range;
                let quantized = (normalized * levels as f32).round() as u8;
                quantized
            })
            .collect()
    }

    pub fn decode(&self, codes: &[u8]) -> Vec<f32> {
        let levels = (1 << self.bits) - 1;
        let range = self.max_val - self.min_val;

        codes
            .iter()
            .map(|&code| {
                let normalized = code as f32 / levels as f32;
                self.min_val + normalized * range
            })
            .collect()
    }
}

/// Binary Quantizer
pub struct BinaryQuantizer {
    threshold: f32,
}

impl BinaryQuantizer {
    pub fn new() -> Self {
        Self { threshold: 0.0 }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        let mut all_values: Vec<f32> = vectors.iter().flat_map(|v| v.iter().copied()).collect();
        all_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.threshold = all_values[all_values.len() / 2];
    }

    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let mut bytes = vec![0u8; (vector.len() + 7) / 8];

        for (i, &val) in vector.iter().enumerate() {
            if val > self.threshold {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                bytes[byte_idx] |= 1 << bit_idx;
            }
        }

        bytes
    }

    pub fn decode(&self, codes: &[u8], dimension: usize) -> Vec<f32> {
        let mut vector = vec![0.0; dimension];

        for i in 0..dimension {
            let byte_idx = i / 8;
            let bit_idx = i % 8;

            if byte_idx < codes.len() {
                let bit_set = (codes[byte_idx] & (1 << bit_idx)) != 0;
                vector[i] = if bit_set { 1.0 } else { -1.0 };
            }
        }

        vector
    }
}

/// Test dataset
struct TestDataset {
    documents: Vec<String>,
    queries: Vec<String>,
    ground_truth: Vec<HashSet<String>>,
}

impl TestDataset {
    fn load_from_workspace(max_docs: usize) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("📂 Loading comprehensive dataset...");

        let test_paths = vec!["../gov", "../vectorizer/docs", "../task-queue/docs"];
        let mut all_documents = Vec::new();
        let temp_store = VectorStore::new();

        for test_path in &test_paths {
            if !Path::new(test_path).exists() {
                continue;
            }

            let config = LoaderConfig {
                collection_name: "combined_benchmark".to_string(),
                max_chunk_size: 1000,
                chunk_overlap: 200,
                include_patterns: vec!["**/*.md".to_string(), "**/*.json".to_string()],
                exclude_patterns: vec![],
                embedding_dimension: 512,
                embedding_type: "bm25".to_string(),
                allowed_extensions: vec![".md".to_string(), ".json".to_string()],
                max_file_size: 1024 * 1024,
            };

            let mut loader = DocumentLoader::new(config);

            if let Ok(_) = loader.load_project(test_path, &temp_store) {
                all_documents.extend(loader.get_processed_documents());
            }
        }

        if all_documents.len() > max_docs {
            all_documents.truncate(max_docs);
        }

        tracing::info!("  ✅ Loaded {} documents", all_documents.len());

        let queries = vec![
            "vector database HNSW indexing performance".to_string(),
            "quantization memory optimization techniques".to_string(),
            "semantic search quality metrics evaluation".to_string(),
            "governance BIP proposal workflow".to_string(),
            "API REST endpoint documentation".to_string(),
            "task queue async Rust implementation".to_string(),
            "authentication security access control".to_string(),
            "error handling logging monitoring".to_string(),
            "testing coverage integration unit".to_string(),
            "deployment infrastructure production".to_string(),
        ];

        let ground_truth = Self::generate_ground_truth(&all_documents, &queries);

        Ok(Self {
            documents: all_documents,
            queries,
            ground_truth,
        })
    }

    fn generate_ground_truth(docs: &[String], queries: &[String]) -> Vec<HashSet<String>> {
        // Create embedding manager for semantic ground truth
        let mut manager = EmbeddingManager::new();
        let bm25 = Bm25Embedding::new(512); // Use standard dimension
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.set_default_provider("bm25").unwrap();

        // Build vocabulary
        if let Some(provider) = manager.get_provider_mut("bm25") {
            if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                bm25.build_vocabulary(docs);
            }
        }

        queries
            .iter()
            .enumerate()
            .map(|(query_idx, query)| {
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
                            relevant.insert(format!("doc_{}", idx));
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
                            // Lower threshold for fallback
                            relevant.insert(format!("doc_{}", idx));
                        }
                    }
                }

                // Ensure at least 3 relevant documents per query
                if relevant.len() < 3 {
                    for i in 0..3.min(docs.len()) {
                        relevant.insert(format!("doc_{}", i));
                    }
                }

                relevant
            })
            .collect()
    }
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

/// Benchmark a specific configuration
fn benchmark_config(
    dataset: &TestDataset,
    dimension: usize,
    quantization: &str,
) -> Result<ConfigBenchmark, Box<dyn std::error::Error>> {
    tracing::info!("\n🧪 Testing: {}D + {}...", dimension, quantization);

    // Create embeddings
    let mut manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(dimension);
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25")?;

    if let Some(provider) = manager.get_provider_mut("bm25") {
        if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
            bm25.build_vocabulary(&dataset.documents);
        }
    }

    // Generate vectors
    let mut vectors = Vec::new();
    let mut vector_ids = Vec::new();

    for (idx, doc) in dataset.documents.iter().enumerate() {
        let embedding = manager.embed(doc)?;
        vectors.push(embedding);
        vector_ids.push(format!("doc_{}", idx));
    }

    // Calculate original memory
    let memory_original_mb = (vectors.len() * dimension * 4) as f64 / 1_048_576.0;

    // Apply quantization
    let (final_vectors, memory_quantized_mb, quant_config) = match quantization {
        "None" => (
            vectors.clone(),
            memory_original_mb,
            "No quantization".to_string(),
        ),

        "SQ-8bit" => {
            let mut sq = ScalarQuantizer::new(8);
            sq.train(&vectors);

            let encoded: Vec<Vec<u8>> = vectors.iter().map(|v| sq.encode(v)).collect();
            let decoded: Vec<Vec<f32>> = encoded.iter().map(|c| sq.decode(c)).collect();

            let mem = (encoded.len() * dimension) as f64 / 1_048_576.0;
            (decoded, mem, "8-bit scalar quantization".to_string())
        }

        "PQ" => {
            let n_subquantizers = 8;
            let n_centroids = 256;

            let mut pq = ProductQuantizer::new(dimension, n_subquantizers, n_centroids);
            pq.train(&vectors);

            let encoded: Vec<Vec<u8>> = vectors.iter().map(|v| pq.encode(v)).collect();
            let decoded: Vec<Vec<f32>> = encoded.iter().map(|c| pq.decode(c)).collect();

            let codes_size = encoded.len() * n_subquantizers;
            let codebook_size = n_subquantizers * n_centroids * (dimension / n_subquantizers) * 4;
            let mem = (codes_size + codebook_size) as f64 / 1_048_576.0;

            (
                decoded,
                mem,
                format!("PQ {}x{}", n_subquantizers, n_centroids),
            )
        }

        "Binary" => {
            let mut binary = BinaryQuantizer::new();
            binary.train(&vectors);

            let encoded: Vec<Vec<u8>> = vectors.iter().map(|v| binary.encode(v)).collect();
            let decoded: Vec<Vec<f32>> = encoded
                .iter()
                .map(|c| binary.decode(c, dimension))
                .collect();

            let mem = (encoded.iter().map(|v| v.len()).sum::<usize>() + 4) as f64 / 1_048_576.0;
            (decoded, mem, "Binary (1-bit)".to_string())
        }

        _ => return Err("Unknown quantization method".into()),
    };

    let compression_ratio = memory_original_mb / memory_quantized_mb;

    // Build index
    let build_start = Instant::now();

    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: vectorizer::models::DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: final_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    let batch_vectors: Vec<(String, Vec<f32>)> = vector_ids
        .iter()
        .zip(final_vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let build_time_ms = build_start.elapsed().as_millis() as f64;

    // Benchmark search
    let mut search_latencies = Vec::new();

    // Warmup
    for _ in 0..5 {
        let query_emb = manager.embed(&dataset.queries[0])?;
        let _ = index.search(&query_emb, 10)?;
    }

    // Actual benchmark
    for query in &dataset.queries {
        for _ in 0..10 {
            // 10 runs per query
            let query_emb = manager.embed(query)?;
            let start = Instant::now();
            let _ = index.search(&query_emb, 10)?;
            search_latencies.push(start.elapsed().as_micros() as f64);
        }
    }

    let avg_search_us = search_latencies.iter().sum::<f64>() / search_latencies.len() as f64;
    let qps = 1_000_000.0 / avg_search_us;

    // Evaluate quality
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

    let eval = evaluate_search_quality(query_results, 10);

    // Calculate scores
    let quality_score = eval.mean_average_precision as f64;
    let efficiency_score = quality_score / memory_quantized_mb;

    // Overall score: 50% quality, 30% efficiency, 20% speed
    let speed_normalized = 1.0 / (avg_search_us / 1000.0); // Lower latency = higher score
    let overall_score =
        quality_score * 0.5 + efficiency_score * 0.1 + (speed_normalized / 2.0) * 0.2;

    tracing::info!(
        "    Memory: {:.2} MB ({:.1}x compression)",
        memory_quantized_mb, compression_ratio
    );
    tracing::info!("    Search: {:.0} μs ({:.0} QPS)", avg_search_us, qps);
    tracing::info!(
        "    Quality: MAP={:.4}, Recall@10={:.4}",
        eval.mean_average_precision,
        eval.recall_at_k.get(9).copied().unwrap_or(0.0)
    );

    Ok(ConfigBenchmark {
        dimension,
        quantization: quantization.to_string(),
        quantization_config: quant_config,
        memory_original_mb,
        memory_quantized_mb,
        compression_ratio,
        bytes_per_vector: (memory_quantized_mb * 1_048_576.0) / vectors.len() as f64,
        index_build_time_ms: build_time_ms,
        avg_search_latency_us: avg_search_us,
        p95_search_latency_us: percentile(&search_latencies, 95),
        p99_search_latency_us: percentile(&search_latencies, 99),
        search_throughput_qps: qps,
        map: eval.mean_average_precision as f64,
        mrr: eval.mean_reciprocal_rank as f64,
        precision_at_5: eval.precision_at_k.get(4).copied().unwrap_or(0.0) as f64,
        recall_at_5: eval.recall_at_k.get(4).copied().unwrap_or(0.0) as f64,
        precision_at_10: eval.precision_at_k.get(9).copied().unwrap_or(0.0) as f64,
        recall_at_10: eval.recall_at_k.get(9).copied().unwrap_or(0.0) as f64,
        quality_score,
        efficiency_score,
        overall_score,
    })
}

/// Generate comprehensive report
fn generate_report(results: &[ConfigBenchmark], dataset_size: usize) -> String {
    let mut md = String::new();

    md.push_str("# Combined Dimension + Quantization Optimization Benchmark\n\n");
    md.push_str(&format!(
        "**Generated**: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    md.push_str(&format!("**Dataset**: {} documents\n\n", dataset_size));

    md.push_str("## Executive Summary\n\n");
    md.push_str("| Config | Memory | Compress | Search (μs) | QPS | MAP | Recall@10 | Score |\n");
    md.push_str("|--------|--------|----------|-------------|-----|-----|-----------|-------|\n");

    let mut sorted_results = results.to_vec();
    sorted_results.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

    for (rank, result) in sorted_results.iter().enumerate().take(10) {
        let medal = match rank {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };

        md.push_str(&format!(
            "| {} {}D+{} | {:.2} MB | {:.1}x | {:.0} | {:.0} | {:.4} | {:.4} | {:.3} |\n",
            medal,
            result.dimension,
            result.quantization,
            result.memory_quantized_mb,
            result.compression_ratio,
            result.avg_search_latency_us,
            result.search_throughput_qps,
            result.map,
            result.recall_at_10,
            result.overall_score,
        ));
    }

    md.push_str("\n## Best Configurations by Category\n\n");

    let best_quality = results
        .iter()
        .max_by(|a, b| a.map.partial_cmp(&b.map).unwrap())
        .unwrap();

    let best_memory = results
        .iter()
        .min_by(|a, b| {
            a.memory_quantized_mb
                .partial_cmp(&b.memory_quantized_mb)
                .unwrap()
        })
        .unwrap();

    let best_speed = results
        .iter()
        .min_by(|a, b| {
            a.avg_search_latency_us
                .partial_cmp(&b.avg_search_latency_us)
                .unwrap()
        })
        .unwrap();

    let best_efficiency = results
        .iter()
        .max_by(|a, b| a.efficiency_score.partial_cmp(&b.efficiency_score).unwrap())
        .unwrap();

    md.push_str(&format!(
        "### 🏆 Best Quality: {}D + {}\n",
        best_quality.dimension, best_quality.quantization
    ));
    md.push_str(&format!("- MAP: {:.4}\n", best_quality.map));
    md.push_str(&format!("- Recall@10: {:.4}\n", best_quality.recall_at_10));
    md.push_str(&format!(
        "- Memory: {:.2} MB\n",
        best_quality.memory_quantized_mb
    ));
    md.push_str(&format!(
        "- Search: {:.0} μs\n\n",
        best_quality.avg_search_latency_us
    ));

    md.push_str(&format!(
        "### 💾 Most Memory Efficient: {}D + {}\n",
        best_memory.dimension, best_memory.quantization
    ));
    md.push_str(&format!(
        "- Memory: {:.2} MB\n",
        best_memory.memory_quantized_mb
    ));
    md.push_str(&format!(
        "- Compression: {:.1}x\n",
        best_memory.compression_ratio
    ));
    md.push_str(&format!("- Quality (MAP): {:.4}\n", best_memory.map));
    md.push_str(&format!(
        "- Search: {:.0} μs\n\n",
        best_memory.avg_search_latency_us
    ));

    md.push_str(&format!(
        "### ⚡ Fastest Search: {}D + {}\n",
        best_speed.dimension, best_speed.quantization
    ));
    md.push_str(&format!(
        "- Latency: {:.0} μs\n",
        best_speed.avg_search_latency_us
    ));
    md.push_str(&format!("- QPS: {:.0}\n", best_speed.search_throughput_qps));
    md.push_str(&format!("- Quality (MAP): {:.4}\n", best_speed.map));
    md.push_str(&format!(
        "- Memory: {:.2} MB\n\n",
        best_speed.memory_quantized_mb
    ));

    md.push_str(&format!(
        "### 🎯 Best Efficiency (Quality/Memory): {}D + {}\n",
        best_efficiency.dimension, best_efficiency.quantization
    ));
    md.push_str(&format!(
        "- Efficiency: {:.4} MAP/MB\n",
        best_efficiency.efficiency_score
    ));
    md.push_str(&format!("- Quality: {:.4} MAP\n", best_efficiency.map));
    md.push_str(&format!(
        "- Memory: {:.2} MB\n",
        best_efficiency.memory_quantized_mb
    ));
    md.push_str(&format!(
        "- Compression: {:.1}x\n\n",
        best_efficiency.compression_ratio
    ));

    md.push_str("## Detailed Results by Dimension\n\n");

    for dimension in [256, 384, 512, 768, 1024] {
        md.push_str(&format!("### {}D Embeddings\n\n", dimension));
        md.push_str("| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |\n");
        md.push_str("|--------------|--------|-------------|-------------|-----|----------|\n");

        let dim_results: Vec<_> = results
            .iter()
            .filter(|r| r.dimension == dimension)
            .collect();

        for result in dim_results {
            md.push_str(&format!(
                "| {} | {:.2} MB | {:.1}x | {:.0} | {:.4} | {:.4} |\n",
                result.quantization,
                result.memory_quantized_mb,
                result.compression_ratio,
                result.avg_search_latency_us,
                result.map,
                result.recall_at_10,
            ));
        }

        md.push_str("\n");
    }

    md.push_str("## Key Trade-offs Analysis\n\n");

    md.push_str("### Quality Loss by Quantization\n\n");

    for dimension in [256, 384, 512, 768, 1024] {
        let baseline = results
            .iter()
            .find(|r| r.dimension == dimension && r.quantization == "None");

        if let Some(base) = baseline {
            md.push_str(&format!("**{}D**:\n", dimension));

            for quant in ["SQ-8bit", "PQ", "Binary"] {
                if let Some(result) = results
                    .iter()
                    .find(|r| r.dimension == dimension && r.quantization == quant)
                {
                    let quality_retention = (result.map / base.map) * 100.0;

                    md.push_str(&format!(
                        "- {}: {:.1}% quality retention, {:.1}x compression\n",
                        quant, quality_retention, result.compression_ratio
                    ));
                }
            }

            md.push_str("\n");
        }
    }

    md.push_str("### Memory Savings Matrix\n\n");
    md.push_str("Comparison to 512D + No Quantization baseline:\n\n");

    let baseline_512 = results
        .iter()
        .find(|r| r.dimension == 512 && r.quantization == "None")
        .unwrap();

    md.push_str("| Config | Memory | vs Baseline | Quality | vs Baseline |\n");
    md.push_str("|--------|--------|-------------|---------|-------------|\n");

    for result in &sorted_results {
        let mem_saving = ((baseline_512.memory_quantized_mb - result.memory_quantized_mb)
            / baseline_512.memory_quantized_mb)
            * 100.0;
        let quality_diff = ((result.map / baseline_512.map) - 1.0) * 100.0;

        md.push_str(&format!(
            "| {}D+{} | {:.2} MB | {:+.1}% | {:.4} | {:+.1}% |\n",
            result.dimension,
            result.quantization,
            result.memory_quantized_mb,
            mem_saving,
            result.map,
            quality_diff,
        ));
    }

    md.push_str("\n## Recommendations\n\n");

    md.push_str("### 🥇 Overall Winner\n\n");
    md.push_str(&format!(
        "**{}D + {}** (Score: {:.3})\n\n",
        sorted_results[0].dimension,
        sorted_results[0].quantization,
        sorted_results[0].overall_score
    ));

    md.push_str("Reasons:\n");
    md.push_str(&format!(
        "- Quality: {:.4} MAP (best balance)\n",
        sorted_results[0].map
    ));
    md.push_str(&format!(
        "- Memory: {:.2} MB ({:.1}x compression)\n",
        sorted_results[0].memory_quantized_mb, sorted_results[0].compression_ratio
    ));
    md.push_str(&format!(
        "- Performance: {:.0} μs ({:.0} QPS)\n",
        sorted_results[0].avg_search_latency_us, sorted_results[0].search_throughput_qps
    ));
    md.push_str(&format!(
        "- Efficiency: {:.4} MAP/MB\n\n",
        sorted_results[0].efficiency_score
    ));

    md.push_str("### Use Case Recommendations\n\n");

    md.push_str("1. **Production Default** (balanced):\n");
    md.push_str(&format!(
        "   - {}D + {}\n",
        sorted_results[0].dimension, sorted_results[0].quantization
    ));
    md.push_str(&format!(
        "   - {:.2} MB memory, {:.4} MAP\n\n",
        sorted_results[0].memory_quantized_mb, sorted_results[0].map
    ));

    md.push_str("2. **Maximum Quality** (when accuracy critical):\n");
    md.push_str(&format!(
        "   - {}D + {}\n",
        best_quality.dimension, best_quality.quantization
    ));
    md.push_str(&format!(
        "   - {:.2} MB memory, {:.4} MAP\n\n",
        best_quality.memory_quantized_mb, best_quality.map
    ));

    md.push_str("3. **Memory Constrained** (< 2 MB target):\n");
    let memory_constrained = results
        .iter()
        .filter(|r| r.memory_quantized_mb < 2.0)
        .max_by(|a, b| a.map.partial_cmp(&b.map).unwrap());

    if let Some(config) = memory_constrained {
        md.push_str(&format!(
            "   - {}D + {}\n",
            config.dimension, config.quantization
        ));
        md.push_str(&format!(
            "   - {:.2} MB memory, {:.4} MAP\n\n",
            config.memory_quantized_mb, config.map
        ));
    }

    md.push_str("4. **Low Latency** (< 500 μs target):\n");
    let low_latency = results
        .iter()
        .filter(|r| r.avg_search_latency_us < 500.0)
        .max_by(|a, b| a.map.partial_cmp(&b.map).unwrap());

    if let Some(config) = low_latency {
        md.push_str(&format!(
            "   - {}D + {}\n",
            config.dimension, config.quantization
        ));
        md.push_str(&format!(
            "   - {:.0} μs latency, {:.4} MAP\n\n",
            config.avg_search_latency_us, config.map
        ));
    }

    md.push_str("---\n\n");
    md.push_str("*Report generated by Vectorizer Combined Optimization Benchmark*\n");

    md
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    tracing::info!("🚀 Combined Dimension + Quantization Optimization");
    tracing::info!("=================================================\n");

    let dataset = TestDataset::load_from_workspace(3000)?;

    tracing::info!(
        "📊 Dataset: {} documents, {} queries",
        dataset.documents.len(),
        dataset.queries.len()
    );

    let dimensions = vec![256, 384, 512, 768, 1024];
    let quantizations = vec!["None", "SQ-8bit", "PQ", "Binary"];

    tracing::info!(
        "\n🧪 Testing {} configurations...",
        dimensions.len() * quantizations.len()
    );
    tracing::info!("   Dimensions: {:?}", dimensions);
    tracing::info!("   Quantizations: {:?}\n", quantizations);

    let mut results = Vec::new();
    let mut completed = 0;
    let total = dimensions.len() * quantizations.len();

    for &dimension in &dimensions {
        for quantization in &quantizations {
            completed += 1;
            tracing::info!(
                "\n[{}/{}] Testing {}D + {}...",
                completed, total, dimension, quantization
            );

            match benchmark_config(&dataset, dimension, quantization) {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    tracing::info!("  ❌ Error: {}", e);
                }
            }
        }
    }

    // Generate report
    tracing::info!("\n\n📊 Generating comprehensive report...");
    let md_report = generate_report(&results, dataset.documents.len());

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");

    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }

    let report_path = report_dir.join(format!("combined_optimization_{}.md", timestamp));
    fs::write(&report_path, &md_report)?;

    let json_path = report_dir.join(format!("combined_optimization_{}.json", timestamp));
    let json_data = serde_json::to_string_pretty(&results)?;
    fs::write(&json_path, json_data)?;

    tracing::info!("✅ Reports saved:");
    tracing::info!("   📄 {}", report_path.display());
    tracing::info!("   📊 {}", json_path.display());

    // Print top 5 configurations
    let mut sorted = results.clone();
    sorted.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

    tracing::info!("\n🏆 TOP 5 CONFIGURATIONS");
    tracing::info!("======================");
    tracing::info!(
        "{:<15} {:<12} {:<12} {:<12} {:<10} {:<10}",
        "Config", "Memory", "Search", "QPS", "MAP", "Score"
    );
    tracing::info!("{}", "-".repeat(75));

    for (i, result) in sorted.iter().take(5).enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };

        tracing::info!(
            "{} {:<13} {:<12} {:<12} {:<12} {:<10} {:.3}",
            medal,
            format!("{}D+{}", result.dimension, result.quantization),
            format!("{:.1}MB", result.memory_quantized_mb),
            format!("{:.0}μs", result.avg_search_latency_us),
            format!("{:.0}", result.search_throughput_qps),
            format!("{:.4}", result.map),
            result.overall_score,
        );
    }

    tracing::info!("\n💡 FINAL RECOMMENDATION:");
    tracing::info!("========================");
    tracing::info!();
    tracing::info!("Use: {}D + {}", sorted[0].dimension, sorted[0].quantization);
    tracing::info!();
    tracing::info!("Benefits:");
    tracing::info!(
        "  ✅ Quality: {:.4} MAP ({:.1}% of best possible)",
        sorted[0].map,
        (sorted[0].map
            / sorted
                .iter()
                .map(|r| r.map)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap())
            * 100.0
    );
    tracing::info!(
        "  ✅ Memory: {:.2} MB ({:.1}x compression)",
        sorted[0].memory_quantized_mb, sorted[0].compression_ratio
    );
    tracing::info!(
        "  ✅ Speed: {:.0} μs ({:.0} QPS)",
        sorted[0].avg_search_latency_us, sorted[0].search_throughput_qps
    );
    tracing::info!("  ✅ Efficiency: {:.4} MAP/MB", sorted[0].efficiency_score);

    // Comparison to current default (512D + None)
    if let Some(current_default) = results
        .iter()
        .find(|r| r.dimension == 512 && r.quantization == "None")
    {
        let mem_improvement = ((current_default.memory_quantized_mb
            - sorted[0].memory_quantized_mb)
            / current_default.memory_quantized_mb)
            * 100.0;
        let quality_improvement = ((sorted[0].map / current_default.map) - 1.0) * 100.0;
        let speed_improvement = ((current_default.avg_search_latency_us
            - sorted[0].avg_search_latency_us)
            / current_default.avg_search_latency_us)
            * 100.0;

        tracing::info!("\nVs Current Default (512D + No Quantization):");
        tracing::info!("  {:+.1}% memory savings", mem_improvement);
        tracing::info!("  {:+.1}% quality change", quality_improvement);
        tracing::info!("  {:+.1}% speed change", speed_improvement);
    }

    tracing::info!("\n✅ Benchmark completed!");

    Ok(())
}
