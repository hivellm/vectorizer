# GitHub Copilot Instructions for Vectorizer

## Overview

Vectorizer is a high-performance, in-memory vector database written in Rust, designed for AI-driven applications requiring fast similarity search and vector operations. This document provides specific instructions for GitHub Copilot to generate consistent, high-quality code that follows the project's standards and patterns.

## ⚠️ CRITICAL ARCHITECTURE NOTE

**REST-First Architecture with MCP Support**

- **REST API**: Primary HTTP interface managing all collections and vector data
- **MCP Server**: Model Context Protocol interface for AI assistants
- **Core Engine**: Rust-based vector store with HNSW indexing

**RULE: REST and MCP must have EXACTLY the same functionality**

When adding features:
1. **Implement in core engine first** (business logic)
2. **Add REST endpoints** (HTTP interface)
3. **Add MCP tools** (AI assistant interface)

**NEVER implement features only in MCP!** Both layers must be kept in sync.

**Architecture**: REST + MCP unified server system

## 🚨 RUST EDITION REQUIREMENT - CRITICAL

**This project MANDATORILY uses Rust Edition 2024**

### Non-Negotiable Requirements:
- **Cargo.toml edition field**: MUST be `"2024"` (never change to 2021 or other editions)
- **Build environment**: Must use Rust toolchain with Edition 2024 support
- **Dependencies**: All dependencies must be compatible with Edition 2024

### Why Edition 2024:
- Required for advanced async patterns used throughout the codebase
- Enables memory optimizations and performance features
- Provides access to latest language improvements

### 🚫 NEVER CHANGE THIS SETTING:
```toml
[package]
edition = "2024"  # ← This MUST stay as "2024"
```

**Consequences of changing edition:**
- Compilation failures for Edition 2024 specific code
- Loss of critical language features
- Performance regressions
- Dependency incompatibility

## Core Principles

### Code Quality Standards
- **Readability First**: Generate self-documenting code with clear, descriptive names
- **Consistency**: Follow established patterns throughout the codebase
- **Performance**: Consider performance implications, especially in hot paths
- **Maintainability**: Write code that can be easily understood and modified

### Documentation Requirements
- **Module docs**: Start with `//!` for crate/module-level documentation
- **Item docs**: Use `///` for all public functions, structs, enums, traits, and their fields
- **Parameter docs**: Document all parameters with their types and purposes
- **Return docs**: Explain what functions return and under what conditions
- **Error docs**: Document all error conditions and edge cases
- **Examples**: Include code examples where helpful

## Language-Specific Patterns

### Rust Idioms and Patterns

#### Struct and Enum Definitions
```rust
// Always include these derives in this order for data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// Unique identifier for the vector
    pub id: String,
    /// The vector data
    pub data: Vec<f32>,
    /// Optional payload associated with the vector
    pub payload: Option<Payload>,
}

// API enums should use lowercase serialization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

// Implement Display for user-facing enums
impl fmt::Display for DistanceMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistanceMetric::Cosine => write!(f, "cosine"),
            DistanceMetric::Euclidean => write!(f, "euclidean"),
            DistanceMetric::DotProduct => write!(f, "dot_product"),
        }
    }
}
```

#### Error Handling
```rust
// Use custom error types with meaningful messages
#[derive(thiserror::Error, Debug)]
pub enum VectorizerError {
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VectorizerError>;

// Error propagation patterns
pub fn create_collection(name: &str, config: CollectionConfig) -> Result<(), VectorizerError> {
    // Validate input first
    if name.is_empty() {
        return Err(VectorizerError::InvalidConfiguration {
            message: "Collection name cannot be empty".to_string(),
        });
    }

    // Use ? operator for error propagation
    let collection = Collection::new(name.to_string(), config)?;
    self.collections.insert(name.to_string(), Arc::new(collection));

    Ok(())
}
```

