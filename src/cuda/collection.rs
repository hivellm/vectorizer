//! CUDA-Enhanced Collection
//! 
//! This module provides a CUDA-enhanced version of the Collection that uses
//! CUDA for accelerated vector similarity search operations.

use std::sync::Arc;
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, SearchResult, Vector, Payload, DistanceMetric};
use crate::cuda::{CudaConfig, CudaVectorOperations};
use crate::db::optimized_hnsw::{OptimizedHnswIndex, OptimizedHnswConfig};
use dashmap::DashMap;
use parking_lot::RwLock;
use chrono::DateTime;
use chrono::Utc;
use tracing::{debug, info};

/// Processed vector data for batch operations
#[derive(Debug, Clone)]
struct ProcessedVector {
    id: String,
    data: Vec<f32>,
    document_id: Option<String>,
}

/// CUDA-enhanced collection with accelerated search
pub struct CudaCollection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vector storage
    vectors: Arc<DashMap<String, Vector>>,
    /// Vector IDs in insertion order
    vector_order: Arc<RwLock<Vec<String>>>,
    /// CUDA-enhanced HNSW index
    cuda_index: Arc<RwLock<OptimizedHnswIndex>>,
    /// CUDA vector operations
    cuda_operations: CudaVectorOperations,
    /// Embedding type
    embedding_type: Arc<RwLock<String>>,
    /// Document IDs
    document_ids: Arc<DashMap<String, ()>>,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<DateTime<Utc>>>,
    /// CUDA configuration
    cuda_config: CudaConfig,
}

impl CudaCollection {
    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    /// Create a new CUDA-enhanced collection
    pub fn new(name: String, config: CollectionConfig, cuda_config: CudaConfig) -> Self {
        Self::new_with_embedding_type(name, config, "bm25".to_string(), cuda_config)
    }

    /// Create a new CUDA-enhanced collection with specific embedding type
    pub fn new_with_embedding_type(
        name: String,
        config: CollectionConfig,
        embedding_type: String,
        cuda_config: CudaConfig,
    ) -> Self {
        // Convert HnswConfig to OptimizedHnswConfig
        let optimized_config = OptimizedHnswConfig {
            max_connections: config.hnsw_config.m,
            max_connections_0: config.hnsw_config.m * 2,
            ef_construction: config.hnsw_config.ef_construction,
            seed: config.hnsw_config.seed,
            distance_metric: config.metric,
            parallel: true,
            initial_capacity: 100_000,
            batch_size: 1000,
        };

        let cuda_index = OptimizedHnswIndex::new(config.dimension, optimized_config)
            .expect("Failed to create CUDA HNSW index");
        
        let cuda_operations = CudaVectorOperations::new(cuda_config.clone());
        let now = Utc::now();

        Self {
            name,
            config,
            vectors: Arc::new(DashMap::new()),
            vector_order: Arc::new(RwLock::new(Vec::new())),
            cuda_index: Arc::new(RwLock::new(cuda_index)),
            cuda_operations,
            embedding_type: Arc::new(RwLock::new(embedding_type)),
            document_ids: Arc::new(DashMap::new()),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
            cuda_config,
        }
    }

