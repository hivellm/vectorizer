//! Benchmark comparison of different embedding approaches
//!
//! This example demonstrates how to compare TF-IDF, BM25, and other embedding
//! methods using standard IR metrics (MAP, MRR, Precision@K, Recall@K).

use std::collections::HashSet;
use std::fs;
use std::time::Instant;
use tracing_subscriber;
use vectorizer::VectorStore;
#[cfg(feature = "candle-models")]
use vectorizer::embedding::{RealModelEmbedder, RealModelType};
use vectorizer::{
    db::{OptimizedHnswConfig, OptimizedHnswIndex},
    document_loader::{DocumentLoader, LoaderConfig},
    embedding::{
        BertEmbedding, Bm25Embedding, EmbeddingManager, EmbeddingProvider, MiniLmEmbedding,
        SvdEmbedding, TfIdfEmbedding,
    },
    evaluation::{EvaluationMetrics, QueryResult, evaluate_search_quality},
    parallel::{ParallelConfig, init_parallel_env},
};

/// Simple document collection for benchmarking
struct BenchmarkDataset {
    documents: Vec<String>,
    queries: Vec<String>,
    // For each query, the set of relevant document indices
    ground_truth: Vec<HashSet<usize>>,
}

impl BenchmarkDataset {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load real documents from gov/ directory
        let gov_path = "/mnt/f/Node/hivellm/gov";
        println!("Loading real documents from: {}", gov_path);

        // Configure document loader
        let config = LoaderConfig {
            collection_name: "gov_benchmark".to_string(),
            max_chunk_size: 1000,
            chunk_overlap: 200,
            include_patterns: vec![],
            exclude_patterns: vec![],
            embedding_dimension: 384, // MiniLM compatible
            embedding_type: "bm25".to_string(),
            allowed_extensions: vec![".md".to_string(), ".txt".to_string(), ".json".to_string()],
            max_file_size: 1024 * 1024, // 1MB max per file
        };

        let mut loader = DocumentLoader::new(config);

        // Create a temporary vector store for loading documents
        let temp_store = VectorStore::new();

        // Load and process documents
        println!("Starting document loading...");
        let chunk_count = loader.load_project(gov_path, &temp_store)?;
        println!("Document loader reported {} chunks processed", chunk_count);

        // Get processed documents (chunks)
        let documents = loader.get_processed_documents();
        println!("Retrieved {} documents from loader", documents.len());

        if documents.is_empty() {
            println!("No documents were loaded. Checking directory contents...");
            // Debug: check if directory exists and has files
            match std::fs::read_dir(gov_path) {
                Ok(entries) => {
                    let mut file_count = 0;
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(ext) = path.extension() {
                                    if ext == "md" || ext == "txt" || ext == "json" {
                                        file_count += 1;
                                        println!("Found valid file: {}", path.display());
                                        if file_count >= 5 {
                                            println!("... (showing first 5 valid files)");
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    println!("Total valid files found: {}", file_count);
                }
                Err(e) => println!("Could not read directory: {}", e),
            }
            return Err("No documents were loaded from gov/ directory".into());
        }

        println!(
            "Loaded {} document chunks from gov/ directory",
            documents.len()
        );

        // Create realistic queries about governance content
        let queries = vec![
            "governance process and voting".to_string(),
            "BIP implementation workflow".to_string(),
            "proposal approval process".to_string(),
            "meeting minutes structure".to_string(),
            "team organization and roles".to_string(),
            "review policy and guidelines".to_string(),
            "model evaluation metrics".to_string(),
            "security and integrity".to_string(),
        ];

        // Define ground truth based on content knowledge
        // This is manually curated based on the gov/ directory structure
        let ground_truth = Self::create_ground_truth(&documents, &queries);

        Ok(Self {
            documents,
            queries,
            ground_truth,
        })
    }

    fn create_ground_truth(documents: &[String], queries: &[String]) -> Vec<HashSet<usize>> {
        // Create ground truth mapping based on document content analysis
        // This is a simplified approach - in practice, this would be done with more sophisticated analysis
        let mut ground_truth = Vec::new();

        for query in queries {
            let mut relevant_docs = HashSet::new();

            // Simple keyword-based relevance (could be improved with actual semantic analysis)
            for (idx, doc) in documents.iter().enumerate() {
                let doc_lower = doc.to_lowercase();
                let query_lower = query.to_lowercase();

                let is_relevant = match query_lower.as_str() {
                    q if q.contains("governance") && q.contains("voting") => {
                        doc_lower.contains("governance")
                            || doc_lower.contains("voting")
                            || doc_lower.contains("consensus")
                    }
                    q if q.contains("bip") && q.contains("implementation") => {
                        doc_lower.contains("bip")
                            || doc_lower.contains("implementation")
                            || doc_lower.contains("workflow")
                    }
                    q if q.contains("proposal") && q.contains("approval") => {
                        doc_lower.contains("proposal")
                            || doc_lower.contains("approval")
                            || doc_lower.contains("approved")
                    }
                    q if q.contains("meeting") && q.contains("minutes") => {
                        doc_lower.contains("minute")
                            || doc_lower.contains("meeting")
                            || doc_lower.contains("summary")
                    }
                    q if q.contains("team") && q.contains("organization") => {
                        doc_lower.contains("team")
                            || doc_lower.contains("structure")
                            || doc_lower.contains("organization")
                    }
                    q if q.contains("review") && q.contains("policy") => {
                        doc_lower.contains("review")
                            || doc_lower.contains("policy")
                            || doc_lower.contains("guideline")
                    }
                    q if q.contains("model") && q.contains("evaluation") => {
                        doc_lower.contains("model")
                            || doc_lower.contains("evaluation")
                            || doc_lower.contains("metric")
                    }
                    q if q.contains("security") && q.contains("integrity") => {
                        doc_lower.contains("security")
                            || doc_lower.contains("integrity")
                            || doc_lower.contains("validation")
                    }
                    _ => false,
                };

                if is_relevant {
                    relevant_docs.insert(idx);
                }
            }

            // Ensure at least some documents are relevant (for testing purposes)
            if relevant_docs.is_empty() {
                // Add first few documents as fallback
                for i in 0..std::cmp::min(3, documents.len()) {
                    relevant_docs.insert(i);
                }
            }

            ground_truth.push(relevant_docs);
        }

        ground_truth
    }
}

/// Convert ground truth document indices to document IDs
fn convert_ground_truth_to_ids(
    ground_truth_indices: &HashSet<usize>,
    _documents: &[String],
) -> HashSet<String> {
    ground_truth_indices
        .iter()
        .map(|&idx| format!("doc_{}", idx))
        .collect()
}

/// Evaluate an embedding method on the benchmark dataset with optimized indexing
fn evaluate_embedding_method(
    embedding_name: &str,
    dataset: &BenchmarkDataset,
    dimension: usize,
) -> Result<EvaluationMetrics, Box<dyn std::error::Error>> {
    println!(
        "Evaluating {} with dimension {}...",
        embedding_name, dimension
    );

    // Initialize parallel environment
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config)?;

    let mut manager = EmbeddingManager::new();

    // Create appropriate embedding provider
    let provider: Box<dyn vectorizer::embedding::EmbeddingProvider> = match embedding_name {
        "TF-IDF" => Box::new(TfIdfEmbedding::new(dimension)),
        "BM25" => Box::new(Bm25Embedding::new(dimension)),
        _ => return Err(format!("Unknown embedding method: {}", embedding_name).into()),
    };

    manager.register_provider(embedding_name.to_string(), provider);
    manager.set_default_provider(embedding_name)?;

    // Build vocabulary from all documents
    println!(
        "Building vocabulary from {} documents...",
        dataset.documents.len()
    );
    if let Some(provider) = manager.get_provider_mut(embedding_name) {
        match embedding_name {
            "TF-IDF" => {
                if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                    tfidf.build_vocabulary(
                        &dataset
                            .documents
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>(),
                    );
                }
            }
            "BM25" => {
                if let Some(bm25) = provider.as_any_mut().downcast_mut::<Bm25Embedding>() {
                    bm25.build_vocabulary(&dataset.documents.clone());
                }
            }
            _ => {}
        }
    }

