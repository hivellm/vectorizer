# Vector System Architecture

## Architecture Overview

The Vectorizer is designed as an in-memory vector database optimized for AI applications, with a modular architecture that allows for scalability and extensibility.

```
┌─────────────────────────────────────────────────────────────┐
│                    Vectorizer System                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   Python    │ │ TypeScript  │ │    CLI      │           │
│  │    SDK      │ │    SDK      │ │   Tools     │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │              REST/gRPC API Layer                    │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │   │
│  │  │   Routes    │ │ Middleware  │ │   Auth      │    │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘    │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Core Engine (Rust)                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │   │
│  │  │ VectorStore │ │  HNSW      │ │ Persistence │    │   │
│  │  │             │ │  Index     │ │             │    │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘    │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                     Persistence Layer                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   Binary    │ │    WAL      │ │ Snapshots   │           │
│  │   Files     │ │   Logs      │ │             │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

## Main Components

### 1. Core Engine (Rust)

#### VectorStore
The central component that manages vector collections and coordinates operations.

**Responsibilities:**
- Collection lifecycle management
- Coordination between indexing and persistence
- Concurrency control for multi-threaded operations
- Input validation and error handling

**Internal Structure:**
```rust
pub struct VectorStore {
    collections: DashMap<String, Arc<Collection>>,
    embedding_engine: Arc<EmbeddingEngine>,
    persistence: Arc<PersistenceManager>,
    metrics: Arc<MetricsCollector>,
}
```

#### Embedding Engine
Component responsible for generating embeddings from text.

**Features:**
- Native embedding models (BOW, Hash, N-gram)
- Smart embedding cache
- Optimized batch processing
- Quantization support for memory reduction

**Structure:**
```rust
pub enum EmbeddingModel {
    Bow { vocab_size: usize, dimension: usize },           // Bag-of-Words with TF-IDF
    Hash { hash_size: usize, dimension: usize },          // Feature hashing
    Ngram { ngram_range: (usize, usize), vocab_size: usize, dimension: usize }, // N-gram features
}

pub enum QuantizationType {
    None,
    PQ { n_centroids: usize, n_subquantizers: usize },    // Product Quantization
    SQ { bits: usize },                                   // Scalar Quantization
    Binary,                                                // Extreme compression
}

pub struct EmbeddingEngine {
    model: EmbeddingModel,
    quantizer: QuantizationType,
    cache: EmbeddingCache,
    metrics: Arc<EmbeddingMetrics>,
}

impl EmbeddingEngine {
    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    pub fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    pub fn quantize_vectors(&self, vectors: &[Vec<f32>]) -> Result<Vec<Vec<f32>>, QuantizationError>;
}
```

#### Text Chunker
Component for splitting long texts into chunks appropriate for embedding.

**Chunking Strategies:**
- **Sentence-based**: Split by sentences, preserves meaning
- **Token-based**: Split by tokens, precise size control
- **Paragraph-based**: Split by paragraphs, maintains structure
- **Hybrid**: Intelligent combination of strategies

**Structure:**
```rust
pub enum ChunkStrategy {
    Sentence { max_sentences: usize },
    Token { max_tokens: usize },
    Paragraph { max_paragraphs: usize },
    Hybrid { primary: Box<ChunkStrategy>, fallback: Box<ChunkStrategy> },
}

pub struct TextChunker {
    strategy: ChunkStrategy,
    overlap: usize,  // Tokens de overlap entre chunks
    language: String,
}

impl TextChunker {
    pub fn chunk_text(&self, text: &str) -> Vec<String> {
        // Implementação da estratégia de chunking
    }
}
```

#### Vector Quantization
Component for compressing vectors to reduce memory usage while maintaining search quality.

**Quantization Strategies:**
- **Product Quantization (PQ)**: Splits vectors into sub-vectors, quantizes each separately
- **Scalar Quantization (SQ)**: Reduces precision of floating-point values
- **Binary Quantization**: Extreme compression using binary codes (32x reduction)

**Memory Reduction Examples:**
- PQ (256 centroids, 8 subquantizers): ~75% memory reduction
- SQ (8-bit): ~50% memory reduction
- Binary: ~97% memory reduction

**Structure:**
```rust
pub trait Quantizer {
    fn quantize(&self, vector: &[f32]) -> Result<Vec<f32>, QuantizationError>;
    fn decompress(&self, quantized: &[f32]) -> Result<Vec<f32>, QuantizationError>;
    fn memory_reduction_ratio(&self) -> f32;
}

pub struct PQQuantizer {
    centroids: Vec<Vec<f32>>,
    n_subquantizers: usize,
}

