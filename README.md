# Vectorizer

## ⚠️ PROJECT STATUS: CONCEPTUAL PHASE

**IMPORTANT**: This project is currently in conceptual/specification phase. **NO CODE IMPLEMENTATION EXISTS YET**. This documentation serves as a complete technical specification for future development.

**Current State**: 
- ✅ Complete technical specification and architecture design
- ✅ Detailed API documentation and configuration system
- ✅ Implementation roadmap and task breakdown
- ❌ **Code implementation (NOT STARTED)**
- ❌ **Installation not possible - no executable code exists**
- ❌ **Performance benchmarks theoretical only**

**For Developers**: This is an excellent foundation for implementation. The specification is production-ready and highly detailed.

**📋 See ROADMAP.md for prioritized implementation plan** - Core server → APIs → Testing → SDKs → Experimental features

**🤖 Documentation Credits**: 
- Technical specification structured by **grok-fast-code-1**
- Documentation reviewed and corrected by **claude-4-sonnet** (September 23, 2025)
- **Second Review** by **gpt-5** (September 23, 2025)
- **Third Review** by **gemini-2.5-pro** (September 23, 2025)

---

A high-performance, in-memory vector database **[PLANNED]** written in Rust with client-server architecture, designed for semantic search and top-k nearest neighbor queries in AI-driven applications. **[WHEN IMPLEMENTED]** Features mandatory API key authentication, automatic LZ4 payload compression, native embedding models, and binary file persistence for durability. Includes pre-configured Python and TypeScript SDKs, optimized for chunking, vectorization, and seamless integrations with LangChain and Aider.

## 🚀 Overview

Vectorizer is a lightweight, scalable vector database with **client-server architecture** tailored for collaborative AI systems, such as multi-LLM architectures. It stores high-dimensional embeddings in memory for sub-millisecond top-k approximate nearest neighbor (ANN) searches, with persistence to binary files for reliable recovery. Built with Rust's safety and performance in mind, it leverages HNSW (Hierarchical Navigable Small World) for efficient indexing and Tokio for async concurrency.

