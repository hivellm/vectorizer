# Vectorizer

## ⚠️ PROJECT STATUS: Phase 1.5 Complete - Ready for APIs

**IMPORTANT**: This project has completed Phase 1.5 with foundation, critical fixes, and embedding system. Ready for Phase 2 (APIs).

**Current State**: 
- ✅ Core vector database engine implemented and tested
- ✅ HNSW index with improved update operations
- ✅ Fixed persistence layer - saves/loads actual vectors
- ✅ Text embedding system with TF-IDF, BoW, and N-grams
- ✅ 30+ unit tests passing
- ✅ Semantic search working with real text
- 🚀 REST/gRPC APIs (Phase 2 - Next)
- ⏳ Client SDKs (Phase 4 - Planned)
- ⏳ Dashboard & CLI (Phase 5 - Planned)

---

A high-performance, in-memory vector database written in Rust with client-server architecture, designed for semantic search and top-k nearest neighbor queries in AI-driven applications. Features mandatory API key authentication, automatic LZ4 payload compression, native embedding models, and binary file persistence for durability. Includes pre-configured Python and TypeScript SDKs, optimized for chunking, vectorization, and seamless integrations with LangChain and Aider.

## 🚀 Overview

Vectorizer is a lightweight, scalable vector database with **client-server architecture** tailored for collaborative AI systems, such as multi-LLM architectures. It stores high-dimensional embeddings in memory for sub-millisecond top-k approximate nearest neighbor (ANN) searches, with persistence to binary files for reliable recovery. Built with Rust's safety and performance in mind, it leverages HNSW (Hierarchical Navigable Small World) for efficient indexing and Tokio for async concurrency.

### Key Features
- **Client-Server Architecture**: Centralized server with lightweight client SDKs
- **Mandatory Security**: API key authentication (coming in Phase 2)
- **Automatic Compression**: LZ4 compression for large payloads (ready for Phase 2)
- **Native Embeddings**: Built-in BOW, Hash, and N-gram models (Phase 3)
- **Memory Optimization**: Vector quantization support (Phase 6)
- **In-Memory Speed**: Operates entirely in RAM for low-latency
- **Top-k ANN Search**: Fast semantic retrieval using HNSW
- **Binary Persistence**: Durable storage with bincode serialization
- **Multi-LLM Ready**: Designed for AI governance systems

## 🎯 Use Case

Vectorizer is ideal for AI projects requiring real-time semantic search and context sharing:
- **Secure AI Governance**: Multi-LLM architectures with authentication
- **Memory-Efficient RAG**: Large knowledge bases with compression
- **Collaborative LLM Discussions**: 27-agent debates for consensus (HiveLLM)
- **Production AI Workflows**: Enterprise-grade vector search
- **Resource-Constrained Deployments**: Optimized memory usage

## 🚀 Implementation Progress

**Current Status**: Phase 1 (Foundation) ✅ COMPLETED

### ✅ Completed Tasks (Phase 1)

- **Project Setup**: Rust project initialized with Cargo.toml and dependencies
- **Core Data Structures**: Vector, Payload, Collection structs implemented
- **VectorStore**: Thread-safe in-memory store with DashMap
- **CRUD Operations**: Insert, retrieve, update, delete operations
- **HNSW Index**: Integration with hnsw_rs v0.3
- **Persistence Layer**: Binary serialization with bincode
- **Unit Tests**: All core components tested
- **CI/CD Pipeline**: GitHub Actions configured

## 📁 Project Structure

```
vectorizer/
├── src/                    # Core Rust server source code
│   ├── db/                # Database engine (in-memory store, HNSW index)
│   ├── api/               # REST/gRPC API handlers (Axum-based) [Phase 2]
│   ├── persistence/       # Binary file serialization with LZ4 compression
│   ├── compression/       # Payload compression engine (LZ4) [Phase 2]
│   ├── auth/              # API key authentication [Phase 2]
│   ├── dashboard/         # Localhost web dashboard [Phase 5]
│   ├── cli/               # Enhanced CLI [Phase 5]
│   └── models/            # Data structures (vectors, payloads, collections)
├── bindings/              # Client SDK bindings [Phase 4]
│   ├── python/            # PyO3-based Python client SDK
│   └── typescript/        # Neon-based TypeScript client SDK
├── integrations/          # LangChain and Aider hooks [Phase 6]
├── docs/                  # Technical documentation
├── examples/              # Example usage [Phase 4]
├── tests/                 # Unit and integration tests
├── benches/               # Performance benchmarks
├── Cargo.toml             # Rust dependencies and config
└── README.md              # You're here!
```

