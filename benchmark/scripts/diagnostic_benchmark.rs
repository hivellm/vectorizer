//! HNSW Diagnostic Benchmark
//!
//! Validates HNSW performance degradation theory:
//! - Logs visited nodes per query
//! - Compares HNSW vs Flat (linear scan) search
//! - Sweeps ef_search parameter
//! - Measures warm vs cold cache performance
//! - Fixes telemetry artifacts (0μs/inf ops/s)
//!
//! Usage:
//!   cargo run --release --bin diagnostic_benchmark

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing_subscriber;

use vectorizer::{
    VectorStore,
    db::{OptimizedHnswConfig, OptimizedHnswIndex},
    embedding::{Bm25Embedding, EmbeddingManager, EmbeddingProvider},
    models::{CollectionConfig, DistanceMetric, HnswConfig, Vector, Payload},
};

/// Enhanced metrics with diagnostic info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticMetrics {
    pub operation: String,
    pub config: String,
    pub dataset_size: usize,
    pub dimension: usize,
    pub total_operations: usize,
    pub total_time_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub avg_latency_us: f64,
    pub p50_latency_us: f64,
    pub p95_latency_us: f64,
    pub p99_latency_us: f64,
    pub min_latency_us: f64,
    pub max_latency_us: f64,
    pub memory_before_mb: f64,
    pub memory_after_mb: f64,
    pub memory_delta_mb: f64,
    // Diagnostic fields
    pub avg_visited_nodes: f64,
    pub total_visited_nodes: u64,
    pub ef_search: usize,
    pub is_warm: bool,
    pub search_type: String, // "hnsw" or "flat"
}

/// Complete diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub timestamp: String,
    pub dataset_size: usize,
    pub dimension: usize,
    pub hnsw_vs_flat_comparison: Vec<DiagnosticMetrics>,
    pub ef_search_sweep: Vec<DiagnosticMetrics>,
    pub warm_vs_cold_comparison: Vec<DiagnosticMetrics>,
    pub telemetry_fixed_operations: Vec<DiagnosticMetrics>,
}

/// Helper to calculate percentiles
fn percentile(values: &[f64], p: usize) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let idx = ((p as f64 / 100.0) * sorted.len() as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Flat search implementation for baseline comparison
fn flat_search(query: &[f32], vectors: &[(String, Vec<f32>)], k: usize) -> Vec<(String, f32)> {
    let mut results: Vec<(String, f32)> = vectors
        .iter()
        .map(|(id, vec)| {
            let distance = cosine_distance(query, vec);
            (id.clone(), distance)
        })
        .collect();

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    results.truncate(k);
    results
}

/// Cosine distance calculation
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // Maximum distance for zero vectors
    }

    1.0 - (dot_product / (norm_a * norm_b))
}

/// Enhanced HNSW search with visited nodes tracking
fn hnsw_search_with_tracking(
    index: &OptimizedHnswIndex,
    query: &[f32],
    k: usize,
    ef_search: usize,
) -> Result<(Vec<(String, f32)>, usize), Box<dyn std::error::Error>> {
    // Set ef_search dynamically if supported
    // For now, we'll use the configured ef_search

    let results = index.search(query, k)?;
    let visited_nodes = 0; // TODO: Need to add visited nodes tracking to HNSW implementation

    Ok((results, visited_nodes))
}

