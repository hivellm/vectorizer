//! Test Data Generation Utilities
//!
//! Provides utilities for generating synthetic test data for benchmarks,
//! including vectors, documents, and queries.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::benchmark::BenchmarkConfig;

/// Test data generator for benchmarks
pub struct TestDataGenerator {
    config: BenchmarkConfig,
    rng: fastrand::Rng,
}

/// Generated test data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestData {
    /// Vector data with IDs
    pub vectors: Vec<(String, Vec<f32>)>,
    /// Document texts (for embedding benchmarks)
    pub documents: Vec<String>,
    /// Test queries
    pub queries: Vec<String>,
    /// Ground truth for quality evaluation
    pub ground_truth: Vec<Vec<String>>,
    /// Metadata about the generated data
    pub metadata: TestDataMetadata,
}

/// Metadata about generated test data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataMetadata {
    /// Number of vectors generated
    pub vector_count: usize,
    /// Vector dimension
    pub dimension: usize,
    /// Generation method used
    pub generation_method: String,
    /// Data source type
    pub data_source: String,
    /// Generation time in seconds
    pub generation_time_sec: f64,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

impl TestDataGenerator {
    /// Create new test data generator
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            rng: fastrand::Rng::new(),
        }
    }

    /// Generate synthetic vectors
    pub fn generate_vectors(
        &mut self,
        count: usize,
        dimension: usize,
    ) -> Result<TestData, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        let mut vectors = Vec::with_capacity(count);
        let mut documents = Vec::with_capacity(count);

        for i in 0..count {
            let id = format!("vec_{}", i);

            // Generate vector using different methods based on count
            let vector = if count < 1000 {
                self.generate_deterministic_vector(i, dimension)
            } else {
                self.generate_random_vector(dimension)
            };

            vectors.push((id, vector));

            // Generate corresponding document text
            documents.push(self.generate_document_text(i));
        }

        // Generate test queries
        let queries = self.generate_test_queries();

        // Generate ground truth (simplified)
        let ground_truth = self.generate_ground_truth(&vectors, &queries);

        let generation_time = start.elapsed().as_secs_f64();

        let mut properties = HashMap::new();
        properties.insert("vector_type".to_string(), "synthetic".to_string());
        properties.insert("distribution".to_string(), "uniform".to_string());

        Ok(TestData {
            vectors,
            documents,
            queries,
            ground_truth,
            metadata: TestDataMetadata {
                vector_count: count,
                dimension,
                generation_method: "synthetic".to_string(),
                data_source: "generated".to_string(),
                generation_time_sec: generation_time,
                properties,
            },
        })
    }

    /// Generate vectors with specific patterns for testing
    pub fn generate_pattern_vectors(
        &mut self,
        count: usize,
        dimension: usize,
        pattern: VectorPattern,
    ) -> Result<TestData, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        let mut vectors = Vec::with_capacity(count);
        let mut documents = Vec::with_capacity(count);

        for i in 0..count {
            let id = format!("vec_{}", i);

            let vector = match pattern {
                VectorPattern::Gaussian => self.generate_gaussian_vector(dimension),
                VectorPattern::Uniform => self.generate_uniform_vector(dimension),
                VectorPattern::Clustered => self.generate_clustered_vector(i, dimension),
                VectorPattern::Sparse => self.generate_sparse_vector(dimension),
                VectorPattern::Dense => self.generate_dense_vector(dimension),
                VectorPattern::Deterministic => self.generate_deterministic_vector(i, dimension),
            };

            vectors.push((id, vector));
            documents.push(self.generate_document_text(i));
        }

        let queries = self.generate_test_queries();
        let ground_truth = self.generate_ground_truth(&vectors, &queries);

        let generation_time = start.elapsed().as_secs_f64();

        let mut properties = HashMap::new();
        properties.insert("vector_type".to_string(), "pattern".to_string());
        properties.insert("pattern".to_string(), format!("{:?}", pattern));

        Ok(TestData {
            vectors,
            documents,
            queries,
            ground_truth,
            metadata: TestDataMetadata {
                vector_count: count,
                dimension,
                generation_method: format!("pattern_{:?}", pattern),
                data_source: "generated".to_string(),
                generation_time_sec: generation_time,
                properties,
            },
        })
    }

    /// Generate vectors from real data (if available)
    pub fn generate_from_real_data(
        &mut self,
        data_source: &str,
        max_count: usize,
        dimension: usize,
    ) -> Result<TestData, Box<dyn std::error::Error>> {
        // This would integrate with actual data loading
        // For now, fall back to synthetic generation
        println!("Real data loading not implemented, using synthetic data");
        self.generate_vectors(max_count, dimension)
    }

    /// Generate vectors for specific benchmark scenarios
    pub fn generate_for_scenario(
        &mut self,
        scenario: BenchmarkScenario,
        dimension: usize,
    ) -> Result<TestData, Box<dyn std::error::Error>> {
        match scenario {
            BenchmarkScenario::SmallDataset => self.generate_vectors(1000, dimension),
            BenchmarkScenario::MediumDataset => self.generate_vectors(10000, dimension),
            BenchmarkScenario::LargeDataset => self.generate_vectors(100000, dimension),
            BenchmarkScenario::HugeDataset => self.generate_vectors(1000000, dimension),
            BenchmarkScenario::MemoryStress => {
                self.generate_pattern_vectors(50000, dimension, VectorPattern::Dense)
            }
            BenchmarkScenario::SearchAccuracy => {
                self.generate_pattern_vectors(10000, dimension, VectorPattern::Clustered)
            }
            BenchmarkScenario::InsertPerformance => self.generate_vectors(50000, dimension),
            BenchmarkScenario::ConcurrentLoad => self.generate_vectors(25000, dimension),
        }
    }

    // Private helper methods

    fn generate_deterministic_vector(&self, index: usize, dimension: usize) -> Vec<f32> {
        (0..dimension)
            .map(|j| ((index * 13 + j * 17) % 1000) as f32 / 1000.0)
            .collect()
    }

    fn generate_random_vector(&mut self, dimension: usize) -> Vec<f32> {
        (0..dimension)
            .map(|_| self.rng.f32() * 2.0 - 1.0) // Range [-1, 1]
            .collect()
    }

    fn generate_gaussian_vector(&mut self, dimension: usize) -> Vec<f32> {
        (0..dimension)
            .map(|_| {
                // Box-Muller transform for Gaussian distribution
                let u1 = self.rng.f32();
                let u2 = self.rng.f32();
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
                z0 * 0.5 // Scale to reasonable range
            })
            .collect()
    }

    fn generate_uniform_vector(&mut self, dimension: usize) -> Vec<f32> {
        (0..dimension).map(|_| self.rng.f32() * 2.0 - 1.0).collect()
    }

    fn generate_clustered_vector(&mut self, index: usize, dimension: usize) -> Vec<f32> {
        let num_clusters = 10;
        let cluster_id = index % num_clusters;
        let cluster_center = (cluster_id as f32) / (num_clusters as f32) * 2.0 - 1.0;

        (0..dimension)
            .map(|_| {
                let noise = (self.rng.f32() - 0.5) * 0.2; // Small noise
                cluster_center + noise
            })
            .collect()
    }

    fn generate_sparse_vector(&mut self, dimension: usize) -> Vec<f32> {
        let sparsity = 0.1; // 10% non-zero elements
        let num_nonzero = (dimension as f32 * sparsity) as usize;

        let mut vector = vec![0.0; dimension];
        let mut indices: Vec<usize> = (0..dimension).collect();
        self.rng.shuffle(&mut indices);

        for i in 0..num_nonzero {
            vector[indices[i]] = self.rng.f32() * 2.0 - 1.0;
        }

        vector
    }

    fn generate_dense_vector(&mut self, dimension: usize) -> Vec<f32> {
        // Generate dense vector with all non-zero elements
        (0..dimension).map(|_| self.rng.f32() * 2.0 - 1.0).collect()
    }

    fn generate_document_text(&self, index: usize) -> String {
        let templates = vec![
            "This is a test document about vector databases and similarity search algorithms.",
            "Machine learning and artificial intelligence applications require efficient vector operations.",
            "The vectorizer project provides high-performance vector database capabilities for Rust applications.",
            "Benchmarking vector operations is crucial for understanding performance characteristics.",
            "HNSW indexing provides logarithmic search complexity for approximate nearest neighbor queries.",
            "Quantization techniques can significantly reduce memory usage while maintaining search quality.",
            "Concurrent access patterns require careful consideration in vector database design.",
            "Memory-mapped files and efficient data structures are key to vector database performance.",
            "The Rust programming language provides excellent performance for vector operations.",
            "Semantic search applications benefit from high-dimensional vector representations.",
        ];

        let template = templates[index % templates.len()];
        format!("{} (Document {})", template, index)
    }

    fn generate_test_queries(&self) -> Vec<String> {
        vec![
            "vector database performance".to_string(),
            "similarity search algorithms".to_string(),
            "machine learning vectors".to_string(),
            "HNSW indexing optimization".to_string(),
            "quantization memory reduction".to_string(),
            "concurrent vector operations".to_string(),
            "semantic search applications".to_string(),
            "Rust vector processing".to_string(),
            "benchmarking vector databases".to_string(),
            "high-dimensional data structures".to_string(),
        ]
    }

    fn generate_ground_truth(
        &self,
        vectors: &[(String, Vec<f32>)],
        queries: &[String],
    ) -> Vec<Vec<String>> {
        // Simplified ground truth generation based on keyword matching
        queries
            .iter()
            .map(|query| {
                let query_lower = query.to_lowercase();
                let keywords: Vec<&str> = query_lower.split_whitespace().collect();

                vectors
                    .iter()
                    .filter(|(id, _)| {
                        // Simple keyword matching for ground truth
                        keywords.iter().any(|kw| id.contains(kw))
                    })
                    .map(|(id, _)| id.clone())
                    .take(10) // Limit to top 10
                    .collect()
            })
            .collect()
    }
}