#### Concurrency Patterns
```rust
// Shared ownership across threads
#[derive(Clone)]
pub struct SharedVectorStore {
    collections: Arc<RwLock<HashMap<String, Arc<Collection>>>>,
}

// Concurrent key-value operations
#[derive(Clone)]
pub struct ConcurrentMetadataStore {
    metadata: Arc<DashMap<String, CollectionMetadata>>,
}

// Async function signatures
pub async fn search_collection(
    &self,
    collection_name: &str,
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchResult>, VectorizerError> {
    let collection = self.get_collection(collection_name).await?;
    let results = collection.search(query, limit).await?;
    Ok(results)
}
```

#### Memory Management
```rust
// Pre-allocate when size is known
pub fn process_batch(vectors: &[Vec<f32>]) -> Vec<Vector> {
    let mut result = Vec::with_capacity(vectors.len());

    for vector_data in vectors {
        let vector = Vector {
            id: generate_id(),
            data: vector_data.clone(),
            payload: None,
        };
        result.push(vector);
    }

    result.shrink_to_fit(); // Minimize memory usage
    result
}

// Use Arc for shared heap data
let shared_data = Arc::new(data);
```

### Naming Conventions

#### Functions and Variables
```rust
// snake_case for all functions and variables
fn create_collection(name: &str) -> Result<(), VectorizerError>
let vector_store = VectorStore::new();
let collection_name = "my_collection";

// Multiple words clearly separated
fn process_documents_batch(documents: &[Document]) -> Result<usize, VectorizerError>
let max_concurrent_requests = 10;
```

#### Types and Traits
```rust
// PascalCase for all types
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub dimension: usize,
    pub metric: DistanceMetric,
}

pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

pub trait VectorStorage {
    fn insert(&self, collection: &str, vectors: Vec<Vector>) -> Result<(), VectorizerError>;
    fn search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<SearchResult>, VectorizerError>;
}
```

#### Constants and Statics
```rust
// SCREAMING_SNAKE_CASE for constants
const DEFAULT_DIMENSION: usize = 512;
const MAX_COLLECTION_NAME_LENGTH: usize = 255;
const MAX_VECTORS_PER_REQUEST: usize = 10000;

// Associated constants in impl blocks
impl VectorStore {
    const MAX_COLLECTIONS: usize = 1000;

    pub fn new() -> Self {
        // Implementation
    }
}
```

## API Design Patterns

### REST API Endpoints

#### URL Patterns
```rust
// Consistent URL patterns
// GET    /api/v1/collections
// POST   /api/v1/collections
// GET    /api/v1/collections/{name}
// PUT    /api/v1/collections/{name}
// DELETE /api/v1/collections/{name}
// POST   /api/v1/collections/{name}/vectors
// POST   /api/v1/collections/{name}/search

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
```

#### Handler Functions
```rust
pub async fn create_collection(
    State(state): State<AppState>,
    Json(request): Json<CreateCollectionRequest>,
) -> Result<Json<CreateCollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    if request.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Collection name cannot be empty".to_string(),
                code: "INVALID_NAME".to_string(),
                details: None,
            }),
        ));
    }

    // Create collection
    let collection_id = state.store.create_collection(&request.name, request.config.into()).await?;

    Ok(Json(CreateCollectionResponse {
        id: collection_id,
        name: request.name,
        created_at: Utc::now(),
    }))
}
```

#### Error Responses
```rust
pub async fn search_collection(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate collection exists
    if !state.store.collection_exists(&collection_name).await {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Collection '{}' not found", collection_name),
                code: "COLLECTION_NOT_FOUND".to_string(),
                details: Some(serde_json::json!({
                    "collection_name": collection_name
                })),
            }),
        ));
    }

    // Perform search
    match state.store.search(&collection_name, &request.query, request.limit).await {
        Ok(results) => Ok(Json(SearchResponse {
            results,
            total_found: results.len(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Search failed: {}", e),
                code: "SEARCH_ERROR".to_string(),
                details: None,
            }),
        )),
    }
}
```

