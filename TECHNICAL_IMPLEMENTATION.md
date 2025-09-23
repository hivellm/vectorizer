# Vectorizer - Technical Implementation Documentation

## Architecture Overview

### Current Project Status
**Status**: Conceptual/Complete Documentation, Implementation Pending

The Vectorizer project is specified in detail in `README.md`, but currently has no code implementation. The conceptual documentation is complete and detailed, defining:

- **Objective**: In-memory vector database for semantic search
- **Technology Stack**: Rust with bindings for Python and TypeScript
- **Use Cases**: Collaborative LLM systems, RAG, knowledge bases

### Dependencies and Integrations Identified

#### 1. UMICP Integration (Existing)
The UMICP project (`../umicp/`) provides fundamental infrastructure that can be leveraged:

**Relevant Resources Found:**
- Efficient embedding communication between AI models
- Optimized binary serialization for vectors and matrices
- Support for BERT, GPT, T5 embeddings
- Similarity operations (cosine similarity)
- Federated embedding aggregation

**Examples Analyzed:**
- `umicp/bindings/rust/examples/embedding_communication.rs`
- `umicp/bindings/typescript/examples/embedding-communication.ts`

## Proposed Architecture

### 1. Core Engine (Rust)

#### Directory Structure
```
vectorizer/src/
├── db/                    # Database engine
│   ├── mod.rs
│   ├── vector_store.rs    # In-memory vector storage
│   ├── hnsw_index.rs      # HNSW implementation
│   └── persistence.rs     # Binary serialization
├── api/                   # REST/gRPC API
│   ├── mod.rs
│   ├── handlers.rs
│   ├── routes.rs
│   └── middleware.rs
├── cli/                   # Command line interface
│   ├── mod.rs
│   ├── commands.rs
│   └── ingestion.rs
├── models/                # Data structures
│   ├── mod.rs
│   ├── vector.rs
│   ├── collection.rs
│   ├── metadata.rs
│   └── query.rs
├── embedding/            # Embedding providers
│   ├── mod.rs
│   ├── providers.rs      # SentenceTransformers, OpenAI, etc.
│   ├── cache.rs          # Embedding cache
│   └── engine.rs         # Embedding orchestration
└── text/                 # Text processing
    ├── mod.rs
    ├── chunker.rs        # Text chunking strategies
    └── tokenizer.rs      # Text tokenization
└── lib.rs
```

#### Main Components

##### VectorStore
```rust
pub struct VectorStore {
    collections: HashMap<String, Collection>,
    index: HNSWIndex,
    persistence: PersistenceManager,
}

impl VectorStore {
    pub fn new() -> Self;
    pub fn create_collection(&mut self, name: &str, config: CollectionConfig) -> Result<(), Error>;
    pub fn insert(&mut self, collection: &str, vectors: &[Vector], payloads: &[Payload]) -> Result<(), Error>;
    pub fn search(&self, collection: &str, query: &Vector, k: usize) -> Result<Vec<SearchResult>, Error>;
    pub fn persist(&self, path: &str) -> Result<(), Error>;
    pub fn load(&mut self, path: &str) -> Result<(), Error>;
}
```

##### Embedding Engine
```rust
pub struct EmbeddingEngine {
    providers: HashMap<String, Box<dyn EmbeddingProvider>>,
    cache: EmbeddingCache,
    metrics: EmbeddingMetrics,
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    fn dimension(&self) -> usize;
    fn name(&self) -> &str;
}

pub struct BowProvider {
    vocab: HashMap<String, usize>,
    idf_weights: Vec<f32>,
    dimension: usize,
    vocab_size: usize,
}

pub struct HashProvider {
    hash_size: usize,
    dimension: usize,
}

pub struct NgramProvider {
    ngram_range: (usize, usize),
    vocab: HashMap<String, usize>,
    dimension: usize,
}
```

##### Text Chunker
```rust
pub enum ChunkStrategy {
    Sentence { max_sentences: usize },
    Token { max_tokens: usize },
    Paragraph { max_paragraphs: usize },
}

pub struct TextChunker {
    strategy: ChunkStrategy,
    overlap: usize,
    language: String,
}

impl TextChunker {
    pub fn chunk_text(&self, text: &str) -> Vec<String>;
    pub fn chunk_document(&self, doc: &Document) -> Vec<DocumentChunk>;
}
```

##### HNSW Index
```rust
pub struct HNSWIndex {
    layers: Vec<Layer>,
    config: HNSWConfig,
    entry_point: Option<NodeId>,
}

impl HNSWIndex {
    pub fn new(config: HNSWConfig) -> Self;
    pub fn insert(&mut self, vector: &Vector, id: NodeId) -> Result<(), Error>;
    pub fn search(&self, query: &Vector, k: usize) -> Result<Vec<(NodeId, f32)>, Error>;
}
```