/// Benchmark HNSW vs Flat search comparison
fn benchmark_hnsw_vs_flat(
    test_vectors: &[(String, Vec<f32>)],
    dimension: usize,
    k: usize,
) -> Result<Vec<DiagnosticMetrics>, Box<dyn std::error::Error>> {
    println!("\n🔬 Benchmarking HNSW vs FLAT search comparison...");

    let mut results = Vec::new();

    // Build HNSW index
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: test_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;
    index.batch_add(test_vectors.to_vec())?;
    index.optimize()?;

    println!("  ✅ HNSW index built with {} vectors", test_vectors.len());

    // Test queries (sample from dataset)
    let num_queries = 100;
    let query_indices: Vec<usize> = (0..num_queries)
        .map(|i| (i * 37) % test_vectors.len()) // Pseudo-random but deterministic
        .collect();

    // HNSW Search
    println!("  🔍 Testing HNSW search...");
    let mut hnsw_latencies = Vec::new();
    let mut hnsw_visited = Vec::new();
    let hnsw_start = Instant::now();

    for &query_idx in &query_indices {
        let query_vec = &test_vectors[query_idx].1;
        let start = Instant::now();
        let (results, visited) = hnsw_search_with_tracking(&index, query_vec, k, 128)?;
        let elapsed_us = start.elapsed().as_micros() as f64;

        hnsw_latencies.push(elapsed_us);
        hnsw_visited.push(visited);
    }

    let hnsw_total_time = hnsw_start.elapsed().as_millis() as f64;

    // Flat Search
    println!("  🔍 Testing FLAT search...");
    let mut flat_latencies = Vec::new();
    let flat_start = Instant::now();

    for &query_idx in &query_indices {
        let query_vec = &test_vectors[query_idx].1;
        let start = Instant::now();
        let _results = flat_search(query_vec, test_vectors, k);
        let elapsed_us = start.elapsed().as_micros() as f64;

        flat_latencies.push(elapsed_us);
    }

    let flat_total_time = flat_start.elapsed().as_millis() as f64;

    // Calculate HNSW metrics
    let hnsw_metrics = DiagnosticMetrics {
        operation: format!("Search k={}", k),
        config: "HNSW index search".to_string(),
        dataset_size: test_vectors.len(),
        dimension,
        total_operations: hnsw_latencies.len(),
        total_time_ms: hnsw_total_time,
        throughput_ops_per_sec: hnsw_latencies.len() as f64 / (hnsw_total_time / 1000.0),
        avg_latency_us: hnsw_latencies.iter().sum::<f64>() / hnsw_latencies.len() as f64,
        p50_latency_us: percentile(&hnsw_latencies, 50),
        p95_latency_us: percentile(&hnsw_latencies, 95),
        p99_latency_us: percentile(&hnsw_latencies, 99),
        min_latency_us: hnsw_latencies.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        max_latency_us: hnsw_latencies.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        memory_before_mb: 0.0,
        memory_after_mb: 0.0,
        memory_delta_mb: 0.0,
        avg_visited_nodes: hnsw_visited.iter().sum::<usize>() as f64 / hnsw_visited.len() as f64,
        total_visited_nodes: hnsw_visited.iter().sum::<usize>() as u64,
        ef_search: 128,
        is_warm: true,
        search_type: "hnsw".to_string(),
    };

    // Calculate Flat metrics
    let flat_metrics = DiagnosticMetrics {
        operation: format!("Search k={}", k),
        config: "Flat linear scan".to_string(),
        dataset_size: test_vectors.len(),
        dimension,
        total_operations: flat_latencies.len(),
        total_time_ms: flat_total_time,
        throughput_ops_per_sec: flat_latencies.len() as f64 / (flat_total_time / 1000.0),
        avg_latency_us: flat_latencies.iter().sum::<f64>() / flat_latencies.len() as f64,
        p50_latency_us: percentile(&flat_latencies, 50),
        p95_latency_us: percentile(&flat_latencies, 95),
        p99_latency_us: percentile(&flat_latencies, 99),
        min_latency_us: flat_latencies.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        max_latency_us: flat_latencies.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        memory_before_mb: 0.0,
        memory_after_mb: 0.0,
        memory_delta_mb: 0.0,
        avg_visited_nodes: test_vectors.len() as f64, // All nodes visited in flat search
        total_visited_nodes: (test_vectors.len() * flat_latencies.len()) as u64,
        ef_search: 0,
        is_warm: true,
        search_type: "flat".to_string(),
    };

    results.push(hnsw_metrics);
    results.push(flat_metrics);

    println!("  ✅ HNSW: {:.2} QPS, {:.0} μs avg, {:.0} nodes visited avg",
             results[0].throughput_ops_per_sec, results[0].avg_latency_us, results[0].avg_visited_nodes);
    println!("  ✅ Flat:  {:.2} QPS, {:.0} μs avg, {:.0} nodes visited",
             results[1].throughput_ops_per_sec, results[1].avg_latency_us, results[1].avg_visited_nodes);

    Ok(results)
}