### REST API Services

#### Service Definitions
```rust
// REST endpoint patterns
pub async fn create_collection(
    State(state): State<AppState>,
    Json(request): Json<CreateCollectionRequest>,
) -> Result<Json<CreateCollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!("REST CreateCollection request: name={}, dimension={}",
                   request.name, request.config.dimension);

    // Validate request
    if request.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Collection name cannot be empty".to_string(),
                code: "INVALID_NAME".to_string(),
                details: None,
            }),
        ));
    }

    // Create collection
    state.vector_store.create_collection(&request.name, request.config.into()).await?;

    let response = CreateCollectionResponse {
        name: request.name,
        created_at: Utc::now(),
    };

    Ok(Json(response))
}
```

## Configuration Patterns

### Struct Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric for similarity calculations
    pub metric: DistanceMetric,
    /// HNSW index configuration
    pub hnsw_config: HnswConfig,
    /// Quantization configuration (enabled by default for memory optimization)
    pub quantization: QuantizationConfig,
    /// Compression configuration
    pub compression: CompressionConfig,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            dimension: 512,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::SQ { bits: 8 }, // Enable Scalar Quantization by default
            compression: CompressionConfig::default(),
        }
    }
}
```

### Builder Pattern for Complex Configuration
```rust
#[derive(Debug)]
pub struct VectorStoreBuilder {
    dimension: Option<usize>,
    metric: Option<DistanceMetric>,
    max_collections: Option<usize>,
    enable_persistence: bool,
}

impl Default for VectorStoreBuilder {
    fn default() -> Self {
        Self {
            dimension: Some(512),
            metric: Some(DistanceMetric::Cosine),
            max_collections: Some(100),
            enable_persistence: true,
        }
    }
}

impl VectorStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dimension(mut self, dimension: usize) -> Self {
        self.dimension = Some(dimension);
        self
    }

    pub fn metric(mut self, metric: DistanceMetric) -> Self {
        self.metric = Some(metric);
        self
    }

    pub fn max_collections(mut self, max: usize) -> Self {
        self.max_collections = Some(max);
        self
    }

    pub fn disable_persistence(mut self) -> Self {
        self.enable_persistence = false;
        self
    }

    pub fn build(self) -> Result<VectorStore, VectorizerError> {
        // Validate configuration
        let dimension = self.dimension.ok_or_else(|| {
            VectorizerError::InvalidConfiguration {
                message: "Dimension must be specified".to_string(),
            }
        })?;

        // Build store
        Ok(VectorStore {
            dimension,
            metric: self.metric.unwrap_or(DistanceMetric::Cosine),
            max_collections: self.max_collections.unwrap_or(100),
            persistence_enabled: self.enable_persistence,
        })
    }
}

// Usage
let store = VectorStoreBuilder::new()
    .dimension(768)
    .metric(DistanceMetric::Cosine)
    .max_collections(50)
    .build()?;
```

## Testing Patterns

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_collection_success() {
        let store = VectorStore::new();
        let config = CollectionConfig::default();

        let result = store.create_collection("test_collection", config);
        assert!(result.is_ok());

        let collections = store.list_collections();
        assert!(collections.contains(&"test_collection".to_string()));
    }

    #[test]
    fn test_create_duplicate_collection() {
        let store = VectorStore::new();
        let config = CollectionConfig::default();

        // First creation should succeed
        assert!(store.create_collection("duplicate", config.clone()).is_ok());

        // Second creation should fail
        let result = store.create_collection("duplicate", config);
        assert!(matches!(result, Err(VectorizerError::CollectionAlreadyExists(_))));
    }

    #[test]
    fn test_vector_dimension_validation() {
        let store = VectorStore::new();
        let config = CollectionConfig {
            dimension: 512,
            ..Default::default()
        };

        store.create_collection("test", config).unwrap();

        // Correct dimension
        let vector_512 = Vector::new("vec1".to_string(), vec![0.0; 512]);
        assert!(store.insert("test", vec![vector_512]).is_ok());

        // Wrong dimension should fail
        let vector_256 = Vector::new("vec2".to_string(), vec![0.0; 256]);
        let result = store.insert("test", vec![vector_256]);
        assert!(matches!(result, Err(VectorizerError::DimensionMismatch { .. })));
    }
}
```

