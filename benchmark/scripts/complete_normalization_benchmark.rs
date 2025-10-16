//! Complete Normalization & Quantization Benchmark
//!
//! Tests all combinations using REAL vectorizer components:
//! - BM25 embeddings (real)
//! - HNSW index (real)
//! - With/Without Normalization (3 levels)
//! - With/Without Quantization (SQ-8)
//!
//! Run with: cargo run --bin complete_normalization_benchmark --features benchmarks --release

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use vectorizer::db::{Collection, CollectionNormalizationHelper};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector,
};
use vectorizer::normalization::{
    ContentTypeDetector, NormalizationConfig, NormalizationLevel, NormalizationPolicy,
    TextNormalizer,
};

/// Configuration for a benchmark scenario
#[derive(Debug, Clone)]
struct ScenarioConfig {
    name: String,
    normalization: Option<NormalizationLevel>,
    quantization: bool,
}

/// Document for testing
#[derive(Debug, Clone)]
struct TestDocument {
    id: String,
    path: PathBuf,
    raw_content: String,
    file_type: String,
}

/// Test query
#[derive(Debug, Clone)]
struct TestQuery {
    raw_query: String,
    expected_docs: Vec<String>,
    description: String,
}

/// Complete scenario result
#[derive(Debug)]
struct ScenarioResult {
    config: ScenarioConfig,
    original_size_bytes: usize,
    processed_size_bytes: usize,
    vector_size_bytes: usize,
    total_storage_bytes: usize,
    preprocessing_time: Duration,
    indexing_time: Duration,
    avg_search_time: Duration,
    avg_precision: f64,
    avg_recall: f64,
    avg_f1: f64,
    duplicates_found: usize,
}

impl ScenarioResult {
    fn storage_vs_baseline(&self, baseline_storage: usize) -> f64 {
        if baseline_storage == 0 {
            return 0.0;
        }
        ((baseline_storage as i64 - self.total_storage_bytes as i64) as f64
            / baseline_storage as f64)
            * 100.0
    }
}

/// Collect real documents from workspace
fn collect_workspace_documents(max_docs: usize) -> Vec<TestDocument> {
    let mut docs = Vec::new();
    let workspace_path = PathBuf::from("..");
    let extensions = vec!["rs", "md", "toml", "json", "js", "ts"];

    fn visit_dir(dir: &Path, docs: &mut Vec<TestDocument>, exts: &[&str], max: usize) {
        if docs.len() >= max {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .to_str()
                    .map(|s| {
                        s.contains("target") || s.contains("node_modules") || s.contains(".git")
                    })
                    .unwrap_or(false)
                {
                    continue;
                }

                if path.is_dir() {
                    visit_dir(&path, docs, exts, max);
                } else if let Some(ext) = path.extension() {
                    if exts.contains(&ext.to_str().unwrap_or("")) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if content.len() > 200 && content.len() < 50_000 {
                                docs.push(TestDocument {
                                    id: format!("doc_{}", docs.len()),
                                    path: path.clone(),
                                    raw_content: content,
                                    file_type: ext.to_str().unwrap_or("unknown").to_string(),
                                });
                            }
                        }
                    }
                }

                if docs.len() >= max {
                    return;
                }
            }
        }
    }

    visit_dir(&workspace_path, &mut docs, &extensions, max_docs);
    docs
}

/// Generate test queries from documents
fn generate_test_queries(docs: &[TestDocument]) -> Vec<TestQuery> {
    let mut queries = Vec::new();

    // Query 1: From code file
    if let Some(doc) = docs
        .iter()
        .find(|d| d.file_type == "rs" && d.raw_content.len() > 1000)
    {
        if let Some(line) = doc
            .raw_content
            .lines()
            .filter(|l| l.len() > 30 && l.len() < 100)
            .nth(10)
        {
            let words: Vec<&str> = line
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .take(4)
                .collect();

            if words.len() >= 3 {
                queries.push(TestQuery {
                    raw_query: words.join(" "),
                    expected_docs: vec![doc.id.clone()],
                    description: "Exact code phrase".to_string(),
                });
            }
        }
    }

    // Query 2: Technical terms
    queries.push(TestQuery {
        raw_query: "collection vector database search".to_string(),
        expected_docs: docs
            .iter()
            .filter(|d| {
                let c = d.raw_content.to_lowercase();
                c.contains("collection") && c.contains("vector")
            })
            .map(|d| d.id.clone())
            .collect(),
        description: "Technical multi-term".to_string(),
    });

    // Query 3: With whitespace
    queries.push(TestQuery {
        raw_query: "  function   async   implementation  ".to_string(),
        expected_docs: docs
            .iter()
            .filter(|d| d.raw_content.contains("function") || d.raw_content.contains("async"))
            .map(|d| d.id.clone())
            .collect(),
        description: "Query with whitespace".to_string(),
    });

    queries
}