/// Vector generation patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorPattern {
    /// Gaussian distribution
    Gaussian,
    /// Uniform distribution
    Uniform,
    /// Clustered data
    Clustered,
    /// Sparse vectors
    Sparse,
    /// Dense vectors
    Dense,
    /// Deterministic pattern
    Deterministic,
}

/// Benchmark scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkScenario {
    /// Small dataset (1K vectors)
    SmallDataset,
    /// Medium dataset (10K vectors)
    MediumDataset,
    /// Large dataset (100K vectors)
    LargeDataset,
    /// Huge dataset (1M vectors)
    HugeDataset,
    /// Memory stress test
    MemoryStress,
    /// Search accuracy test
    SearchAccuracy,
    /// Insert performance test
    InsertPerformance,
    /// Concurrent load test
    ConcurrentLoad,
}

impl TestData {
    /// Get vector count
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// Get vector dimension
    pub fn dimension(&self) -> usize {
        self.metadata.dimension
    }

    /// Get vectors as slice
    pub fn vectors(&self) -> &[(String, Vec<f32>)] {
        &self.vectors
    }

    /// Get documents as slice
    pub fn documents(&self) -> &[String] {
        &self.documents
    }

    /// Get queries as slice
    pub fn queries(&self) -> &[String] {
        &self.queries
    }