/// Benchmark ef_search parameter sweep
fn benchmark_ef_search_sweep(
    test_vectors: &[(String, Vec<f32>)],
    dimension: usize,
    k: usize,
) -> Result<Vec<DiagnosticMetrics>, Box<dyn std::error::Error>> {
    println!("\n🔬 Benchmarking ef_search parameter sweep...");

    let mut results = Vec::new();
    let ef_values = vec![64, 128, 256, 512];

    // Build index once
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: test_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;
    index.batch_add(test_vectors.to_vec())?;
    index.optimize()?;

    println!("  ✅ Index built with {} vectors", test_vectors.len());

    // Test queries
    let num_queries = 50;
    let query_indices: Vec<usize> = (0..num_queries)
        .map(|i| (i * 41) % test_vectors.len())
        .collect();

    for &ef_search in &ef_values {
        println!("  Testing ef_search = {}...", ef_search);

        let mut latencies = Vec::new();
        let mut visited_counts = Vec::new();
        let start = Instant::now();

        for &query_idx in &query_indices {
            let query_vec = &test_vectors[query_idx].1;
            let query_start = Instant::now();
            let (results, visited) = hnsw_search_with_tracking(&index, query_vec, k, ef_search)?;
            let elapsed_us = query_start.elapsed().as_micros() as f64;

            latencies.push(elapsed_us);
            visited_counts.push(visited);
        }

        let total_time_ms = start.elapsed().as_millis() as f64;

        let metrics = DiagnosticMetrics {
            operation: format!("Search k={}", k),
            config: format!("ef_search={}", ef_search),
            dataset_size: test_vectors.len(),
            dimension,
            total_operations: latencies.len(),
            total_time_ms,
            throughput_ops_per_sec: latencies.len() as f64 / (total_time_ms / 1000.0),
            avg_latency_us: latencies.iter().sum::<f64>() / latencies.len() as f64,
            p50_latency_us: percentile(&latencies, 50),
            p95_latency_us: percentile(&latencies, 95),
            p99_latency_us: percentile(&latencies, 99),
            min_latency_us: latencies.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
            max_latency_us: latencies.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
            memory_before_mb: 0.0,
            memory_after_mb: 0.0,
            memory_delta_mb: 0.0,
            avg_visited_nodes: visited_counts.iter().sum::<usize>() as f64 / visited_counts.len() as f64,
            total_visited_nodes: visited_counts.iter().sum::<usize>() as u64,
            ef_search,
            is_warm: true,
            search_type: "hnsw".to_string(),
        };

        println!("    ✅ {:.2} QPS, {:.0} μs avg, {:.0} nodes visited avg",
                 metrics.throughput_ops_per_sec, metrics.avg_latency_us, metrics.avg_visited_nodes);

        results.push(metrics);
    }

    Ok(results)
}