/// Run benchmark using REAL vectorizer components
async fn run_scenario(
    config: ScenarioConfig,
    raw_docs: &[TestDocument],
    embedding_manager: &EmbeddingManager,
) -> ScenarioResult {
    // Create collection
    let mut coll_config = CollectionConfig {
        dimension: 512,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: if config.quantization {
            QuantizationConfig::SQ { bits: 8 }
        } else {
            QuantizationConfig::None
        },
        compression: Default::default(),
        normalization: config
            .normalization
            .as_ref()
            .map(|level| NormalizationConfig {
                enabled: true,
                policy: NormalizationPolicy {
                    version: 1,
                    level: *level,
                    preserve_case: !matches!(level, NormalizationLevel::Aggressive),
                    collapse_whitespace: !matches!(level, NormalizationLevel::Conservative),
                    remove_html: matches!(level, NormalizationLevel::Aggressive),
                },
                cache_enabled: false,
                hot_cache_size: 0,
                normalize_queries: true,
                store_raw_text: true,
            }),
    };

    let collection = Collection::new(
        format!("bench_{}", config.name.replace(" ", "_")),
        coll_config.clone(),
    );

    // Normalization helper
    let temp_dir = std::env::temp_dir().join("vectorizer_benchmark");
    let norm_helper = CollectionNormalizationHelper::from_config(&coll_config, &temp_dir).ok();

    // Step 1: Process and index documents
    let prep_start = Instant::now();
    let mut original_size = 0;
    let mut processed_size = 0;

    for doc in &raw_docs[..raw_docs.len().min(50)] {
        original_size += doc.raw_content.len();

        // Process text (normalize if enabled)
        let text_to_embed: String = if norm_helper.is_some() {
            let helper = norm_helper.as_ref().unwrap();
            match helper
                .process_document(&doc.raw_content, Some(&doc.path))
                .await
            {
                Ok(p) => {
                    processed_size += p.normalized_text.len();
                    p.normalized_text
                }
                Err(_) => {
                    processed_size += doc.raw_content.len();
                    doc.raw_content.clone()
                }
            }
        } else {
            processed_size += doc.raw_content.len();
            doc.raw_content.clone()
        };

        // Generate real BM25 embedding
        match embedding_manager.embed(&text_to_embed) {
            Ok(embedding) => {
                let vector = Vector {
                    id: doc.id.clone(),
                    data: embedding,
                    payload: Some(Payload {
                        data: serde_json::json!({
                            "file_path": doc.path.to_string_lossy(),
                            "file_type": doc.file_type,
                            "text": text_to_embed[..text_to_embed.len().min(200)].to_string(),
                        }),
                    }),
                };

                if let Err(e) = collection.insert(vector) {
                    eprintln!("Failed to insert vector {}: {}", doc.id, e);
                }
            }
            Err(e) => {
                eprintln!("Failed to generate embedding for {}: {}", doc.id, e);
            }
        }
    }

    let indexed_count = collection.vector_count();
    if indexed_count == 0 {
        eprintln!(
            "⚠️  Warning: No vectors indexed for scenario '{}'",
            config.name
        );
    }

    let prep_time = prep_start.elapsed();
    let index_time = Duration::from_millis(0); // Already included in prep

    // Calculate storage
    let vector_count = collection.vector_count();
    let dimension = coll_config.dimension;
    let bytes_per_dim = if config.quantization { 1 } else { 4 };
    let vector_size = vector_count * dimension * bytes_per_dim;

    // Step 2: Test queries
    let queries = generate_test_queries(raw_docs);
    let mut search_times = Vec::new();
    let mut precisions = Vec::new();
    let mut recalls = Vec::new();

    if vector_count > 0 && !queries.is_empty() {
        for query in &queries {
            let search_start = Instant::now();

            // Process query (normalize if enabled)
            let query_text: String = if norm_helper.is_some() {
                let helper = norm_helper.as_ref().unwrap();
                helper.process_query(&query.raw_query)
            } else {
                query.raw_query.clone()
            };

            // Generate query embedding
            if let Ok(query_embedding) = embedding_manager.embed(&query_text) {
                // Search using real HNSW index
                if let Ok(results) = collection.search(&query_embedding, 10) {
                    let search_time = search_start.elapsed();
                    search_times.push(search_time);

                    // Calculate metrics
                    let found: Vec<String> = results.iter().map(|r| r.id.clone()).collect();
                    let found_set: HashSet<_> = found.iter().collect();
                    let expected_set: HashSet<_> = query.expected_docs.iter().collect();

                    let true_positives = found_set.intersection(&expected_set).count();

                    let precision = if !found.is_empty() {
                        true_positives as f64 / found.len() as f64
                    } else {
                        0.0
                    };

                    let recall = if !query.expected_docs.is_empty() {
                        true_positives as f64 / query.expected_docs.len() as f64
                    } else {
                        0.0
                    };

                    precisions.push(precision);
                    recalls.push(recall);
                }
            }
        }
    }

    let avg_search_time = if !search_times.is_empty() {
        search_times.iter().sum::<Duration>() / search_times.len() as u32
    } else {
        Duration::from_secs(0)
    };

    let avg_precision = if !precisions.is_empty() {
        precisions.iter().sum::<f64>() / precisions.len() as f64
    } else {
        0.0
    };

    let avg_recall = if !recalls.is_empty() {
        recalls.iter().sum::<f64>() / recalls.len() as f64
    } else {
        0.0
    };

    let avg_f1 = if avg_precision + avg_recall > 0.0 {
        2.0 * (avg_precision * avg_recall) / (avg_precision + avg_recall)
    } else {
        0.0
    };

    ScenarioResult {
        config,
        original_size_bytes: original_size,
        processed_size_bytes: processed_size,
        vector_size_bytes: vector_size,
        total_storage_bytes: processed_size + vector_size,
        preprocessing_time: prep_time,
        indexing_time: Duration::from_millis(0),
        avg_search_time,
        avg_precision,
        avg_recall,
        avg_f1,
        duplicates_found: 0,
    }
}

