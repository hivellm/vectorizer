# Documentation Generation Examples

## Example 1: Documenting a Simple Function

### Before

```rust
pub fn create_collection(name: &str, config: CollectionConfig) -> Result<(), VectorizerError> {
    if store.collections.contains_key(name) {
        return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
    }
    store.collections.insert(name.to_string(), Collection::new(config));
    Ok(())
}
```

### After

```rust
/// Creates a new vector collection with the specified name and configuration.
///
/// This function registers a new collection in the vector store. The collection
/// must have a unique name and valid configuration parameters. Once created,
/// the collection is immediately available for vector operations.
///
/// # Arguments
///
/// * `name` - Unique identifier for the collection. Must be a valid string
///   and must not already exist in the store.
/// * `config` - Configuration specifying vector dimension, distance metric,
///   and indexing parameters. See `CollectionConfig` for details.
///
/// # Returns
///
/// Returns `Ok(())` if the collection was created successfully.
///
/// # Errors
///
/// * `VectorizerError::CollectionAlreadyExists(name)` - If a collection with
///   the specified name already exists in the store.
/// * `VectorizerError::InvalidConfig` - If the configuration contains invalid
///   parameters (e.g., dimension is 0).
/// * `VectorizerError::IoError` - If there was an I/O error during collection
///   creation (e.g., disk write failure).
///
/// # Examples
///
/// ```rust
/// use vectorizer::db::vector_store::VectorStore;
/// use vectorizer::models::{CollectionConfig, DistanceMetric};
///
/// let store = VectorStore::new();
/// let config = CollectionConfig {
///     dimension: 512,
///     metric: DistanceMetric::Cosine,
///     ..Default::default()
/// };
///
/// store.create_collection("my_collection", config)?;
/// ```
pub fn create_collection(name: &str, config: CollectionConfig) -> Result<(), VectorizerError> {
    if store.collections.contains_key(name) {
        return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
    }
    store.collections.insert(name.to_string(), Collection::new(config));
    Ok(())
}
```

## Example 2: Documenting an Async Function

### Before

```rust
pub async fn search_vectors(
    collection: &str,
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchResult>, VectorizerError> {
    let collection = store.get_collection(collection)?;
    let results = collection.search(query, limit).await?;
    Ok(results)
}
```

### After

```rust
/// Searches for similar vectors in the specified collection.
///
/// Performs a similarity search using the collection's configured distance
/// metric. The search is performed asynchronously and returns the top `limit`
/// most similar vectors.
///
/// # Arguments
///
/// * `collection` - Name of the collection to search in. The collection must
///   exist and be accessible.
/// * `query` - Query vector to search for. Must match the collection's
///   dimension exactly.
/// * `limit` - Maximum number of results to return. Must be greater than 0.
///
/// # Returns
///
/// Returns `Ok(Vec<SearchResult>)` containing the search results, ordered by
/// similarity (most similar first). Each result includes the vector ID, score,
/// and optional payload.
///
/// # Errors
///
/// * `VectorizerError::CollectionNotFound(name)` - If the specified collection
///   does not exist.
/// * `VectorizerError::DimensionMismatch { expected, actual }` - If the query
///   vector dimension does not match the collection's dimension.
/// * `VectorizerError::IndexError` - If there was an error accessing the
///   vector index.
///
/// # Examples
///
/// ```rust
/// use vectorizer::db::vector_store::VectorStore;
///
/// let store = VectorStore::new();
/// let query_vector = vec![0.1, 0.2, 0.3, /* ... */];
///
/// let results = store.search_vectors("my_collection", &query_vector, 10).await?;
/// for result in results {
///     println!("ID: {}, Score: {}", result.id, result.score);
/// }
/// ```
///
/// # Performance
///
/// Search performance depends on the collection's index configuration. For
/// HNSW indexes, typical search time is sub-3ms for collections with millions
/// of vectors.
pub async fn search_vectors(
    collection: &str,
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchResult>, VectorizerError> {
    let collection = store.get_collection(collection)?;
    let results = collection.search(query, limit).await?;
    Ok(results)
}
```

## Example 3: Documenting a Struct

### Before

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionConfig {
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub hnsw_config: HnswConfig,
}
```

### After

```rust
/// Configuration for a vector collection.
///
/// Specifies the properties and behavior of a vector collection, including
/// vector dimensions, distance metrics, and indexing parameters. This
/// configuration is used when creating a new collection and cannot be
/// changed after creation.
///
/// # Fields
///
/// * `dimension` - The dimensionality of vectors in this collection. Must be
///   greater than 0. Common values are 128, 256, 512, or 1536 depending on
///   the embedding model used.
/// * `metric` - The distance metric used for similarity calculations. See
///   `DistanceMetric` for available options.
/// * `hnsw_config` - Configuration for the HNSW (Hierarchical Navigable Small
///   World) index. Controls index quality vs. memory trade-offs.
///
/// # Examples
///
/// ```rust
/// use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig};
///
/// let config = CollectionConfig {
///     dimension: 512,
///     metric: DistanceMetric::Cosine,
///     hnsw_config: HnswConfig::default(),
/// };
/// ```
///
/// # Default Values
///
/// Use `CollectionConfig::default()` for sensible defaults:
/// - Dimension: 512
/// - Metric: Cosine
/// - HNSW: Balanced quality/memory configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionConfig {
    /// Vector dimension (must be > 0)
    pub dimension: usize,
    /// Distance metric for similarity calculations
    pub metric: DistanceMetric,
    /// HNSW index configuration
    pub hnsw_config: HnswConfig,
}
```

## Example 4: Documenting an Enum

### Before

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}
```

### After

```rust
/// Distance metric used for vector similarity calculations.
///
/// Different metrics are suitable for different use cases:
/// - **Cosine**: Best for normalized vectors, measures angle between vectors
/// - **Euclidean**: Measures straight-line distance, good for general use
/// - **DotProduct**: Fast but requires normalized vectors for meaningful results
///
/// # Variants
///
/// * `Cosine` - Cosine similarity. Computes the cosine of the angle between
///   two vectors. Values range from -1 to 1, where 1 means identical
///   direction. Best for text embeddings and normalized vectors.
/// * `Euclidean` - Euclidean distance (L2 norm). Computes the straight-line
///   distance between two vectors. Lower values indicate greater similarity.
///   Suitable for general-purpose vector search.
/// * `DotProduct` - Dot product of two vectors. Fast computation but requires
///   normalized vectors for meaningful similarity scores. Higher values
///   indicate greater similarity.
///
/// # Examples
///
/// ```rust
/// use vectorizer::models::DistanceMetric;
///
/// let metric = DistanceMetric::Cosine;
/// ```
///
/// # Serialization
///
/// Serializes to lowercase strings: `"cosine"`, `"euclidean"`, `"dotproduct"`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    /// Cosine similarity metric
    Cosine,
    /// Euclidean distance metric
    Euclidean,
    /// Dot product metric
    DotProduct,
}
```

## Example 5: Documenting a Module

### Before

```rust
// src/db/vector_store.rs
use std::collections::HashMap;