    /// Search for similar vectors with CUDA acceleration using hybrid approach
    pub async fn search_with_cuda(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query_vector.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_vector.len(),
            });
        }

        // Normalize query vector for cosine similarity
        let search_vector = if matches!(self.config.metric, DistanceMetric::Cosine) {
            crate::models::vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Step 1: Use HNSW to get candidates (top-1k)
        let index = self.cuda_index.read();
        let candidate_count = std::cmp::min(2048, self.vectors.len()); // Increased to 2k candidates
        let candidates = index.search(&search_vector, candidate_count)?;
        
        if candidates.is_empty() {
            return Ok(vec![]);
        }

        // Gating logic: only use GPU if work is substantial enough
        let work_operations = 1 * candidates.len() * self.config.dimension; // Q=1, K'=candidates.len(), D=dimension
        let gpu_threshold = 5_000_000; // 5M operations threshold
        
        if work_operations < gpu_threshold {
            // Fall back to CPU for small workloads
            return self.search_cpu_fallback(&search_vector, &candidates, k);
        }

        // Step 2: Extract candidate vectors for GPU re-ranking
        let candidate_vectors: Vec<Vec<f32>> = candidates.iter()
            .filter_map(|(id, _)| self.vectors.get(id).map(|v| v.data.clone()))
            .collect();
        
        let candidate_ids: Vec<String> = candidates.iter()
            .map(|(id, _)| id.clone())
            .collect();

        // Step 3: Use CUDA for exact re-ranking of candidates
        let similarities = self.cuda_operations
            .parallel_similarity_search(&search_vector, &candidate_vectors, 0.0, self.config.metric)
            .await?;

        // Step 4: Combine results with IDs and sort by similarity
        let mut results: Vec<(String, f32)> = candidate_ids
            .into_iter()
            .zip(similarities.into_iter())
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k results
        results.truncate(k);

        // Build SearchResult objects
        let mut search_results = Vec::with_capacity(results.len());
        for (id, score) in results {
            if let Some(vector) = self.vectors.get(&id) {
                search_results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }

        Ok(search_results)
    }

    /// CPU fallback for small workloads
    fn search_cpu_fallback(&self, query_vector: &[f32], candidates: &[(String, f32)], k: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::with_capacity(candidates.len());
        
        for (id, _) in candidates {
            if let Some(vector) = self.vectors.get(id) {
                let similarity = match self.config.metric {
                    DistanceMetric::Cosine => {
                        let dot_product: f32 = query_vector.iter().zip(vector.data.iter()).map(|(a, b)| a * b).sum();
                        let query_norm: f32 = query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                        let vector_norm: f32 = vector.data.iter().map(|x| x * x).sum::<f32>().sqrt();
                        if query_norm > 0.0 && vector_norm > 0.0 {
                            dot_product / (query_norm * vector_norm)
                        } else {
                            0.0
                        }
                    }
                    DistanceMetric::Euclidean => {
                        let sum_squared_diff: f32 = query_vector.iter()
                            .zip(vector.data.iter())
                            .map(|(a, b)| (a - b).powi(2))
                            .sum();
                        sum_squared_diff.sqrt()
                    }
                    DistanceMetric::DotProduct => {
                        query_vector.iter().zip(vector.data.iter()).map(|(a, b)| a * b).sum()
                    }
                };
                
                results.push(SearchResult {
                    id: id.clone(),
                    score: similarity,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k results
        results.truncate(k);
        
        Ok(results)
    }

    /// Batch search with CUDA acceleration for multiple queries
    pub async fn batch_search_with_cuda(&self, queries: &[Vec<f32>], k: usize) -> Result<Vec<Vec<SearchResult>>> {
        if queries.is_empty() {
            return Ok(vec![]);
        }

        // For now, use simple sequential processing to avoid complexity
        let mut results = Vec::with_capacity(queries.len());
        
        for query in queries {
            let query_results = self.search_with_cuda(query, k).await?;
            results.push(query_results);
        }
        
        Ok(results)
    }

    /// Regular search (fallback to HNSW)
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query_vector.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_vector.len(),
            });
        }

        // Normalize query vector for cosine similarity
        let search_vector = if matches!(self.config.metric, DistanceMetric::Cosine) {
            crate::models::vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Search in HNSW index
        let index = self.cuda_index.read();
        let neighbors = index.search(&search_vector, k)?;

        // Build results
        let mut results = Vec::with_capacity(neighbors.len());
        for (id, score) in neighbors {
            if let Some(vector) = self.vectors.get(&id) {
                results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Add a vector to the collection
    pub fn add_vector(&self, vector: Vector) -> Result<()> {
        let id = vector.id.clone();

        // Extract document ID from payload
        if let Some(payload) = &vector.payload {
            if let Some(file_path) = payload.data.get("file_path") {
                if let Some(file_path_str) = file_path.as_str() {
                    self.document_ids.insert(file_path_str.to_string(), ());
                }
            }
        }

        // Store vector
        self.vectors.insert(id.clone(), vector.clone());

        // Add to CUDA index
        let index = self.cuda_index.read();
        index.add(id.clone(), vector.data.clone())?;

        // Track insertion order
        let mut vector_order = self.vector_order.write();
        vector_order.push(id);

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        Ok(())
    }

    /// Batch add vectors with CUDA acceleration (parallelized)
    pub async fn batch_add_vectors_with_cuda(&self, vectors: Vec<Vector>) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        debug!("Adding {} vectors to CUDA collection '{}' (parallel)", vectors.len(), self.name);

        // Validate dimensions first
        for vector in &vectors {
            if vector.dimension() != self.config.dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: self.config.dimension,
                    got: vector.dimension(),
                });
            }
        }

        // Process vectors in parallel chunks
        let chunk_size = 1000; // Process in chunks to avoid memory issues
        let mut tasks = Vec::new();
        let metric = self.config.metric.clone();

        for chunk in vectors.chunks(chunk_size) {
            let chunk_vectors: Vec<Vector> = chunk.to_vec();
            let metric_clone = metric.clone();
            let task = tokio::spawn(async move {
                Self::process_vector_chunk(chunk_vectors, metric_clone).await
            });
            tasks.push(task);
        }

        // Collect results from all tasks
        let mut all_processed_vectors = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(processed)) => all_processed_vectors.extend(processed),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(VectorizerError::InternalError(format!("Task join error: {}", e))),
            }
        }

        // Now insert all processed vectors into the collection
        self.insert_processed_vectors(all_processed_vectors)?;

        debug!("Successfully added {} vectors to CUDA collection '{}' (parallel)", self.vectors.len(), self.name);
        Ok(())
    }

    /// Process a chunk of vectors in parallel
    async fn process_vector_chunk(vectors: Vec<Vector>, metric: DistanceMetric) -> Result<Vec<ProcessedVector>> {
        let mut processed = Vec::new();

        for vector in vectors {
            let id = vector.id.clone();
            let mut data = vector.data.clone();

            // Normalize vector for cosine similarity if needed
            if matches!(metric, DistanceMetric::Cosine) {
                data = crate::models::vector_utils::normalize_vector(&data);
            }

            // Extract document ID from payload
            let document_id = vector.payload.as_ref()
                .and_then(|p| p.data.get("file_path"))
                .and_then(|fp| fp.as_str())
                .map(|s| s.to_string());

            processed.push(ProcessedVector {
                id,
                data,
                document_id,
            });
        }

        Ok(processed)
    }

    /// Insert processed vectors into the collection
    fn insert_processed_vectors(&self, processed_vectors: Vec<ProcessedVector>) -> Result<()> {
        // Extract document IDs
        for pv in &processed_vectors {
            if let Some(doc_id) = &pv.document_id {
                self.document_ids.insert(doc_id.clone(), ());
            }
        }

        // Prepare data for CUDA index batch insertion
        let ids: Vec<String> = processed_vectors.iter().map(|pv| pv.id.clone()).collect();
        let datas: Vec<Vec<f32>> = processed_vectors.iter().map(|pv| pv.data.clone()).collect();

        // Add to CUDA index (batch operation if supported)
        let index = self.cuda_index.read();
        for (id, data) in ids.iter().zip(datas.iter()) {
            index.add(id.clone(), data.clone())?;
        }

        // Store vectors and track order
        let mut vector_order = self.vector_order.write();
        for pv in processed_vectors {
            // Create full vector for storage
            let vector = Vector {
                id: pv.id.clone(),
                data: pv.data,
                payload: None, // We'll reconstruct if needed
            };

            self.vectors.insert(pv.id.clone(), vector);
            vector_order.push(pv.id);
        }

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        Ok(())
    }

    /// Get collection metadata
    pub fn metadata(&self) -> crate::models::CollectionMetadata {
        crate::models::CollectionMetadata {
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
            vector_count: self.vectors.len(),
            document_count: self.document_ids.len(),
            config: self.config.clone(),
        }
    }

    /// Get CUDA configuration
    pub fn cuda_config(&self) -> &CudaConfig {
        &self.cuda_config
    }

    /// Check if CUDA is available
    pub fn is_cuda_available(&self) -> bool {
        self.cuda_operations.is_cuda_available()
    }

    /// Get CUDA device info
    pub fn get_cuda_device_info(&self) -> Result<crate::cuda::CudaDeviceInfo> {
        self.cuda_operations.get_device_info()
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// Get all vectors in the collection
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        self.vectors
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get the embedding type
    pub fn get_embedding_type(&self) -> String {
        self.embedding_type.read().clone()
    }

    /// Set the embedding type
    pub fn set_embedding_type(&self, embedding_type: String) {
        *self.embedding_type.write() = embedding_type;
    }

    /// Remove a vector by ID
    pub fn remove_vector(&self, vector_id: &str) -> Result<()> {
        // Remove from vectors
        if self.vectors.remove(vector_id).is_none() {
            return Err(VectorizerError::VectorNotFound(vector_id.to_string()));
        }

        // Remove from order tracking
        let mut vector_order = self.vector_order.write();
        vector_order.retain(|id| id != vector_id);

        // Remove from CUDA index
        let index = self.cuda_index.read();
        index.remove(vector_id)?;

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        self.vectors
            .get(vector_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))
    }

    /// Estimate memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_size = std::mem::size_of::<f32>() * self.config.dimension;
        let entry_overhead = std::mem::size_of::<String>() + std::mem::size_of::<Vector>();
        let total_per_vector = vector_size + entry_overhead;

        self.vectors.len() * total_per_vector
    }

    /// Initialize CUHNSW with data if available
    pub fn initialize_cuhnsw(&mut self, data: &[f32], num_data: usize, num_dims: usize) -> Result<()> {
        self.cuda_operations.initialize_cuhnsw(data, num_data, num_dims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CollectionConfig, HnswConfig, CompressionConfig};

    fn create_test_config() -> CollectionConfig {
        CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 50,
                seed: Some(42),
            },
            compression: CompressionConfig::default(),
            quantization: None,
        }
    }

    #[tokio::test]
    async fn test_cuda_collection_creation() {
        let config = create_test_config();
        let cuda_config = CudaConfig::default();
        
        let collection = CudaCollection::new("test".to_string(), config, cuda_config);
        assert_eq!(collection.name, "test");
        assert_eq!(collection.vector_count(), 0);
    }

    #[tokio::test]
    async fn test_cuda_collection_search() {
        let config = create_test_config();
        let cuda_config = CudaConfig::default();
        
        let collection = CudaCollection::new("test".to_string(), config, cuda_config);
        
        // Add test vectors
        let vector1 = Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            payload: None,
        };
        
        let vector2 = Vector {
            id: "vec2".to_string(),
            data: vec![0.0, 1.0, 0.0],
            payload: None,
        };
        
        collection.add_vector(vector1).unwrap();
        collection.add_vector(vector2).unwrap();
        
        // Search with CUDA
        let results = collection.search_with_cuda(&[1.0, 0.0, 0.0], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // First result should be the identical vector
        assert_eq!(results[0].id, "vec1");
        assert!(results[0].score > 0.9);
    }

    #[tokio::test]
    async fn test_cuda_collection_batch_add() {
        let config = create_test_config();
        let cuda_config = CudaConfig::default();
        
        let collection = CudaCollection::new("test".to_string(), config, cuda_config);
        
        // Create test vectors
        let vectors = vec![
            Vector {
                id: "vec1".to_string(),
                data: vec![1.0, 0.0, 0.0],
                payload: None,
            },
            Vector {
                id: "vec2".to_string(),
                data: vec![0.0, 1.0, 0.0],
                payload: None,
            },
        ];
        
        // Batch add
        collection.batch_add_vectors_with_cuda(vectors).await.unwrap();
        
        assert_eq!(collection.vector_count(), 2);
    }
}