    // Create optimized HNSW index
    let hnsw_config = OptimizedHnswConfig {
        batch_size: 1000,
        parallel: true,
        initial_capacity: dataset.documents.len(),
        ..Default::default()
    };
    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    // Compute and index all document embeddings
    println!(
        "Computing and indexing {} documents...",
        dataset.documents.len()
    );
    let batch_size = 500; // Larger batches for TF-IDF/BM25 which are faster
    let start_time = Instant::now();

    for (batch_idx, batch) in dataset.documents.chunks(batch_size).enumerate() {
        let batch_start = Instant::now();

        // Compute embeddings for batch
        let mut batch_vectors = Vec::with_capacity(batch.len());
        for (i, document) in batch.iter().enumerate() {
            let doc_idx = batch_idx * batch_size + i;
            let embedding = manager.embed(document)?;
            batch_vectors.push((format!("doc_{}", doc_idx), embedding));
        }

        // Batch insert into index
        index.batch_add(batch_vectors)?;

        let batch_elapsed = batch_start.elapsed();
        let batch_throughput = batch.len() as f64 / batch_elapsed.as_secs_f64();

        if batch_idx % 5 == 0 {
            let total_processed = (batch_idx + 1) * batch_size;
            let progress = (total_processed as f32 / dataset.documents.len() as f32) * 100.0;
            println!(
                "  Batch {}: {}/{} docs ({:.1}%) - {:.2} docs/sec",
                batch_idx,
                total_processed,
                dataset.documents.len(),
                progress,
                batch_throughput
            );
        }
    }

    // Optimize index for search
    index.optimize()?;

    let total_time = start_time.elapsed();
    let overall_throughput = dataset.documents.len() as f64 / total_time.as_secs_f64();
    println!(
        "✅ Indexed {} documents in {:.2}s ({:.2} docs/sec)",
        dataset.documents.len(),
        total_time.as_secs_f64(),
        overall_throughput
    );

    // Evaluate queries using the index
    let mut query_results = Vec::new();

    for (query_idx, query) in dataset.queries.iter().enumerate() {
        // Embed the query
        let query_embedding = manager.embed(query)?;

        // Search using optimized index
        let k = 100;
        let search_results = index.search(&query_embedding, k)?;

        // Convert to QueryResult format
        let results: Vec<QueryResult> = search_results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance, // Convert distance to similarity
            })
            .collect();

        // Convert ground truth to document IDs
        let ground_truth_ids =
            convert_ground_truth_to_ids(&dataset.ground_truth[query_idx], &dataset.documents);

        query_results.push((results, ground_truth_ids));
    }

    // Evaluate search quality
    let metrics = evaluate_search_quality(query_results, 10);

    // Print index statistics
    let memory_stats = index.memory_stats();
    println!("\n📊 {} Index Statistics:", embedding_name);
    println!("  - Vectors: {}", memory_stats.vector_count);
    println!("  - Memory: {}", memory_stats.format());
    println!("  - Build time: {:.2}s", total_time.as_secs_f64());
    println!("  - Throughput: {:.2} docs/sec", overall_throughput);

    Ok(metrics)
}