### Integration Tests
```rust
// tests/integration_tests.rs
use vectorizer::{VectorStore, CollectionConfig, Vector};
use std::sync::Arc;

#[test]
fn test_full_indexing_workflow() {
    let store = Arc::new(VectorStore::new());

    // Create test collection
    let config = CollectionConfig {
        dimension: 384,
        ..Default::default()
    };
    store.create_collection("integration_test", config).unwrap();

    // Generate test vectors
    let vectors: Vec<Vector> = (0..100)
        .map(|i| {
            let data = (0..384)
                .map(|j| (i as f32 * 0.001) + (j as f32 * 0.0001))
                .collect();
            Vector::new(format!("vec_{}", i), data)
        })
        .collect();

    // Insert vectors
    store.insert("integration_test", vectors).unwrap();

    // Test search
    let query = vec![0.5; 384];
    let results = store.search("integration_test", &query, 10).unwrap();

    assert_eq!(results.len(), 10);
    assert!(results[0].score > 0.0);

    // Verify collection metadata
    let metadata = store.get_collection_metadata("integration_test").unwrap();
    assert_eq!(metadata.vector_count, 100);
}
```

### Benchmark Tests
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search_operations(c: &mut Criterion) {
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: 512,
        ..Default::default()
    };

    store.create_collection("benchmark", config).unwrap();

    // Generate test data
    let vectors: Vec<Vector> = (0..10000)
        .map(|i| {
            let data = (0..512)
                .map(|j| (i as f32 * 0.001) + (j as f32 * 0.0001))
                .collect();
            Vector::new(format!("vec_{}", i), data)
        })
        .collect();

    store.insert("benchmark", vectors).unwrap();

    c.bench_function("search_10k_vectors", |b| {
        let query = vec![0.1; 512];
        b.iter(|| {
            black_box(store.search("benchmark", &query, 10).unwrap());
        });
    });
}

criterion_group!(benches, benchmark_search_operations);
criterion_main!(benches);
```

## Search Optimization Patterns

### Intelligent Search Configuration
The Vectorizer project includes advanced search capabilities with multiple optimization levels. When implementing search functionality, use these patterns for optimal performance:

#### Fast Search Configuration (Recommended for most cases)
```rust
// Optimized intelligent search parameters
let search_config = IntelligentSearchConfig {
    query: "search query".to_string(),
    max_results: 5,           // Reduced from default 15 for speed
    mmr_enabled: false,       // Disable MMR for faster processing
    domain_expansion: false,   // Disable domain expansion for speed
    technical_focus: true,     // Keep for technical relevance
    mmr_lambda: 0.7,          // Not used when MMR disabled
};
```

#### Performance Comparison Guidelines
- **Standard Config**: 100% time, High quality (complex research)
- **Optimized Config**: ~30-40% time, Good quality (quick queries)  
- **Minimal Config**: ~20% time, Medium quality (simple searches)

#### Alternative Search Methods
```rust
// For single-collection deep research
let semantic_results = semantic_search(
    collection_name,
    query,
    SemanticSearchConfig {
        max_results: 5,
        semantic_reranking: false,  // Disable for speed
        similarity_threshold: 0.3,
    }
).await?;

// For maximum speed with basic relevance
let basic_results = search_vectors(
    collection_name,
    query,
    SearchConfig { limit: 5 }
).await?;

