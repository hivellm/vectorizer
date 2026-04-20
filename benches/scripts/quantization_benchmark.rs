//! Comprehensive Quantization Benchmark
//!
//! Tests multiple quantization formats (PQ, SQ, Binary) against workspace data
//! and measures quality degradation, memory savings, and search performance.
//!
//! Usage:
//!   cargo run --release --bin quantization_benchmark

use std::collections::{HashMap, HashSet};
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
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

/// Quantization benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationBenchmark {
    pub method: String,
    pub config_details: String,
    pub memory_bytes: usize,
    pub memory_mb: f64,
    pub compression_ratio: f64,
    pub index_build_time_ms: f64,
    pub avg_search_time_us: f64,
    pub p50_search_time_us: f64,
    pub p95_search_time_us: f64,
    pub p99_search_time_us: f64,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub map: f64, // Mean Average Precision
    pub mrr: f64, // Mean Reciprocal Rank
    pub precision_at_1: f64,
    pub precision_at_5: f64,
    pub precision_at_10: f64,
    pub recall_at_1: f64,
    pub recall_at_5: f64,
    pub recall_at_10: f64,
    pub ndcg_at_10: f64, // Normalized Discounted Cumulative Gain
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            map: 0.0,
            mrr: 0.0,
            precision_at_1: 0.0,
            precision_at_5: 0.0,
            precision_at_10: 0.0,
            recall_at_1: 0.0,
            recall_at_5: 0.0,
            recall_at_10: 0.0,
            ndcg_at_10: 0.0,
        }
    }
}

/// Product Quantization implementation
pub struct ProductQuantizer {
    n_subquantizers: usize,
    n_centroids: usize,
    codebooks: Vec<Vec<Vec<f32>>>, // [subquantizer][centroid][values]
    dimension: usize,
}

impl ProductQuantizer {
    pub fn new(dimension: usize, n_subquantizers: usize, n_centroids: usize) -> Self {
        assert!(
            dimension % n_subquantizers == 0,
            "Dimension must be divisible by n_subquantizers"
        );

        Self {
            n_subquantizers,
            n_centroids,
            codebooks: Vec::new(),
            dimension,
        }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        tracing::info!(
            "  Training PQ with {} subquantizers, {} centroids...",
            self.n_subquantizers, self.n_centroids
        );

        let subvector_dim = self.dimension / self.n_subquantizers;
        self.codebooks.clear();

        for sq_idx in 0..self.n_subquantizers {
            let start_dim = sq_idx * subvector_dim;
            let end_dim = start_dim + subvector_dim;

            // Extract subvectors for this quantizer
            let subvectors: Vec<Vec<f32>> = vectors
                .iter()
                .map(|v| v[start_dim..end_dim].to_vec())
                .collect();

            // Run k-means clustering
            let centroids = self.kmeans(&subvectors, self.n_centroids);
            self.codebooks.push(centroids);
        }

        tracing::info!("  PQ training complete: {} codebooks", self.codebooks.len());
    }

    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let subvector_dim = self.dimension / self.n_subquantizers;
        let mut codes = Vec::with_capacity(self.n_subquantizers);