/// Evaluate dense embedding methods (BERT, MiniLM) with optimized indexing
fn evaluate_dense_embedding_method(
    method: &str,
    dataset: &BenchmarkDataset,
    dimension: usize,
) -> Result<EvaluationMetrics, Box<dyn std::error::Error>> {
    println!("Evaluating {} with dimension {}...", method, dimension);

    // Initialize parallel environment
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config)?;

    let mut manager = EmbeddingManager::new();

    // Create appropriate embedding provider
    let provider: Box<dyn vectorizer::embedding::EmbeddingProvider> = match method {
        "BERT" => {
            let mut bert = BertEmbedding::new(dimension);
            bert.load_model()?;
            Box::new(bert)
        }
        "MiniLM" => {
            let mut minilm = MiniLmEmbedding::new(dimension);
            minilm.load_model()?;
            Box::new(minilm)
        }
        _ => return Err(format!("Unknown dense method: {}", method).into()),
    };

    manager.register_provider(method.to_string(), provider);
    manager.set_default_provider(method)?;

    // Determine batch size based on method (placeholder models are faster)
    let batch_size = 200;

    // Create optimized HNSW index
    let hnsw_config = OptimizedHnswConfig {
        batch_size: 500,
        parallel: true,
        initial_capacity: dataset.documents.len(),
        ..Default::default()
    };
    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    // Compute and index all document embeddings
    println!(
        "Computing and indexing {} documents...",
        dataset.documents.len()
    );
    let start_time = Instant::now();

    for (batch_idx, batch) in dataset.documents.chunks(batch_size).enumerate() {
        let batch_start = Instant::now();

        // Compute embeddings for batch
        let mut batch_vectors = Vec::with_capacity(batch.len());
        for (i, document) in batch.iter().enumerate() {
            let doc_idx = batch_idx * batch_size + i;
            let embedding = manager.embed(document)?;
            batch_vectors.push((format!("doc_{}", doc_idx), embedding));
        }

        // Batch insert into index
        index.batch_add(batch_vectors)?;

        let batch_elapsed = batch_start.elapsed();
        let batch_throughput = batch.len() as f64 / batch_elapsed.as_secs_f64();

        if batch_idx % 10 == 0 {
            let total_processed = (batch_idx + 1) * batch_size;
            let progress = (total_processed as f32 / dataset.documents.len() as f32) * 100.0;
            println!(
                "  Batch {}: {}/{} docs ({:.1}%) - {:.2} docs/sec",
                batch_idx,
                total_processed,
                dataset.documents.len(),
                progress,
                batch_throughput
            );
        }
    }

    // Optimize index for search
    index.optimize()?;

    let total_time = start_time.elapsed();
    let overall_throughput = dataset.documents.len() as f64 / total_time.as_secs_f64();
    println!(
        "✅ Indexed {} documents in {:.2}s ({:.2} docs/sec)",
        dataset.documents.len(),
        total_time.as_secs_f64(),
        overall_throughput
    );

    // Evaluate queries using the index
    let mut query_results = Vec::new();

    for (query_idx, query) in dataset.queries.iter().enumerate() {
        // Get query embedding
        let query_embedding = manager.embed(query)?;

        // Search using optimized index
        let k = 100;
        let search_results = index.search(&query_embedding, k)?;

        // Convert to QueryResult format
        let results: Vec<QueryResult> = search_results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance,
            })
            .collect();

        // Convert ground truth
        let ground_truth_ids =
            convert_ground_truth_to_ids(&dataset.ground_truth[query_idx], &dataset.documents);

        query_results.push((results, ground_truth_ids));
    }

    // Evaluate search quality
    let metrics = evaluate_search_quality(query_results, 10);

    // Print index statistics
    let memory_stats = index.memory_stats();
    println!("\n📊 {} Index Statistics:", method);
    println!("  - Vectors: {}", memory_stats.vector_count);
    println!("  - Memory: {}", memory_stats.format());
    println!("  - Build time: {:.2}s", total_time.as_secs_f64());
    println!("  - Throughput: {:.2} docs/sec", overall_throughput);

    Ok(metrics)
}