pub struct VectorStore {
    collections: HashMap<String, Collection>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }
}
```

### After

```rust
//! # Vector Store
//!
//! Core data structure for managing vector collections and performing
//! similarity searches.
//!
//! ## Overview
//!
//! The `VectorStore` is the primary interface for working with vector
//! collections. It provides functionality for:
//!
//! - Creating and managing collections
//! - Adding vectors to collections
//! - Performing similarity searches
//! - Managing collection metadata
//!
//! ## Key Components
//!
//! - **VectorStore**: Main store instance managing all collections
//! - **Collection**: Individual vector collection with its own configuration
//! - **SearchResult**: Results from similarity searches
//!
//! ## Usage
//!
//! ```rust
//! use vectorizer::db::vector_store::VectorStore;
//! use vectorizer::models::CollectionConfig;
//!
//! let store = VectorStore::new();
//! let config = CollectionConfig::default();
//!
//! store.create_collection("my_collection", config)?;
//! ```
//!
//! ## Thread Safety
//!
//! `VectorStore` uses internal synchronization and is safe to share across
//! threads using `Arc<VectorStore>`.
//!
//! ## Examples
//!
//! ### Creating a Collection
//!
//! ```rust
//! use vectorizer::db::vector_store::VectorStore;
//!
//! let store = VectorStore::new();
//! let config = CollectionConfig::default();
//! store.create_collection("docs", config)?;
//! ```
//!
//! ### Adding Vectors
//!
//! ```rust
//! let vector = vec![0.1, 0.2, 0.3, /* ... */];
//! store.add_vector("docs", "doc1", &vector, None)?;
//! ```
//!
//! ### Searching
//!
//! ```rust
//! let query = vec![0.1, 0.2, 0.3, /* ... */];
//! let results = store.search_vectors("docs", &query, 10).await?;
//! ```

use std::collections::HashMap;

pub struct VectorStore {
    collections: HashMap<String, Collection>,
}