## 🛠️ Installation & Usage

### Prerequisites
- Rust 1.82+ (using nightly for edition 2024)
- Cargo for dependency management

### Current Installation (Phase 1)

```bash
# Clone repository
git clone https://github.com/hivellm/vectorizer
cd vectorizer

# Use Rust nightly
rustup override set nightly

# Build and run tests
cargo test

# Run the server (placeholder only)
cargo run
```

### Planned Usage (When Phase 2+ Complete)

#### CLI for Server Management
```bash
# Start server
vectorizer server --host 127.0.0.1 --port 15001

# Create API key
vectorizer api-keys create --name "production" --description "Production app"

# Ingest files
vectorizer ingest --file document.txt --collection my_docs --api-key <key>
```

#### Python SDK Example
```python
from vectorizer import VectorizerClient

# Connect to server (API key required in Phase 2)
client = VectorizerClient(
    host="localhost",
    port=15001,
    api_key="your-api-key-here"
)

# Create collection
client.create_collection(
    name="documents",
    dimension=768,
    metric="cosine"
)

# Insert documents
documents = [{
    "id": "doc_001",
    "text": "Machine learning is a method of data analysis...",
    "metadata": {"source": "ml_guide.pdf"}
}]

client.insert_documents("documents", documents)

# Search
results = client.search_by_text(
    "documents",
    "machine learning algorithms",
    k=5
)
```

## 🏗️ Technical Details

- **Rust Edition**: 2024 (nightly)
- **Architecture**: Client-server with REST/gRPC APIs
- **Storage**: In-memory with binary persistence
- **Indexing**: HNSW for ANN search
- **Concurrency**: Thread-safe with DashMap and RwLock
- **Compression**: LZ4 for payloads >1KB
- **Security**: API key authentication (Phase 2)

### Core Dependencies
- `tokio` 1.40 - Async runtime
- `axum` 0.7 - Web framework (ready for Phase 2)
- `hnsw_rs` 0.3 - HNSW index implementation
- `dashmap` 6.1 - Concurrent HashMap
- `bincode` 1.3 - Binary serialization
- `lz4_flex` 0.11 - Compression
- `serde` 1.0 - Serialization framework

## 🧪 Testing & Quality

Currently implemented:
- ✅ Unit tests for core components (13 passing)
- ✅ CI/CD with GitHub Actions
- ✅ Code formatting with rustfmt
- ✅ Linting with clippy

### Running Tests
```bash
cargo test        # Run all tests
cargo bench       # Run benchmarks
cargo clippy      # Run linter
```

## 📊 Performance Targets

Based on architecture design (actual benchmarks pending):

### Core Performance
- **Insert**: ~10µs per vector
- **Top-10 Query**: ~0.8ms (HNSW index)
- **Memory Footprint**: ~1.2GB for 1M vectors (before quantization)
- **Network Latency**: <1ms for local API calls

### Compression Performance (Phase 2)
- **LZ4 Compression**: <10µs per KB
- **Storage Reduction**: 40-70% for payloads >1KB
- **Network Savings**: 40-70% bandwidth reduction

### Quantization Impact (Phase 6)
- **PQ Quantization**: 75% memory reduction, 10-15% slower queries
- **SQ Quantization**: 50% memory reduction, 5% slower queries
- **Binary Quantization**: 97% memory reduction, 50% faster queries

## ⚙️ Configuration

Vectorizer uses YAML configuration (see `config.example.yml`):

### Quick Configuration
```yaml
server:
  host: "127.0.0.1"
  port: 15001
  mode: "internal"  # or "cloud"

storage:
  persistence_path: "./data"
  compression:
    enabled: true
    threshold_bytes: 1024

collections:
  default_dimension: 768
  default_metric: "cosine"
```

## 🚀 Roadmap

### Phase 1: Foundation ✅ COMPLETED
- Core engine, HNSW index, persistence
- All critical bugs fixed
- 30+ tests passing

### Phase 1.5: Enhancements ✅ COMPLETED
- Fixed persistence layer (grok-code-fast-1)
- Corrected distance metrics (grok-code-fast-1)
- Improved HNSW operations (grok-code-fast-1)
- Text embedding system (Claude)
- TF-IDF, BoW, N-gram providers

