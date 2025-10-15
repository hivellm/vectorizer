//! Quantized collection implementation for memory-optimized vector storage
//! 
//! This module provides a collection implementation that uses quantization
//! to reduce memory usage while maintaining search quality.

use crate::{
    models::{Vector, CollectionConfig},
    quantization::{
        QuantizationType, QuantizationConfig, QuantizationResult,
        traits::{QuantizationMethod, QuantizedVectors},
        scalar::ScalarQuantization,
        storage::{QuantizedVectorStorage, StorageConfig},
        hnsw_integration::{QuantizedHnswIndex, HnswQuantizationConfig},
    },
    error::{Result, VectorizerError},
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use chrono;

/// A collection that uses quantization for memory optimization
#[derive(Clone, Debug)]
pub struct QuantizedCollection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Quantization configuration
    quantization_config: QuantizationConfig,
    /// Quantized vector storage
    quantized_storage: Arc<QuantizedVectorStorage>,
    /// Quantized HNSW index
    quantized_index: Arc<RwLock<Option<QuantizedHnswIndex>>>,
    /// Vector metadata (ID -> metadata mapping)
    vector_metadata: Arc<DashMap<String, VectorMetadata>>,
    /// Vector IDs in insertion order
    vector_order: Arc<RwLock<Vec<String>>>,
    /// Embedding type used for this collection
    embedding_type: Arc<RwLock<String>>,
    /// Set of unique document IDs
    document_ids: Arc<DashMap<String, ()>>,
    /// Persistent vector count
    vector_count: Arc<RwLock<usize>>,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

/// Metadata for quantized vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    /// Vector ID
    pub id: String,
    /// Document ID (if any)
    pub document_id: Option<String>,
    /// Vector dimension
    pub dimension: usize,
    /// Quantization method used
    pub quantization_type: QuantizationType,
    /// Quality metrics
    pub quality_score: Option<f32>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl QuantizedCollection {
    /// Create a new quantized collection
    pub fn new(name: String, config: CollectionConfig) -> Result<Self> {
        Self::new_with_quantization(name, config, QuantizationConfig::default())
    }

    /// Create a new quantized collection with specific quantization config
    pub fn new_with_quantization(
        name: String,
        config: CollectionConfig,
        quantization_config: QuantizationConfig,
    ) -> Result<Self> {
        // Create storage configuration
        let storage_config = StorageConfig {
            storage_dir: std::env::temp_dir(),
            max_cache_size_mb: 100,
            enable_memory_mapping: true,
            auto_cleanup: true,
            max_file_size_mb: 1000,
        };

        // Create quantized storage
        let quantized_storage = Arc::new(QuantizedVectorStorage::new(storage_config)?);

        Ok(Self {
            name,
            config,
            quantization_config,
            quantized_storage,
            quantized_index: Arc::new(RwLock::new(None)),
            vector_metadata: Arc::new(DashMap::new()),
            vector_order: Arc::new(RwLock::new(Vec::new())),
            embedding_type: Arc::new(RwLock::new("bm25".to_string())),
            document_ids: Arc::new(DashMap::new()),
            vector_count: Arc::new(RwLock::new(0)),
            created_at: chrono::Utc::now(),
            updated_at: Arc::new(RwLock::new(chrono::Utc::now())),
        })
    }

    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    /// Get quantization config
    pub fn quantization_config(&self) -> &QuantizationConfig {
        &self.quantization_config
    }

    /// Add vectors to the collection with quantization
    pub fn add_vectors(&self, vectors: &[Vector]) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        // Extract vector data and metadata
        let vector_data: Vec<Vec<f32>> = vectors.iter()
            .map(|v| v.data.clone())
            .collect();

        // Initialize quantized index if needed
        {
            let mut index_guard = self.quantized_index.write().unwrap();
            if index_guard.is_none() {
                let hnsw_config = HnswQuantizationConfig {
                    quantization_type: self.quantization_config.method.clone(),
                    enable_quantized_search: true,
                    cache_size: 10000,
                    enable_hybrid_search: true,
                    quality_threshold: 0.8,
                };
                
                *index_guard = Some(QuantizedHnswIndex::new(hnsw_config, self.quantized_storage.clone())?);
            }
        }

        // Add vectors to quantized index
        {
            let mut index_guard = self.quantized_index.write().unwrap();
            if let Some(ref mut index) = *index_guard {
                index.insert(&vector_data)?;
            }
        }

        // Store metadata for each vector
        for (i, vector) in vectors.iter().enumerate() {
            let metadata = VectorMetadata {
                id: vector.id.clone(),
                document_id: None, // TODO: Add document_id to Vector struct
                dimension: vector.data.len(),
                quantization_type: self.quantization_config.method.clone(),
                quality_score: None,
                created_at: chrono::Utc::now(),
            };

            self.vector_metadata.insert(vector.id.clone(), metadata);
            
            // Track document IDs (TODO: Add document_id to Vector struct)
            // if let Some(ref doc_id) = vector.document_id {
            //     self.document_ids.insert(doc_id.clone(), ());
            // }

            // Update vector order
            {
                let mut order_guard = self.vector_order.write().unwrap();
                order_guard.push(vector.id.clone());
            }
        }

        // Update counts and timestamp
        {
            let mut count_guard = self.vector_count.write().unwrap();
            *count_guard += vectors.len();
        }

        {
            let mut updated_guard = self.updated_at.write().unwrap();
            *updated_guard = chrono::Utc::now();
        }

        Ok(())
    }

    /// Search for similar vectors using quantized search
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        let index_guard = self.quantized_index.read().unwrap();
        if let Some(ref index) = *index_guard {
            let results = index.search(query, k)?;
            
            // Convert internal IDs to vector IDs
            let mut search_results = Vec::new();
            for (internal_id, score) in results {
                if let Some(metadata_entry) = self.vector_metadata.iter().nth(internal_id) {
                    search_results.push((metadata_entry.key().clone(), score));
                }
            }
            
            Ok(search_results)
        } else {
            Err(VectorizerError::CollectionNotInitialized(self.name.clone()))
        }
    }

    /// Get vector count
    pub fn vector_count(&self) -> usize {
        *self.vector_count.read().unwrap()
    }

    /// Get document count
    pub fn document_count(&self) -> usize {
        self.document_ids.len()
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            name: self.name.clone(),
            config: self.config.clone(),
            vector_count: self.vector_count(),
            document_count: self.document_count(),
            dimension: self.config.dimension,
            created_at: self.created_at,
            updated_at: *self.updated_at.read().unwrap(),
            quantization_type: Some(self.quantization_config.method.clone()),
            memory_usage_mb: self.estimate_memory_usage(),
        }
    }

    /// Estimate memory usage in MB
    pub fn estimate_memory_usage(&self) -> f64 {
        // Rough estimation based on quantized storage
        let vector_count = self.vector_count();
        let dimension = self.config.dimension;
        
        match self.quantization_config.method {
            QuantizationType::Scalar(8) => (vector_count * dimension) as f64 / (1024.0 * 1024.0) * 0.25, // 4x compression
            QuantizationType::Scalar(4) => (vector_count * dimension) as f64 / (1024.0 * 1024.0) * 0.125, // 8x compression
            QuantizationType::Scalar(2) => (vector_count * dimension) as f64 / (1024.0 * 1024.0) * 0.0625, // 16x compression
            QuantizationType::Scalar(1) => (vector_count * dimension) as f64 / (1024.0 * 1024.0) * 0.03125, // 32x compression
            _ => (vector_count * dimension) as f64 / (1024.0 * 1024.0) * 4.0, // Default to float32 size
        }
    }

    /// Set embedding type
    pub fn set_embedding_type(&self, embedding_type: String) {
        let mut guard = self.embedding_type.write().unwrap();
        *guard = embedding_type;
    }

    /// Get embedding type
    pub fn get_embedding_type(&self) -> String {
        self.embedding_type.read().unwrap().clone()
    }
}

/// Collection metadata for quantized collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub name: String,
    pub config: CollectionConfig,
    pub vector_count: usize,
    pub document_count: usize,
    pub dimension: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub quantization_type: Option<QuantizationType>,
    pub memory_usage_mb: f64,
}
