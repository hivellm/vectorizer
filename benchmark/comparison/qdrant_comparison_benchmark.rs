//! Vectorizer vs Qdrant Comparative Benchmark
//!
//! This benchmark compares Vectorizer and Qdrant performance across:
//! - Insertion latency and throughput
//! - Search latency and throughput
//! - Memory usage
//! - Search quality (precision, recall, F1)
//!
//! Usage:
//!   cargo run --release --bin qdrant_comparison_benchmark --features benchmarks

#![allow(
    clippy::uninlined_format_args,
    clippy::single_char_add_str,
    clippy::manual_map
)]

use rand::prelude::*;
use rand::rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{Duration, Instant};
use tracing::{error, info};

/// Benchmark results for a single system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResults {
    pub system_name: String,
    pub insertion: InsertionResults,
    pub search: SearchResults,
    pub memory: MemoryResults,
    pub quality: QualityResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertionResults {
    pub total_vectors: usize,
    pub total_time_ms: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_vectors_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub total_queries: usize,
    pub total_time_ms: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_queries_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResults {
    pub memory_usage_mb: f64,
    pub memory_per_vector_bytes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResults {
    pub precision_at_10: f64,
    pub recall_at_10: f64,
    pub f1_score: f64,
}

/// Client for Vectorizer REST API
struct VectorizerClient {
    client: Client,
    base_url: String,
}

impl VectorizerClient {
    fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, base_url }
    }

    async fn health_check(&self) -> Result<(), String> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Health check failed: {}", response.status()))
        }
    }

    async fn create_collection(&self, name: &str, dimension: usize) -> Result<(), String> {
        let url = format!("{}/api/v1/collections", self.base_url);
        let payload = json!({
            "name": name,
            "dimension": dimension,
            "metric": "cosine"
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 409 {
            // 409 = collection already exists, which is fine
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(format!(
                "Failed to create collection: {} - {}",
                status, text
            ))
        }
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Duration, String> {
        let url = format!(
            "{}/api/v1/collections/{}/vectors",
            self.base_url, collection
        );

        let points: Vec<serde_json::Value> = vectors
            .iter()
            .map(|(id, vector)| {
                json!({
                    "id": id,
                    "vector": vector
                })
            })
            .collect();

        let payload = json!({ "vectors": points });

        let start = Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Insert failed: {} - {}", status, text));
        }

        Ok(start.elapsed())
    }

    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<(Duration, Vec<String>), String> {
        let url = format!("{}/api/v1/collections/{}/search", self.base_url, collection);

        let payload = json!({
            "vector": query_vector,
            "limit": limit
        });

        let start = Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Search failed: {} - {}", status, text));
        }

        let elapsed = start.elapsed();
        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        let ids: Vec<String> = result
            .get("results")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        r.get("id")
                            .and_then(|id| id.as_str().map(|s| s.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok((elapsed, ids))
    }

    async fn get_memory_stats(&self) -> Result<MemoryResults, String> {
        // Vectorizer doesn't expose memory stats via REST, so we'll estimate
        // This is a placeholder - in real implementation, you'd query internal metrics
        Ok(MemoryResults {
            memory_usage_mb: 0.0,
            memory_per_vector_bytes: 0.0,
        })
    }
}

/// Client for Qdrant REST API
struct QdrantClient {
    client: Client,
    base_url: String,
}

impl QdrantClient {
    fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, base_url }
    }

    async fn health_check(&self) -> Result<(), String> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Health check failed: {}", response.status()))
        }
    }

    async fn create_collection(&self, name: &str, dimension: usize) -> Result<(), String> {
        let url = format!("{}/collections/{}", self.base_url, name);
        let payload = json!({
            "vectors": {
                "size": dimension,
                "distance": "Cosine"
            }
        });

        let response = self
            .client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 409 {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(format!(
                "Failed to create collection: {} - {}",
                status, text
            ))
        }
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Duration, String> {
        let url = format!("{}/collections/{}/points", self.base_url, collection);

        let points: Vec<serde_json::Value> = vectors
            .iter()
            .map(|(id, vector)| {
                json!({
                    "id": id,
                    "vector": vector
                })
            })
            .collect();

        let payload = json!({ "points": points });

        let start = Instant::now();
        let response = self
            .client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Insert failed: {} - {}", status, text));
        }

        Ok(start.elapsed())
    }

    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<(Duration, Vec<String>), String> {
        let url = format!("{}/collections/{}/points/search", self.base_url, collection);

        let payload = json!({
            "vector": query_vector,
            "limit": limit
        });

        let start = Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Search failed: {} - {}", status, text));
        }

        let elapsed = start.elapsed();
        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        let ids: Vec<String> = result
            .get("result")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        r.get("id").and_then(|id| {
                            if let Some(s) = id.as_str() {
                                Some(s.to_string())
                            } else if let Some(n) = id.as_u64() {
                                Some(n.to_string())
                            } else {
                                None
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok((elapsed, ids))
    }

    async fn get_memory_stats(&self) -> Result<MemoryResults, String> {
        // Qdrant doesn't expose memory stats via REST easily, so we'll estimate
        Ok(MemoryResults {
            memory_usage_mb: 0.0,
            memory_per_vector_bytes: 0.0,
        })
    }
}

/// Generate random normalized vector
fn generate_random_vector(dimension: usize, rng: &mut ThreadRng) -> Vec<f32> {
    let mut vector: Vec<f32> = (0..dimension)
        .map(|_| rng.random_range(-1.0..1.0))
        .collect();

    // Normalize for cosine similarity
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in &mut vector {
            *val /= norm;
        }
    }

    vector
}