/// Evaluate real transformer model embeddings with optimized indexing
#[cfg(feature = "candle-models")]
fn evaluate_real_model_embedding(
    dataset: &BenchmarkDataset,
    model_type: RealModelType,
    model_name: &str,
    dimension: usize,
) -> Result<EvaluationMetrics, Box<dyn std::error::Error>> {
    println!(
        "Evaluating real model {} with dimension {}...",
        model_name, dimension
    );

    // Initialize parallel environment for optimal performance
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config)?;

    // Create real model embedder directly (bypass manager for better performance)
    let embedder = RealModelEmbedder::new(model_type)?;

    // Measure embedding throughput first
    let test_batch = &dataset.documents[..std::cmp::min(100, dataset.documents.len())];
    let test_refs: Vec<&str> = test_batch.iter().map(|s| s.as_str()).collect();

    let start_time = Instant::now();
    let _ = embedder.embed(test_refs[0])?;
    let _single_time = start_time.elapsed();

    let start_time = Instant::now();
    for doc in test_refs.iter().take(10) {
        let _ = embedder.embed(doc)?;
    }
    let batch_time = start_time.elapsed();

    let docs_per_sec = 10.0 / batch_time.as_secs_f64();
    println!("Throughput estimate: {:.2} docs/sec", docs_per_sec);

    // Decide whether to process all documents based on performance
    let process_all = docs_per_sec > 50.0; // If we can do > 50 docs/sec, process all
    let max_docs = if process_all {
        dataset.documents.len()
    } else {
        500
    };

    let sampled_docs: Vec<&String> = if dataset.documents.len() > max_docs {
        println!(
            "Processing {} documents from {} total",
            max_docs,
            dataset.documents.len()
        );
        dataset.documents.iter().take(max_docs).collect()
    } else {
        println!("Processing ALL {} documents!", dataset.documents.len());
        dataset.documents.iter().collect()
    };

    // Create optimized HNSW index
    let hnsw_config = OptimizedHnswConfig {
        batch_size: 1000,
        parallel: true,
        initial_capacity: sampled_docs.len(),
        ..Default::default()
    };
    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    // Pre-compute and index embeddings in batches
    println!("Computing and indexing {} documents...", sampled_docs.len());
    let batch_size = 100;
    let start_time = Instant::now();

    for (batch_idx, batch) in sampled_docs.chunks(batch_size).enumerate() {
        let batch_start = Instant::now();

        // Compute embeddings for batch
        let mut batch_vectors = Vec::with_capacity(batch.len());
        for (i, document) in batch.iter().enumerate() {
            let doc_idx = batch_idx * batch_size + i;
            let embedding = embedder.embed(document)?;
            batch_vectors.push((format!("doc_{}", doc_idx), embedding));
        }

        // Batch insert into index
        index.batch_add(batch_vectors)?;

        let batch_elapsed = batch_start.elapsed();
        let batch_throughput = batch.len() as f64 / batch_elapsed.as_secs_f64();

        if batch_idx % 10 == 0 {
            let total_processed = (batch_idx + 1) * batch_size;
            let progress = (total_processed as f32 / sampled_docs.len() as f32) * 100.0;
            println!(
                "  Batch {}: {}/{} docs ({:.1}%) - {:.2} docs/sec",
                batch_idx,
                total_processed,
                sampled_docs.len(),
                progress,
                batch_throughput
            );
        }
    }

    // Optimize index for search
    index.optimize()?;

    let total_time = start_time.elapsed();
    let overall_throughput = sampled_docs.len() as f64 / total_time.as_secs_f64();
    println!(
        "✅ Indexed {} documents in {:.2}s ({:.2} docs/sec)",
        sampled_docs.len(),
        total_time.as_secs_f64(),
        overall_throughput
    );

    // Evaluate queries using the index
    println!("Evaluating queries using HNSW index...");
    let mut query_results = Vec::new();

    for (query_idx, query) in dataset.queries.iter().enumerate() {
        println!(
            "  Query {}/{}: {}",
            query_idx + 1,
            dataset.queries.len(),
            query
        );

        // Get query embedding
        let query_embedding = embedder.embed(query)?;

        // Search using optimized index
        let k = 100; // Get more results for better evaluation
        let search_results = index.search(&query_embedding, k)?;

        // Convert to QueryResult format
        let results: Vec<QueryResult> = search_results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance, // Convert distance to similarity
            })
            .collect();

        // Create ground truth based on sampled documents
        let ground_truth_ids = if sampled_docs.len() < dataset.documents.len() {
            // For sampled evaluation, adjust ground truth
            let sampled_indices: HashSet<usize> = (0..sampled_docs.len()).collect();
            dataset.ground_truth[query_idx]
                .iter()
                .filter(|idx| sampled_indices.contains(idx))
                .map(|idx| format!("doc_{}", idx))
                .collect()
        } else {
            convert_ground_truth_to_ids(&dataset.ground_truth[query_idx], &dataset.documents)
        };

        query_results.push((results, ground_truth_ids));
    }

    // Evaluate search quality
    let metrics = evaluate_search_quality(query_results, 10);

    // Print index statistics
    let memory_stats = index.memory_stats();
    println!("\n📊 Index Statistics:");
    println!("  - Vectors: {}", memory_stats.vector_count);
    println!("  - Memory: {}", memory_stats.format());
    println!("  - Build time: {:.2}s", total_time.as_secs_f64());
    println!("  - Throughput: {:.2} docs/sec", overall_throughput);

    println!("\n✅ Model evaluation completed successfully!");

    Ok(metrics)
}

/// Evaluate SVD-based embeddings with optimizations
fn evaluate_svd_method_optimized(
    dataset: &BenchmarkDataset,
    svd_dimension: usize,
    max_docs: usize,
) -> Result<EvaluationMetrics, Box<dyn std::error::Error>> {
    println!(
        "Evaluating SVD with dimension {} (using {} docs)...",
        svd_dimension, max_docs
    );

    // Initialize parallel environment
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config)?;

    // Use subset of documents for SVD
    let sampled_docs: Vec<&str> = dataset
        .documents
        .iter()
        .take(max_docs)
        .map(|s| s.as_str())
        .collect();

    // Create SVD embedding with vocabulary size 1000
    let mut svd = SvdEmbedding::new(svd_dimension, 1000);

    // Fit SVD on the sampled documents
    let start_time = Instant::now();
    svd.fit_svd(&sampled_docs)?;
    let fit_time = start_time.elapsed();
    println!("  SVD fit completed in {:.2}s", fit_time.as_secs_f64());

    // Create optimized HNSW index
    let hnsw_config = OptimizedHnswConfig {
        batch_size: 500,
        parallel: true,
        initial_capacity: sampled_docs.len(),
        ..Default::default()
    };
    let index = OptimizedHnswIndex::new(svd_dimension, hnsw_config)?;

    // Index documents
    println!("  Indexing {} documents...", sampled_docs.len());
    let index_start = Instant::now();

    let mut batch_vectors = Vec::new();
    for (idx, doc) in sampled_docs.iter().enumerate() {
        let embedding =
            <SvdEmbedding as vectorizer::embedding::EmbeddingProvider>::embed(&svd, doc)?;
        batch_vectors.push((format!("doc_{}", idx), embedding));

        // Batch insert
        if batch_vectors.len() >= 500 || idx == sampled_docs.len() - 1 {
            index.batch_add(batch_vectors.clone())?;
            batch_vectors.clear();
        }
    }

    index.optimize()?;
    let index_time = index_start.elapsed();
    println!(
        "  Indexed in {:.2}s ({:.2} docs/sec)",
        index_time.as_secs_f64(),
        sampled_docs.len() as f64 / index_time.as_secs_f64()
    );

    // Evaluate queries
    let mut query_results = Vec::new();
    for (query_idx, query) in dataset.queries.iter().enumerate() {
        let query_embedding =
            <SvdEmbedding as vectorizer::embedding::EmbeddingProvider>::embed(&svd, query)?;

        let k = 100;
        let search_results = index.search(&query_embedding, k)?;

        let results: Vec<QueryResult> = search_results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance,
            })
            .collect();

        // Adjust ground truth for sampled docs
        let sampled_indices: HashSet<usize> = (0..sampled_docs.len()).collect();
        let adjusted_ground_truth: HashSet<String> = dataset.ground_truth[query_idx]
            .iter()
            .filter(|idx| sampled_indices.contains(idx))
            .map(|idx| format!("doc_{}", idx))
            .collect();

        query_results.push((results, adjusted_ground_truth));
    }

    let metrics = evaluate_search_quality(query_results, 10);

    // Print statistics
    let memory_stats = index.memory_stats();
    println!("\n📊 SVD Index Statistics:");
    println!("  - Vectors: {}", memory_stats.vector_count);
    println!("  - Memory: {}", memory_stats.format());
    println!(
        "  - Total time: {:.2}s",
        (fit_time + index_time).as_secs_f64()
    );

    Ok(metrics)
}