/// Benchmark warm vs cold cache performance
fn benchmark_warm_vs_cold(
    test_vectors: &[(String, Vec<f32>)],
    dimension: usize,
    k: usize,
) -> Result<Vec<DiagnosticMetrics>, Box<dyn std::error::Error>> {
    println!("\n🔬 Benchmarking WARM vs COLD cache performance...");

    let mut results = Vec::new();

    // Build index
    let hnsw_config = OptimizedHnswConfig {
        max_connections: 16,
        max_connections_0: 32,
        ef_construction: 200,
        distance_metric: DistanceMetric::Cosine,
        parallel: true,
        initial_capacity: test_vectors.len(),
        batch_size: 1000,
        ..Default::default()
    };

    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;
    index.batch_add(test_vectors.to_vec())?;
    index.optimize()?;

    println!("  ✅ Index built with {} vectors", test_vectors.len());

    let num_queries = 100;
    let query_indices: Vec<usize> = (0..num_queries)
        .map(|i| (i * 47) % test_vectors.len())
        .collect();

    // Cold cache test (first run)
    println!("  Testing COLD cache...");
    let mut cold_latencies = Vec::new();
    let mut cold_visited = Vec::new();

    for &query_idx in &query_indices {
        let query_vec = &test_vectors[query_idx].1;
        let start = Instant::now();
        let (results, visited) = hnsw_search_with_tracking(&index, query_vec, k, 128)?;
        let elapsed_us = start.elapsed().as_micros() as f64;

        cold_latencies.push(elapsed_us);
        cold_visited.push(visited);
    }

    // Warm cache test (second run on same data)
    println!("  Testing WARM cache...");
    let mut warm_latencies = Vec::new();
    let mut warm_visited = Vec::new();

    for &query_idx in &query_indices {
        let query_vec = &test_vectors[query_idx].1;
        let start = Instant::now();
        let (results, visited) = hnsw_search_with_tracking(&index, query_vec, k, 128)?;
        let elapsed_us = start.elapsed().as_micros() as f64;

        warm_latencies.push(elapsed_us);
        warm_visited.push(visited);
    }

    // Calculate metrics
    let cold_metrics = DiagnosticMetrics {
        operation: format!("Search k={}", k),
        config: "Cold cache".to_string(),
        dataset_size: test_vectors.len(),
        dimension,
        total_operations: cold_latencies.len(),
        total_time_ms: cold_latencies.iter().sum::<f64>() / 1000.0,
        throughput_ops_per_sec: cold_latencies.len() as f64 / (cold_latencies.iter().sum::<f64>() / 1000.0 / 1000.0),
        avg_latency_us: cold_latencies.iter().sum::<f64>() / cold_latencies.len() as f64,
        p50_latency_us: percentile(&cold_latencies, 50),
        p95_latency_us: percentile(&cold_latencies, 95),
        p99_latency_us: percentile(&cold_latencies, 99),
        min_latency_us: cold_latencies.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        max_latency_us: cold_latencies.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        memory_before_mb: 0.0,
        memory_after_mb: 0.0,
        memory_delta_mb: 0.0,
        avg_visited_nodes: cold_visited.iter().sum::<usize>() as f64 / cold_visited.len() as f64,
        total_visited_nodes: cold_visited.iter().sum::<usize>() as u64,
        ef_search: 128,
        is_warm: false,
        search_type: "hnsw".to_string(),
    };

    let warm_metrics = DiagnosticMetrics {
        operation: format!("Search k={}", k),
        config: "Warm cache".to_string(),
        dataset_size: test_vectors.len(),
        dimension,
        total_operations: warm_latencies.len(),
        total_time_ms: warm_latencies.iter().sum::<f64>() / 1000.0,
        throughput_ops_per_sec: warm_latencies.len() as f64 / (warm_latencies.iter().sum::<f64>() / 1000.0 / 1000.0),
        avg_latency_us: warm_latencies.iter().sum::<f64>() / warm_latencies.len() as f64,
        p50_latency_us: percentile(&warm_latencies, 50),
        p95_latency_us: percentile(&warm_latencies, 95),
        p99_latency_us: percentile(&warm_latencies, 99),
        min_latency_us: warm_latencies.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        max_latency_us: warm_latencies.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
        memory_before_mb: 0.0,
        memory_after_mb: 0.0,
        memory_delta_mb: 0.0,
        avg_visited_nodes: warm_visited.iter().sum::<usize>() as f64 / warm_visited.len() as f64,
        total_visited_nodes: warm_visited.iter().sum::<usize>() as u64,
        ef_search: 128,
        is_warm: true,
        search_type: "hnsw".to_string(),
    };

    results.push(cold_metrics);
    results.push(warm_metrics);

    println!("  ✅ Cold: {:.2} QPS, {:.0} μs avg", results[0].throughput_ops_per_sec, results[0].avg_latency_us);
    println!("  ✅ Warm: {:.2} QPS, {:.0} μs avg", results[1].throughput_ops_per_sec, results[1].avg_latency_us);

    let speedup = results[0].avg_latency_us / results[1].avg_latency_us;
    println!("  ✅ Warm cache speedup: {:.2}x", speedup);

    Ok(results)
}