##### Vector Quantization
```rust
pub enum QuantizationType {
    None,
    PQ { n_centroids: usize, n_subquantizers: usize },
    SQ { bits: usize },
    Binary,
}

pub trait Quantizer {
    fn quantize(&self, vector: &[f32]) -> Result<Vec<f32>, QuantizationError>;
    fn decompress(&self, quantized: &[f32]) -> Result<Vec<f32>, QuantizationError>;
    fn memory_reduction_ratio(&self) -> f32;
}

pub struct PQQuantizer {
    centroids: Vec<Vec<f32>>,
    n_subquantizers: usize,
}

pub struct SQQuantizer {
    bits: usize,
    min_val: f32,
    max_val: f32,
}
```

### 2. Bindings e SDKs

#### Python SDK (PyO3)
**Arquivo**: `bindings/python/vectorizer/__init__.py`

```python
from .vectorizer import (
    VectorizerDB,
    BowEmbedder,
    HashEmbedder,
    NgramEmbedder,
    PQQuantizer,
    SQQuantizer,
    IntelligentChunker,
    chunk_text,
    vectorize
)

class BowEmbedder:
    def __init__(self, vocab_size: int = 50000, dimension: int = 768, max_sequence_length: int = 512):
        ...

    def build_vocab(self, texts: List[str]) -> None:
        ...

    def embed_text(self, text: str) -> List[float]:
        ...

    def embed_texts(self, texts: List[str]) -> List[List[float]]:
        ...

class HashEmbedder:
    def __init__(self, hash_size: int = 1024, dimension: int = 768):
        ...

    def embed_text(self, text: str) -> List[float]:
        ...

    def embed_texts(self, texts: List[str]) -> List[List[float]]:
        ...

class NgramEmbedder:
    def __init__(self, ngram_range: tuple = (1, 3), vocab_size: int = 10000, dimension: int = 768):
        ...

    def build_vocab(self, texts: List[str]) -> None:
        ...

    def embed_text(self, text: str) -> List[float]:
        ...

    def embed_texts(self, texts: List[str]) -> List[List[float]]:
        ...

class PQQuantizer:
    def __init__(self, n_centroids: int = 256, n_subquantizers: int = 8):
        ...

    def quantize(self, vectors: List[List[float]]) -> List[List[float]]:
        ...

    def decompress(self, quantized: List[List[float]]) -> List[List[float]]:
        ...

class SQQuantizer:
    def __init__(self, bits: int = 8):
        ...

    def quantize(self, vectors: List[List[float]]) -> List[List[float]]:
        ...

    def decompress(self, quantized: List[List[float]]) -> List[List[float]]:
        ...

class IntelligentChunker:
    def __init__(self, chunk_size: int = 512, chunk_overlap: int = 50, strategy: str = "sentence"):
        ...

    def chunk_text(self, text: str) -> List[str]:
        ...

class VectorizerDB:
    def __init__(self, persist_path: Optional[str] = None):
        ...

    def insert_documents(self, collection: str, documents: List[Dict], embedder, chunk_size: int = 512, chunk_overlap: int = 50) -> None:
        ...

    def search_by_text(self, collection: str, query_text: str, embedder, k: int = 5) -> List[Dict[str, Any]]:
        ...

    def insert(self, collection: str, ids: List[str], vectors: List[List[float]], payloads: List[Dict]) -> None:
        ...

    def query(self, collection: str, query_text: str, k: int = 5) -> List[Dict[str, Any]]:
        ...

    def search(self, collection: str, query_vector: List[float], k: int = 5) -> List[Dict[str, Any]]:
        ...
```

#### TypeScript SDK (Neon)
**Arquivo**: `bindings/typescript/src/index.ts`