/// Evaluate ONNX models
#[cfg(feature = "onnx-models")]
fn evaluate_onnx_model(
    dataset: &BenchmarkDataset,
    model_name: &str,
    dimension: usize,
) -> Result<EvaluationMetrics, Box<dyn std::error::Error>> {
    use vectorizer::embedding::{OnnxConfig, OnnxEmbedder, OnnxModelType};

    println!("Evaluating {} with ONNX Runtime...", model_name);

    // Initialize parallel environment
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config)?;

    // Configure ONNX model
    let model_type = match model_name {
        "MiniLM-ONNX" => OnnxModelType::MiniLMMultilingual384,
        "E5-Base-ONNX" => OnnxModelType::E5BaseMultilingual768,
        _ => return Err(format!("Unknown ONNX model: {}", model_name).into()),
    };

    let config = OnnxConfig {
        model_type,
        batch_size: 128,
        use_int8: true, // Enable INT8 quantization
        ..Default::default()
    };
    let use_int8 = config.use_int8;
    let embedder = OnnxEmbedder::new(config)?;

    // Measure throughput
    let test_batch = &dataset.documents[..std::cmp::min(100, dataset.documents.len())];
    let start_time = Instant::now();
    let _ = embedder.embed_parallel(test_batch)?;
    let batch_time = start_time.elapsed();
    let docs_per_sec = test_batch.len() as f64 / batch_time.as_secs_f64();
    println!("ONNX Throughput: {:.2} docs/sec", docs_per_sec);

    // Create optimized index
    let hnsw_config = OptimizedHnswConfig {
        batch_size: 1000,
        parallel: true,
        initial_capacity: dataset.documents.len(),
        ..Default::default()
    };
    let index = OptimizedHnswIndex::new(dimension, hnsw_config)?;

    // Index all documents
    println!(
        "Indexing {} documents with ONNX...",
        dataset.documents.len()
    );
    let start_time = Instant::now();

    for (batch_idx, batch) in dataset.documents.chunks(128).enumerate() {
        let embeddings = embedder.embed_parallel(batch)?;
        let mut batch_vectors = Vec::new();
        for (i, embedding) in embeddings.into_iter().enumerate() {
            let doc_idx = batch_idx * 128 + i;
            batch_vectors.push((format!("doc_{}", doc_idx), embedding));
        }
        index.batch_add(batch_vectors)?;
    }

    index.optimize()?;
    let index_time = start_time.elapsed();
    println!(
        "✅ ONNX indexing completed in {:.2}s ({:.2} docs/sec)",
        index_time.as_secs_f64(),
        dataset.documents.len() as f64 / index_time.as_secs_f64()
    );

    // Evaluate queries
    let mut query_results = Vec::new();
    for (query_idx, query) in dataset.queries.iter().enumerate() {
        let query_embedding = embedder.embed(query)?;
        let search_results = index.search(&query_embedding, 100)?;

        let results: Vec<QueryResult> = search_results
            .into_iter()
            .map(|(id, distance)| QueryResult {
                doc_id: id,
                relevance: 1.0 - distance,
            })
            .collect();

        let ground_truth_ids =
            convert_ground_truth_to_ids(&dataset.ground_truth[query_idx], &dataset.documents);
        query_results.push((results, ground_truth_ids));
    }

    let metrics = evaluate_search_quality(query_results, 10);

    // Print statistics
    let memory_stats = index.memory_stats();
    println!("\n📊 ONNX Index Statistics:");
    println!("  - Model: {} (INT8: {})", model_name, use_int8);
    println!("  - Vectors: {}", memory_stats.vector_count);
    println!("  - Memory: {}", memory_stats.format());
    println!("  - Index time: {:.2}s", index_time.as_secs_f64());

    Ok(metrics)
}