/// Print results table
fn print_results_table(results: &[ScenarioResult]) {
    println!("\n{}", "=".repeat(120));
    println!("📊 COMPLETE BENCHMARK RESULTS - Using REAL BM25 + HNSW");
    println!("{}\n", "=".repeat(120));

    let baseline_storage = results[0].total_storage_bytes;

    println!("💾 STORAGE IMPACT:");
    println!("{}", "-".repeat(120));
    println!(
        "{:<40} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "Scenario", "Text Size", "Vector Size", "Total", "Saved", "vs Baseline"
    );
    println!("{}", "-".repeat(120));

    for result in results {
        let saved = baseline_storage as i64 - result.total_storage_bytes as i64;
        println!(
            "{:<40} {:>12} {:>12} {:>12} {:>12} {:>11.1}%",
            result.config.name,
            format!("{} KB", result.processed_size_bytes / 1024),
            format!("{} KB", result.vector_size_bytes / 1024),
            format!("{} KB", result.total_storage_bytes / 1024),
            format!("{} KB", saved / 1024),
            result.storage_vs_baseline(baseline_storage)
        );
    }

    println!("\n⚡ PERFORMANCE:");
    println!("{}", "-".repeat(120));
    println!(
        "{:<40} {:>15} {:>15} {:>15}",
        "Scenario", "Preprocessing", "Search Time", "Total Time"
    );
    println!("{}", "-".repeat(120));

    for result in results {
        println!(
            "{:<40} {:>15?} {:>15?} {:>15?}",
            result.config.name,
            result.preprocessing_time,
            result.avg_search_time,
            result.preprocessing_time
        );
    }

    println!("\n🎯 SEARCH QUALITY:");
    println!("{}", "-".repeat(120));
    println!(
        "{:<40} {:>12} {:>12} {:>12}",
        "Scenario", "Precision", "Recall", "F1-Score"
    );
    println!("{}", "-".repeat(120));

    for result in results {
        println!(
            "{:<40} {:>11.1}% {:>11.1}% {:>11.1}%",
            result.config.name,
            result.avg_precision * 100.0,
            result.avg_recall * 100.0,
            result.avg_f1 * 100.0
        );
    }

    println!();
}