// For cross-domain research
let cross_results = multi_collection_search(
    collections,
    query,
    MultiCollectionConfig {
        max_total_results: 15,
        cross_collection_reranking: false,  // Disable for speed
        max_per_collection: 5,
    }
).await?;
```

#### Search Strategy Guidelines
- **Use intelligent_search optimized** for quick technical queries
- **Use semantic_search** for single-collection deep research
- **Use search_vectors** for maximum speed with basic relevance
- **Use multi_collection_search** for cross-domain research

#### Query Optimization Tips
- Be specific with technical terms for better relevance
- Use domain-specific vocabulary when available
- Combine multiple search methods for comprehensive results
- Cache frequent queries when possible
- Implement query preprocessing for better performance

### Search Implementation Patterns
```rust
// Search service with optimization levels
pub struct SearchService {
    config: SearchConfig,
}

impl SearchService {
    /// Fast search for quick queries
    pub async fn fast_search(
        &self,
        query: &str,
        collection: &str,
    ) -> Result<Vec<SearchResult>, VectorizerError> {
        let config = IntelligentSearchConfig {
            query: query.to_string(),
            max_results: 5,
            mmr_enabled: false,
            domain_expansion: false,
            technical_focus: true,
            mmr_lambda: 0.7,
        };
        
        self.intelligent_search(collection, config).await
    }
    
    /// Deep search for comprehensive research
    pub async fn deep_search(
        &self,
        query: &str,
        collection: &str,
    ) -> Result<Vec<SearchResult>, VectorizerError> {
        let config = IntelligentSearchConfig {
            query: query.to_string(),
            max_results: 15,
            mmr_enabled: true,
            domain_expansion: true,
            technical_focus: true,
            mmr_lambda: 0.7,
        };
        
        self.intelligent_search(collection, config).await
    }
}
```

## Common Patterns and Idioms

### Iterator Usage
```rust
// Prefer iterator chains over manual loops
pub fn process_vectors(vectors: &[Vector]) -> Vec<ProcessedVector> {
    vectors
        .iter()
        .filter(|v| v.data.len() > 0) // Remove empty vectors
        .map(|v| ProcessedVector {
            id: v.id.clone(),
            data: normalize_vector(&v.data),
            score: calculate_score(&v.data),
        })
        .collect()
}

// Use collect when you need to change types
pub fn extract_ids(vectors: &[Vector]) -> Vec<String> {
    vectors.iter().map(|v| v.id.clone()).collect()
}

// Use for loops when you need side effects or complex logic
pub fn validate_vectors(vectors: &[Vector]) -> Result<(), VectorizerError> {
    for (i, vector) in vectors.iter().enumerate() {
        if vector.data.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("Vector at index {} has empty data", i),
            });
        }
    }
    Ok(())
}
```

### Option and Result Handling
```rust
// Use if let for Option patterns
pub fn get_vector_payload(vector: &Vector) -> Option<&serde_json::Value> {
    if let Some(payload) = &vector.payload {
        Some(&payload.data)
    } else {
        None
    }
}