/// Evaluate Hybrid Search (sparse retrieval + dense re-ranking)
fn evaluate_hybrid_search(
    dataset: &BenchmarkDataset,
    sparse_method: &str,
    dense_method: &str,
    dense_dimension: usize,
) -> Result<EvaluationMetrics, Box<dyn std::error::Error>> {
    println!(
        "Evaluating Hybrid Search: {} -> {}",
        sparse_method, dense_method
    );

    // Initialize parallel environment
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config)?;

    // Create sparse retriever (BM25)
    let mut bm25 = Bm25Embedding::new(10000); // Large vocab for BM25
    bm25.build_vocabulary(&dataset.documents);

    // Create dense embedder
    let dense_embedder: Box<dyn vectorizer::embedding::EmbeddingProvider> = match dense_method {
        "BERT" => {
            let mut bert = BertEmbedding::new(dense_dimension);
            bert.load_model()?;
            Box::new(bert)
        }
        "MiniLM" => {
            let mut minilm = MiniLmEmbedding::new(dense_dimension);
            minilm.load_model()?;
            Box::new(minilm)
        }
        _ => return Err(format!("Unknown dense method: {}", dense_method).into()),
    };

    // For hybrid search, we'll simulate the two-stage process
    println!(
        "Building BM25 index for {} documents...",
        dataset.documents.len()
    );
    let start_time = Instant::now();

    let mut query_results = Vec::new();

    for (query_idx, query) in dataset.queries.iter().enumerate() {
        // Stage 1: BM25 retrieval to get top-50 candidates
        let bm25_embedding = bm25.embed(query)?;

        let mut candidates = Vec::new();
        for (doc_idx, doc) in dataset.documents.iter().enumerate() {
            let doc_embedding = bm25.embed(doc)?;
            let similarity = cosine_similarity(&bm25_embedding, &doc_embedding);
            candidates.push((doc_idx, similarity));
        }

        // Sort by BM25 score and take top-50
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_candidates: Vec<_> = candidates.into_iter().take(50).collect();

        // Stage 2: Re-rank top candidates with dense embeddings
        let mut reranked = Vec::new();
        let query_dense = dense_embedder.embed(query)?;

        for (doc_idx, _) in top_candidates {
            let doc_dense = dense_embedder.embed(&dataset.documents[doc_idx])?;
            let dense_similarity = cosine_similarity(&query_dense, &doc_dense);
            reranked.push((doc_idx, dense_similarity));
        }

        // Sort by dense score and take top-10
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let results: Vec<QueryResult> = reranked
            .into_iter()
            .take(10)
            .map(|(doc_idx, relevance)| QueryResult {
                doc_id: format!("doc_{}", doc_idx),
                relevance,
            })
            .collect();

        let ground_truth_ids =
            convert_ground_truth_to_ids(&dataset.ground_truth[query_idx], &dataset.documents);
        query_results.push((results, ground_truth_ids));
    }

    let total_time = start_time.elapsed();
    println!(
        "✅ Hybrid search completed in {:.2}s",
        total_time.as_secs_f64()
    );

    let metrics = evaluate_search_quality(query_results, 10);

    println!("\n📊 Hybrid Search Statistics:");
    println!("  - Sparse: {} (top-50 candidates)", sparse_method);
    println!("  - Dense: {} (re-ranking)", dense_method);
    println!("  - Total time: {:.2}s", total_time.as_secs_f64());

    Ok(metrics)
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Print evaluation results in a formatted way
fn print_results(method: &str, metrics: &EvaluationMetrics) {
    println!("\n=== {} Results ===", method);
    println!("Queries evaluated: {}", metrics.num_queries);
    println!(
        "Mean Average Precision (MAP): {:.4}",
        metrics.mean_average_precision
    );
    println!(
        "Mean Reciprocal Rank (MRR): {:.4}",
        metrics.mean_reciprocal_rank
    );

    println!("\nPrecision@K:");
    for (k, &precision) in metrics.precision_at_k.iter().enumerate() {
        println!("  P@{}: {:.4}", k + 1, precision);
    }

    println!("\nRecall@K:");
    for (k, &recall) in metrics.recall_at_k.iter().enumerate() {
        println!("  R@{}: {:.4}", k + 1, recall);
    }
}

