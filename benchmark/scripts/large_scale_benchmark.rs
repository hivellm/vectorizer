//! Large Scale Benchmark - Millions of Vectors
//!
//! Real-world benchmark with 100K-1M+ vectors to test:
//! - Quantization quality at scale
//! - Memory usage patterns
//! - Search performance degradation
//! - Index build time
//!
//! Usage:
//!   cargo run --release --bin large_scale_benchmark -- [vector_count]

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tracing_subscriber;

use vectorizer::{
    VectorStore,
    db::{OptimizedHnswConfig, OptimizedHnswIndex},
    document_loader::{DocumentLoader, LoaderConfig},
    embedding::{Bm25Embedding, EmbeddingManager, EmbeddingProvider},
    evaluation::{EvaluationMetrics, QueryResult, evaluate_search_quality},
};

/// Corrected Binary Quantizer - maintains proper normalization
pub struct BinaryQuantizer {
    threshold: f32,
}

impl BinaryQuantizer {
    pub fn new() -> Self {
        Self { threshold: 0.0 }
    }
    
    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        // Use mean as threshold for normalized vectors
        let total: f32 = vectors.iter()
            .flat_map(|v| v.iter().copied())
            .sum();
        let count = vectors.iter().map(|v| v.len()).sum::<usize>();
        
        self.threshold = total / count as f32;
        println!("  Binary threshold (mean): {:.6}", self.threshold);
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
        
        // Decode and normalize properly
        let mut pos_count = 0;
        
        for i in 0..dimension {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            
            if byte_idx < codes.len() {
                let bit_set = (codes[byte_idx] & (1 << bit_idx)) != 0;
                if bit_set {
                    vector[i] = 1.0;
                    pos_count += 1;
                }
            }
        }
        
        // Normalize to unit length for cosine similarity
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut vector {
                *val /= norm;
            }
        }
        
        vector
    }
}

/// Product Quantizer (corrected)
pub struct ProductQuantizer {
    n_subquantizers: usize,
    n_centroids: usize,
    codebooks: Vec<Vec<Vec<f32>>>,
    dimension: usize,
}

impl ProductQuantizer {
    pub fn new(dimension: usize, n_subquantizers: usize, n_centroids: usize) -> Self {
        assert!(dimension % n_subquantizers == 0);
        Self {
            n_subquantizers,
            n_centroids,
            codebooks: Vec::new(),
            dimension,
        }
    }
    
    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        println!("  Training PQ: {} subq, {} centroids, {} training vectors", 
            self.n_subquantizers, self.n_centroids, vectors.len());
        
        let subvector_dim = self.dimension / self.n_subquantizers;
        self.codebooks.clear();
        
        // Train each subquantizer
        for sq_idx in 0..self.n_subquantizers {
            let start_dim = sq_idx * subvector_dim;
            let end_dim = start_dim + subvector_dim;
            
            let subvectors: Vec<Vec<f32>> = vectors.iter()
                .map(|v| v[start_dim..end_dim].to_vec())
                .collect();
            
            let centroids = self.kmeans(&subvectors, self.n_centroids);
            self.codebooks.push(centroids);
            
            if sq_idx % 2 == 0 {
                println!("    Trained subquantizer {}/{}", sq_idx + 1, self.n_subquantizers);
            }
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
        
        // Renormalize for cosine similarity
        let norm: f32 = reconstructed.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut reconstructed {
                *val /= norm;
            }
        }
        