impl VectorStore {
    /// Creates a new empty vector store.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vectorizer::db::vector_store::VectorStore;
    ///
    /// let store = VectorStore::new();
    /// ```
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }
}
```

## Example 6: Documenting a Trait

### Before

```rust
pub trait EmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn dimension(&self) -> usize;
}
```

### After

```rust
/// Trait for embedding providers that convert text to vectors.
///
/// Implementations of this trait provide functionality to generate vector
/// embeddings from text. Different providers may use different models,
/// dimensions, and processing strategies.
///
/// # Required Methods
///
/// * `embed` - Generates a vector embedding for the given text
/// * `dimension` - Returns the dimensionality of embeddings produced by this
///   provider
///
/// # Examples
///
/// ```rust
/// use vectorizer::embedding::EmbeddingProvider;
///
/// struct MyProvider;
///
/// impl EmbeddingProvider for MyProvider {
///     fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
///         // Implementation
///     }
///
///     fn dimension(&self) -> usize {
///         512
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// Implementations should be thread-safe and safe to share across threads
/// using `Arc<dyn EmbeddingProvider>`.
pub trait EmbeddingProvider: Send + Sync {
    /// Generates a vector embedding for the given text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to embed. Can be of any length, though very long
    ///   texts may be truncated depending on the model's context window.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<f32>)` containing the embedding vector with
    /// dimensionality matching `dimension()`.
    ///
    /// # Errors
    ///
    /// * `EmbeddingError::ModelError` - If there was an error with the
    ///   embedding model
    /// * `EmbeddingError::InvalidInput` - If the input text is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// let provider = MyProvider;
    /// let embedding = provider.embed("Hello, world!")?;
    /// assert_eq!(embedding.len(), provider.dimension());
    /// ```
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    
    /// Returns the dimensionality of embeddings produced by this provider.
    ///
    /// # Returns
    ///
    /// The dimension of vectors returned by `embed()`. This value is constant
    /// for a given provider instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let provider = MyProvider;
    /// assert_eq!(provider.dimension(), 512);
    /// ```
    fn dimension(&self) -> usize;
}
```

## Example 7: Complete Module Documentation

### Before

```rust
// src/api/endpoints.rs
use axum::{extract::Path, response::Json};

pub async fn get_collection(Path(name): Path<String>) -> Json<CollectionInfo> {
    // Implementation
}

pub async fn create_collection(Json(request): Json<CreateRequest>) -> Json<CreateResponse> {
    // Implementation
}
```

### After

```rust
//! # API Endpoints
//!
//! REST API endpoints for the Vectorizer service.
//!
//! ## Overview
//!
//! This module provides HTTP handlers for the Vectorizer REST API. All
//! endpoints follow RESTful conventions and return JSON responses. The API
//! supports operations for:
//!
//! - Collection management (create, read, update, delete)
//! - Vector operations (add, search, delete)
//! - Health checks and status
//!
//! ## Endpoint Structure
//!
//! All endpoints are prefixed with `/api/v1/`. The API follows RESTful
//! conventions:
//!
//! - `GET /api/v1/collections/{name}` - Get collection info
//! - `POST /api/v1/collections` - Create collection
//! - `POST /api/v1/collections/{name}/vectors` - Add vector
//! - `POST /api/v1/collections/{name}/search` - Search vectors
//!
//! ## Error Handling
//!
//! All endpoints return appropriate HTTP status codes:
//!
//! - `200 OK` - Success
//! - `400 Bad Request` - Invalid input
//! - `404 Not Found` - Resource not found
//! - `500 Internal Server Error` - Server error
//!
//! ## Examples
//!
//! ### Get Collection
//!
//! ```bash
//! curl http://localhost:15002/api/v1/collections/my_collection
//! ```
//!
//! ### Create Collection
//!
//! ```bash
//! curl -X POST http://localhost:15002/api/v1/collections \
//!   -H "Content-Type: application/json" \
//!   -d '{"name": "my_collection", "config": {...}}'
//! ```

use axum::{extract::Path, response::Json};

/// Retrieves information about a collection.
///
/// Returns metadata about the specified collection including its configuration,
/// vector count, and index statistics.
///
/// # Path Parameters
///
/// * `name` - Name of the collection to retrieve
///
/// # Returns
///
/// Returns `200 OK` with collection information, or `404 Not Found` if the
/// collection doesn't exist.
///
/// # Examples
///
/// ```bash
/// curl http://localhost:15002/api/v1/collections/my_collection
/// ```
pub async fn get_collection(Path(name): Path<String>) -> Json<CollectionInfo> {
    // Implementation
}