    /// Get ground truth for a specific query
    pub fn ground_truth_for_query(&self, query_index: usize) -> Option<&[String]> {
        self.ground_truth.get(query_index).map(|v| v.as_slice())
    }

    /// Split data into training and test sets
    pub fn split(&self, train_ratio: f64) -> (TestData, TestData) {
        let split_point = (self.vectors.len() as f64 * train_ratio) as usize;

        let train_vectors = self.vectors[..split_point].to_vec();
        let test_vectors = self.vectors[split_point..].to_vec();

        let train_docs = self.documents[..split_point].to_vec();
        let test_docs = self.documents[split_point..].to_vec();

        let train_data = TestData {
            vectors: train_vectors,
            documents: train_docs,
            queries: self.queries.clone(),
            ground_truth: self.ground_truth.clone(),
            metadata: TestDataMetadata {
                vector_count: split_point,
                dimension: self.metadata.dimension,
                generation_method: format!("{}_train", self.metadata.generation_method),
                data_source: self.metadata.data_source.clone(),
                generation_time_sec: 0.0,
                properties: self.metadata.properties.clone(),
            },
        };

        let test_data = TestData {
            vectors: test_vectors,
            documents: test_docs,
            queries: self.queries.clone(),
            ground_truth: self.ground_truth.clone(),
            metadata: TestDataMetadata {
                vector_count: self.vectors.len() - split_point,
                dimension: self.metadata.dimension,
                generation_method: format!("{}_test", self.metadata.generation_method),
                data_source: self.metadata.data_source.clone(),
                generation_time_sec: 0.0,
                properties: self.metadata.properties.clone(),
            },
        };

        (train_data, test_data)
    }

    /// Sample a subset of the data
    pub fn sample(&self, count: usize) -> TestData {
        let actual_count = count.min(self.vectors.len());
        let step = self.vectors.len() / actual_count;

        let sampled_vectors: Vec<_> = self
            .vectors
            .iter()
            .step_by(step)
            .take(actual_count)
            .cloned()
            .collect();

        let sampled_docs: Vec<_> = self
            .documents
            .iter()
            .step_by(step)
            .take(actual_count)
            .cloned()
            .collect();

        TestData {
            vectors: sampled_vectors,
            documents: sampled_docs,
            queries: self.queries.clone(),
            ground_truth: self.ground_truth.clone(),
            metadata: TestDataMetadata {
                vector_count: actual_count,
                dimension: self.metadata.dimension,
                generation_method: format!("{}_sampled", self.metadata.generation_method),
                data_source: self.metadata.data_source.clone(),
                generation_time_sec: 0.0,
                properties: self.metadata.properties.clone(),
            },
        }
    }
}
