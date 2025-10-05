# Vectorizer Coding Guidelines

This document outlines the coding standards and best practices for the Vectorizer project. These guidelines ensure consistency, maintainability, and performance across the codebase.

## ⚠️ CRITICAL ARCHITECTURE RULE

### REST + MCP is the Primary Architecture - Unified Server

**Architecture Overview:**
- **Unified Server**: Manages all real collections and vector data
- **REST API**: HTTP interface for direct access
- **MCP Server**: Model Context Protocol interface for AI integration

**MANDATORY RULE: GRPC, REST, and MCP must have EXACTLY the same functionality**

**Implementation Order:**
1. **Implement in GRPC first** (core business logic in `src/grpc/`)
2. **Add REST endpoints** (HTTP interface in `src/api/`)
3. **Add MCP tools** (AI assistant interface in `src/mcp/`)

**🚫 NEVER implement features only in REST or MCP!**

**Violations to Fix:**
- `memory-analysis` endpoint exists only in REST API but not in MCP
- Any feature that exists in only one layer breaks the architecture

**Why This Matters:**
- GRPC is the single source of truth for data operations
- REST and MCP are just different interfaces to the same GRPC functionality
- Ensures all interfaces stay synchronized
- Prevents feature fragmentation across layers

## 🚨 RUST EDITION REQUIREMENT

### This Project Uses Rust Edition 2024 - NON-NEGOTIABLE

**MANDATORY REQUIREMENT:**
- **Edition**: `2024` (in `Cargo.toml`)
- **Why**: This project specifically requires Rust Edition 2024 features
- **DO NOT CHANGE**: Never downgrade to 2021 or other editions
- **Build Environment**: Must use Rust toolchain that supports Edition 2024

**Technical Details:**
- Edition 2024 provides access to the latest language features
- Required for advanced async patterns, memory optimizations, and performance features
- All dependencies must be compatible with Edition 2024
- CI/CD must use Rust toolchain with Edition 2024 support

**🚫 NEVER CHANGE THIS SETTING:**
```toml
[package]
name = "vectorizer"
version = "0.21.0"
edition = "2024"  # ← This MUST stay as "2024"
```

**Consequences of Changing Edition:**
- Breaks compilation of Edition 2024 specific code
- Loses access to critical language features
- Performance regressions
- Incompatibility with project dependencies

## Table of Contents