```typescript
export interface EmbeddingProvider {
    embedText(text: string): Promise<number[]>;
    embedTexts(texts: string[]): Promise<number[][]>;
    dimension: number;
    name: string;
}

export class BowEmbedder implements EmbeddingProvider {
    constructor(options: {
        vocabSize?: number;
        dimension?: number;
        maxSequenceLength?: number;
    });

    buildVocab(texts: string[]): Promise<void>;
    embedText(text: string): Promise<number[]>;
    embedTexts(texts: string[]): Promise<number[][]>;
    dimension: number;
    name: string;
}

export class HashEmbedder implements EmbeddingProvider {
    constructor(options: {
        hashSize?: number;
        dimension?: number;
    });

    embedText(text: string): Promise<number[]>;
    embedTexts(texts: string[]): Promise<number[][]>;
    dimension: number;
    name: string;
}

export class NgramEmbedder implements EmbeddingProvider {
    constructor(options: {
        ngramRange?: [number, number];
        vocabSize?: number;
        dimension?: number;
    });

    buildVocab(texts: string[]): Promise<void>;
    embedText(text: string): Promise<number[]>;
    embedTexts(texts: string[]): Promise<number[][]>;
    dimension: number;
    name: string;
}

export interface Quantizer {
    quantize(vectors: number[][]): Promise<number[][]>;
    decompress(quantized: number[][]): Promise<number[][]>;
    memoryReductionRatio: number;
}

export class PQQuantizer implements Quantizer {
    constructor(options: {
        nCentroids?: number;
        nSubquantizers?: number;
    });

    quantize(vectors: number[][]): Promise<number[][]>;
    decompress(quantized: number[][]): Promise<number[][]>;
    memoryReductionRatio: number;
}

export class SQQuantizer implements Quantizer {
    constructor(options: {
        bits?: number;
    });

    quantize(vectors: number[][]): Promise<number[][]>;
    decompress(quantized: number[][]): Promise<number[][]>;
    memoryReductionRatio: number;
}

export class IntelligentChunker {
    constructor(options: {
        chunkSize?: number;
        chunkOverlap?: number;
        strategy?: 'sentence' | 'paragraph' | 'token';
    });

    chunkText(text: string): string[];
    chunkDocument(document: Document): DocumentChunk[];
}

export class VectorizerDB {
    constructor(options: { persistPath?: string });

    insertDocuments(
        collection: string,
        documents: Array<{id: string, text: string, metadata?: any}>,
        options: {
            embedder: EmbeddingProvider;
            chunkSize?: number;
            chunkOverlap?: number;
        }
    ): Promise<void>;

    searchByText(
        collection: string,
        queryText: string,
        embedder: EmbeddingProvider,
        k?: number
    ): Promise<Array<{
        id: string;
        score: number;
        payload: any;
    }>>;

    insert(collection: string, ids: string[], vectors: number[][], payloads: any[]): Promise<void>;

    query(collection: string, queryText: string, k?: number): Promise<Array<{
        score: number;
        payload: any;
        vector: number[];
    }>>;

    search(collection: string, queryVector: number[], k?: number): Promise<Array<{
        score: number;
        payload: any;
        vector: number[];
    }>>;
}
```

### 3. Integrations

#### LangChain Integration
**Python**: `integrations/langchain-py/vectorizer_store.py`
**TypeScript**: `integrations/langchain-ts/vectorizer_store.ts`

```python
from langchain.vectorstores import VectorStore
from vectorizer import VectorizerDB

class VectorizerStore(VectorStore):
    def __init__(self, db: VectorizerDB, embedding_function):
        self.db = db
        self.embedding_function = embedding_function

    def similarity_search(self, query: str, k: int = 5) -> List[Document]:
        results = self.db.query("default", query, k)
        return [Document(page_content=result["payload"]["text"],
                        metadata=result["payload"]) for result in results]
```

#### Aider Integration
Hooks for automated code generation with contextual embeddings.

## Communication Protocols

### Envelope Format (Based on UMICP)

```rust
#[derive(Serialize, Deserialize)]
pub struct VectorEnvelope {
    pub header: EnvelopeHeader,
    pub payload: VectorPayload,
    pub capabilities: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct VectorPayload {
    pub collection: String,
    pub vectors: Vec<Vector>,
    pub payloads: Vec<Payload>,
    pub operation: VectorOperation,
}
```

### Supported Operations
- `INSERT`: Add vectors to collection
- `SEARCH`: Similarity search
- `DELETE`: Remove vectors
- `UPDATE`: Update vectors/payloads
- `BATCH`: Batch operations

## Persistence and Recovery

### Binary Format (bincode)
```rust
#[derive(Serialize, Deserialize)]
pub struct PersistedCollection {
    pub name: String,
    pub config: CollectionConfig,
    pub vectors: Vec<Vector>,
    pub payloads: Vec<Payload>,
    pub index_data: HNSWPersistedData,
    pub metadata: CollectionMetadata,
}
```

### Recovery Strategy
1. **Cold Start**: Load complete indexes from memory
2. **Incremental**: Apply operation logs since last checkpoint
3. **Lazy Loading**: Load vectors on demand

### Payload Compression Strategy
Vectorizer implements automatic payload compression to optimize storage and network performance:

```rust
#[derive(Serialize, Deserialize)]
pub struct CompressedPayload {
    pub data: Vec<u8>,
    pub compressed: bool,
    pub original_size: usize,
    pub algorithm: CompressionAlgorithm,
}

#[derive(Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Lz4,
}

impl PayloadCompressor {
    pub fn compress(&self, payload: &Payload, threshold: usize) -> Result<CompressedPayload, Error> {
        let json_data = serde_json::to_vec(payload)?;
        if json_data.len() > threshold {
            let compressed = lz4_flex::compress_prepend_size(&json_data);
            Ok(CompressedPayload {
                data: compressed,
                compressed: true,
                original_size: json_data.len(),
                algorithm: CompressionAlgorithm::Lz4,
            })
        } else {
            Ok(CompressedPayload {
                data: json_data,
                compressed: false,
                original_size: json_data.len(),
                algorithm: CompressionAlgorithm::None,
            })
        }
    }

    pub fn decompress(&self, compressed: &CompressedPayload) -> Result<Payload, Error> {
        match compressed.algorithm {
            CompressionAlgorithm::Lz4 => {
                let decompressed = lz4_flex::decompress_size_prepended(&compressed.data)?;
                Ok(serde_json::from_slice(&decompressed)?)
            }
            CompressionAlgorithm::None => {
                Ok(serde_json::from_slice(&compressed.data)?)
            }
        }
    }
}
```

**Compression Benefits:**
- **Storage Reduction**: 40-70% reduction for JSON payloads >1KB
- **Network Efficiency**: Faster API responses with compressed payloads
- **Transparent Operation**: Automatic compression/decompression
- **Configurable Thresholds**: Per-collection compression settings
- **Fast Algorithm**: LZ4 provides <10µs compression/decompression

## Performance Considerations

### Target Benchmarks (Based on Specification)
- **Insertion**: ~10µs per vector (1M vectors)
- **Top-10 Query**: ~0.8ms
- **Memory Footprint**: ~1.2GB for 1M 768-dim vectors
- **Latency**: Sub-millisecond for local queries

### Planned Optimizations
1. **SIMD Operations**: Optimized vector processing
2. **Memory Pooling**: Efficient memory management
3. **Concurrent Indexing**: Parallel index construction
4. **Quantization**: Vector compression (PQ, SQ)
5. **Payload Compression**: LZ4 compression for large payloads (>1KB)
6. **Network Optimization**: Compressed payload transmission in APIs

## Security and Robustness

### Input Validation
- Vector dimension verification
- Payload sanitization
- Collection size limits

### Error Handling
```rust
#[derive(Debug)]
pub enum VectorizerError {
    InvalidDimension { expected: usize, got: usize },
    CollectionNotFound(String),
    PersistenceError(String),
    IndexError(String),
}
```

### Logging and Monitoring
- Performance metrics per collection
- Structured logs for debugging
- Health checks for index integrity

## Implementation Strategy

### Phase 1: Core Engine (Rust)
1. Implement basic data structures
2. Create basic HNSW index
3. Implement CRUD operations
4. Add binary persistence

### Phase 2: APIs and CLI
1. REST API with Axum
2. CLI for file ingestion
3. Unit and integration tests

### Phase 3: Bindings
1. Python SDK (PyO3)
2. TypeScript SDK (Neon)
3. Compatibility tests

### Phase 4: Integrations
1. LangChain VectorStore implementations
2. Aider hooks
3. Usage examples

### Phase 5: Optimizations
1. Benchmarks and profiling
2. Performance optimizations
3. Compression and quantization

## Critical Dependencies

### Rust Dependencies (Cargo.toml)
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
hnsw_rs = "0.1"  # Para HNSW implementation
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
parking_lot = "0.12"
dashmap = "5.5"
```

### Python Dependencies
```txt
numpy>=1.21.0
sentence-transformers>=2.2.0
langchain>=0.0.300
pyo3>=0.19.0
```

### TypeScript Dependencies
```json
{
  "dependencies": {
    "@langchain/core": "^0.1.0",
    "neon-cli": "^0.2.0",
    "axios": "^1.6.0"
  }
}
```

## Testing and Quality

### Testing Strategy
1. **Unit Tests**: Individual components
2. **Integration Tests**: Complete workflows
3. **Performance Tests**: Automated benchmarks
4. **Fuzzing**: Robustness tests with random inputs

### Quality Metrics
- Code coverage: >90%
- Consistent performance with benchmarks
- Zero crashes in stress tests
- Compatibility with LangChain specifications

## Next Steps

1. **Review by specialized Rust LLM**: Validate architecture design
2. **Prototyping**: Implement core engine proof of concept
3. **Performance Validation**: Initial benchmark tests
4. **Iteration**: Refine based on feedback and measurements

---

**Note**: This documentation was generated through static analysis of the conceptual project and existing integrations (UMICP). It serves as a basis for implementation and review by other specialized LLMs.