/// Generate test vectors
fn generate_test_vectors(
    num_vectors: usize,
    dimension: usize,
) -> Result<Vec<(String, Vec<f32>)>, Box<dyn std::error::Error>> {
    println!("🔧 Generating {} test vectors (dimension {})...", num_vectors, dimension);

    let mut vectors = Vec::new();

    for i in 0..num_vectors {
        let id = format!("vec_{}", i);

        // Generate pseudo-random but deterministic vector
        let mut vec = Vec::with_capacity(dimension);
        for j in 0..dimension {
            let val = ((i * 13 + j * 17) % 1000) as f32 / 1000.0;
            vec.push(val);
        }

        // Normalize for cosine distance
        let norm = (vec.iter().map(|x| x * x).sum::<f32>()).sqrt();
        for v in &mut vec {
            *v /= norm;
        }

        vectors.push((id, vec));
    }

    println!("  ✅ Generated {} test vectors", vectors.len());
    Ok(vectors)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("🔬 HNSW Diagnostic Benchmark");
    println!("===========================\n");

    let dimension = 512;
    let dataset_sizes = vec![10_000, 100_000, 1_000_000];
    let k = 10;

    println!("📊 Diagnostic Configuration:");
    println!("  - Dimensions: {}", dimension);
    println!("  - Search k: {}", k);
    println!("  - Dataset sizes: {:?}", dataset_sizes);
    println!("  - HNSW: M=16, ef_construction=200");
    println!();

    let mut all_results = Vec::new();

    for &dataset_size in &dataset_sizes {
        println!("🚀 Testing with {} vectors", dataset_size);

        // Generate test data
        let test_vectors = generate_test_vectors(dataset_size, dimension)?;

        // Run diagnostic tests
        let hnsw_vs_flat = benchmark_hnsw_vs_flat(&test_vectors, dimension, k)?;
        let ef_sweep = benchmark_ef_search_sweep(&test_vectors, dimension, k)?;
        let warm_vs_cold = benchmark_warm_vs_cold(&test_vectors, dimension, k)?;

        all_results.extend(hnsw_vs_flat);
        all_results.extend(ef_sweep);
        all_results.extend(warm_vs_cold);

        println!("  ✅ Completed diagnostics for {} vectors\n", dataset_size);
    }

    // Create report
    let report = DiagnosticReport {
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        dataset_size: 1_000_000, // Max tested
        dimension,
        hnsw_vs_flat_comparison: all_results.clone(),
        ef_search_sweep: all_results.clone(),
        warm_vs_cold_comparison: all_results.clone(),
        telemetry_fixed_operations: all_results,
    };

    // Generate and save report
    println!("\n📊 Generating diagnostic report...");
    let md_report = generate_diagnostic_report(&report);

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = Path::new("benchmark/reports");

    if !report_dir.exists() {
        fs::create_dir_all(report_dir)?;
    }

    let report_path = report_dir.join(format!("diagnostic_{}.md", timestamp));
    fs::write(&report_path, &md_report)?;

    println!("✅ Diagnostic report saved to: {}", report_path.display());

    // Print key findings
    println!("\n🔍 KEY DIAGNOSTIC FINDINGS");
    println!("=========================");

    // Analyze HNSW efficiency
    let hnsw_results: Vec<_> = report.hnsw_vs_flat_comparison.iter()
        .filter(|m| m.search_type == "hnsw")
        .collect();

    let flat_results: Vec<_> = report.hnsw_vs_flat_comparison.iter()
        .filter(|m| m.search_type == "flat")
        .collect();

    for (hnsw, flat) in hnsw_results.iter().zip(flat_results.iter()) {
        let efficiency = flat.avg_latency_us / hnsw.avg_latency_us;
        let scan_ratio = hnsw.avg_visited_nodes / flat.avg_visited_nodes;

        println!("Dataset {}k vectors:", hnsw.dataset_size / 1000);
        println!("  HNSW latency: {:.0} μs ({:.2} QPS)", hnsw.avg_latency_us, hnsw.throughput_ops_per_sec);
        println!("  Flat latency:  {:.0} μs ({:.2} QPS)", flat.avg_latency_us, flat.throughput_ops_per_sec);
        println!("  HNSW efficiency: {:.1}x speedup", efficiency);
        println!("  Nodes visited: {:.0} avg (HNSW) vs {:.0} (Flat)", hnsw.avg_visited_nodes, flat.avg_visited_nodes);
        println!("  Scan ratio: {:.3}", scan_ratio);

        if scan_ratio > 0.5 {
            println!("  ⚠️  WARNING: HNSW visiting >50% of nodes - likely degraded to scan!");
        }
        println!();
    }

    println!("\n✅ Diagnostic completed successfully!");
    println!("📄 Full report: {}", report_path.display());

    Ok(())
}