1. [General Principles](#general-principles)
2. [Rust-Specific Patterns](#rust-specific-patterns)
3. [Code Organization](#code-organization)
4. [Error Handling](#error-handling)
5. [Concurrency and Performance](#concurrency-and-performance)
6. [API Design](#api-design)
7. [Testing](#testing)
8. [Documentation](#documentation)
9. [Build and Deployment](#build-and-deployment)

## General Principles

### Code Quality

- **Readability First**: Code should be self-documenting. Choose clear, descriptive names over clever abbreviations.
- **Consistency**: Follow the established patterns in the codebase. When in doubt, look at existing similar code.
- **Performance**: Consider performance implications of design decisions, especially in hot paths.
- **Maintainability**: Write code that can be easily understood and modified by other developers.

### Naming Conventions

```rust
// Functions and variables: snake_case
fn create_collection(name: &str) -> Result<(), VectorizerError>
let vector_store = VectorStore::new();

// Structs, enums, traits: PascalCase
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub dimension: usize,
    pub metric: DistanceMetric,
}

// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_DIMENSION: usize = 512;
const MAX_CONNECTIONS: usize = 1000;

// Modules: snake_case
mod vector_store;
mod grpc_client;
```

## Rust-Specific Patterns

### Struct and Enum Design

```rust
// Always include these derives in this order
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

### Memory Management

```rust
// Pre-allocate when size is known
pub fn process_batch(vectors: &[Vec<f32>]) -> Vec<Vector> {
    let mut result = Vec::with_capacity(vectors.len());

    for vector_data in vectors {
        let vector = Vector {
            id: generate_id(),
            data: vector_data.clone(), // Consider using references if possible
            payload: None,
        };
        result.push(vector);
    }

    result.shrink_to_fit(); // Minimize memory usage
    result
}

// Use Arc for shared ownership
#[derive(Clone)]
pub struct SharedVectorStore {
    store: Arc<RwLock<VectorStore>>,
}

// Prefer stack allocation when possible
pub fn calculate_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Use local variables instead of heap allocation
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}
```

## Code Organization

### Module Structure

The codebase is organized into logical modules:

```
src/
├── api/           # HTTP/gRPC API endpoints and handlers
├── db/            # Core database operations and storage
├── grpc/          # gRPC service definitions and clients
├── models/        # Data models and configurations
├── embedding/     # Embedding providers and algorithms
├── quantization/  # Vector quantization implementations
├── persistence/   # Data persistence and caching
├── mcp/           # Model Context Protocol integration
├── batch/         # Batch processing utilities
├── error.rs       # Error types and handling
├── lib.rs         # Library exports and module organization
└── main.rs        # Binary entry point
```

### Function Organization

```rust
// Keep functions focused and under 50 lines
// Break complex functions into smaller, testable units

// ❌ Bad: Single large function
pub fn process_documents(docs: &[Document]) -> Result<Vec<Vector>, VectorizerError> {
    // 100+ lines of mixed concerns
}

// ✅ Good: Focused, composable functions
pub fn process_documents(docs: &[Document]) -> Result<Vec<Vector>, VectorizerError> {
    let chunks = chunk_documents(docs)?;
    let embeddings = generate_embeddings(&chunks)?;
    let vectors = create_vectors(embeddings)?;
    Ok(vectors)
}

fn chunk_documents(docs: &[Document]) -> Result<Vec<DocumentChunk>, VectorizerError> {
    // Focused on chunking logic only
}

fn generate_embeddings(chunks: &[DocumentChunk]) -> Result<Vec<Vec<f32>>, VectorizerError> {
    // Focused on embedding generation only
}

fn create_vectors(embeddings: Vec<Vec<f32>>) -> Vec<Vector> {
    // Focused on vector creation only
}
```

## Error Handling

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VectorizerError {
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Vector not found: {0}")]
    VectorNotFound(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VectorizerError>;
```

### Error Propagation

```rust
// Use ? operator for concise error propagation
pub fn create_collection(
    &self,
    name: &str,
    config: CollectionConfig,
) -> Result<(), VectorizerError> {
    // Validate input
    self.validate_config(&config)?;

    // Check if collection exists
    if self.collections.contains_key(name) {
        return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
    }

    // Create collection
    let collection = Collection::new(name.to_string(), config)?;
    self.collections.insert(name.to_string(), Arc::new(collection));

    Ok(())
}

// Add context to errors
pub fn load_collection(&self, name: &str) -> Result<Arc<Collection>, VectorizerError> {
    self.store
        .get_collection(name)
        .context(format!("Failed to load collection '{}'", name))
}
```

## Concurrency and Performance

### Async/Await Patterns

```rust
// Use async/await consistently
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

// Handle cancellation properly
pub async fn process_batch_with_timeout<T>(
    items: Vec<T>,
    timeout_duration: Duration,
) -> Result<Vec<ProcessedItem>, VectorizerError> {
    use tokio::time::{timeout, Duration};

    timeout(timeout_duration, async {
        let mut results = Vec::new();
        for item in items {
            results.push(process_item(item).await?);
        }
        Ok(results)
    })
    .await
    .map_err(|_| VectorizerError::Timeout)?
}
```

### Concurrent Data Structures

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;

// For shared read-heavy data
#[derive(Clone)]
pub struct SharedVectorStore {
    collections: Arc<RwLock<HashMap<String, Arc<Collection>>>>,
}

// For concurrent key-value operations
#[derive(Clone)]
pub struct ConcurrentMetadataStore {
    metadata: Arc<DashMap<String, CollectionMetadata>>,
}
```

## API Design

### Implementation Across All Layers

When adding new functionality, you **MUST** implement it in all three layers:

#### 1. GRPC Layer (Primary)
```rust
// src/grpc/server.rs
pub async fn new_feature(
    &self,
    request: Request<NewFeatureRequest>,
) -> Result<Response<NewFeatureResponse>, Status> {
    // Core business logic here
    // This is the single source of truth
}
```

#### 2. REST API Layer
```rust
// src/api/handlers.rs
pub async fn new_feature_handler(
    State(state): State<AppState>,
    Json(request): Json<NewFeatureRequest>,
) -> Result<Json<NewFeatureResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Proxy to GRPC
    let grpc_response = state.grpc_client.new_feature(request).await?;
    Ok(Json(grpc_response))
}
```

#### 3. MCP Layer
```rust
// src/mcp/service.rs
pub async fn new_feature_tool(
    &self,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, MCPError> {
    // Parse arguments
    let request: NewFeatureRequest = serde_json::from_value(arguments)?;

    // Proxy to GRPC
    let grpc_response = self.grpc_client.new_feature(request).await?;

    // Return MCP-formatted response
    serde_json::to_value(grpc_response)
}
```

### REST API Patterns

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};

// Consistent URL patterns
// GET    /api/v1/collections
// POST   /api/v1/collections
// GET    /api/v1/collections/{name}
// PUT    /api/v1/collections/{name}
// DELETE /api/v1/collections/{name}
// POST   /api/v1/collections/{name}/vectors
// POST   /api/v1/collections/{name}/search

pub async fn list_collections(
    State(state): State<AppState>,
) -> Result<Json<ListCollectionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let collections = state.store.list_collections().await?;
    Ok(Json(ListCollectionsResponse { collections }))
}

pub async fn search_collection(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    if request.query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Query vector cannot be empty".to_string(),
                code: "INVALID_QUERY".to_string(),
                details: None,
            }),
        ));
    }

    // Perform search
    let results = state.store.search(&collection_name, &request.query, request.limit).await?;

    Ok(Json(SearchResponse {
        results,
        query: request.query,
        limit: request.limit,
    }))
}
```

### gRPC Service Patterns

```rust
// .proto definition
service VectorizerService {
    rpc CreateCollection(CreateCollectionRequest) returns (CreateCollectionResponse);
    rpc SearchVectors(SearchRequest) returns (SearchResponse);
    rpc GetCollectionStats(GetCollectionStatsRequest) returns (CollectionStatsResponse);
}

// Implementation
pub async fn create_collection(
    &self,
    request: Request<CreateCollectionRequest>,
) -> Result<Response<CreateCollectionResponse>, Status> {
    let req = request.into_inner();

    tracing::debug!("GRPC CreateCollection request: name={}, dimension={}",
                   req.name, req.dimension);

    // Validate request
    if req.name.is_empty() {
        return Err(Status::invalid_argument("Collection name cannot be empty"));
    }

    // Convert to internal types
    let config = CollectionConfig::from_proto(req.config)?;

    // Create collection
    self.vector_store.create_collection(&req.name, config).await?;

    let response = CreateCollectionResponse {
        name: req.name,
        created_at: Utc::now().timestamp(),
    };

    Ok(Response::new(response))
}
```

## Testing

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
    fn test_search_empty_collection() {
        let store = VectorStore::new();
        let config = CollectionConfig::default();

        store.create_collection("empty", config).unwrap();

        let query = vec![0.1; 512];
        let results = store.search("empty", &query, 5).unwrap();

        assert_eq!(results.len(), 0);
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
use tempfile::tempdir;

#[test]
fn test_full_indexing_workflow() {
    // Setup
    let temp_dir = tempdir().unwrap();
    let store = Arc::new(VectorStore::new());

    // Create test collection
    let config = CollectionConfig {
        dimension: 384,
        ..Default::default()
    };
    store.create_collection("integration_test", config).unwrap();

    // Generate test vectors
    let mut test_vectors = Vec::new();
    for i in 0..100 {
        let data = (0..384)
            .map(|j| (i as f32 * 0.01) + (j as f32 * 0.001))
            .collect::<Vec<f32>>();
        let vector = Vector::new(format!("vec_{}", i), data);
        test_vectors.push(vector);
    }

    // Insert vectors
    store.insert("integration_test", test_vectors).unwrap();

    // Test search
    let query = vec![0.5; 384];
    let results = store.search("integration_test", &query, 10).unwrap();

    assert_eq!(results.len(), 10);
    assert!(results[0].score > 0.0);

    // Verify collection metadata
    let metadata = store.get_collection_metadata("integration_test").unwrap();
    assert_eq!(metadata.vector_count, 100);
    assert_eq!(metadata.config.dimension, 384);
}
```

### Benchmark Tests

```rust
// benches/search_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vectorizer::{VectorStore, CollectionConfig, Vector};

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

## Documentation

### Code Documentation

```rust
//! Vectorizer - High-performance vector database
//!
//! This crate provides fast similarity search and vector operations
//! optimized for AI and machine learning applications.

/// A collection of vectors with associated metadata
///
/// Collections provide the primary interface for storing and searching
/// vectors in Vectorizer. Each collection has a fixed dimensionality
/// and distance metric.
///
/// # Examples
///
/// ```
/// use vectorizer::{VectorStore, CollectionConfig, Vector};
///
/// let store = VectorStore::new();
/// let config = CollectionConfig::default();
///
/// // Create collection
/// store.create_collection("my_collection", config).unwrap();
///
/// // Add vectors
/// let vectors = vec![
///     Vector::new("vec1".to_string(), vec![1.0, 2.0, 3.0]),
///     Vector::new("vec2".to_string(), vec![4.0, 5.0, 6.0]),
/// ];
/// store.insert("my_collection", vectors).unwrap();
///
/// // Search
/// let results = store.search("my_collection", &[1.5, 2.5, 3.5], 5).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Collection {
    // ... fields
}

impl Collection {
    /// Create a new collection with the specified configuration
    ///
    /// # Arguments
    /// * `name` - Unique name for the collection
    /// * `config` - Configuration specifying dimension, metric, etc.
    ///
    /// # Returns
    /// Returns the new collection instance
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use vectorizer::{Collection, CollectionConfig};
    ///
    /// let config = CollectionConfig::default();
    /// let collection = Collection::new("example".to_string(), config)?;
    /// # Ok::<(), vectorizer::VectorizerError>(())
    /// ```
    pub fn new(name: String, config: CollectionConfig) -> Result<Self> {
        // Implementation
    }

    /// Search for similar vectors in the collection
    ///
    /// # Arguments
    /// * `query` - Query vector to search for
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Returns a vector of search results ordered by similarity
    ///
    /// # Performance
    /// This operation uses HNSW indexing for sub-linear search time.
    /// Typical search latency is under 10ms for collections up to 1M vectors.
    pub fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        // Implementation
    }
}
```

### API Documentation

```rust
/// Search vectors in a collection
///
/// Searches for vectors similar to the query vector using the configured
/// distance metric and HNSW index.
///
/// # Request
/// ```json
/// {
///   "query": [0.1, 0.2, 0.3, ...],
///   "limit": 10,
///   "score_threshold": 0.8
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "results": [
///     {
///       "id": "vec_123",
///       "score": 0.95,
///       "payload": {"metadata": "value"}
///     }
///   ],
///   "total_found": 1
/// }
/// ```
///
/// # Errors
/// - `400 Bad Request`: Invalid query vector or parameters
/// - `404 Not Found`: Collection doesn't exist
/// - `500 Internal Server Error`: Search operation failed
#[utoipa::path(
    post,
    path = "/api/v1/collections/{collection_name}/search",
    params(("collection_name" = String, Path, description = "Collection name")),
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search successful", body = SearchResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn search_collection(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Implementation
}
```

## Build and Deployment

### Cargo Configuration

```toml
# Cargo.toml organization
[package]
name = "vectorizer"
version = "0.21.0"
edition = "2024"  # Use latest stable edition
authors = ["Vectorizer Contributors"]
license = "MIT"

[dependencies]
# Core dependencies - group related functionality
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

# Data structures and algorithms
dashmap = "6.1"
hnsw_rs = "0.3"

# Networking and APIs
axum = { version = "0.8", features = ["ws", "json"] }
tonic = { version = "0.11", features = ["tls"] }

# Error handling
anyhow = { version = "1.0", features = ["backtrace"] }
thiserror = "2.0"

# Optional features
candle-core = { version = "0.9.1", optional = true }
gudarc = { version = "0.17", features = ["gpu-12080"], optional = true }

[features]
default = ["gpu_real"]
gpu = []
gpu_real = ["gpu"]
candle-models = ["candle-core", "candle-nn", "candle-transformers"]
onnx-models = ["ort"]
```

### Build Optimization

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization, slower compile
panic = "abort"       # Smaller binaries
strip = true          # Remove debug symbols
```

### Cross-Platform Considerations

```rust
// Platform-specific code
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;

// Conditional compilation for CUDA
#[cfg(feature = "cuda")]
mod cuda_kernels {
    // CUDA-specific code
}

#[cfg(not(feature = "cuda"))]
mod fallback_kernels {
    // CPU fallback implementation
}
```

## Security Considerations

### Input Validation

```rust
pub async fn create_collection(
    Json(request): Json<CreateCollectionRequest>,
) -> Result<Json<CreateCollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate collection name
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

    // Validate dimension
    if request.dimension == 0 || request.dimension > MAX_DIMENSION {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Dimension must be between 1 and {}", MAX_DIMENSION),
                code: "INVALID_DIMENSION".to_string(),
                details: Some(serde_json::json!({
                    "min_dimension": 1,
                    "max_dimension": MAX_DIMENSION,
                    "provided": request.dimension
                })),
            }),
        ));
    }

    // Validate vector data if provided
    if let Some(vectors) = &request.initial_vectors {
        for vector in vectors {
            if vector.data.len() != request.dimension as usize {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!(
                            "Vector dimension mismatch: expected {}, got {}",
                            request.dimension,
                            vector.data.len()
                        ),
                        code: "DIMENSION_MISMATCH".to_string(),
                        details: Some(serde_json::json!({
                            "expected_dimension": request.dimension,
                            "actual_dimension": vector.data.len(),
                            "vector_id": vector.id
                        })),
                    }),
                ));
            }
        }
    }

    // Proceed with creation
    // ...
}
```

### Resource Limits

```rust
// Define constants for resource limits
const MAX_COLLECTION_NAME_LENGTH: usize = 255;
const MAX_DIMENSION: usize = 4096;
const MAX_VECTORS_PER_REQUEST: usize = 10000;
const MAX_SEARCH_LIMIT: usize = 1000;
const MAX_PAYLOAD_SIZE_BYTES: usize = 1024 * 1024; // 1MB

pub fn validate_resource_limits(request: &SearchRequest) -> Result<(), VectorizerError> {
    if request.limit > MAX_SEARCH_LIMIT {
        return Err(VectorizerError::InvalidConfiguration {
            message: format!("Search limit exceeds maximum of {}", MAX_SEARCH_LIMIT),
        });
    }

    // Additional validations...
    Ok(())
}
```

## Performance Guidelines

### Hot Path Optimizations

```rust
// Avoid allocations in hot search paths
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    // Unroll loop for better SIMD utilization
    let len = a.len();
    let mut i = 0;
    while i + 4 <= len {
        dot_product += a[i] * b[i] + a[i + 1] * b[i + 1] + a[i + 2] * b[i + 2] + a[i + 3] * b[i + 3];
        norm_a += a[i] * a[i] + a[i + 1] * a[i + 1] + a[i + 2] * a[i + 2] + a[i + 3] * a[i + 3];
        norm_b += b[i] * b[i] + b[i + 1] * b[i + 1] + b[i + 2] * b[i + 2] + b[i + 3] * b[i + 3];
        i += 4;
    }

    // Handle remaining elements
    for j in i..len {
        dot_product += a[j] * b[j];
        norm_a += a[j] * a[j];
        norm_b += b[j] * b[j];
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}
```

### Memory Pool Management

```rust
// Reuse allocation patterns
pub struct VectorPool {
    pool: Vec<Vec<f32>>,
    dimension: usize,
}

impl VectorPool {
    pub fn new(dimension: usize, initial_capacity: usize) -> Self {
        let mut pool = Vec::with_capacity(initial_capacity);
        for _ in 0..initial_capacity {
            pool.push(Vec::with_capacity(dimension));
        }

        Self { pool, dimension }
    }

    pub fn get_vector(&mut self) -> Vec<f32> {
        self.pool
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.dimension))
    }

    pub fn return_vector(&mut self, mut vector: Vec<f32>) {
        if vector.capacity() >= self.dimension {
            vector.clear();
            self.pool.push(vector);
        }
    }
}
```

## Code Review Checklist

- [ ] **Functionality**: Code does what it claims to do
- [ ] **Documentation**: All public APIs documented with examples
- [ ] **Error Handling**: Proper error propagation and meaningful messages
- [ ] **Performance**: No obvious performance bottlenecks
- [ ] **Security**: Input validation and resource limits
- [ ] **Testing**: Unit tests for new functionality
- [ ] **Style**: Follows established naming and formatting conventions
- [ ] **Concurrency**: Thread-safe where required
- [ ] **Memory**: No memory leaks or excessive allocations
- [ ] **Dependencies**: No unnecessary dependencies added

### ⚠️ Architecture Compliance Checklist

- [ ] **GRPC Implementation**: New features implemented in GRPC server first (`src/grpc/`)
- [ ] **REST API**: Corresponding REST endpoints added (`src/api/`)
- [ ] **MCP Tools**: MCP tools implemented for AI assistants (`src/mcp/`)
- [ ] **Feature Parity**: All three layers (GRPC, REST, MCP) have exactly the same functionality
- [ ] **No Single-Layer Features**: No features exist only in REST or MCP (like `memory-analysis` currently does)
- [ ] **Consistent APIs**: Request/response formats match across all layers
- [ ] **Documentation Sync**: All layers properly documented with same information

## Tools and Automation

### Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run clippy
cargo clippy -- -D warnings

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run security audit
cargo audit
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cargo test --verbose
      - name: Run benchmarks
        run: cargo bench
      - name: Check formatting
        run: cargo fmt --check
      - name: Run clippy
        run: cargo clippy -- -D warnings

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Security audit
        run: cargo audit
```

These guidelines ensure that Vectorizer maintains high code quality, performance, and maintainability. When contributing to the project, always consider these principles and consult existing code for examples of proper implementation patterns.