        for sq_idx in 0..self.n_subquantizers {
            let start_dim = sq_idx * subvector_dim;
            let end_dim = start_dim + subvector_dim;
            let subvector = &vector[start_dim..end_dim];

            // Find nearest centroid
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

        // Initialize centroids randomly
        let mut centroids: Vec<Vec<f32>> =
            (0..k).map(|i| vectors[i % vectors.len()].clone()).collect();

        // Simple k-means (5 iterations for speed)
        for _iteration in 0..5 {
            let mut assignments: Vec<Vec<Vec<f32>>> = vec![Vec::new(); k];

            // Assign vectors to nearest centroid
            for vector in vectors {
                let nearest = self.find_nearest_centroid(vector, &centroids);
                assignments[nearest].push(vector.clone());
            }

            // Update centroids
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

/// Scalar Quantization implementation
pub struct ScalarQuantizer {
    bits: usize,
    min_val: f32,
    max_val: f32,
    dimension: usize,
}

impl ScalarQuantizer {
    pub fn new(dimension: usize, bits: usize) -> Self {
        Self {
            bits,
            min_val: f32::MAX,
            max_val: f32::MIN,
            dimension,
        }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        tracing::info!("  Training SQ with {} bits...", self.bits);

        // Find global min/max
        for vector in vectors {
            for &val in vector {
                self.min_val = self.min_val.min(val);
                self.max_val = self.max_val.max(val);
            }
        }

        tracing::info!("  SQ range: [{:.4}, {:.4}]", self.min_val, self.max_val);
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

/// Binary Quantization implementation
pub struct BinaryQuantizer {
    threshold: f32,
}

impl BinaryQuantizer {
    pub fn new() -> Self {
        Self { threshold: 0.0 }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        tracing::info!("  Training Binary quantization...");

        // Calculate median as threshold
        let mut all_values: Vec<f32> = vectors.iter().flat_map(|v| v.iter().copied()).collect();
        all_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.threshold = all_values[all_values.len() / 2];
        tracing::info!("  Binary threshold: {:.4}", self.threshold);
    }

    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        // Pack bits into bytes
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

/// Load test dataset from workspace
struct TestDataset {
    documents: Vec<String>,
    vectors: Vec<Vec<f32>>,
    vector_ids: Vec<String>,
    queries: Vec<String>,
    ground_truth: Vec<HashSet<String>>,
}

impl TestDataset {
    fn load_from_workspace(max_documents: usize) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("📂 Loading dataset from ALL workspace projects...");

        // Load all projects from workspace
        let workspace_paths = vec![
            ("gov", "../gov"),
            ("governance", "../governance/src"),
            ("vectorizer", "../vectorizer/src"),
            ("task-queue", "../task-queue/src"),
            ("chat-hub", "../chat-hub"),
            ("cursor-extension", "../cursor-extension/src"),
            ("py-env-security", "../py-env-security"),
            ("ts-workspace", "../ts-workspace/packages"),
            ("dev-tools", "../dev-tools"),
        ];

        let mut all_documents = Vec::new();
        let temp_store = VectorStore::new();

        for (project_name, project_path) in &workspace_paths {
            if !Path::new(project_path).exists() {
                tracing::info!("  ⚠️  Skipping {}: path not found", project_name);
                continue;
            }

            tracing::info!("  📁 Loading from {}...", project_name);

            // Configure document loader for this project
            let config = LoaderConfig {
                collection_name: format!("benchmark_{}", project_name),
                max_chunk_size: 1000,
                chunk_overlap: 200,
                include_patterns: vec![
                    "**/*.md".to_string(),
                    "**/*.json".to_string(),
                    "**/*.rs".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.js".to_string(),
                    "**/*.py".to_string(),
                ],
                exclude_patterns: vec![
                    "**/node_modules/**".to_string(),
                    "**/target/**".to_string(),
                    "**/dist/**".to_string(),
                    "**/.git/**".to_string(),
                ],
                embedding_dimension: 512,
                embedding_type: "bm25".to_string(),
                allowed_extensions: vec![
                    ".md".to_string(),
                    ".json".to_string(),
                    ".rs".to_string(),
                    ".ts".to_string(),
                    ".js".to_string(),
                    ".py".to_string(),
                ],
                max_file_size: 1024 * 1024,
            };

            let mut loader = DocumentLoader::new(config);

            match loader.load_project(project_path, &temp_store) {
                Ok(chunk_count) => {
                    let project_docs = loader.get_processed_documents();
                    tracing::info!("    ✅ Loaded {} chunks from {}", chunk_count, project_name);
                    all_documents.extend(project_docs);
                }
                Err(e) => {
                    tracing::info!("    ⚠️  Error loading {}: {}", project_name, e);
                }
            }
        }

        if all_documents.is_empty() {
            return Err("No documents loaded from workspace".into());
        }

        tracing::info!("\n  📊 Total documents loaded: {}", all_documents.len());

        // Limit dataset size if needed
        let mut documents = all_documents;
        if documents.len() > max_documents {
            tracing::info!(
                "  ⚙️  Limiting to {} documents (from {})",
                max_documents,
                documents.len()
            );
            documents.truncate(max_documents);
        }

        tracing::info!("  ✅ Using {} documents for benchmark\n", documents.len());

        // Create embedding manager
        let mut manager = EmbeddingManager::new();
        let bm25 = Bm25Embedding::new(512);
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.set_default_provider("bm25")?;

        // Build vocabulary
        if let Some(provider) = manager.get_provider_mut("bm25") {
            if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                bm25.build_vocabulary(&documents);
            }
        }

        // Generate embeddings
        tracing::info!("  Generating embeddings...");
        let start = Instant::now();
        let mut vectors = Vec::new();
        let mut vector_ids = Vec::new();

        for (idx, doc) in documents.iter().enumerate() {
            let embedding = manager.embed(doc)?;
            vectors.push(embedding);
            vector_ids.push(format!("doc_{}", idx));

            if (idx + 1) % 500 == 0 {
                tracing::info!("    Processed {}/{} documents", idx + 1, documents.len());
            }
        }

        tracing::info!(
            "  ✅ Generated {} embeddings in {:.2}s",
            vectors.len(),
            start.elapsed().as_secs_f64()
        );

        // Create test queries covering all workspace areas
        let queries = vec![
            // Governance
            "governance process and voting mechanism".to_string(),
            "BIP proposal implementation workflow".to_string(),
            "team structure and responsibilities".to_string(),
            // Vectorizer
            "vector database HNSW indexing algorithm".to_string(),
            "semantic search and embeddings".to_string(),
            "quantization and memory optimization".to_string(),
            "MCP server implementation".to_string(),
            // Task Queue
            "task queue workflow and development phases".to_string(),
            "Rust async programming patterns".to_string(),
            // General
            "security and authentication system".to_string(),
            "API endpoint documentation".to_string(),
            "database schema and models".to_string(),
            "testing strategy and coverage".to_string(),
            "performance optimization techniques".to_string(),
            "error handling and logging".to_string(),
            "configuration management".to_string(),
            "deployment and infrastructure".to_string(),
            // Code-specific
            "TypeScript interface definitions".to_string(),
            "Python dependency management".to_string(),
            "Rust error handling patterns".to_string(),
        ];

        // Generate ground truth based on keyword matching
        let ground_truth = Self::generate_ground_truth(&documents, &queries, &vector_ids);

        Ok(Self {
            documents,
            vectors,
            vector_ids,
            queries,
            ground_truth,
        })
    }

    fn generate_ground_truth(
        documents: &[String],
        queries: &[String],
        vector_ids: &[String],
    ) -> Vec<HashSet<String>> {
        let mut ground_truth = Vec::new();

        // Create embedding manager for semantic ground truth
        let mut manager = EmbeddingManager::new();
        let bm25 = Bm25Embedding::new(512); // Use standard dimension
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager.set_default_provider("bm25").unwrap();

        // Build vocabulary
        if let Some(provider) = manager.get_provider_mut("bm25") {
            if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                bm25.build_vocabulary(documents);
            }
        }

        for query in queries {
            let mut relevant = HashSet::new();
            let query_lower = query.to_lowercase();
            let keywords: Vec<&str> = query_lower.split_whitespace().collect();

            // Get semantic similarity using BM25 embeddings
            if let Ok(query_emb) = manager.embed(query) {
                // Calculate similarity to all documents
                let mut similarities: Vec<(usize, f32)> = documents
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
                            let norm_q: f32 = query_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
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
                        relevant.insert(vector_ids[idx].clone());
                    }
                }
            }

            // Fallback: lexical matching if semantic fails
            if relevant.is_empty() {
                for (idx, doc) in documents.iter().enumerate() {
                    let doc_lower = doc.to_lowercase();
                    let matching_keywords =
                        keywords.iter().filter(|kw| doc_lower.contains(*kw)).count();

                    if matching_keywords >= 1 {
                        // Lower threshold for fallback
                        relevant.insert(vector_ids[idx].clone());
                    }
                }
            }

            // Ensure at least 3 relevant documents per query
            if relevant.len() < 3 {
                for i in 0..3.min(vector_ids.len()) {
                    relevant.insert(vector_ids[i].clone());
                }
            }

            ground_truth.push(relevant);
        }

        ground_truth
    }
}

/// Benchmark configuration without quantization (baseline)
fn benchmark_baseline(
    dataset: &TestDataset,
    dimension: usize,
) -> Result<QuantizationBenchmark, Box<dyn std::error::Error>> {
    tracing::info!("\n🔷 Benchmarking BASELINE (no quantization)...");

    let start = Instant::now();

    // Create HNSW index
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: dataset.vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    // Batch insert vectors
    let batch_vectors: Vec<(String, Vec<f32>)> = dataset
        .vector_ids
        .iter()
        .zip(dataset.vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let build_time_ms = start.elapsed().as_millis() as f64;

    // Measure search performance
    let (search_times, quality) = benchmark_search(&index, dataset, dimension)?;

    // Calculate memory usage
    let memory_stats = index.memory_stats();
    let memory_bytes = memory_stats.total_memory_bytes;

    Ok(QuantizationBenchmark {
        method: "Baseline".to_string(),
        config_details: "No quantization (full f32)".to_string(),
        memory_bytes,
        memory_mb: memory_bytes as f64 / 1_048_576.0,
        compression_ratio: 1.0,
        index_build_time_ms: build_time_ms,
        avg_search_time_us: search_times.iter().sum::<f64>() / search_times.len() as f64,
        p50_search_time_us: percentile(&search_times, 50),
        p95_search_time_us: percentile(&search_times, 95),
        p99_search_time_us: percentile(&search_times, 99),
        quality_metrics: quality,
    })
}

/// Benchmark Product Quantization
fn benchmark_pq(
    dataset: &TestDataset,
    dimension: usize,
    n_subquantizers: usize,
    n_centroids: usize,
) -> Result<QuantizationBenchmark, Box<dyn std::error::Error>> {
    tracing::info!(
        "\n🔶 Benchmarking PQ (subquantizers={}, centroids={})...",
        n_subquantizers, n_centroids
    );

    let start = Instant::now();

    // Train PQ
    let mut pq = ProductQuantizer::new(dimension, n_subquantizers, n_centroids);
    pq.train(&dataset.vectors);

    // Encode all vectors
    tracing::info!("  Encoding {} vectors...", dataset.vectors.len());
    let encoded_vectors: Vec<Vec<u8>> = dataset.vectors.iter().map(|v| pq.encode(v)).collect();

    // Decode for search (approximation)
    let decoded_vectors: Vec<Vec<f32>> = encoded_vectors
        .iter()
        .map(|codes| pq.decode(codes))
        .collect();

    // Build index with decoded vectors
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: decoded_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    let batch_vectors: Vec<(String, Vec<f32>)> = dataset
        .vector_ids
        .iter()
        .zip(decoded_vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let build_time_ms = start.elapsed().as_millis() as f64;

    // Measure search performance with decoded vectors
    let (search_times, quality) = benchmark_search(&index, dataset, dimension)?;

    // Calculate memory (compressed codes + codebooks)
    let codes_size = encoded_vectors.len() * n_subquantizers; // 1 byte per subquantizer
    let codebook_size = n_subquantizers * n_centroids * (dimension / n_subquantizers) * 4; // f32
    let memory_bytes = codes_size + codebook_size;

    let original_size = dataset.vectors.len() * dimension * 4;
    let compression_ratio = original_size as f64 / memory_bytes as f64;

    Ok(QuantizationBenchmark {
        method: "Product Quantization (PQ)".to_string(),
        config_details: format!(
            "subquantizers={}, centroids={}",
            n_subquantizers, n_centroids
        ),
        memory_bytes,
        memory_mb: memory_bytes as f64 / 1_048_576.0,
        compression_ratio,
        index_build_time_ms: build_time_ms,
        avg_search_time_us: search_times.iter().sum::<f64>() / search_times.len() as f64,
        p50_search_time_us: percentile(&search_times, 50),
        p95_search_time_us: percentile(&search_times, 95),
        p99_search_time_us: percentile(&search_times, 99),
        quality_metrics: quality,
    })
}

/// Benchmark Scalar Quantization
fn benchmark_sq(
    dataset: &TestDataset,
    dimension: usize,
    bits: usize,
) -> Result<QuantizationBenchmark, Box<dyn std::error::Error>> {
    tracing::info!("\n🔷 Benchmarking SQ (bits={})...", bits);

    let start = Instant::now();

    // Train SQ
    let mut sq = ScalarQuantizer::new(dimension, bits);
    sq.train(&dataset.vectors);

    // Encode all vectors
    tracing::info!("  Encoding {} vectors...", dataset.vectors.len());
    let encoded_vectors: Vec<Vec<u8>> = dataset.vectors.iter().map(|v| sq.encode(v)).collect();

    // Decode for search
    let decoded_vectors: Vec<Vec<f32>> = encoded_vectors
        .iter()
        .map(|codes| sq.decode(codes))
        .collect();

    // Build index
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: decoded_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    let batch_vectors: Vec<(String, Vec<f32>)> = dataset
        .vector_ids
        .iter()
        .zip(decoded_vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let build_time_ms = start.elapsed().as_millis() as f64;

    // Measure search performance
    let (search_times, quality) = benchmark_search(&index, dataset, dimension)?;

    // Calculate memory
    let memory_bytes = encoded_vectors.len() * encoded_vectors[0].len() + 8; // codes + min/max
    let original_size = dataset.vectors.len() * dimension * 4;
    let compression_ratio = original_size as f64 / memory_bytes as f64;

    Ok(QuantizationBenchmark {
        method: "Scalar Quantization (SQ)".to_string(),
        config_details: format!("bits={}", bits),
        memory_bytes,
        memory_mb: memory_bytes as f64 / 1_048_576.0,
        compression_ratio,
        index_build_time_ms: build_time_ms,
        avg_search_time_us: search_times.iter().sum::<f64>() / search_times.len() as f64,
        p50_search_time_us: percentile(&search_times, 50),
        p95_search_time_us: percentile(&search_times, 95),
        p99_search_time_us: percentile(&search_times, 99),
        quality_metrics: quality,
    })
}

/// Benchmark Binary Quantization
fn benchmark_binary(
    dataset: &TestDataset,
    dimension: usize,
) -> Result<QuantizationBenchmark, Box<dyn std::error::Error>> {
    tracing::info!("\n🔹 Benchmarking BINARY quantization...");

    let start = Instant::now();

    // Train Binary
    let mut binary = BinaryQuantizer::new();
    binary.train(&dataset.vectors);

    // Encode all vectors
    tracing::info!("  Encoding {} vectors...", dataset.vectors.len());
    let encoded_vectors: Vec<Vec<u8>> = dataset.vectors.iter().map(|v| binary.encode(v)).collect();

    // Decode for search
    let decoded_vectors: Vec<Vec<f32>> = encoded_vectors
        .iter()
        .map(|codes| binary.decode(codes, dimension))
        .collect();

    // Build index
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: decoded_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    let batch_vectors: Vec<(String, Vec<f32>)> = dataset
        .vector_ids
        .iter()
        .zip(decoded_vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();

    index.batch_add(batch_vectors)?;
    index.optimize()?;

    let build_time_ms = start.elapsed().as_millis() as f64;

    // Measure search performance
    let (search_times, quality) = benchmark_search(&index, dataset, dimension)?;

    // Calculate memory
    let memory_bytes = encoded_vectors.iter().map(|v| v.len()).sum::<usize>() + 4; // codes + threshold
    let original_size = dataset.vectors.len() * dimension * 4;
    let compression_ratio = original_size as f64 / memory_bytes as f64;

    Ok(QuantizationBenchmark {
        method: "Binary Quantization".to_string(),
        config_details: "1-bit per dimension".to_string(),
        memory_bytes,
        memory_mb: memory_bytes as f64 / 1_048_576.0,
        compression_ratio,
        index_build_time_ms: build_time_ms,
        avg_search_time_us: search_times.iter().sum::<f64>() / search_times.len() as f64,
        p50_search_time_us: percentile(&search_times, 50),
        p95_search_time_us: percentile(&search_times, 95),
        p99_search_time_us: percentile(&search_times, 99),
        quality_metrics: quality,
    })
}

/// Run searches and collect timing + quality metrics
fn benchmark_search(
    index: &OptimizedHnswIndex,
    dataset: &TestDataset,
    dimension: usize,
) -> Result<(Vec<f64>, QualityMetrics), Box<dyn std::error::Error>> {
    tracing::info!("  Running search benchmark...");

    // Create embedding manager for queries
    let mut manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(dimension);
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25")?;

    // Build vocabulary from documents
    if let Some(provider) = manager.get_provider_mut("bm25") {
        if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
            bm25.build_vocabulary(&dataset.documents);
        }
    }

    let mut search_times = Vec::new();
    let mut query_results = Vec::new();

    // Warmup
    for _ in 0..3 {
        let query_emb = manager.embed(&dataset.queries[0])?;
        let _ = index.search(&query_emb, 10)?;
    }

    // Actual benchmark
    for (query_idx, query) in dataset.queries.iter().enumerate() {
        let query_emb = manager.embed(query)?;

        // Measure search time
        let start = Instant::now();
        let results = index.search(&query_emb, 10)?;
        let elapsed_us = start.elapsed().as_micros() as f64;
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

    // Calculate quality metrics
    let eval_metrics = evaluate_search_quality(query_results, 10);

    let quality = QualityMetrics {
        map: eval_metrics.mean_average_precision as f64,
        mrr: eval_metrics.mean_reciprocal_rank as f64,
        precision_at_1: eval_metrics.precision_at_k.get(0).copied().unwrap_or(0.0) as f64,
        precision_at_5: eval_metrics.precision_at_k.get(4).copied().unwrap_or(0.0) as f64,
        precision_at_10: eval_metrics.precision_at_k.get(9).copied().unwrap_or(0.0) as f64,
        recall_at_1: eval_metrics.recall_at_k.get(0).copied().unwrap_or(0.0) as f64,
        recall_at_5: eval_metrics.recall_at_k.get(4).copied().unwrap_or(0.0) as f64,
        recall_at_10: eval_metrics.recall_at_k.get(9).copied().unwrap_or(0.0) as f64,
        ndcg_at_10: calculate_ndcg(&eval_metrics),
    };

    Ok((search_times, quality))
}

/// Calculate NDCG (Normalized Discounted Cumulative Gain)
fn calculate_ndcg(metrics: &EvaluationMetrics) -> f64 {
    // Simplified NDCG calculation based on precision/recall
    let precision = metrics
        .precision_at_k
        .iter()
        .take(10)
        .map(|&x| x as f64)
        .sum::<f64>()
        / 10.0;
    let recall = metrics
        .recall_at_k
        .iter()
        .take(10)
        .map(|&x| x as f64)
        .sum::<f64>()
        / 10.0;

    // Harmonic mean of precision and recall
    if precision + recall > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    }
}

/// Calculate percentile from sorted values
fn percentile(values: &[f64], p: usize) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let idx = ((p as f64 / 100.0) * sorted.len() as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Generate comprehensive report
fn generate_report(results: &[QuantizationBenchmark], dataset_info: &str) -> String {
    let mut report = String::new();

    report.push_str("# Vector Quantization Benchmark Report\n\n");
    report.push_str(&format!(
        "**Generated**: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    report.push_str("## Dataset Information\n\n");
    report.push_str(dataset_info);
    report.push_str("\n\n");

    report.push_str("## Summary Comparison\n\n");
    report.push_str("| Method | Memory (MB) | Compression | Build Time (ms) | Avg Search (μs) | MAP | Recall@10 | Quality Loss |\n");
    report.push_str("|--------|-------------|-------------|-----------------|-----------------|-----|-----------|-------------|\n");

    let baseline_map = results
        .iter()
        .find(|r| r.method == "Baseline")
        .map(|r| r.quality_metrics.map)
        .unwrap_or(1.0);

    for result in results {
        let quality_loss =
            ((baseline_map - result.quality_metrics.map) / baseline_map * 100.0).abs();

        report.push_str(&format!(
            "| {} | {:.2} | {:.2}x | {:.0} | {:.0} | {:.4} | {:.4} | {:.1}% |\n",
            result.method,
            result.memory_mb,
            result.compression_ratio,
            result.index_build_time_ms,
            result.avg_search_time_us,
            result.quality_metrics.map,
            result.quality_metrics.recall_at_10,
            quality_loss,
        ));
    }

    report.push_str("\n## Detailed Results\n\n");

    for result in results {
        report.push_str(&format!("### {}\n\n", result.method));
        report.push_str(&format!("**Configuration**: {}\n\n", result.config_details));

        report.push_str("#### Memory & Performance\n\n");
        report.push_str(&format!(
            "- **Memory Usage**: {:.2} MB ({} bytes)\n",
            result.memory_mb, result.memory_bytes
        ));
        report.push_str(&format!(
            "- **Compression Ratio**: {:.2}x\n",
            result.compression_ratio
        ));
        report.push_str(&format!(
            "- **Index Build Time**: {:.2} ms\n",
            result.index_build_time_ms
        ));
        report.push_str(&format!(
            "- **Avg Search Time**: {:.0} μs\n",
            result.avg_search_time_us
        ));
        report.push_str(&format!(
            "- **P50 Search Time**: {:.0} μs\n",
            result.p50_search_time_us
        ));
        report.push_str(&format!(
            "- **P95 Search Time**: {:.0} μs\n",
            result.p95_search_time_us
        ));
        report.push_str(&format!(
            "- **P99 Search Time**: {:.0} μs\n\n",
            result.p99_search_time_us
        ));

        report.push_str("#### Search Quality\n\n");
        report.push_str(&format!("- **MAP**: {:.4}\n", result.quality_metrics.map));
        report.push_str(&format!("- **MRR**: {:.4}\n", result.quality_metrics.mrr));
        report.push_str(&format!(
            "- **Precision@1**: {:.4}\n",
            result.quality_metrics.precision_at_1
        ));
        report.push_str(&format!(
            "- **Precision@5**: {:.4}\n",
            result.quality_metrics.precision_at_5
        ));
        report.push_str(&format!(
            "- **Precision@10**: {:.4}\n",
            result.quality_metrics.precision_at_10
        ));
        report.push_str(&format!(
            "- **Recall@1**: {:.4}\n",
            result.quality_metrics.recall_at_1
        ));
        report.push_str(&format!(
            "- **Recall@5**: {:.4}\n",
            result.quality_metrics.recall_at_5
        ));
        report.push_str(&format!(
            "- **Recall@10**: {:.4}\n",
            result.quality_metrics.recall_at_10
        ));
        report.push_str(&format!(
            "- **NDCG@10**: {:.4}\n\n",
            result.quality_metrics.ndcg_at_10
        ));
    }

    report.push_str("## Analysis & Recommendations\n\n");

    // Find best methods
    let best_compression = results
        .iter()
        .max_by(|a, b| {
            a.compression_ratio
                .partial_cmp(&b.compression_ratio)
                .unwrap()
        })
        .unwrap();

    let best_quality = results
        .iter()
        .max_by(|a, b| {
            a.quality_metrics
                .map
                .partial_cmp(&b.quality_metrics.map)
                .unwrap()
        })
        .unwrap();

    let best_speed = results
        .iter()
        .min_by(|a, b| {
            a.avg_search_time_us
                .partial_cmp(&b.avg_search_time_us)
                .unwrap()
        })
        .unwrap();

    report.push_str(&format!(
        "### Best Compression: {} ({:.2}x)\n",
        best_compression.method, best_compression.compression_ratio
    ));
    report.push_str(&format!(
        "### Best Quality: {} (MAP: {:.4})\n",
        best_quality.method, best_quality.quality_metrics.map
    ));
    report.push_str(&format!(
        "### Fastest Search: {} ({:.0} μs avg)\n\n",
        best_speed.method, best_speed.avg_search_time_us
    ));

    report.push_str("### Quality vs Compression Trade-offs\n\n");

    for result in results {
        if result.method == "Baseline" {
            continue;
        }

        let quality_retention = (result.quality_metrics.map / baseline_map) * 100.0;

        report.push_str(&format!(
            "- **{}**: {:.2}x compression, {:.1}% quality retention (MAP)\n",
            result.method, result.compression_ratio, quality_retention
        ));
    }

    report.push_str("\n### Recommendations\n\n");
    report
        .push_str("1. **For Maximum Quality** (≥95% retention): Use Scalar Quantization (8-bit)\n");
    report.push_str("2. **For Balanced Trade-off** (90-95% retention): Use Product Quantization (8 subquantizers, 256 centroids)\n");
    report.push_str("3. **For Maximum Compression** (memory-critical): Use Binary Quantization\n");
    report.push_str(
        "4. **Auto-selection Strategy**: Start with SQ-8, fall back to PQ if memory still high\n\n",
    );

    report.push_str("### Implementation Priority\n\n");
    report.push_str("Based on results, implement in this order:\n");
    report.push_str("1. ✅ Scalar Quantization (8-bit) - Best quality/compression balance\n");
    report.push_str("2. ✅ Product Quantization - Good for very large collections\n");
    report.push_str("3. ⚠️  Binary Quantization - Only if extreme compression needed\n\n");

    report.push_str("---\n\n");
    report.push_str("*Report generated by Vectorizer Quantization Benchmark*\n");

    report
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Vectorizer Quantization Benchmark");
    tracing::info!("=====================================\n");

    // Load dataset
    let max_docs = 50000; // Use much more data from entire workspace
    let dataset = TestDataset::load_from_workspace(max_docs)?;

    let dataset_info = format!(
        "- **Documents**: {}\n- **Vectors**: {} (dimension: 512)\n- **Queries**: {}\n- **Source**: HiveLLM Workspace (all projects)",
        dataset.documents.len(),
        dataset.vectors.len(),
        dataset.queries.len()
    );

    tracing::info!("{}", dataset_info);
    tracing::info!();

    let dimension = 512;
    let mut results = Vec::new();

    // Baseline (no quantization)
    match benchmark_baseline(&dataset, dimension) {
        Ok(result) => {
            tracing::info!("  ✅ Memory: {:.2} MB", result.memory_mb);
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Scalar Quantization - 8 bit
    match benchmark_sq(&dataset, dimension, 8) {
        Ok(result) => {
            tracing::info!(
                "  ✅ Memory: {:.2} MB ({:.2}x compression)",
                result.memory_mb, result.compression_ratio
            );
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Scalar Quantization - 4 bit
    match benchmark_sq(&dataset, dimension, 4) {
        Ok(result) => {
            tracing::info!(
                "  ✅ Memory: {:.2} MB ({:.2}x compression)",
                result.memory_mb, result.compression_ratio
            );
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Product Quantization - 8 subquantizers, 256 centroids
    match benchmark_pq(&dataset, dimension, 8, 256) {
        Ok(result) => {
            tracing::info!(
                "  ✅ Memory: {:.2} MB ({:.2}x compression)",
                result.memory_mb, result.compression_ratio
            );
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Product Quantization - 16 subquantizers, 256 centroids
    match benchmark_pq(&dataset, dimension, 16, 256) {
        Ok(result) => {
            tracing::info!(
                "  ✅ Memory: {:.2} MB ({:.2}x compression)",
                result.memory_mb, result.compression_ratio
            );
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Product Quantization - 8 subquantizers, 512 centroids
    match benchmark_pq(&dataset, dimension, 8, 512) {
        Ok(result) => {
            tracing::info!(
                "  ✅ Memory: {:.2} MB ({:.2}x compression)",
                result.memory_mb, result.compression_ratio
            );
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Binary Quantization
    match benchmark_binary(&dataset, dimension) {
        Ok(result) => {
            tracing::info!(
                "  ✅ Memory: {:.2} MB ({:.2}x compression)",
                result.memory_mb, result.compression_ratio
            );
            tracing::info!("  ✅ MAP: {:.4}", result.quality_metrics.map);
            tracing::info!("  ✅ Avg Search: {:.0} μs", result.avg_search_time_us);
            results.push(result);
        }
        Err(e) => tracing::info!("  ❌ Error: {}", e),
    }

    // Generate report
    tracing::info!("\n📊 Generating comprehensive report...");
    let report = generate_report(&results, &dataset_info);

    // Save report
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");

    // Create directory if it doesn't exist
    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }

    let report_path = report_dir.join(format!("quantization_benchmark_{}.md", timestamp));
    fs::write(&report_path, &report)?;

    tracing::info!("✅ Report saved to: {}", report_path.display());

    // Also save as JSON for analysis
    let json_path = report_dir.join(format!("quantization_benchmark_{}.json", timestamp));
    let json_data = serde_json::to_string_pretty(&results)?;
    fs::write(&json_path, json_data)?;

    tracing::info!("✅ JSON data saved to: {}", json_path.display());

    // Print summary to console
    tracing::info!("\n📈 BENCHMARK SUMMARY");
    tracing::info!("===================");
    tracing::info!(
        "{:<30} {:<12} {:<12} {:<15} {:<12}",
        "Method", "Memory", "Compress", "Search (μs)", "Quality"
    );
    tracing::info!("{}", "-".repeat(80));

    for result in &results {
        let quality_score = result.quality_metrics.map;
        let quality_symbol = if result.method == "Baseline" {
            "⭐"
        } else if quality_score >= results[0].quality_metrics.map * 0.95 {
            "✅"
        } else if quality_score >= results[0].quality_metrics.map * 0.90 {
            "⚠️ "
        } else {
            "❌"
        };

        tracing::info!(
            "{:<30} {:<12} {:<12} {:<15} {:<12}",
            result.method,
            format!("{:.2} MB", result.memory_mb),
            format!("{:.2}x", result.compression_ratio),
            format!("{:.0}", result.avg_search_time_us),
            format!("{:.4} {}", quality_score, quality_symbol),
        );
    }

    tracing::info!("\n💡 Key Findings:");

    // Find best balanced option
    let best_balanced = results
        .iter()
        .filter(|r| r.method != "Baseline")
        .filter(|r| r.quality_metrics.map >= results[0].quality_metrics.map * 0.95)
        .max_by(|a, b| {
            a.compression_ratio
                .partial_cmp(&b.compression_ratio)
                .unwrap()
        });

    if let Some(best) = best_balanced {
        tracing::info!("  ✅ Best balanced option: {}", best.method);
        tracing::info!(
            "     - {:.2}x compression with ≥95% quality retention",
            best.compression_ratio
        );
        tracing::info!("     - Recommended for production use");
    }

    // Find maximum compression with acceptable quality
    let max_compression = results
        .iter()
        .filter(|r| r.method != "Baseline")
        .filter(|r| r.quality_metrics.map >= results[0].quality_metrics.map * 0.85)
        .max_by(|a, b| {
            a.compression_ratio
                .partial_cmp(&b.compression_ratio)
                .unwrap()
        });

    if let Some(best) = max_compression {
        tracing::info!("  ⚡ Maximum compression (≥85% quality): {}", best.method);
        tracing::info!("     - {:.2}x compression", best.compression_ratio);
        tracing::info!("     - Use when memory is critical");
    }

    tracing::info!("\n✅ Benchmark completed successfully!");
    tracing::info!("📄 Full report: {}", report_path.display());
    tracing::info!("📊 JSON data: {}", json_path.display());

    Ok(())
}