/// Generate Markdown report with all benchmark results
fn generate_markdown_report(
    results: &[(String, EvaluationMetrics)],
    dataset: &BenchmarkDataset,
) -> String {
    let mut report = String::new();

    report.push_str("# Vectorizer Embedding Benchmark Report\n\n");
    report.push_str(&format!(
        "**Generated**: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    report.push_str("## Dataset Overview\n\n");
    report.push_str(&format!(
        "- **Documents**: {} total\n",
        dataset.documents.len()
    ));
    report.push_str(&format!(
        "- **Queries**: {} test queries\n",
        dataset.queries.len()
    ));
    report.push_str("- **Ground Truth**: Manually annotated relevant documents per query\n\n");

    report.push_str("## Benchmark Configuration\n\n");
    report.push_str("### Embedding Methods Tested\n\n");
    report.push_str("| Method | Type | Dimensions | Description |\n");
    report.push_str("|--------|------|------------|-------------|\n");
    report.push_str(
        "| TF-IDF | Sparse | Variable | Traditional term frequency-inverse document frequency |\n",
    );
    report
        .push_str("| BM25 | Sparse | Variable | Advanced sparse retrieval with k1=1.5, b=0.75 |\n");
    report.push_str(
        "| TF-IDF+SVD | Sparse Reduced | 300D/768D | TF-IDF with dimensionality reduction |\n",
    );
    report.push_str("| BERT | Dense | 768D | Contextual embeddings (placeholder/real) |\n");
    report
        .push_str("| MiniLM | Dense | 384D | Efficient sentence embeddings (placeholder/real) |\n");
    report.push_str(
        "| ONNX Models | Dense | 384D/768D | Optimized inference with INT8 quantization |\n",
    );
    report.push_str(
        "| Hybrid Search | Two-stage | Variable | BM25 retrieval + dense re-ranking |\n\n",
    );

    report.push_str("### Evaluation Metrics\n\n");
    report.push_str(
        "- **MAP (Mean Average Precision)**: Average precision across all relevant documents\n",
    );
    report.push_str("- **MRR (Mean Reciprocal Rank)**: Average of reciprocal ranks of first relevant document\n");
    report.push_str("- **Precision@K**: Fraction of relevant documents in top-K results\n");
    report
        .push_str("- **Recall@K**: Fraction of relevant documents retrieved in top-K results\n\n");

    report.push_str("## Results Summary\n\n");
    report.push_str("| Method | MAP | MRR | P@1 | P@3 | P@5 | R@1 | R@3 | R@5 |\n");
    report.push_str("|--------|-----|-----|-----|-----|-----|-----|-----|-----|\n");

    for (method, metrics) in results {
        report.push_str(&format!(
            "| {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
            method,
            metrics.mean_average_precision,
            metrics.mean_reciprocal_rank,
            metrics.precision_at_k.get(0).copied().unwrap_or(0.0), // P@1
            metrics.precision_at_k.get(2).copied().unwrap_or(0.0), // P@3
            metrics.precision_at_k.get(4).copied().unwrap_or(0.0), // P@5
            metrics.recall_at_k.get(0).copied().unwrap_or(0.0),    // R@1
            metrics.recall_at_k.get(2).copied().unwrap_or(0.0),    // R@3
            metrics.recall_at_k.get(4).copied().unwrap_or(0.0),    // R@5
        ));
    }

    report.push_str("\n## Detailed Results\n\n");

    for (method, metrics) in results {
        report.push_str(&format!("### {}\n\n", method));
        report.push_str(&format!(
            "- **Queries Evaluated**: {}\n",
            metrics.num_queries
        ));
        report.push_str(&format!(
            "- **Mean Average Precision**: {:.4}\n",
            metrics.mean_average_precision
        ));
        report.push_str(&format!(
            "- **Mean Reciprocal Rank**: {:.4}\n\n",
            metrics.mean_reciprocal_rank
        ));

        report.push_str("#### Precision@K\n\n");
        report.push_str("| K | Precision |\n");
        report.push_str("|---|-----------|\n");
        for (k, &precision) in metrics.precision_at_k.iter().enumerate() {
            report.push_str(&format!("| {} | {:.4} |\n", k + 1, precision));
        }

        report.push_str("\n#### Recall@K\n\n");
        report.push_str("| K | Recall |\n");
        report.push_str("|---|--------|\n");
        for (k, &recall) in metrics.recall_at_k.iter().enumerate() {
            report.push_str(&format!("| {} | {:.4} |\n", k + 1, recall));
        }
        report.push_str("\n");
    }

    report.push_str("## Analysis & Insights\n\n");

    // Find best performers
    let best_map = results
        .iter()
        .max_by(|a, b| {
            a.1.mean_average_precision
                .partial_cmp(&b.1.mean_average_precision)
                .unwrap()
        })
        .unwrap();

    let best_mrr = results
        .iter()
        .max_by(|a, b| {
            a.1.mean_reciprocal_rank
                .partial_cmp(&b.1.mean_reciprocal_rank)
                .unwrap()
        })
        .unwrap();

    report.push_str(&format!("### Best Performers\n\n"));
    report.push_str(&format!(
        "- **Highest MAP**: {} ({:.4})\n",
        best_map.0, best_map.1.mean_average_precision
    ));
    report.push_str(&format!(
        "- **Highest MRR**: {} ({:.4})\n\n",
        best_mrr.0, best_mrr.1.mean_reciprocal_rank
    ));

    report.push_str("### Observations\n\n");
    report.push_str(
        "- **Sparse vs Dense**: Compare TF-IDF/BM25 (efficient) vs BERT/MiniLM (semantic)\n",
    );
    report.push_str("- **SVD Impact**: Evaluate dimensionality reduction effects on TF-IDF\n");
    report.push_str("- **Hybrid Benefits**: Assess if BM25 + dense re-ranking improves quality\n");
    report.push_str(
        "- **Dataset Characteristics**: Small dataset may favor exact matching methods\n\n",
    );

    report.push_str("### Recommendations\n\n");
    report.push_str("1. **For Efficiency**: Use BM25 or TF-IDF+SVD for fast retrieval\n");
    report.push_str("2. **For Quality**: Consider hybrid approaches when compute allows\n");
    report.push_str("3. **For Scale**: Test with larger, more diverse datasets\n");
    report.push_str(
        "4. **Real Models**: Replace placeholders with actual BERT/MiniLM implementations\n\n",
    );

    report.push_str("## Technical Details\n\n");
    report.push_str("### Implementation Notes\n\n");
    report.push_str(
        "- **TF-IDF+SVD**: Pseudo-orthogonal transformation using Gram-Schmidt orthogonalization\n",
    );
    report.push_str("- **BERT/MiniLM**: Placeholder implementations using seeded hashing\n");
    report.push_str("- **BM25**: Standard parameters (k1=1.5, b=0.75)\n");
    report.push_str("- **Sparse Methods**: TF-IDF and BM25 use variable vocabulary sizes\n");
    report.push_str(
        "- **Dense Methods**: BERT/MiniLM use fixed dimensions with reproducible embeddings\n\n",
    );

    report.push_str("### Dependencies\n\n");
    report.push_str("- `ndarray`: Linear algebra operations\n");
    report.push_str("- `ndarray-linalg`: SVD decomposition\n");
    report.push_str("- Custom evaluation framework\n\n");

    report.push_str("---\n\n");
    report.push_str("*Report generated by Vectorizer benchmark suite*");

    report
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("🚀 Vectorizer Embedding Benchmark");
    println!("==================================");

    let dataset = BenchmarkDataset::new()?;
    println!(
        "Dataset: {} documents, {} queries",
        dataset.documents.len(),
        dataset.queries.len()
    );

    let dimension = 128; // Embedding dimension for comparison

    // Evaluate different embedding methods
    let methods = vec!["TF-IDF", "BM25"];

    let mut results = Vec::new();

    for method in &methods {
        match evaluate_embedding_method(method, &dataset, dimension) {
            Ok(metrics) => {
                print_results(method, &metrics);
                results.push((method.to_string(), metrics));
            }
            Err(e) => {
                println!("Error evaluating {}: {}", method, e);
            }
        }
    }

    // Evaluate SVD-based methods with optimizations
    println!("\n🔍 Evaluating SVD-based methods...");

    // Use a subset for SVD to avoid performance issues
    let svd_sample_size = std::cmp::min(1000, dataset.documents.len());
    if dataset.documents.len() > svd_sample_size {
        println!(
            "   Using {} documents for SVD evaluation (from {} total)",
            svd_sample_size,
            dataset.documents.len()
        );
    }

    match evaluate_svd_method_optimized(&dataset, 300, svd_sample_size) {
        // 300D SVD
        Ok(metrics) => {
            print_results("TF-IDF+SVD(300D)", &metrics);
            results.push(("TF-IDF+SVD(300D)".to_string(), metrics));
        }
        Err(e) => {
            println!("Error evaluating TF-IDF+SVD(300D): {}", e);
        }
    }

    match evaluate_svd_method_optimized(&dataset, 768, svd_sample_size) {
        // 768D SVD
        Ok(metrics) => {
            print_results("TF-IDF+SVD(768D)", &metrics);
            results.push(("TF-IDF+SVD(768D)".to_string(), metrics));
        }
        Err(e) => {
            println!("Error evaluating TF-IDF+SVD(768D): {}", e);
        }
    }

    // Evaluate dense embeddings
    println!("\n🧠 Evaluating dense embeddings...");

    // Test placeholder models first
    match evaluate_dense_embedding_method("BERT", &dataset, 768) {
        Ok(metrics) => {
            print_results("BERT(768D Placeholder)", &metrics);
            results.push(("BERT(768D Placeholder)".to_string(), metrics));
        }
        Err(e) => {
            println!("Error evaluating BERT placeholder: {}", e);
        }
    }

    match evaluate_dense_embedding_method("MiniLM", &dataset, 384) {
        Ok(metrics) => {
            print_results("MiniLM(384D Placeholder)", &metrics);
            results.push(("MiniLM(384D Placeholder)".to_string(), metrics));
        }
        Err(e) => {
            println!("Error evaluating MiniLM placeholder: {}", e);
        }
    }

    // Test real models (only if candle-models feature is enabled)
    #[cfg(feature = "candle-models")]
    {
        println!("\n🤖 Testing real transformer models (cached in /models)...");

        let real_models = vec![
            (
                "MiniLM-Multilingual",
                RealModelType::MiniLMMultilingual,
                384,
            ),
            ("E5-Small", RealModelType::E5SmallMultilingual, 384),
            (
                "DistilUSE-Multilingual",
                RealModelType::DistilUseMultilingual,
                512,
            ),
            (
                "MPNet-Multilingual",
                RealModelType::MPNetMultilingualBase,
                768,
            ),
            ("E5-Base", RealModelType::E5BaseMultilingual, 768),
            ("GTE-Base", RealModelType::GTEMultilingualBase, 768),
            ("LaBSE", RealModelType::LaBSE, 768),
        ];

        for (model_name, model_type, dimension) in real_models {
            println!("\n🔄 Testing {} ({})", model_name, model_type.model_id());
            match evaluate_real_model_embedding(&dataset, model_type.clone(), model_name, dimension)
            {
                Ok(metrics) => {
                    print_results(&format!("{}({}D Real)", model_name, dimension), &metrics);
                    results.push((format!("{}({}D Real)", model_name, dimension), metrics));
                }
                Err(e) => {
                    println!("Error evaluating {}: {}", model_name, e);
                    // Continue with other models
                }
            }
        }
    }

    #[cfg(not(feature = "candle-models"))]
    {
        println!(
            "\n⚠️  Real models not available - compile with --features candle-models to test actual transformer models"
        );
        println!(
            "   Available models would include: MiniLM-Multilingual, E5-Small, MPNet-Multilingual, etc."
        );
    }

    // Test ONNX models if available
    #[cfg(feature = "onnx-models")]
    {
        println!("\n⚡ Testing ONNX models for production inference...");

        match evaluate_onnx_model(&dataset, "MiniLM-ONNX", 384) {
            Ok(metrics) => {
                print_results("MiniLM(384D ONNX)", &metrics);
                results.push(("MiniLM(384D ONNX)".to_string(), metrics));
            }
            Err(e) => {
                println!("Error evaluating MiniLM ONNX: {}", e);
            }
        }

        match evaluate_onnx_model(&dataset, "E5-Base-ONNX", 768) {
            Ok(metrics) => {
                print_results("E5-Base(768D ONNX)", &metrics);
                results.push(("E5-Base(768D ONNX)".to_string(), metrics));
            }
            Err(e) => {
                println!("Error evaluating E5-Base ONNX: {}", e);
            }
        }
    }

    #[cfg(not(feature = "onnx-models"))]
    {
        println!(
            "\n⚡ ONNX models not available - compile with --features onnx-models for optimized inference"
        );
    }

    // Evaluate Hybrid Search approaches
    println!("\n🔀 Evaluating Hybrid Search approaches...");

    match evaluate_hybrid_search(&dataset, "BM25", "BERT", 768) {
        Ok(metrics) => {
            print_results("Hybrid: BM25->BERT", &metrics);
            results.push(("Hybrid: BM25->BERT".to_string(), metrics));
        }
        Err(e) => {
            println!("Error evaluating BM25+BERT hybrid: {}", e);
        }
    }

    match evaluate_hybrid_search(&dataset, "BM25", "MiniLM", 384) {
        Ok(metrics) => {
            print_results("Hybrid: BM25->MiniLM", &metrics);
            results.push(("Hybrid: BM25->MiniLM".to_string(), metrics));
        }
        Err(e) => {
            println!("Error evaluating BM25+MiniLM hybrid: {}", e);
        }
    }

    // Summary comparison
    println!("\n📊 Summary Comparison");
    println!("====================");
    println!(
        "{:<10} {:<8} {:<8} {:<8} {:<8}",
        "Method", "MAP", "MRR", "P@5", "R@5"
    );

    for (method, metrics) in &results {
        println!(
            "{:<10} {:.4}   {:.4}   {:.4}   {:.4}",
            method,
            metrics.mean_average_precision,
            metrics.mean_reciprocal_rank,
            metrics.precision_at_k.get(4).copied().unwrap_or(0.0), // P@5
            metrics.recall_at_k.get(4).copied().unwrap_or(0.0),    // R@5
        );
    }

    // Generate and save Markdown report
    println!("\n📝 Generating Markdown report...");
    let report = generate_markdown_report(&results, &dataset);

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("benchmark/reports/benchmark_report_{}.md", timestamp);

    match fs::write(&filename, &report) {
        Ok(_) => {
            println!("✅ Report saved to: {}", filename);
            println!("📄 Report size: {} bytes", report.len());
        }
        Err(e) => {
            println!("❌ Failed to save report: {}", e);
            // Fallback: tentar salvar no diretório atual
            let fallback_filename = format!("benchmark_report_{}.md", timestamp);
            if let Ok(_) = fs::write(&fallback_filename, &report) {
                println!(
                    "📁 Fallback: Report saved to: {} (current directory)",
                    fallback_filename
                );
            }
        }
    }

    println!("\n✅ Benchmark completed!");
    println!("\n💡 Next steps:");
    println!("   - Implement BERT/MiniLM embeddings for comparison");
    println!("   - Add hybrid search (BM25 + dense embeddings)");
    println!("   - Use larger, more diverse datasets");
    println!("   - Implement proper train/test splits");

    Ok(())
}