// Use map for transforming Options
pub fn get_vector_metadata(vector: &Vector) -> Option<String> {
    vector.payload.as_ref()
        .and_then(|p| p.data.get("metadata"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// Use and_then for chaining operations
pub fn find_vector_by_id(store: &VectorStore, collection: &str, id: &str) -> Option<Vector> {
    store.get_collection(collection)
        .and_then(|coll| coll.get_vector(id).ok())
}
```

### Async Patterns
```rust
// Use async/await consistently
pub async fn batch_insert(
    &self,
    collection: &str,
    vectors: Vec<Vector>,
) -> Result<(), VectorizerError> {
    // Validate first
    self.validate_batch(collection, &vectors).await?;

    // Insert in chunks to avoid memory issues
    for chunk in vectors.chunks(1000) {
        self.insert_chunk(collection, chunk).await?;
    }

    Ok(())
}

// Handle timeouts properly
pub async fn search_with_timeout(
    &self,
    collection: &str,
    query: &[f32],
    timeout_ms: u64,
) -> Result<Vec<SearchResult>, VectorizerError> {
    use tokio::time::{timeout, Duration};

    let search_future = self.search(collection, query, 100);
    let duration = Duration::from_millis(timeout_ms);

    match timeout(duration, search_future).await {
        Ok(result) => result,
        Err(_) => Err(VectorizerError::Timeout {
            operation: "search".to_string(),
            duration_ms: timeout_ms,
        }),
    }
}
```

## Performance Considerations

### Memory Optimization
```rust
// Pre-allocate vectors
pub fn create_vector_batch(count: usize, dimension: usize) -> Vec<Vector> {
    let mut vectors = Vec::with_capacity(count);

    for i in 0..count {
        let mut data = Vec::with_capacity(dimension);
        // Fill data...
        vectors.push(Vector {
            id: format!("vec_{}", i),
            data,
            payload: None,
        });
    }

    vectors.shrink_to_fit();
    vectors
}

// Use SmallVec for small collections
use smallvec::SmallVec;

// Reuse buffers
pub struct VectorProcessor {
    buffer: Vec<f32>,
}

impl VectorProcessor {
    pub fn new(dimension: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(dimension),
        }
    }

    pub fn process(&mut self, vector: &Vector) -> ProcessedVector {
        self.buffer.clear();
        self.buffer.extend_from_slice(&vector.data);

        // Process buffer...
        ProcessedVector {
            id: vector.id.clone(),
            processed_data: self.buffer.clone(),
        }
    }
}
```

### CPU Optimization
```rust
// Use parallel processing when beneficial
use rayon::prelude::*;

pub fn batch_normalize(vectors: &mut [Vector]) {
    vectors.par_iter_mut().for_each(|vector| {
        normalize_inplace(&mut vector.data);
    });
}

// SIMD-friendly operations
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    use std::simd::*;

    let mut sum_ab = f32x4::splat(0.0);
    let mut sum_aa = f32x4::splat(0.0);
    let mut sum_bb = f32x4::splat(0.0);

    let chunks = a.chunks_exact(4).zip(b.chunks_exact(4));

    for (a_chunk, b_chunk) in chunks {
        let va = f32x4::from_slice(a_chunk);
        let vb = f32x4::from_slice(b_chunk);

        sum_ab += va * vb;
        sum_aa += va * va;
        sum_bb += vb * vb;
    }

    // Handle remaining elements...
    let dot_product = sum_ab.reduce_sum();
    let norm_a = sum_aa.reduce_sum().sqrt();
    let norm_b = sum_bb.reduce_sum().sqrt();

    dot_product / (norm_a * norm_b)
}
```

## Module Organization

### File Structure
```
src/
├── api/
│   ├── handlers.rs     # HTTP request handlers
│   ├── routes.rs       # Route definitions
│   └── types.rs        # API request/response types
├── db/
│   ├── collection.rs   # Collection implementation
│   ├── vector_store.rs # Main vector store
│   └── mod.rs
├── models/
│   ├── collection.rs   # Collection models
│   ├── vector.rs       # Vector models
│   └── mod.rs
├── quantization/
│   ├── scalar.rs       # Scalar quantization
│   ├── traits.rs       # Quantization traits
│   └── mod.rs
├── embedding/
│   ├── bm25.rs         # BM25 embedding
│   ├── bert.rs         # BERT embedding
│   └── mod.rs
├── error.rs            # Error types
├── lib.rs              # Library exports
└── main.rs             # Binary entry point
```

### Import Organization
```rust
// Group imports logically
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

// External crates
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use dashmap::DashMap;

// Internal modules (group by functionality)
use crate::{
    // Models
    models::{CollectionConfig, Vector, SearchResult},

    // Core functionality
    db::{VectorStore, Collection},
    embedding::EmbeddingManager,

    // Error handling
    error::{Result, VectorizerError},

    // Utilities
    utils::validation,
};
```

## Security Patterns

### Input Validation
```rust
pub fn validate_collection_name(name: &str) -> Result<(), VectorizerError> {
    if name.is_empty() {
        return Err(VectorizerError::InvalidConfiguration {
            message: "Collection name cannot be empty".to_string(),
        });
    }

    if name.len() > MAX_COLLECTION_NAME_LENGTH {
        return Err(VectorizerError::InvalidConfiguration {
            message: format!(
                "Collection name too long: {} > {}",
                name.len(),
                MAX_COLLECTION_NAME_LENGTH
            ),
        });
    }

    // Check for valid characters (alphanumeric, underscore, dash)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(VectorizerError::InvalidConfiguration {
            message: "Collection name contains invalid characters".to_string(),
        });
    }

    Ok(())
}

