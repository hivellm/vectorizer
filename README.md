# Vectorizer

A high-performance, in-memory vector database written in Rust with client-server architecture, designed for semantic search and top-k nearest neighbor queries in AI-driven applications. Features mandatory API key authentication, automatic LZ4 payload compression, native embedding models, and binary file persistence for durability. Includes pre-configured Python and TypeScript SDKs, optimized for chunking, vectorization, and seamless integrations with LangChain and Aider.

## 🚀 Overview

Vectorizer is a lightweight, scalable vector database with **client-server architecture** tailored for collaborative AI systems, such as multi-LLM architectures. It stores high-dimensional embeddings in memory for sub-millisecond top-k approximate nearest neighbor (ANN) searches, with persistence to binary files for reliable recovery. Built with Rust's safety and performance in mind, it leverages HNSW (Hierarchical Navigable Small World) for efficient indexing and Tokio for async concurrency.

Expanded technical specifications:
- **Client-Server Architecture**: Centralized Rust server with mandatory API key authentication; lightweight Python/TypeScript SDKs connect via REST/gRPC APIs.
- **Security**: Mandatory API keys for all operations; localhost dashboard (http://localhost:3000) for secure key management.
- **Automatic Compression**: LZ4 compression for payloads >1KB, reducing storage by 40-70% and network bandwidth.
- **Native Embeddings**: Built-in BOW, Hash, and N-gram models - no external transformer dependencies.
- **Vector Optimization**: PQ/SQ/Binary quantization options for memory-efficient storage (50-97% reduction).
- **Bindings for Other Languages**: Pre-configured bindings via PyO3 for Python and Neon for TypeScript/Node.js.
- **SDK Features**: Server-backed chunking, vectorization, and queries with automatic embedding and compression.
- **Network Configuration**: Internal mode (localhost:15001) and cloud mode (0.0.0.0:15001) with configurable security.
- **CLI Tool**: Enhanced CLI for API key management, server administration, and secure file ingestion.
- **Integrations**: Python and TypeScript SDKs include native support for LangChain and Aider with server-backed processing.

Key features:
- **Client-Server Architecture**: Centralized server with lightweight client SDKs for maintainable multi-language support.
- **Mandatory Security**: API key authentication with localhost dashboard for secure key management.
- **Automatic Compression**: LZ4 compression reduces storage and network usage by 40-70% for large payloads.
- **Native Embeddings**: Built-in BOW, Hash, and N-gram models - no external dependencies.
- **Memory Optimization**: Vector quantization (PQ/SQ/Binary) for 50-97% memory reduction.
- **In-Memory Speed**: Operates entirely in RAM, rivaling Redis for low-latency queries.
- **Top-k ANN Search**: Fast semantic retrieval using HNSW (via `hnsw_rs`).
- **Binary Persistence**: Durable storage with `bincode` serialization to disk.
- **Network Flexible**: Configurable for internal (localhost) or cloud deployment.
- **Multi-LLM Ready**: Designed for AI governance systems (e.g., HiveLLM), with API-driven integration.
- **Scalable and Safe**: Built with Rust, using Tokio for concurrency and Serde for robust serialization.

## 🎯 Use Case

Vectorizer is ideal for AI projects requiring real-time semantic search and context sharing, such as:
- **Secure AI Governance**: Multi-LLM architectures with mandatory authentication and audit trails.
- **Memory-Efficient RAG**: Large knowledge bases with automatic compression and quantization optimization.
- **Collaborative LLM Discussions**: 27-agent debates for consensus with server-backed processing (HiveLLM governance).
- **Production AI Workflows**: Enterprise-grade vector search with native embeddings and network flexibility.
- **Resource-Constrained Deployments**: Optimized memory usage through quantization (50-97% reduction).

## 📁 Project Structure

```
vectorizer/
├── src/                    # Core Rust source code
│   ├── db/                # Database engine (in-memory store, HNSW index)
│   ├── api/               # REST/gRPC API handlers (Axum-based)
│   ├── persistence/       # Binary file serialization (bincode)
│   ├── cli/               # CLI implementation (using clap)
│   └── models/            # Data structures (vector, metadata, collections)
├── bindings/              # Language bindings
│   ├── python/            # PyO3-based Python SDK
│   └── typescript/        # Neon-based TypeScript/Node.js SDK
├── integrations/          # LangChain and Aider hooks
│   ├── langchain-py/      # Python LangChain VectorStore implementation
│   └── langchain-ts/      # TypeScript LangChain.js VectorStore implementation
├── examples/              # Example usage (e.g., LLM integration, top-k queries, CLI ingestion)
├── tests/                 # Unit and integration tests (proptest)
├── benches/               # Performance benchmarks (criterion)
├── Cargo.toml             # Rust dependencies and config
└── README.md              # You're here!
```

## 🛠️ Installation

### Prerequisites
- Rust (stable, 1.82+ recommended, 2025)
- Cargo for dependency management
- For Python SDK: Python 3.12+ and pip
- For TypeScript SDK: Node.js 20+ and npm/yarn
- Optional: Docker for containerized deployment

### Steps
1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/vectorizer.git
   cd vectorizer
   ```
2. Build the core Rust project:
   ```bash
   cargo build --release
   ```
3. Install Python SDK (via bindings):
   ```bash
   cd bindings/python
   maturin develop --release  # Or pip install .
   ```
4. Install TypeScript SDK:
   ```bash
   cd bindings/typescript
   npm install
   ```
5. Run the server:
   ```bash
   cargo run --release
   ```

## 📚 Usage

### CLI for File Ingestion and Queries
Ingest a file (e.g., text or PDF) into a collection: The CLI chunks the content, generates embeddings (using a default model like Sentence Transformers via integrated wrappers), upserts to the DB, and updates memory.
```bash
cargo run -- --cli ingest --file path/to/document.txt --collection my_collection --chunk-size 512 --vector-dim 768
```

Simple text query:
```bash
cargo run -- --cli query --text "Your search query" --collection my_collection --k 5
```
Output includes results with scores, payloads, and vectors.

### Python SDK Example (with LangChain Integration)
```python
from vectorizer import VectorizerDB, chunk_text, vectorize

# Initialize DB
db = VectorizerDB(persist_path="data/vectorizer.bin")

# Chunk and vectorize text
chunks = chunk_text("Your long text here", chunk_size=512)
vectors = vectorize(chunks)  # Uses default embedding model

# Insert with payload
db.insert(collection="my_collection", ids=["id1"], vectors=vectors, payloads=[{"source": "doc"}])

# Query with k results
results = db.query(collection="my_collection", query_text="Search term", k=5)
# Results: list of dicts with score, payload, vector

# LangChain integration
from langchain.vectorstores import VectorizerStore
store = VectorizerStore(db, embedding_function=vectorize)
docs = store.similarity_search("Query", k=5)
```

### TypeScript SDK Example (with LangChain.js Integration)
```typescript
import { VectorizerClient } from '@hivellm/vectorizer';

// Connect to Vectorizer server (API key required)
const client = new VectorizerClient({
  host: 'localhost',
  port: 15001,
  apiKey: 'your-api-key-here'
});

// Insert documents (server handles chunking and embedding)
await client.insertDocuments('my_collection', [
  {
    id: 'doc1',
    text: 'Your long text here...',
    metadata: { source: 'doc' }
  }
], {
  chunkSize: 512,
  chunkOverlap: 50
});

// Query with text (server handles embedding)
const results = await client.searchByText('my_collection', 'Search term', 5);
// Results: array of { id, score, payload }

// LangChain.js integration
import { VectorizerStore } from '@langchain/vectorizer';
const store = new VectorizerStore(client);
const docs = await store.similaritySearch('Query', 5);
```

### Aider Integration
In Python/TypeScript SDKs, use provided hooks for Aider-assisted code generation, e.g., `aider_hook(client)` to automate embedding pipelines in AI coding sessions with server-backed processing.

## ⚙️ Configuration

- **Default Vector Size**: Fixed at 768 dims (configurable via --vector-dim).
- **Chunking**: Default size 512 tokens; customizable.
- **Embedding Model**: Defaults to a lightweight Sentence Transformers model; configurable to OpenAI or custom.
- **Persistence**: Set --persist-path for binary files.
- **Index Parameters**: Configure HNSW via config.toml (e.g., m, ef_construction).
- **API Port**: Default 8080, override with --port <PORT>.

## 🧪 Benchmarks
Using criterion on a dataset of 1M 768-dim vectors (2025 hardware, M1 Max):
- Insert: ~10µs per vector
- Top-10 query: ~0.8ms
- Memory footprint: ~1.2GB for 1M vectors
- CLI Ingestion: ~500ms for a 10MB text file (chunk + embed + upsert)

## 🤝 Contributing
We follow a governance model inspired by HiveLLM (see [hivellm/gov](https://github.com/hivellm/gov)). To contribute:
1. Submit a proposal in proposals/ (use JSON schemas in schemas/).
2. Pass Peer and Final Review (see gov/ for details).
3. Open a PR with tests and docs.

Issues and PRs are welcome! Check issues/ for templates.

## 📜 License
MIT License. See [LICENSE](LICENSE) for details.

## 📬 Contact
For questions or collaboration, open an issue or join the discussion at [hivellm/gov](https://github.com/hivellm/gov).