/// Creates a new vector collection.
///
/// Registers a new collection with the specified name and configuration. The
/// collection name must be unique.
///
/// # Request Body
///
/// * `name` - Unique collection name (required)
/// * `config` - Collection configuration (required)
///
/// # Returns
///
/// Returns `201 Created` with collection information, or `400 Bad Request` if
/// the request is invalid, or `409 Conflict` if the collection already exists.
///
/// # Examples
///
/// ```bash
/// curl -X POST http://localhost:15002/api/v1/collections \
///   -H "Content-Type: application/json" \
///   -d '{"name": "my_collection", "config": {...}}'
/// ```
pub async fn create_collection(Json(request): Json<CreateRequest>) -> Json<CreateResponse> {
    // Implementation
}
```

## Workflow Example: Documenting Entire Module

1. **Read the module file** using `get_file_content` or `read_file`
2. **Identify all public items** (functions, structs, enums, traits)
3. **Check existing documentation** - what's missing?
4. **Generate module-level docs** (`//!`) if missing
5. **Document each public item** following templates
6. **Add examples** for complex functions
7. **Verify** with `cargo doc --no-deps`

## Common Patterns

### Documenting Error Conditions

```rust
/// # Errors
///
/// * `VectorizerError::CollectionNotFound(name)` - If the collection `name`
///   does not exist in the store
/// * `VectorizerError::DimensionMismatch { expected: 512, actual: 256 }` -
///   If the vector dimension doesn't match the collection's dimension
/// * `VectorizerError::IoError(err)` - If there was an I/O error, with the
///   underlying error details
```

### Documenting Performance

```rust
/// # Performance
///
/// This function performs O(n) operations where n is the number of vectors
/// in the collection. For large collections, consider using the async
/// version `search_vectors_async` which uses parallel processing.
```

### Documenting Thread Safety

```rust
/// # Thread Safety
///
/// This function is thread-safe and can be called concurrently from multiple
/// threads. Internal synchronization ensures data consistency.
```

### Documenting Panics

```rust
/// # Panics
///
/// Panics if `limit` is 0, as this would result in an empty result set with
/// no meaningful behavior.
```

## Example 8: Generating Markdown Documentation

### Code Documentation (Source)

```rust
//! # Vector Store
//!
//! Core data structure for managing vector collections and performing
//! similarity searches.

/// Creates a new vector collection with the specified name and configuration.
///
/// # Arguments
///
/// * `name` - Unique identifier for the collection
/// * `config` - Configuration specifying dimension, metric, and index settings
///
/// # Returns
///
/// Returns `Ok(())` if the collection was created successfully.
///
/// # Errors
///
/// * `VectorizerError::CollectionAlreadyExists` - If collection already exists
/// * `VectorizerError::InvalidConfig` - If configuration is invalid
pub fn create_collection(name: &str, config: CollectionConfig) -> Result<(), VectorizerError> {
    // Implementation
}
```

### Generated Markdown Documentation

**File:** `docs/modules/vector_store.md`

```markdown
# Vector Store

## Overview

Core data structure for managing vector collections and performing similarity searches.

## Purpose

The `VectorStore` is the primary interface for working with vector collections. It provides functionality for creating and managing collections, adding vectors, performing similarity searches, and managing collection metadata.

## Key Components

- **VectorStore**: Main store instance managing all collections
- **Collection**: Individual vector collection with its own configuration
- **SearchResult**: Results from similarity searches

## Functions

### `create_collection`

Creates a new vector collection with the specified name and configuration.

**Signature:**
```rust
pub fn create_collection(
    name: &str,
    config: CollectionConfig
) -> Result<(), VectorizerError>
```

**Parameters:**
- `name` - `&str` - Unique identifier for the collection. Must be a valid string and must not already exist in the store.
- `config` - `CollectionConfig` - Configuration specifying vector dimension, distance metric, and indexing parameters.

**Returns:**
Returns `Ok(())` if the collection was created successfully.

**Errors:**
- `VectorizerError::CollectionAlreadyExists(name)` - If a collection with the specified name already exists in the store.
- `VectorizerError::InvalidConfig` - If the configuration contains invalid parameters (e.g., dimension is 0).
- `VectorizerError::IoError` - If there was an I/O error during collection creation.

**Example:**
```rust
use vectorizer::db::vector_store::VectorStore;
use vectorizer::models::CollectionConfig;

let store = VectorStore::new();
let config = CollectionConfig::default();