Expanded technical specifications:
- **Client-Server Architecture**: Centralized Rust server with mandatory API key authentication; lightweight Python/TypeScript SDKs connect via REST/gRPC APIs.
- **Security**: Mandatory API keys for all operations; localhost dashboard (http://localhost:15002) for secure key management.
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
├── src/                    # Core Rust server source code
│   ├── db/                # Database engine (in-memory store, HNSW index)
│   ├── api/               # REST/gRPC API handlers (Axum-based)
│   ├── persistence/       # Binary file serialization with LZ4 compression
│   ├── compression/       # Payload compression engine (LZ4)
│   ├── auth/              # API key authentication and management
│   ├── dashboard/         # Localhost web dashboard (localhost:15002)
│   ├── cli/               # Enhanced CLI (API keys, server management)
│   └── models/            # Data structures (vectors, payloads, collections)
├── bindings/              # Client SDK bindings
│   ├── python/            # PyO3-based Python client SDK
│   └── typescript/        # Neon-based TypeScript client SDK
├── integrations/          # LangChain and Aider hooks
│   ├── langchain-py/      # Python LangChain VectorStore (server client)
│   └── langchain-ts/      # TypeScript LangChain.js VectorStore (server client)
├── docs/                  # Technical documentation
│   ├── APIS.md           # Complete API reference with compression
│   ├── ARCHITECTURE.md   # System architecture and security
│   ├── DASHBOARD.md      # Dashboard technical documentation
│   ├── PERFORMANCE.md    # Benchmarks and optimization guides
│   └── INTEGRATIONS.md   # Integration examples
├── examples/              # Example usage (server-backed processing)
├── tests/                 # Unit and integration tests (proptest)
├── benches/               # Performance benchmarks (criterion)
├── Cargo.toml             # Rust dependencies and config
└── README.md              # You're here!
```

## 🛠️ Implementation Requirements

### Prerequisites for Future Implementation
- Rust (stable, 1.82+ recommended, 2025)
- Cargo for dependency management
- For Python SDK: Python 3.12+ and pip
- For TypeScript SDK: Node.js 20+ and npm/yarn
- Optional: Docker for containerized deployment
- Optional: LZ4 library for payload compression (automatically handled)

### Installation Status
❌ **NOT YET AVAILABLE** - Implementation required

The complete technical specification exists, but no code has been implemented. To implement this project:

1. **Clone Repository**: `git clone [repository-url]`
2. **Review Specification**: Start with `TECHNICAL_DOCUMENTATION_INDEX.md`
3. **Follow Implementation Plan**: See `IMPLEMENTATION_CHECKLIST.md` (380 tasks)
4. **Use Roadmap**: Follow `ROADMAP.md` for phased approach

### For Contributors
```bash
# Current state - specification only
git clone [repository-url]
cd vectorizer
ls src/  # Empty - needs implementation

# Review documentation
cat TECHNICAL_DOCUMENTATION_INDEX.md
cat IMPLEMENTATION_CHECKLIST.md
cat ROADMAP.md
```

When implemented, installation will follow the standard Rust build process.

## 📚 Usage (Planned)

### First Time Setup - When Implemented
1. **Start the server**: `cargo run --release`
2. **Create an API key** via dashboard (`http://localhost:15002`) or CLI:
   ```bash
   vectorizer api-keys create --name "my-app" --description "Development key"
   ```
3. **Use the API key** in all SDK operations

**Note**: All usage examples below are planned functionality based on the technical specification.

### CLI for Server Management and API Keys
Manage API keys and server operations:
```bash
# Create API key
vectorizer api-keys create --name "production" --description "Production app"

# List API keys
vectorizer api-keys list

# Delete API key
vectorizer api-keys delete <key-id>

# Start server (internal mode)
vectorizer server --host 127.0.0.1 --port 15001

# Start server (cloud mode)
vectorizer server --host 0.0.0.0 --port 15001
```

### CLI for Secure File Ingestion
Ingest files securely through the server (requires API key):
```bash
# Ingest with server-backed processing
vectorizer ingest \
  --file document.txt \
  --collection my_docs \
  --api-key your-api-key-here \
  --chunk-size 512 \
  --embedding native_bow

# Query with text (server handles embedding)
vectorizer query \
  --collection my_docs \
  --text "machine learning algorithms" \
  --api-key your-api-key-here \
  --k 5
```

### Python SDK Example (Server-Client Architecture)
```python
from vectorizer import VectorizerClient

# Connect to Vectorizer server (API key REQUIRED)
client = VectorizerClient(
    host="localhost",
    port=15001,
    api_key="your-api-key-here"  # MANDATORY for all operations
)

# Create collection with compression and native embeddings
client.create_collection(
    name="documents",
    dimension=768,
    metric="cosine",
    quantization={"type": "pq", "n_centroids": 256, "n_subquantizers": 8},
    embedding={"model": "native_bow", "vocab_size": 50000},
    compression={"enabled": True, "threshold_bytes": 1024, "algorithm": "lz4"}
)

# Insert documents (server handles chunking, embedding, and compression)
documents = [
    {
        "id": "doc_001",
        "text": "Machine learning is a method of data analysis that automates analytical model building...",
        "metadata": {"source": "ml_guide.pdf", "chapter": 1}
    }
]

# Server processes: chunks → embeds → quantizes → compresses → stores
client.insert_documents(
    collection="documents",
    documents=documents,
    chunk_size=512,
    chunk_overlap=50
)

# Semantic search (server handles text embedding automatically)
results = client.search_by_text(
    collection="documents",
    query_text="machine learning algorithms",
    k=5
)

for result in results:
    print(f"ID: {result['id']}, Score: {result['score']:.3f}")
    print(f"Text: {result['payload']['text'][:100]}...")
    print("---")

# LangChain integration (server-backed)
from langchain.vectorstores import VectorizerStore
store = VectorizerStore(client)  # Uses server client
docs = store.similarity_search("Query", k=5)
```

### TypeScript SDK Example (Server-Client Architecture)
```typescript
import { VectorizerClient } from '@hivellm/vectorizer';

// Connect to Vectorizer server (API key MANDATORY)
const client = new VectorizerClient({
  host: 'localhost',
  port: 15001,
  apiKey: 'your-api-key-here'  // REQUIRED for all operations
});

// Create collection with compression and native embeddings
await client.createCollection('documents', {
  dimension: 768,
  metric: 'cosine' as const,
  quantization: { type: 'pq', nCentroids: 256, nSubquantizers: 8 },
  embedding: { model: 'native_bow', vocabSize: 50000 },
  compression: { enabled: true, thresholdBytes: 1024, algorithm: 'lz4' }
});

// Insert documents (server handles everything: chunking, embedding, compression)
const documents = [
  {
    id: 'doc_001',
    text: 'Machine learning is a method of data analysis that automates analytical model building...',
    metadata: { source: 'ml_guide.pdf', chapter: 1 }
  }
];

// Server processes: chunks → embeds → quantizes → compresses → stores
await client.insertDocuments('documents', documents, {
  chunkSize: 512,
  chunkOverlap: 50
});

// Semantic search (server handles text embedding automatically)
const results = await client.searchByText(
  'documents',
  'machine learning algorithms',
  5
);

results.forEach(result => {
  console.log(`ID: ${result.id}, Score: ${result.score.toFixed(3)}`);
  console.log(`Text: ${result.payload.text?.substring(0, 100)}...`);
  console.log('---');
});

// LangChain.js integration (server-backed)
import { VectorizerStore } from '@langchain/vectorizer';
const store = new VectorizerStore(client);
const docs = await store.similaritySearch('Query', 5);
```

### Aider Integration
In Python/TypeScript SDKs, use provided hooks for Aider-assisted code generation, e.g., `aider_hook(client)` to automate embedding pipelines in AI coding sessions with server-backed processing.

## ⚙️ Configuration

Vectorizer uses a comprehensive YAML configuration system that allows you to customize every aspect of the server. See `docs/CONFIGURATION.md` for complete documentation and `IMPLEMENTATION_CHECKLIST.md` for implementation status. Copy `config.example.yml` to `config.yml` and modify it according to your needs.

### Quick Configuration
```bash
# Start with example configuration
cp config.example.yml config.yml

# Start server with custom config
cargo run --release -- --config config.yml
```

### Server Configuration
- **Default Port**: 15001 (above 15000 as requested)
- **Network Modes**: Internal (127.0.0.1) or Cloud (0.0.0.0)
- **Dashboard**: Localhost-only at http://localhost:(server_port + 1)
- **API Keys**: Mandatory authentication for all operations

### Collection Configuration
- **Vector Dimensions**: Configurable (default: 768 for text)
- **Distance Metrics**: Cosine, Euclidean, Dot Product
- **Quantization**: PQ (75% memory reduction), SQ (50%), Binary (97%)
- **Compression**: LZ4 automatic compression (>1KB payloads, 40-70% reduction)
- **Native Embeddings**: BOW, Hash, N-gram (no external dependencies)

### Performance Tuning
- **Chunking**: Default 512 tokens, configurable overlap
- **Index Parameters**: HNSW m, ef_construction, ef_search
- **Memory Pooling**: Efficient memory management
- **Caching**: Query result caching with TTL

### Security Settings
- **API Key Management**: CLI and dashboard-based key lifecycle
- **Audit Logging**: Complete operation tracking
- **Rate Limiting**: Configurable per API key
- **Network Isolation**: Internal vs external deployment modes

## 🧪 Performance Targets (Theoretical)

**Note**: These are theoretical estimates based on architecture design. No actual benchmarks exist yet.

Projected performance on reference hardware (M1 Max, 2025):

### Core Performance Targets
- **Insert**: ~10µs per vector (estimated)
- **Top-10 Query**: ~0.8ms (HNSW index, estimated)
- **Memory Footprint**: ~1.2GB for 1M vectors (before quantization, estimated)
- **Network Latency**: <1ms for local API calls (estimated)

### Compression Performance (Estimated)
- **LZ4 Compression**: <10µs per KB (target)
- **Storage Reduction**: 40-70% for payloads >1KB (target)
- **Network Savings**: 40-70% bandwidth reduction (target)
- **Decompression**: <5µs per KB (target)

### Quantization Impact (Estimated)
- **PQ Quantization**: 75% memory reduction, 10-15% slower queries (target)
- **SQ Quantization**: 50% memory reduction, 5% slower queries (target)
- **Binary Quantization**: 97% memory reduction, 50% faster queries (target)

### File Processing (Estimated)
- **CLI Ingestion**: ~500ms for 10MB text file (target)
- **Batch Operations**: 2-3x faster than individual operations (target)
- **Concurrent Clients**: Scales to 100+ simultaneous connections (target)

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