        reconstructed
    }
    
    fn kmeans(&self, vectors: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        if vectors.is_empty() { return Vec::new(); }
        
        let dim = vectors[0].len();
        let k = k.min(vectors.len());
        
        // Initialize with k-means++
        let mut centroids: Vec<Vec<f32>> = Vec::new();
        centroids.push(vectors[0].clone());
        
        for _ in 1..k {
            centroids.push(vectors[centroids.len() % vectors.len()].clone());
        }
        
        // Run k-means (10 iterations for better quality)
        for _ in 0..10 {
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
        centroids.iter()
            .enumerate()
            .map(|(idx, centroid)| {
                let dist: f32 = vector.iter()
                    .zip(centroid.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>();
                (idx, dist)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
    
    fn compute_mean(vectors: &[Vec<f32>], dim: usize) -> Vec<f32> {
        let mut mean = vec![0.0; dim];
        for vector in vectors {
            for (i, &val) in vector.iter().enumerate() {
                mean[i] += val;
            }
        }
        let count = vectors.len() as f32;
        for val in &mut mean {
            *val /= count;
        }
        mean
    }
}

/// Scalar Quantizer (corrected)
pub struct ScalarQuantizer {
    bits: usize,
    min_val: f32,
    max_val: f32,
}

impl ScalarQuantizer {
    pub fn new(bits: usize) -> Self {
        Self { bits, min_val: f32::MAX, max_val: f32::MIN }
    }
    
    pub fn train(&mut self, vectors: &[Vec<f32>]) {
        for vector in vectors {
            for &val in vector {
                self.min_val = self.min_val.min(val);
                self.max_val = self.max_val.max(val);
            }
        }
        println!("  SQ range: [{:.6}, {:.6}]", self.min_val, self.max_val);
    }
    
    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let levels = (1 << self.bits) - 1;
        let range = self.max_val - self.min_val;
        
        if range == 0.0 {
            return vec![0u8; vector.len()];
        }
        
        vector.iter()
            .map(|&val| {
                let normalized = ((val - self.min_val) / range).clamp(0.0, 1.0);
                (normalized * levels as f32).round() as u8
            })
            .collect()
    }
    
    pub fn decode(&self, codes: &[u8]) -> Vec<f32> {
        let levels = (1 << self.bits) - 1;
        let range = self.max_val - self.min_val;
        
        let mut decoded: Vec<f32> = codes.iter()
            .map(|&code| {
                let normalized = code as f32 / levels as f32;
                self.min_val + normalized * range
            })
            .collect();
        
        // Renormalize for cosine similarity
        let norm: f32 = decoded.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut decoded {
                *val /= norm;
            }
        }
        
        decoded
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleBenchmark {
    pub dataset_size: usize,
    pub dimension: usize,
    pub quantization: String,
    pub memory_mb: f64,
    pub compression_ratio: f64,
    pub index_build_time_s: f64,
    pub avg_search_latency_ms: f64,
    pub p95_search_latency_ms: f64,
    pub p99_search_latency_ms: f64,
    pub throughput_qps: f64,
    pub map: f64,
    pub recall_at_10: f64,
}

fn percentile(values: &[f64], p: usize) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((p as f64 / 100.0) * sorted.len() as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Load massive dataset from entire workspace
fn load_massive_dataset(target_size: usize) -> Result<(Vec<String>, Vec<String>, Vec<HashSet<String>>), Box<dyn std::error::Error>> {
    println!("📂 Loading LARGE dataset (target: {} docs)...", target_size);
    
    let workspace_paths = vec![
        "../gov",
        "../governance/src",
        "../vectorizer/src",
        "../vectorizer/docs",
        "../task-queue/src",
        "../task-queue/docs",
        "../chat-hub",
        "../cursor-extension/src",
        "../py-env-security",
        "../ts-workspace/packages",
        "../dev-tools",
    ];
    
    let mut all_documents = Vec::new();
    let temp_store = VectorStore::new();
    
    for path in &workspace_paths {
        if !Path::new(path).exists() {
            continue;
        }
        
        println!("  Loading from {}...", path);
        
        let config = LoaderConfig {
            collection_name: "large_scale_bench".to_string(),
            max_chunk_size: 500, // Smaller chunks = more vectors
            chunk_overlap: 100,
            include_patterns: vec![
                "**/*.md".to_string(), "**/*.json".to_string(),
                "**/*.rs".to_string(), "**/*.ts".to_string(),
                "**/*.js".to_string(), "**/*.py".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
            ],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            allowed_extensions: vec![
                ".md".to_string(), ".json".to_string(),
                ".rs".to_string(), ".ts".to_string(),
                ".js".to_string(), ".py".to_string(),
            ],
            max_file_size: 2 * 1024 * 1024,
        };
        
        let mut loader = DocumentLoader::new(config);
        
        if let Ok(_) = loader.load_project(path, &temp_store) {
            let docs = loader.get_processed_documents();
            println!("    Loaded {} chunks", docs.len());
            all_documents.extend(docs);
        }
        
        if all_documents.len() >= target_size {
            break;
        }
    }
    
    // If still not enough, duplicate documents
    if all_documents.len() < target_size {
        println!("  📊 Duplicating documents to reach target size...");
        let original_len = all_documents.len();
        let needed = target_size - original_len;
        
        for i in 0..needed {
            all_documents.push(all_documents[i % original_len].clone());
        }
    }
    
    all_documents.truncate(target_size);
    
    println!("  ✅ Final dataset: {} documents\n", all_documents.len());
    
    // Create realistic queries
    let queries = vec![
        "HNSW graph construction algorithm implementation".to_string(),
        "quantization compression memory optimization".to_string(),
        "semantic search vector similarity cosine distance".to_string(),
        "BM25 embedding term frequency inverse document".to_string(),
        "governance proposal voting consensus protocol".to_string(),
        "async tokio runtime task scheduling".to_string(),
        "REST API endpoint authentication authorization".to_string(),
        "database schema persistence transaction".to_string(),
        "error handling result option unwrap".to_string(),
        "testing benchmark performance profiling".to_string(),
    ];
    
    // Generate ground truth with TF-IDF-like relevance
    let ground_truth = queries.iter().map(|query| {
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut relevant = HashSet::new();
        
        for (idx, doc) in all_documents.iter().enumerate() {
            let doc_lower = doc.to_lowercase();
            
            // Calculate term overlap
            let matches = query_terms.iter()
                .filter(|term| doc_lower.contains(&term.to_lowercase()))
                .count();
            
            // Require at least 30% term overlap
            if matches as f64 / query_terms.len() as f64 >= 0.3 {
                relevant.insert(format!("doc_{}", idx));
            }
        }
        
        // Ensure minimum relevant docs
        if relevant.len() < 10 {
            for i in 0..10.min(all_documents.len()) {
                relevant.insert(format!("doc_{}", i));
            }
        }
        
        relevant
    }).collect();
    
    Ok((all_documents, queries, ground_truth))
}

/// Benchmark at scale
fn benchmark_at_scale(
    documents: &[String],
    queries: &[String],
    ground_truth: &[HashSet<String>],
    dimension: usize,
    quantization: &str,
) -> Result<ScaleBenchmark, Box<dyn std::error::Error>> {
    println!("\n🔬 Benchmarking: {} docs, {}D, {}", documents.len(), dimension, quantization);
    
    // Generate embeddings
    println!("  Step 1: Generating embeddings...");
    let embed_start = Instant::now();
    
    let mut manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(dimension);
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25")?;
    
    if let Some(provider) = manager.get_provider_mut("bm25") {
        if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
            bm25.build_vocabulary(documents);
        }
    }
    
    let mut vectors = Vec::new();
    let mut vector_ids = Vec::new();
    
    for (idx, doc) in documents.iter().enumerate() {
        let embedding = manager.embed(doc)?;
        vectors.push(embedding);
        vector_ids.push(format!("doc_{}", idx));
        
        if (idx + 1) % 10000 == 0 {
            println!("    Generated {}/{} embeddings", idx + 1, documents.len());
        }
    }
    
    println!("    ✅ Embeddings: {:.2}s", embed_start.elapsed().as_secs_f64());
    
    // Apply quantization
    println!("  Step 2: Applying quantization...");
    let quant_start = Instant::now();
    
    let (final_vectors, memory_mb, compression_ratio) = match quantization {
        "Baseline" => {
            let mem = (vectors.len() * dimension * 4) as f64 / 1_048_576.0;
            (vectors.clone(), mem, 1.0)
        }
        
        "Binary" => {
            let mut binary = BinaryQuantizer::new();
            binary.train(&vectors);
            
            let encoded: Vec<_> = vectors.iter().map(|v| binary.encode(v)).collect();
            let decoded: Vec<_> = encoded.iter().map(|c| binary.decode(c, dimension)).collect();
            
            let mem = encoded.iter().map(|v| v.len()).sum::<usize>() as f64 / 1_048_576.0;
            let orig = (vectors.len() * dimension * 4) as f64 / 1_048_576.0;
            
            (decoded, mem, orig / mem)
        }
        
        "PQ" => {
            let mut pq = ProductQuantizer::new(dimension, 8, 256);
            
            // Train on sample for speed
            let sample_size = 5000.min(vectors.len());
            pq.train(&vectors[..sample_size]);
            
            let encoded: Vec<_> = vectors.iter().map(|v| pq.encode(v)).collect();
            let decoded: Vec<_> = encoded.iter().map(|c| pq.decode(c)).collect();
            
            let codes_size = encoded.len() * 8; // 8 subquantizers
            let codebook_size = 8 * 256 * (dimension / 8) * 4;
            let mem = (codes_size + codebook_size) as f64 / 1_048_576.0;
            let orig = (vectors.len() * dimension * 4) as f64 / 1_048_576.0;
            
            (decoded, mem, orig / mem)
        }
        
        "SQ" => {
            let mut sq = ScalarQuantizer::new(8);
            sq.train(&vectors);
            
            let encoded: Vec<_> = vectors.iter().map(|v| sq.encode(v)).collect();
            let decoded: Vec<_> = encoded.iter().map(|c| sq.decode(c)).collect();
            
            let mem = (encoded.len() * dimension) as f64 / 1_048_576.0;
            let orig = (vectors.len() * dimension * 4) as f64 / 1_048_576.0;
            
            (decoded, mem, orig / mem)
        }
        
        _ => return Err("Unknown quantization".into()),
    };
    
    println!("    ✅ Quantization: {:.2}s, {:.2}x compression", 
        quant_start.elapsed().as_secs_f64(), compression_ratio);
    
    // Build index
    println!("  Step 3: Building HNSW index...");
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
    
    let batch_vectors: Vec<_> = vector_ids.iter()
        .zip(final_vectors.iter())
        .map(|(id, vec)| (id.clone(), vec.clone()))
        .collect();
    
    index.batch_add(batch_vectors)?;
    index.optimize()?;
    
    let build_time_s = build_start.elapsed().as_secs_f64();
    println!("    ✅ Index built: {:.2}s ({:.0} vectors/sec)", 
        build_time_s, documents.len() as f64 / build_time_s);
    
    // Benchmark search
    println!("  Step 4: Benchmarking search...");
    let mut search_latencies = Vec::new();
    
    // Warmup
    for _ in 0..10 {
        let query_emb = manager.embed(&queries[0])?;
        let _ = index.search(&query_emb, 10)?;
    }
    
    // Actual benchmark (100 searches)
    for i in 0..100 {
        let query_idx = i % queries.len();
        let query_emb = manager.embed(&queries[query_idx])?;
        
        let start = Instant::now();
        let _ = index.search(&query_emb, 10)?;
        search_latencies.push(start.elapsed().as_micros() as f64 / 1000.0); // Convert to ms
    }
    
    let avg_latency_ms = search_latencies.iter().sum::<f64>() / search_latencies.len() as f64;
    let qps = 1000.0 / avg_latency_ms;
    
    println!("    ✅ Search: {:.2}ms avg, {:.0} QPS", avg_latency_ms, qps);
    
    // Evaluate quality
    println!("  Step 5: Evaluating quality...");
    let mut query_results = Vec::new();
    
    for (query_idx, query) in queries.iter().enumerate() {
        let query_emb = manager.embed(query)?;
        let results = index.search(&query_emb, 10)?;
        
        let query_result: Vec<QueryResult> = results.into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance,
            })
            .collect();
        
        query_results.push((query_result, ground_truth[query_idx].clone()));
    }
    
    let eval = evaluate_search_quality(query_results, 10);
    
    println!("    ✅ Quality: MAP={:.4}, Recall@10={:.4}", 
        eval.mean_average_precision,
        eval.recall_at_k.get(9).copied().unwrap_or(0.0));
    
    Ok(ScaleBenchmark {
        dataset_size: documents.len(),
        dimension,
        quantization: quantization.to_string(),
        memory_mb,
        compression_ratio,
        index_build_time_s: build_time_s,
        avg_search_latency_ms: avg_latency_ms,
        p95_search_latency_ms: percentile(&search_latencies, 95),
        p99_search_latency_ms: percentile(&search_latencies, 99),
        throughput_qps: qps,
        map: eval.mean_average_precision as f64,
        recall_at_10: eval.recall_at_k.get(9).copied().unwrap_or(0.0) as f64,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();
    
    println!("🚀 Large Scale Vectorizer Benchmark");
    println!("===================================\n");
    
    // Parse vector count from args
    let args: Vec<String> = std::env::args().collect();
    let vector_count = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(100_000)
    } else {
        100_000 // Default: 100K vectors
    };
    
    println!("🎯 Target: {} vectors", vector_count);
    println!("📊 This will test REAL performance at scale\n");
    
    // Load data
    let (documents, queries, ground_truth) = load_massive_dataset(vector_count)?;
    
    println!("\n🧪 Testing configurations:");
    println!("  Dimension: 384D (from previous benchmarks)");
    println!("  Quantizations: Baseline, SQ, PQ, Binary\n");
    
    let dimension = 384;
    let mut results = Vec::new();
    
    // Test each quantization method
    for quantization in ["Baseline", "SQ", "PQ", "Binary"] {
        match benchmark_at_scale(&documents, &queries, &ground_truth, dimension, quantization) {
            Ok(result) => {
                println!("  ✅ {} completed", quantization);
                results.push(result);
            }
            Err(e) => {
                println!("  ❌ {} failed: {}", quantization, e);
            }
        }
    }
    
    // Generate Markdown report
    println!("\n📊 Generating reports...");
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");
    
    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }
    
    // Generate MD report
    let mut md = String::new();
    md.push_str(&format!("# Large Scale Benchmark - {}K Vectors\n\n", vector_count / 1000));
    md.push_str(&format!("**Generated**: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    md.push_str("## Test Configuration\n\n");
    md.push_str(&format!("- **Dataset Size**: {} vectors\n", vector_count));
    md.push_str("- **Dimension**: 384D\n");
    md.push_str("- **Embedding Model**: BM25\n");
    md.push_str("- **HNSW Config**: M=16, ef_construction=200\n\n");
    
    md.push_str("## Results Summary\n\n");
    md.push_str("| Method | Memory | Compression | Build Time | Search (ms) | QPS | MAP | Recall@10 |\n");
    md.push_str("|--------|--------|-------------|------------|-------------|-----|-----|----------|\n");
    
    for result in &results {
        md.push_str(&format!(
            "| {} | {:.1} MB | {:.1}x | {:.1}s | {:.2} | {:.0} | {:.4} | {:.4} |\n",
            result.quantization,
            result.memory_mb,
            result.compression_ratio,
            result.index_build_time_s,
            result.avg_search_latency_ms,
            result.throughput_qps,
            result.map,
            result.recall_at_10,
        ));
    }
    
    if let Some(baseline) = results.iter().find(|r| r.quantization == "Baseline") {
        md.push_str("\n## Quality Analysis\n\n");
        
        for result in &results {
            if result.quantization == "Baseline" { continue; }
            
            let quality_retention = (result.map / baseline.map) * 100.0;
            let mem_saving = ((baseline.memory_mb - result.memory_mb) / baseline.memory_mb) * 100.0;
            
            md.push_str(&format!("### {}\n\n", result.quantization));
            md.push_str(&format!("- **Quality Retention**: {:.1}%\n", quality_retention));
            md.push_str(&format!("- **Memory Savings**: {:.1}%\n", mem_saving));
            md.push_str(&format!("- **Compression**: {:.1}x\n", result.compression_ratio));
            md.push_str(&format!("- **Search Speed**: {:.2}ms ({:.1}% vs baseline)\n\n", 
                result.avg_search_latency_ms,
                ((result.avg_search_latency_ms / baseline.avg_search_latency_ms) - 1.0) * 100.0
            ));
        }
    }
    
    md.push_str("## Scaling Projections\n\n");
    md.push_str("### For 1M Vectors\n\n");
    
    for result in &results {
        let mem_1m = (result.memory_mb / vector_count as f64) * 1_000_000.0;
        let build_1m = (result.index_build_time_s / vector_count as f64) * 1_000_000.0;
        
        md.push_str(&format!(
            "**{}**:\n- Memory: {:.2} GB\n- Build Time: {:.0}s (~{:.1} min)\n- Search: {:.2}ms ({:.0} QPS)\n\n",
            result.quantization,
            mem_1m / 1024.0,
            build_1m,
            build_1m / 60.0,
            result.avg_search_latency_ms,
            result.throughput_qps,
        ));
    }
    
    md.push_str("## Recommendations\n\n");
    
    if let Some(baseline) = results.iter().find(|r| r.quantization == "Baseline") {
        let best_balanced = results.iter()
            .filter(|r| r.quantization != "Baseline")
            .filter(|r| (r.map / baseline.map) >= 0.95)
            .max_by(|a, b| a.compression_ratio.partial_cmp(&b.compression_ratio).unwrap());
        
        if let Some(best) = best_balanced {
            md.push_str(&format!("**Best Balanced**: {} (≥95% quality retention)\n", best.quantization));
            md.push_str(&format!("- {:.1}x compression\n", best.compression_ratio));
            md.push_str(&format!("- {:.1}% memory savings\n", 
                ((baseline.memory_mb - best.memory_mb) / baseline.memory_mb) * 100.0));
            md.push_str(&format!("- {:.1}% quality retention\n\n", (best.map / baseline.map) * 100.0));
        }
    }
    
    md.push_str("---\n\n");
    md.push_str("*Report generated by Large Scale Vectorizer Benchmark*\n");
    
    let md_path = report_dir.join(format!("large_scale_{}k_{}.md", vector_count / 1000, timestamp));
    fs::write(&md_path, &md)?;
    
    println!("✅ Markdown report saved to: {}", md_path.display());
    
    // Save JSON
    let json_path = report_dir.join(format!("large_scale_{}k_{}.json", vector_count / 1000, timestamp));
    let json_data = serde_json::to_string_pretty(&results)?;
    fs::write(&json_path, json_data)?;
    
    println!("✅ JSON data saved to: {}", json_path.display());
    
    // Print summary
    println!("\n📈 LARGE SCALE RESULTS ({} vectors, {}D)", vector_count, dimension);
    println!("========================================");
    println!("{:<12} {:<12} {:<12} {:<12} {:<10} {:<10}", 
        "Method", "Memory", "Build", "Search", "MAP", "QPS");
    println!("{}", "-".repeat(70));
    
    for result in &results {
        println!("{:<12} {:<12} {:<12} {:<12} {:<10} {:<10}",
            result.quantization,
            format!("{:.1}MB", result.memory_mb),
            format!("{:.1}s", result.index_build_time_s),
            format!("{:.2}ms", result.avg_search_latency_ms),
            format!("{:.4}", result.map),
            format!("{:.0}", result.throughput_qps),
        );
    }
    
    // Find baseline for comparison
    if let Some(baseline) = results.iter().find(|r| r.quantization == "Baseline") {
        println!("\n💡 Quality Retention Analysis:");
        
        for result in &results {
            if result.quantization == "Baseline" {
                continue;
            }
            
            let quality_retention = (result.map / baseline.map) * 100.0;
            let quality_symbol = if quality_retention >= 95.0 {
                "✅"
            } else if quality_retention >= 85.0 {
                "⚠️"
            } else {
                "❌"
            };
            
            println!("  {} {}: {:.1}% quality, {:.1}x compression, {:.2}ms search",
                quality_symbol,
                result.quantization,
                quality_retention,
                result.compression_ratio,
                result.avg_search_latency_ms,
            );
        }
    }
    
    println!("\n💾 Memory Projections for 1M vectors:");
    for result in &results {
        let mem_1m = (result.memory_mb / documents.len() as f64) * 1_000_000.0;
        println!("  {}: {:.2} GB", result.quantization, mem_1m / 1024.0);
    }
    
    println!("\n⚡ Throughput Projections:");
    for result in &results {
        let queries_per_hour = result.throughput_qps * 3600.0;
        println!("  {}: {:.0} queries/hour ({:.0} QPS)", 
            result.quantization, queries_per_hour, result.throughput_qps);
    }
    
    println!("\n✅ Benchmark completed!");
    println!("📊 Data saved to: {}", json_path.display());
    
    Ok(())
}