pub fn validate_vector_data(data: &[f32], expected_dim: usize) -> Result<(), VectorizerError> {
    if data.is_empty() {
        return Err(VectorizerError::InvalidConfiguration {
            message: "Vector data cannot be empty".to_string(),
        });
    }

    if data.len() != expected_dim {
        return Err(VectorizerError::DimensionMismatch {
            expected: expected_dim,
            actual: data.len(),
        });
    }

    // Check for invalid values
    if data.iter().any(|&x| !x.is_finite()) {
        return Err(VectorizerError::InvalidConfiguration {
            message: "Vector contains non-finite values".to_string(),
        });
    }

    Ok(())
}
```

### Resource Limits
```rust
// Define constants for resource limits
const MAX_COLLECTION_NAME_LENGTH: usize = 255;
const MAX_DIMENSION: usize = 4096;
const MAX_VECTORS_PER_REQUEST: usize = 10000;
const MAX_PAYLOAD_SIZE_BYTES: usize = 1024 * 1024; // 1MB
const MAX_SEARCH_LIMIT: usize = 1000;
const REQUEST_TIMEOUT_SECONDS: u64 = 300;

pub struct RateLimiter {
    requests: DashMap<String, Vec<u64>>, // IP -> timestamps
    max_requests_per_minute: u32,
}

impl RateLimiter {
    pub fn check_rate_limit(&self, ip: &str) -> Result<(), VectorizerError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut timestamps = self.requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove timestamps older than 1 minute
        timestamps.retain(|&timestamp| now - timestamp < 60);

        if timestamps.len() >= self.max_requests_per_minute as usize {
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: "requests_per_minute".to_string(),
                limit: self.max_requests_per_minute,
            });
        }

        timestamps.push(now);
        Ok(())
    }
}
```

## Logging Patterns

```rust
use tracing::{debug, info, warn, error, instrument};