/// Print analysis
fn print_analysis(results: &[ScenarioResult]) {
    println!("\n{}", "=".repeat(120));
    println!("📈 KEY FINDINGS");
    println!("{}\n", "=".repeat(120));

    let baseline = &results[0];

    for result in results.iter().skip(1) {
        let storage_diff = result.storage_vs_baseline(baseline.total_storage_bytes);
        let time_diff = ((result.avg_search_time.as_nanos() as f64
            - baseline.avg_search_time.as_nanos() as f64)
            / baseline.avg_search_time.as_nanos() as f64)
            * 100.0;
        let quality_diff = ((result.avg_f1 - baseline.avg_f1) / baseline.avg_f1.max(0.01)) * 100.0;

        println!("vs {}", result.config.name);
        println!(
            "   Storage:  {:>+7.1}% {}",
            storage_diff,
            if storage_diff > 0.0 { "✅" } else { "→" }
        );
        println!(
            "   Latency:  {:>+7.1}% {}",
            time_diff,
            if time_diff.abs() < 10.0 {
                "✅"
            } else {
                "⚠️"
            }
        );
        println!(
            "   Quality:  {:>+7.1}% {}",
            quality_diff,
            if quality_diff.abs() < 5.0 {
                "✅"
            } else if quality_diff > 0.0 {
                "📈"
            } else {
                "📉"
            }
        );
        println!();
    }
}