store.create_collection("my_collection", config)?;
```

## Types

### `CollectionConfig`

Configuration for a vector collection.

**Fields:**
- `dimension` - `usize` - The dimensionality of vectors in this collection (must be > 0)
- `metric` - `DistanceMetric` - The distance metric used for similarity calculations
- `hnsw_config` - `HnswConfig` - Configuration for the HNSW index

## Usage Examples

### Creating a Collection

```rust
use vectorizer::db::vector_store::VectorStore;
use vectorizer::models::CollectionConfig;

let store = VectorStore::new();
let config = CollectionConfig {
    dimension: 512,
    metric: DistanceMetric::Cosine,
    ..Default::default()
};

store.create_collection("my_collection", config)?;
```

### Adding Vectors

```rust
let vector = vec![0.1, 0.2, 0.3, /* ... */];
store.add_vector("my_collection", "doc1", &vector, None)?;
```

### Searching Vectors

```rust
let query = vec![0.1, 0.2, 0.3, /* ... */];
let results = store.search_vectors("my_collection", &query, 10).await?;
```

## Thread Safety

`VectorStore` uses internal synchronization and is safe to share across threads using `Arc<VectorStore>`.

## See Also

- [API Reference](../api/README.md)
- [Collection Configuration](../reference/collection_config.md)
- [Distance Metrics](../reference/distance_metrics.md)
```

## Example 9: Module Documentation in Markdown

### Generated from Module Documentation

**Source Code:**
```rust
//! # API Endpoints
//!
//! REST API endpoints for the Vectorizer service.
//!
//! ## Overview
//!
//! This module provides HTTP handlers for the Vectorizer REST API.
```

**Generated Markdown:** `docs/modules/api_endpoints.md`

```markdown
# API Endpoints

## Overview

REST API endpoints for the Vectorizer service.

## Purpose

This module provides HTTP handlers for the Vectorizer REST API. All endpoints follow RESTful conventions and return JSON responses.

## Endpoint Structure

All endpoints are prefixed with `/api/v1/`. The API follows RESTful conventions:

- `GET /api/v1/collections/{name}` - Get collection info
- `POST /api/v1/collections` - Create collection
- `POST /api/v1/collections/{name}/vectors` - Add vector
- `POST /api/v1/collections/{name}/search` - Search vectors

## Functions

### `get_collection`

Retrieves information about a collection.

**Signature:**
```rust
pub async fn get_collection(
    Path(name): Path<String>
) -> Json<CollectionInfo>
```

**Path Parameters:**
- `name` - `String` - Name of the collection to retrieve

**Returns:**
Returns `200 OK` with collection information, or `404 Not Found` if the collection doesn't exist.

**Example Request:**
```bash
curl http://localhost:15002/api/v1/collections/my_collection
```

### `create_collection`

Creates a new vector collection.

**Signature:**
```rust
pub async fn create_collection(
    Json(request): Json<CreateRequest>
) -> Json<CreateResponse>
```

**Request Body:**
- `name` - `String` - Unique collection name (required)
- `config` - `CollectionConfig` - Collection configuration (required)

**Returns:**
Returns `201 Created` with collection information, or `400 Bad Request` if the request is invalid, or `409 Conflict` if the collection already exists.

**Example Request:**
```bash
curl -X POST http://localhost:15002/api/v1/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "my_collection", "config": {...}}'
```

## Error Handling

All endpoints return appropriate HTTP status codes:

- `200 OK` - Success
- `400 Bad Request` - Invalid input
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

## See Also

- [API Reference](../api/README.md)
- [Collection Management](../modules/vector_store.md)
```

## Example 10: Complete Documentation Workflow

### Step-by-Step Process

1. **Read source code** from `src/db/vector_store.rs`
2. **Generate code documentation** (add `//!` and `///` comments)
3. **Generate Markdown documentation** in `docs/modules/vector_store.md`
4. **Create cross-references** to related modules
5. **Update index** in `docs/modules/README.md`

### Generated Files

**Code:** `src/db/vector_store.rs` (with documentation comments)
**Markdown:** `docs/modules/vector_store.md` (standalone documentation)
**Index:** `docs/modules/README.md` (navigation)

### Markdown Index Example

```markdown
# Module Documentation

## Core Modules

- [Vector Store](vector_store.md) - Core data structure for managing collections
- [API Endpoints](api_endpoints.md) - REST API handlers
- [Embedding Providers](embedding_providers.md) - Embedding generation

## Reference

- [Collection Configuration](../reference/collection_config.md)
- [Distance Metrics](../reference/distance_metrics.md)
- [Error Types](../reference/error_types.md)
```