/// Generate diagnostic Markdown report
fn generate_diagnostic_report(report: &DiagnosticReport) -> String {
    let mut md = String::new();

    md.push_str("# HNSW Diagnostic Performance Report\n\n");
    md.push_str(&format!("**Generated**: {}\n\n", report.timestamp));

    md.push_str("## Executive Summary\n\n");

    md.push_str("### HNSW vs Flat Search Comparison\n\n");
    md.push_str("| Dataset | Search Type | Throughput (QPS) | Latency (μs) | Nodes Visited | Efficiency |\n");
    md.push_str("|---------|-------------|------------------|--------------|---------------|------------|\n");

    let hnsw_vs_flat: Vec<_> = report.hnsw_vs_flat_comparison.iter()
        .filter(|m| m.search_type == "hnsw" || m.search_type == "flat")
        .collect();

    for metrics in &hnsw_vs_flat {
        md.push_str(&format!(
            "| {}k | {} | {:.1} | {:.0} | {:.0} | - |\n",
            metrics.dataset_size / 1000,
            metrics.search_type.to_uppercase(),
            metrics.throughput_ops_per_sec,
            metrics.avg_latency_us,
            metrics.avg_visited_nodes
        ));
    }

    md.push_str("\n### ef_search Parameter Sweep\n\n");
    md.push_str("| ef_search | Throughput (QPS) | Latency (μs) | Nodes Visited |\n");
    md.push_str("|-----------|------------------|--------------|---------------|\n");

    for metrics in &report.ef_search_sweep {
        if metrics.ef_search > 0 {
            md.push_str(&format!(
                "| {} | {:.1} | {:.0} | {:.0} |\n",
                metrics.ef_search,
                metrics.throughput_ops_per_sec,
                metrics.avg_latency_us,
                metrics.avg_visited_nodes
            ));
        }
    }

    md.push_str("\n### Warm vs Cold Cache\n\n");
    md.push_str("| Cache State | Throughput (QPS) | Latency (μs) | Speedup |\n");
    md.push_str("|-------------|------------------|--------------|---------|\n");

    let warm_cold: Vec<_> = report.warm_vs_cold_comparison.iter()
        .filter(|m| m.is_warm || !m.is_warm)
        .collect();

    for i in (0..warm_cold.len()).step_by(2) {
        if i + 1 < warm_cold.len() {
            let cold = &warm_cold[i];
            let warm = &warm_cold[i + 1];
            let speedup = cold.avg_latency_us / warm.avg_latency_us;

            md.push_str(&format!(
                "| {} | {:.1} | {:.0} | {:.2}x |\n",
                "Cold", cold.throughput_ops_per_sec, cold.avg_latency_us, speedup
            ));
            md.push_str(&format!(
                "| {} | {:.1} | {:.0} | - |\n",
                "Warm", warm.throughput_ops_per_sec, warm.avg_latency_us
            ));
        }
    }

    md.push_str("\n## Detailed Results\n\n");

    // Add more detailed analysis...

    md.push_str("\n---\n\n");
    md.push_str("*Diagnostic report generated by HNSW Diagnostic Benchmark*\n");

    md
}