impl Quantizer for PQQuantizer {
    // Implementation for product quantization
}
```

#### HNSW Index
Implementation of the Hierarchical Navigable Small World algorithm for approximate nearest neighbor search.

**Features:**
- **Hierarchical**: Multiple layers with decreasing granularity
- **Navigable**: Optimized connections for efficient exploration
- **Small World**: Property that allows efficient graph jumps

**Configuration Parameters:**
```rust
pub struct HNSWConfig {
    pub max_layers: usize,        // Maximum hierarchical layers
    pub m: usize,                // Maximum connections per node
    pub m_max: usize,           // Maximum connections in layer 0
    pub ef_construction: usize,  // Candidate list size during construction
    pub ef_search: usize,       // Dynamic list size during search
}
```

#### Persistence Manager
Manages durable persistence of data in binary format.

**Strategies:**
- **Full Snapshot**: Complete image of current state
- **Incremental WAL**: Write-Ahead Logging for operations
- **Hybrid**: Combination of snapshots + logs for fast recovery

### 2. API Layer

#### REST API (Axum-based)
HTTP interface for database operations.

**Main Endpoints:**
```
POST   /collections/{name}/vectors     # Insert vectors
GET    /collections/{name}/search      # Semantic search
DELETE /collections/{name}/vectors     # Delete vectors
GET    /collections/{name}/stats       # Collection statistics
POST   /collections/{name}/batch       # Batch operations
```

#### Protocol Buffers (gRPC)
High-performance binary interface for inter-system communication.

**Defined Services:**
```protobuf
service VectorService {
  rpc InsertVectors(InsertRequest) returns (InsertResponse);
  rpc SearchVectors(SearchRequest) returns (SearchResponse);
  rpc DeleteVectors(DeleteRequest) returns (DeleteResponse);
  rpc GetCollectionStats(StatsRequest) returns (StatsResponse);
}

message InsertRequest {
  string collection = 1;
  repeated Vector vectors = 2;
  repeated Payload payloads = 3;
  map<string, string> metadata = 4;
}
```

### 3. Language Bindings

#### Python SDK (PyO3)
Native bindings that provide high-level Python interface.

**Features:**
- **Zero-copy**: When possible, avoids unnecessary data copying
- **Async Support**: Integration with asyncio for non-blocking operations
- **Type Hints**: Complete annotations for better development experience

#### TypeScript SDK (Neon)
Node.js bindings focused on web and server-side applications.

**Features:**
- **Promise-based**: Promise-based asynchronous API
- **Browser Compatible**: Works in Node.js and browser environments (via bundlers)
- **Type Safe**: Complete TypeScript definitions

### 4. CLI Tools

#### File Ingestion
Command-line tool for processing and indexing documents.

**Operation Flow:**
1. **Chunking**: Split text into smaller chunks
2. **Embedding**: Generate vectors using configurable models
3. **Indexing**: Insert into HNSW index
4. **Persistence**: Save to binary file

**Available Commands:**
```bash
vectorizer ingest --file document.txt --collection docs --chunk-size 512
vectorizer query --text "search query" --collection docs --k 5
vectorizer stats --collection docs
vectorizer export --collection docs --format json
```

## Data Models

### Core Structures

#### Vector
Fundamental representation of a high-dimensional vector.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub data: Vec<f32>,
    pub dimension: usize,
    pub norm: Option<f32>,  // Cached L2 norm for performance
}
```

#### Collection
Logical grouping of related vectors.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub config: CollectionConfig,
    pub vectors: Vec<Vector>,
    pub index: HNSWIndex,
    pub metadata: CollectionMetadata,
}
```

#### Payload
Data associated with each vector (metadata, original text, etc.).

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payload {
    pub id: String,
    pub data: serde_json::Value,  // Flexible JSON payload
    pub timestamp: i64,
}
```

### Search Operations

#### Similarity Search
```rust
pub struct SearchRequest {
    pub collection: String,
    pub query: QueryType,
    pub k: usize,
    pub filter: Option<SearchFilter>,
}

pub enum QueryType {
    Vector(Vec<f32>),
    Text(String),  // Requires embedding function
}
```

#### Search Results
```rust
pub struct SearchResult {
    pub vector_id: String,
    pub score: f32,        // Similarity score (cosine, dot product, etc.)
    pub payload: Payload,
    pub vector: Option<Vec<f32>>,  // Optional vector return
}
```

## Indexing Strategies

### Incremental Construction
The HNSW index is built incrementally as vectors are added.