### Phase 2: Server & APIs 🚀 NEXT
- REST API with Axum
- Authentication system
- Rate limiting

### Phase 3: Testing & Quality (Month 3)
- Integration tests
- Performance benchmarks
- Load testing

### Phase 4: Client SDKs (Month 4)
- Python SDK (PyO3)
- TypeScript SDK
- SDK packaging

### Phase 5: Production Features (Month 5)
- Dashboard (localhost)
- CLI tools
- Monitoring

### Phase 6: Experimental (Month 6+)
- Vector quantization
- UMICP integration
- GPU acceleration

---

## 📚 Documentation

### 📋 Core Documentation
- [Roadmap](docs/ROADMAP.md) - Current implementation plan and status
- [Technical Documentation Index](docs/TECHNICAL_DOCUMENTATION_INDEX.md) - Complete documentation overview

### 🏗️ Phase 1 - Foundation
- [Architecture](docs/phase1/ARCHITECTURE.md) - System architecture details
- [Technical Implementation](docs/phase1/TECHNICAL_IMPLEMENTATION.md) - Detailed technical specs
- [Configuration](docs/phase1/CONFIGURATION.md) - Configuration guide
- [Performance](docs/phase1/PERFORMANCE.md) - Performance characteristics
- [QA Guidelines](docs/phase1/QA_GUIDELINES.md) - Quality assurance standards

### 🔍 Implementation Reviews
- [grok-code-fast-1 Review](docs/reviews/REVIEW_REPORT.md) - Critical issues analysis
- [Claude Validation](docs/reviews/CLAUDE_REVIEW_ANALYSIS.md) - Implementation fixes validation
- [Embedding System](docs/reviews/EMBEDDING_IMPLEMENTATION.md) - Text embedding documentation
- [Project Status Summary](docs/reviews/PROJECT_STATUS_SUMMARY.md) - Current project status

### 🚀 Future Phases
- [API Specifications](docs/future/APIS.md) - REST/gRPC API designs
- [Dashboard](docs/future/DASHBOARD.md) - Web interface specifications
- [Integrations](docs/future/INTEGRATIONS.md) - External system integrations
- [Implementation Checklist](docs/future/IMPLEMENTATION_CHECKLIST.md) - Complete task tracking (380+ items)
- [Implementation Tasks](docs/future/IMPLEMENTATION_TASKS.md) - Task management board

## 🤖 AI Implementation Review: Sonnet-4.1-Opus Integration

**Status**: Phase 1 Foundation Complete ✅ | **Critical Bugs**: Fixed ✅ | **Next Phase**: Ready for Phase 2 (APIs)

### Implementation Status
- ✅ **Phase 1 (Foundation)**: Core engine, HNSW index, persistence, basic operations
- ✅ **Comprehensive Testing**: 29 unit tests + 4 integration tests (all passing)
- ✅ **Code Review**: grok-code-fast-1 review completed with detailed feedback
- ✅ **Documentation**: Technical specs and roadmap finalized

### ✅ Critical Issues Resolved
**Status**: All critical bugs from grok-code-fast-1 review have been fixed!

1. **✅ Persistence Layer Fixed**
   - Implemented proper vector iteration in `save()` method
   - Now correctly serializes all vector data instead of placeholder

2. **✅ Distance Metrics Corrected**
   - Fixed metric conversions in HNSW search with proper mathematical formulas
   - Added automatic vector normalization for cosine similarity
   - Implemented correct similarity score calculations

3. **✅ HNSW Operations Improved**
   - Added index rebuild tracking and statistics
   - Implemented foundation for efficient update operations
   - Added rebuild monitoring capabilities

**Full review details**: See `REVIEW_REPORT.md` for complete implementation history.

---

## 🤝 Contributing

We follow a governance model inspired by HiveLLM. To contribute:
1. Review the documentation in `docs/`
2. Check `IMPLEMENTATION_TASKS.md` for pending work
3. Submit PRs with tests and documentation
4. Follow Rust best practices and conventions

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

## 📬 Contact

For questions or collaboration, open an issue or join the discussion at [hivellm/gov](https://github.com/hivellm/gov).

---

**Note**: This project is part of the HiveLLM ecosystem. The implementation follows the detailed specification available in the `docs/` directory.