// Instrument functions for tracing
#[instrument(skip(self, query), fields(collection_name = %collection_name))]
pub async fn search(
    &self,
    collection_name: &str,
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchResult>, VectorizerError> {
    info!("Starting search in collection '{}'", collection_name);

    // Validate inputs
    if query.is_empty() {
        warn!("Empty query vector provided");
        return Err(VectorizerError::InvalidConfiguration {
            message: "Query vector cannot be empty".to_string(),
        });
    }

    // Perform search
    let start_time = std::time::Instant::now();
    let results = self.perform_search(collection_name, query, limit).await?;
    let duration = start_time.elapsed();

    info!(
        "Search completed in {:.2}ms, found {} results",
        duration.as_millis(),
        results.len()
    );

    debug!("Search results: {:?}", results);

    Ok(results)
}

// Structured logging
pub async fn create_collection(
    &self,
    name: &str,
    config: CollectionConfig,
) -> Result<(), VectorizerError> {
    info!(
        collection_name = %name,
        dimension = config.dimension,
        metric = %config.metric,
        "Creating new collection"
    );

    // Implementation...

    info!(
        collection_name = %name,
        "Collection created successfully"
    );

    Ok(())
}
```

## Common Anti-Patterns to Avoid

### ❌ Don't write this:
```rust
// Unclear variable names
fn proc(vecs: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let mut res = vec![];
    for v in vecs {
        let mut nv = vec![];
        for &x in &v {
            nv.push(x * x);
        }
        res.push(nv);
    }
    res
}

// Missing error handling
pub fn divide(a: f32, b: f32) -> f32 {
    a / b // Will panic on division by zero
}

// Blocking operations in async code
pub async fn slow_operation() -> Result<(), Error> {
    std::thread::sleep(std::time::Duration::from_secs(1)); // Blocks!
    Ok(())
}

// Manual loop when iterator would be clearer
pub fn sum_positive(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &num in numbers {
        if num > 0 {
            sum += num;
        }
    }
    sum
}
```

### ✅ Write this instead:
```rust
// Clear names and documentation
/// Calculate the element-wise square of multiple vectors
///
/// # Arguments
/// * `vectors` - A slice of vectors to process
///
/// # Returns
/// A vector containing the squared vectors
pub fn square_vectors(vectors: &[Vec<f32>]) -> Vec<Vec<f32>> {
    vectors
        .iter()
        .map(|vector| {
            vector.iter().map(|&value| value * value).collect()
        })
        .collect()
}

// Proper error handling
pub fn divide(a: f32, b: f32) -> Result<f32, MathError> {
    if b == 0.0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(a / b)
}

// Async sleep
pub async fn slow_operation() -> Result<(), Error> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Ok(())
}

// Use iterators
pub fn sum_positive(numbers: &[i32]) -> i32 {
    numbers.iter().filter(|&&x| x > 0).sum()
}
```

## Project-Specific Conventions

### Vectorizer Architecture Patterns

1. **API Layer**: HTTP endpoints in `src/api/`
2. **Business Logic**: Core operations in `src/db/` and `src/embedding/`
3. **Data Access**: Persistence and caching in `src/persistence/`
4. **Infrastructure**: MCP in `src/mcp/`

### Configuration Hierarchy

1. **Workspace Config**: `workspace.yml` - project definitions
2. **Runtime Config**: Environment variables and CLI flags
3. **Code Defaults**: Sensible defaults in Rust code

### Performance Priorities

1. **Memory Efficiency**: Quantization, pre-allocation, buffer reuse
2. **Search Speed**: HNSW indexing, SIMD operations, parallel processing
3. **Scalability**: Async operations, connection pooling, resource limits

## 🚨 CRITICAL: Server Execution Instructions

### Vectorizer REST-First Architecture
**CRITICAL REQUIREMENT**: Vectorizer uses a REST-first architecture with integrated MCP support. Both REST and MCP are part of a unified server process.

### ✅ CORRECT: Starting the Server
```bash
# Start unified server
./target/release/vectorizer

# Development mode
cargo run

# This starts:
# - REST API on http://127.0.0.1:15002
# - MCP Server on ws://127.0.0.1:15002/mcp
```

### ✅ CORRECT: Stopping the Server
```bash
# Kill vectorizer process
pkill vectorizer

# Alternative: Ctrl+C if running in foreground
```

### Architecture Flow
```
Client → REST/MCP → Core Engine → Vector Store
```

### Why This Matters for Code Generation
- **ALWAYS generate code** that works with unified server architecture
- **Architecture**: REST + MCP integrated in single process
- **Service lifecycle**: Managed by single vectorizer process
- **Simplified deployment**: No need for orchestrator or multiple processes

These instructions ensure GitHub Copilot generates code that seamlessly integrates with the Vectorizer codebase, following established patterns and maintaining high quality standards. Always consider the performance implications and security requirements when generating new code.