/// Generate comprehensive report
fn generate_report(results: &[ScenarioResult], docs_count: usize) -> anyhow::Result<()> {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let report_path = PathBuf::from(format!(
        "benchmark/reports/complete_benchmark_{}.md",
        timestamp
    ));

    fs::create_dir_all("benchmark/reports")?;
    let mut report = fs::File::create(&report_path)?;

    writeln!(report, "# Complete Normalization & Quantization Benchmark")?;
    writeln!(report)?;
    writeln!(
        report,
        "**Date**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(report, "**Documents**: {}", docs_count)?;
    writeln!(report, "**Embedding**: BM25 (real)")?;
    writeln!(report, "**Index**: HNSW (real)")?;
    writeln!(report, "**Scenarios**: {}", results.len())?;
    writeln!(report)?;
    writeln!(report, "---")?;
    writeln!(report)?;

    let baseline_storage = results[0].total_storage_bytes;

    // Storage
    writeln!(report, "## 💾 Storage Impact")?;
    writeln!(report)?;
    writeln!(
        report,
        "| Scenario | Text | Vectors | Total | Saved | Reduction |"
    )?;
    writeln!(
        report,
        "|----------|------|---------|-------|-------|-----------|"
    )?;

    for result in results {
        let saved = baseline_storage as i64 - result.total_storage_bytes as i64;
        writeln!(
            report,
            "| {} | {} KB | {} KB | {} KB | {} KB | {:.1}% |",
            result.config.name,
            result.processed_size_bytes / 1024,
            result.vector_size_bytes / 1024,
            result.total_storage_bytes / 1024,
            saved / 1024,
            result.storage_vs_baseline(baseline_storage)
        )?;
    }
    writeln!(report)?;

    // Performance
    writeln!(report, "## ⚡ Performance")?;
    writeln!(report)?;
    writeln!(report, "| Scenario | Preprocessing | Search | Total |")?;
    writeln!(report, "|----------|---------------|--------|-------|")?;

    for result in results {
        writeln!(
            report,
            "| {} | {:?} | {:?} | {:?} |",
            result.config.name,
            result.preprocessing_time,
            result.avg_search_time,
            result.preprocessing_time
        )?;
    }
    writeln!(report)?;

    // Quality
    writeln!(report, "## 🎯 Search Quality")?;
    writeln!(report)?;
    writeln!(report, "| Scenario | Precision | Recall | F1-Score |")?;
    writeln!(report, "|----------|-----------|--------|----------|")?;

    for result in results {
        writeln!(
            report,
            "| {} | {:.1}% | {:.1}% | {:.1}% |",
            result.config.name,
            result.avg_precision * 100.0,
            result.avg_recall * 100.0,
            result.avg_f1 * 100.0
        )?;
    }
    writeln!(report)?;

    // Key Findings
    writeln!(report, "## ✅ Key Findings")?;
    writeln!(report)?;
    writeln!(report, "**Moderate + SQ-8 (Default Configuration)**:")?;

    let baseline = &results[0];

    if let Some(recommended) = results.iter().find(|r| {
        matches!(r.config.normalization, Some(NormalizationLevel::Moderate))
            && r.config.quantization
    }) {
        let storage_saved = baseline_storage as i64 - recommended.total_storage_bytes as i64;
        let quality_change =
            ((recommended.avg_f1 - baseline.avg_f1) / baseline.avg_f1.max(0.01)) * 100.0;
        let latency_change = ((recommended.avg_search_time.as_micros() as f64
            - baseline.avg_search_time.as_micros() as f64)
            / baseline.avg_search_time.as_micros() as f64)
            * 100.0;

        writeln!(
            report,
            "- Storage: -{} KB ({:.1}% reduction)",
            storage_saved / 1024,
            recommended.storage_vs_baseline(baseline_storage)
        )?;
        writeln!(
            report,
            "- Quality: {:.1}% F1 ({:+.1}% vs baseline)",
            recommended.avg_f1 * 100.0,
            quality_change
        )?;
        writeln!(
            report,
            "- Latency: {:?} ({:+.1}% vs baseline)",
            recommended.avg_search_time, latency_change
        )?;
    }

    writeln!(report)?;
    writeln!(report, "---")?;
    writeln!(
        report,
        "**Generated**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;

    println!("\n📄 Report saved to: {}", report_path.display());

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n🔥 COMPLETE NORMALIZATION & QUANTIZATION BENCHMARK");
    println!("Using REAL BM25 embeddings and HNSW index\n");

    // Collect documents
    println!("📁 Collecting documents...");
    let docs = collect_workspace_documents(50);
    println!(
        "   ✓ {} documents ({} KB)\n",
        docs.len(),
        docs.iter().map(|d| d.raw_content.len()).sum::<usize>() / 1024
    );

    // Create embedding manager with BM25
    println!("🔧 Initializing BM25 embedding manager (512D)...");
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25")?;
    println!("   ✓ BM25 ready (512D)\n");

    // Define scenarios
    let scenarios = vec![
        ScenarioConfig {
            name: "Baseline (No Norm, No Quant)".to_string(),
            normalization: None,
            quantization: false,
        },
        ScenarioConfig {
            name: "Quantization Only (SQ-8)".to_string(),
            normalization: None,
            quantization: true,
        },
        ScenarioConfig {
            name: "Normalization Conservative".to_string(),
            normalization: Some(NormalizationLevel::Conservative),
            quantization: false,
        },
        ScenarioConfig {
            name: "Normalization Moderate".to_string(),
            normalization: Some(NormalizationLevel::Moderate),
            quantization: false,
        },
        ScenarioConfig {
            name: "Normalization Aggressive".to_string(),
            normalization: Some(NormalizationLevel::Aggressive),
            quantization: false,
        },
        ScenarioConfig {
            name: "Moderate + SQ-8 (DEFAULT)".to_string(),
            normalization: Some(NormalizationLevel::Moderate),
            quantization: true,
        },
    ];

    println!("🧪 Running {} scenarios...\n", scenarios.len());

    let mut results = Vec::new();
    for (i, scenario) in scenarios.iter().enumerate() {
        print!("   [{}/{}] {}... ", i + 1, scenarios.len(), scenario.name);
        let result = run_scenario(scenario.clone(), &docs, &embedding_manager).await;
        println!("✓");
        results.push(result);
    }

    print_results_table(&results);
    print_analysis(&results);
    generate_report(&results, docs.len())?;

    println!("\n{}", "=".repeat(120));
    println!("✅ Benchmark Complete!");
    println!("{}", "=".repeat(120));
    println!();

    Ok(())
}