**Algorithm:**
1. **Hierarchical Insertion**: Starting from the top layer
2. **Neighbor Search**: Find best connections in each layer
3. **Local Optimization**: Refine connections to maximize quality

### Performance Optimizations

#### SIMD Operations
Use of SIMD instructions for vector operations.

```rust
#[cfg(target_arch = "x86_64")]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    // SIMD implementation for AVX2/AVX-512
}
```

#### Memory Pooling
Efficient memory management to reduce allocations.

```rust
pub struct MemoryPool {
    vectors: Vec<Vec<f32>>,
    payloads: Vec<Payload>,
    free_indices: Vec<usize>,
}
```

## Concurrency and Parallelism

### Concurrency Model
- **Readers-Writer Lock**: Multiple simultaneous reads, exclusive write
- **Async Operations**: I/O operations don't block threads
- **Work Stealing**: Balanced workload distribution in batch operations

### Thread Safety
```rust
pub struct ThreadSafeVectorStore {
    collections: Arc<DashMap<String, Arc<RwLock<Collection>>>>,
}
```

## Persistence Strategies

### Binary Format (Bincode)
Efficient serialization specifically developed for Rust.

**Advantages:**
- **Zero-copy**: When possible, data is mapped directly
- **Compact**: Optimized binary format
- **Type-safe**: Preserves type information

### Write-Ahead Logging
Durability guarantee for critical operations.

**Log Structure:**
```rust
#[derive(Serialize, Deserialize)]
pub enum WALEntry {
    Insert { collection: String, vectors: Vec<Vector> },
    Delete { collection: String, ids: Vec<String> },
    Update { collection: String, updates: Vec<VectorUpdate> },
}
```

## Monitoring and Observability

### Collected Metrics
- **Performance**: Operation latency, throughput
- **Resource Usage**: Memory, CPU, I/O
- **Quality**: Search accuracy, index size
- **Health**: Collection status, errors

### Structured Logging
```rust
#[derive(Serialize)]
pub struct OperationLog {
    pub timestamp: i64,
    pub operation: String,
    pub collection: String,
    pub duration_ms: f64,
    pub success: bool,
    pub error: Option<String>,
}
```

## Scalability Considerations

### Limits and Constraints
- **Memory**: Limited by available RAM
- **Dimensionality**: Quadratic impact on some operations
- **Connectivity**: Number of HNSW connections per node

### Scalability Strategies
1. **Sharding**: Distribution of collections across multiple instances
2. **Quantization**: Precision reduction for smaller footprint
3. **Approximate Search**: Quality vs performance trade-off
4. **Caching**: Cache of frequent results

## Security & Authentication

### API Key Management
The server implements mandatory API key authentication for all operations:

- **API Key Storage**: Securely stored in encrypted database
- **Key Generation**: Cryptographically secure random keys
- **Key Validation**: Fast lookup with caching
- **Key Revocation**: Immediate invalidation of compromised keys

### Dashboard Security
- **Localhost Only**: Dashboard accessible only from 127.0.0.1
- **No Authentication**: No login required (localhost restriction provides security)
- **Read-Only Operations**: Dashboard can only view and manage API keys, not data

### Network Security
- **Internal Mode**: Server binds to 127.0.0.1, accessible only locally
- **Cloud Mode**: Server binds to 0.0.0.0 with configurable CORS and rate limiting
- **TLS Support**: Optional TLS encryption for cloud deployments
- **Rate Limiting**: Configurable request limits per API key

### Payload Compression
Vectorizer implements automatic payload compression for large payloads to optimize storage and network performance:

- **Threshold-Based Compression**: Payloads larger than configurable threshold are automatically compressed
- **Fast Compression Algorithms**: Uses LZ4 for high-speed compression/decompression
- **Transparent Operation**: Compression is transparent to API users - payloads are decompressed automatically
- **Configurable Thresholds**: Collection-level configuration for compression thresholds
- **Performance Optimization**: Reduces memory usage and network bandwidth for large payloads

### Input Validation
- **Sanitization**: JSON payloads are validated and sanitized
- **Limits**: Maximum size of vectors, collections, and payloads
- **Type Checking**: Runtime type validation for all inputs
- **SQL Injection Prevention**: Parameterized queries for metadata filtering

### Audit Logging
- **Operation Logs**: All API operations logged with timestamps
- **API Key Tracking**: Which key performed which operation
- **Error Logging**: Failed authentication attempts and errors
- **Performance Metrics**: Response times and resource usage per API key

---

This architecture provides enterprise-grade security while maintaining the performance benefits of native vector processing. The modularity allows for incremental iterations and future extensions with proper authentication and authorization built-in from the ground up.