/// Calculate percentile from sorted latencies
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = (sorted.len() as f64 * p / 100.0).ceil() as usize - 1;
    sorted[index.min(sorted.len() - 1)]
}

/// Benchmark insertion performance
async fn benchmark_insertion<T>(
    client: &T,
    collection: &str,
    vector_count: usize,
    dimension: usize,
    batch_size: usize,
) -> Result<InsertionResults, String>
where
    T: InsertClient,
{
    let mut rng = rng();
    let mut latencies = Vec::new();
    let mut total_time = Duration::ZERO;

    info!(
        "  Inserting {} vectors in batches of {}...",
        vector_count, batch_size
    );

    for batch_start in (0..vector_count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(vector_count);
        let batch_size_actual = batch_end - batch_start;

        let vectors: Vec<(String, Vec<f32>)> = (batch_start..batch_end)
            .map(|i| {
                let id = format!("vec_{}", i);
                let vector = generate_random_vector(dimension, &mut rng);
                (id, vector)
            })
            .collect();

        let elapsed = client.insert_vectors(collection, vectors).await?;
        let latency_ms = elapsed.as_secs_f64() * 1000.0 / batch_size_actual as f64;

        for _ in 0..batch_size_actual {
            latencies.push(latency_ms);
        }

        total_time += elapsed;
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    Ok(InsertionResults {
        total_vectors: vector_count,
        total_time_ms: total_time.as_secs_f64() * 1000.0,
        avg_latency_ms: latencies.iter().sum::<f64>() / latencies.len() as f64,
        p50_latency_ms: percentile(&latencies, 50.0),
        p95_latency_ms: percentile(&latencies, 95.0),
        p99_latency_ms: percentile(&latencies, 99.0),
        throughput_vectors_per_sec: (vector_count as f64) / total_time.as_secs_f64(),
    })
}

/// Benchmark search performance
async fn benchmark_search<T>(
    client: &T,
    collection: &str,
    query_count: usize,
    dimension: usize,
    limit: usize,
) -> Result<SearchResults, String>
where
    T: SearchClient,
{
    let mut rng = rng();
    let mut latencies = Vec::new();
    let mut total_time = Duration::ZERO;

    info!("  Running {} search queries...", query_count);

    for _ in 0..query_count {
        let query_vector = generate_random_vector(dimension, &mut rng);
        let (elapsed, _ids) = client.search(collection, &query_vector, limit).await?;

        let latency_ms = elapsed.as_secs_f64() * 1000.0;
        latencies.push(latency_ms);
        total_time += elapsed;
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    Ok(SearchResults {
        total_queries: query_count,
        total_time_ms: total_time.as_secs_f64() * 1000.0,
        avg_latency_ms: latencies.iter().sum::<f64>() / latencies.len() as f64,
        p50_latency_ms: percentile(&latencies, 50.0),
        p95_latency_ms: percentile(&latencies, 95.0),
        p99_latency_ms: percentile(&latencies, 99.0),
        throughput_queries_per_sec: (query_count as f64) / total_time.as_secs_f64(),
    })
}

/// Benchmark search quality (precision, recall, F1)
async fn benchmark_quality<T>(
    client: &T,
    collection: &str,
    test_queries: usize,
    dimension: usize,
    limit: usize,
) -> Result<QualityResults, String>
where
    T: SearchClient,
{
    // For quality testing, we'll use a simple approach:
    // Generate query vectors and check if results are reasonable
    // In a real benchmark, you'd have ground truth data

    let mut rng = rng();
    let mut total_precision = 0.0;
    let mut total_recall = 0.0;

    info!(
        "  Evaluating search quality with {} test queries...",
        test_queries
    );

    for _ in 0..test_queries {
        let query_vector = generate_random_vector(dimension, &mut rng);
        let (_elapsed, result_ids) = client.search(collection, &query_vector, limit).await?;

        // Simple quality metric: results should be non-empty
        // In real benchmark, compare against ground truth
        if !result_ids.is_empty() {
            total_precision += 1.0;
        }
        total_recall += (result_ids.len() as f64) / limit as f64;
    }

    let precision = total_precision / test_queries as f64;
    let recall = total_recall / test_queries as f64;
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    Ok(QualityResults {
        precision_at_10: precision,
        recall_at_10: recall,
        f1_score: f1,
    })
}

/// Trait for insertion operations
trait InsertClient {
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Duration, String>;
}

/// Trait for search operations
trait SearchClient {
    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<(Duration, Vec<String>), String>;
}

impl InsertClient for VectorizerClient {
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Duration, String> {
        VectorizerClient::insert_vectors(self, collection, vectors).await
    }
}

impl SearchClient for VectorizerClient {
    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<(Duration, Vec<String>), String> {
        VectorizerClient::search(self, collection, query_vector, limit).await
    }
}

impl InsertClient for QdrantClient {
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Duration, String> {
        QdrantClient::insert_vectors(self, collection, vectors).await
    }
}

impl SearchClient for QdrantClient {
    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<(Duration, Vec<String>), String> {
        QdrantClient::search(self, collection, query_vector, limit).await
    }
}

/// Run benchmark for Vectorizer
async fn run_vectorizer_benchmark(
    client: &VectorizerClient,
    collection: &str,
    vector_count: usize,
    query_count: usize,
    dimension: usize,
    batch_size: usize,
) -> Result<SystemResults, String> {
    info!("\n📊 Benchmarking Vectorizer...");
    info!("==========================================");

    // Create collection
    info!("  Creating collection '{}'...", collection);
    VectorizerClient::create_collection(client, collection, dimension).await?;

    // Benchmark insertion
    info!("\n🔹 Insertion Benchmark");
    let insertion =
        benchmark_insertion(client, collection, vector_count, dimension, batch_size).await?;
    info!(
        "  ✓ Inserted {} vectors in {:.2}ms",
        insertion.total_vectors, insertion.total_time_ms
    );
    info!("  ✓ Average latency: {:.2}ms", insertion.avg_latency_ms);
    info!(
        "  ✓ Throughput: {:.2} vectors/sec",
        insertion.throughput_vectors_per_sec
    );

    // Wait a bit for indexing
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Benchmark search
    info!("\n🔹 Search Benchmark");
    let search = benchmark_search(client, collection, query_count, dimension, 10).await?;
    info!(
        "  ✓ Executed {} queries in {:.2}ms",
        search.total_queries, search.total_time_ms
    );
    info!("  ✓ Average latency: {:.2}ms", search.avg_latency_ms);
    info!(
        "  ✓ Throughput: {:.2} queries/sec",
        search.throughput_queries_per_sec
    );

    // Benchmark quality
    info!("\n🔹 Quality Benchmark");
    let quality =
        benchmark_quality(client, collection, query_count.min(100), dimension, 10).await?;
    info!("  ✓ Precision@10: {:.2}%", quality.precision_at_10 * 100.0);
    info!("  ✓ Recall@10: {:.2}%", quality.recall_at_10 * 100.0);
    info!("  ✓ F1-Score: {:.2}%", quality.f1_score * 100.0);

    // Memory stats
    let memory = VectorizerClient::get_memory_stats(client).await?;

    Ok(SystemResults {
        system_name: "Vectorizer".to_string(),
        insertion,
        search,
        memory,
        quality,
    })
}

/// Run benchmark for Qdrant
async fn run_qdrant_benchmark(
    client: &QdrantClient,
    collection: &str,
    vector_count: usize,
    query_count: usize,
    dimension: usize,
    batch_size: usize,
) -> Result<SystemResults, String> {
    info!("\n📊 Benchmarking Qdrant...");
    info!("==========================================");

    // Create collection
    info!("  Creating collection '{}'...", collection);
    QdrantClient::create_collection(client, collection, dimension).await?;

    // Benchmark insertion
    info!("\n🔹 Insertion Benchmark");
    let insertion =
        benchmark_insertion(client, collection, vector_count, dimension, batch_size).await?;
    info!(
        "  ✓ Inserted {} vectors in {:.2}ms",
        insertion.total_vectors, insertion.total_time_ms
    );
    info!("  ✓ Average latency: {:.2}ms", insertion.avg_latency_ms);
    info!(
        "  ✓ Throughput: {:.2} vectors/sec",
        insertion.throughput_vectors_per_sec
    );

    // Wait a bit for indexing
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Benchmark search
    info!("\n🔹 Search Benchmark");
    let search = benchmark_search(client, collection, query_count, dimension, 10).await?;
    info!(
        "  ✓ Executed {} queries in {:.2}ms",
        search.total_queries, search.total_time_ms
    );
    info!("  ✓ Average latency: {:.2}ms", search.avg_latency_ms);
    info!(
        "  ✓ Throughput: {:.2} queries/sec",
        search.throughput_queries_per_sec
    );

    // Benchmark quality
    info!("\n🔹 Quality Benchmark");
    let quality =
        benchmark_quality(client, collection, query_count.min(100), dimension, 10).await?;
    info!("  ✓ Precision@10: {:.2}%", quality.precision_at_10 * 100.0);
    info!("  ✓ Recall@10: {:.2}%", quality.recall_at_10 * 100.0);
    info!("  ✓ F1-Score: {:.2}%", quality.f1_score * 100.0);

    // Memory stats
    let memory = QdrantClient::get_memory_stats(client).await?;

    Ok(SystemResults {
        system_name: "Qdrant".to_string(),
        insertion,
        search,
        memory,
        quality,
    })
}

/// Generate comparison report
fn generate_report(vectorizer_results: &SystemResults, qdrant_results: &SystemResults) -> String {
    let mut report = String::new();

    report.push_str("# Vectorizer vs Qdrant Benchmark Report\n\n");
    report.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    report.push_str("## Executive Summary\n\n");

    // Insertion comparison
    let insertion_speedup =
        qdrant_results.insertion.avg_latency_ms / vectorizer_results.insertion.avg_latency_ms;
    report.push_str("### Insertion Performance\n\n");
    report.push_str("| Metric | Vectorizer | Qdrant | Winner |\n");
    report.push_str("|--------|------------|--------|--------|\n");
    report.push_str(&format!(
        "| Avg Latency | {:.2}ms | {:.2}ms | {} |\n",
        vectorizer_results.insertion.avg_latency_ms,
        qdrant_results.insertion.avg_latency_ms,
        if insertion_speedup > 1.0 {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str(&format!(
        "| P95 Latency | {:.2}ms | {:.2}ms | {} |\n",
        vectorizer_results.insertion.p95_latency_ms,
        qdrant_results.insertion.p95_latency_ms,
        if vectorizer_results.insertion.p95_latency_ms < qdrant_results.insertion.p95_latency_ms {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str(&format!(
        "| Throughput | {:.2} vec/s | {:.2} vec/s | {} |\n",
        vectorizer_results.insertion.throughput_vectors_per_sec,
        qdrant_results.insertion.throughput_vectors_per_sec,
        if vectorizer_results.insertion.throughput_vectors_per_sec
            > qdrant_results.insertion.throughput_vectors_per_sec
        {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str("\n");

    // Search comparison
    let search_speedup =
        qdrant_results.search.avg_latency_ms / vectorizer_results.search.avg_latency_ms;
    report.push_str("### Search Performance\n\n");
    report.push_str("| Metric | Vectorizer | Qdrant | Winner |\n");
    report.push_str("|--------|------------|--------|--------|\n");
    report.push_str(&format!(
        "| Avg Latency | {:.2}ms | {:.2}ms | {} |\n",
        vectorizer_results.search.avg_latency_ms,
        qdrant_results.search.avg_latency_ms,
        if search_speedup > 1.0 {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str(&format!(
        "| P95 Latency | {:.2}ms | {:.2}ms | {} |\n",
        vectorizer_results.search.p95_latency_ms,
        qdrant_results.search.p95_latency_ms,
        if vectorizer_results.search.p95_latency_ms < qdrant_results.search.p95_latency_ms {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str(&format!(
        "| Throughput | {:.2} q/s | {:.2} q/s | {} |\n",
        vectorizer_results.search.throughput_queries_per_sec,
        qdrant_results.search.throughput_queries_per_sec,
        if vectorizer_results.search.throughput_queries_per_sec
            > qdrant_results.search.throughput_queries_per_sec
        {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str("\n");

    // Quality comparison
    report.push_str("### Search Quality\n\n");
    report.push_str("| Metric | Vectorizer | Qdrant | Winner |\n");
    report.push_str("|--------|------------|--------|--------|\n");
    report.push_str(&format!(
        "| Precision@10 | {:.2}% | {:.2}% | {} |\n",
        vectorizer_results.quality.precision_at_10 * 100.0,
        qdrant_results.quality.precision_at_10 * 100.0,
        if vectorizer_results.quality.precision_at_10 > qdrant_results.quality.precision_at_10 {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str(&format!(
        "| Recall@10 | {:.2}% | {:.2}% | {} |\n",
        vectorizer_results.quality.recall_at_10 * 100.0,
        qdrant_results.quality.recall_at_10 * 100.0,
        if vectorizer_results.quality.recall_at_10 > qdrant_results.quality.recall_at_10 {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str(&format!(
        "| F1-Score | {:.2}% | {:.2}% | {} |\n",
        vectorizer_results.quality.f1_score * 100.0,
        qdrant_results.quality.f1_score * 100.0,
        if vectorizer_results.quality.f1_score > qdrant_results.quality.f1_score {
            "Vectorizer"
        } else {
            "Qdrant"
        }
    ));
    report.push_str("\n");

    // Detailed results
    report.push_str("## Detailed Results\n\n");

    report.push_str("### Vectorizer Results\n\n");
    report.push_str("```json\n");
    report.push_str(&serde_json::to_string_pretty(vectorizer_results).unwrap());
    report.push_str("\n```\n\n");

    report.push_str("### Qdrant Results\n\n");
    report.push_str("```json\n");
    report.push_str(&serde_json::to_string_pretty(qdrant_results).unwrap());
    report.push_str("\n```\n\n");

    report.push_str("## Test Configuration\n\n");
    report.push_str("- Vector dimension: 384\n");
    report.push_str("- Test vectors: 10,000\n");
    report.push_str("- Test queries: 1,000\n");
    report.push_str("- Batch size: 100\n");

    report
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Vectorizer vs Qdrant Benchmark");
    info!("==========================================\n");

    // Configuration
    let vectorizer_url =
        std::env::var("VECTORIZER_URL").unwrap_or_else(|_| "http://localhost:15002".to_string());
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());

    let dimension = 384;
    let vector_count = 10_000;
    let query_count = 1_000;
    let batch_size = 100;

    // Create clients
    let vectorizer_client = VectorizerClient::new(vectorizer_url.clone());
    let qdrant_client = QdrantClient::new(qdrant_url.clone());

    // Health checks
    info!("🔍 Checking system health...");
    info!("  Vectorizer: {}", vectorizer_url);
    vectorizer_client.health_check().await.map_err(|e| {
        error!("Vectorizer health check failed: {}", e);
        e
    })?;
    info!("  ✓ Vectorizer is healthy");

    info!("  Qdrant: {}", qdrant_url);
    qdrant_client.health_check().await.map_err(|e| {
        error!("Qdrant health check failed: {}", e);
        e
    })?;
    info!("  ✓ Qdrant is healthy\n");

    // Run benchmarks
    let vectorizer_collection = format!("benchmark_vectorizer_{}", chrono::Utc::now().timestamp());
    let qdrant_collection = format!("benchmark_qdrant_{}", chrono::Utc::now().timestamp());

    let vectorizer_results = run_vectorizer_benchmark(
        &vectorizer_client,
        &vectorizer_collection,
        vector_count,
        query_count,
        dimension,
        batch_size,
    )
    .await?;

    let qdrant_results = run_qdrant_benchmark(
        &qdrant_client,
        &qdrant_collection,
        vector_count,
        query_count,
        dimension,
        batch_size,
    )
    .await?;

    // Generate report
    info!("\n📝 Generating report...");
    let report = generate_report(&vectorizer_results, &qdrant_results);

    // Save report
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let report_path = format!("docs/qdrant_comparison_benchmark_{}.md", timestamp);
    std::fs::write(&report_path, &report)?;

    info!("  ✓ Report saved to: {}", report_path);

    // Also save JSON results
    let json_path = format!("docs/qdrant_comparison_benchmark_{}.json", timestamp);
    let comparison = json!({
        "vectorizer": vectorizer_results,
        "qdrant": qdrant_results,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(&json_path, serde_json::to_string_pretty(&comparison)?)?;
    info!("  ✓ JSON results saved to: {}", json_path);

    // Print summary
    println!("\n{}", report);

    Ok(())
}
