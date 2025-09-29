# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## 🚀 **Key Features**

- **🔍 Semantic Search**: Advanced vector similarity search with multiple distance metrics
- **📚 Document Indexing**: Intelligent chunking and processing of various file types
- **🧠 Multiple Embeddings**: Support for TF-IDF, BM25, BERT, MiniLM, and custom models
- **⚡ High Performance**: Sub-3ms search times with optimized HNSW indexing
- **🏗️ GRPC Architecture**: High-performance binary communication between services
- **🔧 MCP Integration**: Model Context Protocol for AI IDE integration (Cursor, VS Code)
- **🌐 REST API**: Complete HTTP API with authentication and security
- **📱 TypeScript SDK**: ✅ Published on npm - Complete TypeScript client for web applications
- **🟨 JavaScript SDK**: ✅ Published on npm - Modern JavaScript client with multiple build formats
- **🦀 Rust SDK**: ✅ Published on crates.io - High-performance native client with memory safety and MCP support
- **🐍 Python SDK**: 🚧 In development - PyPI publishing in progress
- **🔗 LangChain Integration**: Complete VectorStore for Python and JavaScript/TypeScript
- **🚀 Advanced Embedding Models**: ONNX and Real Models (MiniLM, E5, MPNet, GTE) with GPU acceleration

## 📝 **Automatic Summarization**

Intelligent content summarization with MMR algorithm:
- **Extractive Summarization**: MMR algorithm for diversity and relevance
- **Keyword Summarization**: Key term extraction for quick overview  
- **Dynamic Collections**: Auto-created summary collections with rich metadata

## 🔗 **Framework Integrations**

Complete integrations with popular AI frameworks:

### **LangChain**
```python
from integrations.langchain.vectorizer_store import VectorizerStore

store = VectorizerStore(host="localhost", port=15001, collection_name="docs")
store.add_documents([{"page_content": "LangChain framework", "metadata": {"source": "intro.txt"}}])
results = store.similarity_search("language models", k=3)
```

### **PyTorch & TensorFlow**
```python
from integrations.pytorch.pytorch_embedder import create_transformer_embedder

embedder = create_transformer_embedder(model_path="sentence-transformers/all-MiniLM-L6-v2")
client = PyTorchVectorizerClient()
client.set_embedder(embedder)
```

## 🏷️ **Releases & Downloads**

### **Latest Release**
[![Latest Release](https://img.shields.io/github/v/release/hivellm/vectorizer)](https://github.com/hivellm/vectorizer/releases/latest)
[![Build Status](https://github.com/hivellm/vectorizer/actions/workflows/tag-release.yml/badge.svg)](https://github.com/hivellm/vectorizer/actions/workflows/tag-release.yml)

**Pre-built Binaries Available:**
- 🐧 **Linux** (x86_64, ARM64)
- 🪟 **Windows** (x86_64) 
- 🍎 **macOS** (x86_64, ARM64)

### **Automatic Releases**
Releases are automatically created when version tags are pushed:
```bash
git tag v0.22.0
git push origin v0.22.0
```

GitHub Actions will automatically:
- ✅ Build all binaries for 6 platforms
- ✅ Create installation scripts
- ✅ Generate GitHub release with downloads
- ✅ Include all configuration files

## 🚀 **Advanced Embedding Models**

Production-ready models with GPU acceleration:

### **Available Models**
- **MiniLM Multilingual** (384D): Fast, efficient multilingual embeddings
- **E5 Small/Base** (384D/768D): Optimized for retrieval tasks
- **MPNet Multilingual** (768D): Superior semantic understanding
- **GTE Multilingual** (768D): Alibaba's high-quality model
- **DistilUSE** (512D): Google's efficient universal embeddings

### **Features**
- **GPU Acceleration**: Automatic GPU detection and utilization
- **Batch Processing**: Optimized batch inference for high throughput
- **Quantization**: INT8 quantization for ONNX models (3x speedup)
- **Multilingual**: Support for 100+ languages

## 📚 **Configuration**

```yaml
vectorizer:
  host: "localhost"
  port: 15001
  grpc_port: 15002
  default_dimension: 512
  default_metric: "cosine"
  
  # GPU Acceleration
  cuda:
    enabled: true
    device_id: 0
  
  # Summarization
  summarization:
    enabled: true
    default_method: "extractive"
```

## 🎯 **Current Status**

**Version**: v0.22.0  
**Status**: ✅ **Production Ready**  
**Collections**: 99 active collections with 47,000+ vectors indexed  
**Performance**: Sub-3ms search with GPU acceleration  
**Architecture**: GRPC + REST + MCP unified server system  
**SDKs**: ✅ **TypeScript (npm), JavaScript (npm), Rust (crates.io)** | 🚧 **Python (PyPI in progress)**  
**Integrations**: ✅ **LangChain, PyTorch, TensorFlow**


## 🚀 Quick Start

```bash
# Start all services
./scripts/start.sh

# Or manually
cargo run --bin vzr -- start --workspace config/vectorize-workspace.yml

# Check status
./scripts/status.sh
```

**Services:**
- **vectorizer-server** (port 15001) - HTTP API and dashboard
- **vectorizer-mcp-server** (port 15002) - Model Context Protocol integration  
- **vzr** (port 15003) - GRPC orchestrator and indexing engine
- **vectorizer-cli** - Command-line interface for management

### MCP Integration
```bash
# MCP endpoint: ws://127.0.0.1:15002/mcp
# Available tools: search_vectors, list_collections, embed_text, create_collection
```



## 🎯 Use Cases

- **RAG Systems**: Large knowledge bases with semantic search
- **AI Applications**: Real-time context sharing and retrieval
- **Document Search**: Intelligent document indexing and search
- **Production Workflows**: Enterprise-grade vector operations



## 🔍 Embedding Methods

**Sparse Embeddings**: TF-IDF, BM25 with SVD dimensionality reduction  
**Dense Embeddings**: BERT, MiniLM with contextual understanding  
**Hybrid Search**: Sparse retrieval + dense re-ranking for optimal results

## 🛠️ Installation

### **Pre-built Binaries (Recommended)**

Download the latest release for your platform:

| Platform | Architecture | Download |
|----------|-------------|----------|
| **Linux** | x86_64 | [Download](https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-linux-x86_64.tar.gz) |
| **Linux** | ARM64 | [Download](https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-linux-aarch64.tar.gz) |
| **Windows** | x86_64 | [Download](https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-windows-x86_64.zip) |
| **Windows** | ARM64 | [Download](https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-windows-aarch64.zip) |
| **macOS** | x86_64 | [Download](https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-macos-x86_64.tar.gz) |
| **macOS** | ARM64 | [Download](https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-macos-aarch64.tar.gz) |

```bash
# Example: Linux x86_64
wget https://github.com/hivellm/vectorizer/releases/latest/download/vectorizer-linux-x86_64.tar.gz
tar -xzf vectorizer-linux-x86_64.tar.gz
./vectorizer-server --config config.yml
```

### **Build from Source**

```bash
# Clone repository
git clone https://github.com/hivellm/vectorizer
cd vectorizer

# Use Rust nightly
rustup override set nightly

# Build the project
./scripts/build.sh

# Start all services
./scripts/start.sh --workspace vectorize-workspace.yml

# Check status
./scripts/status.sh
```

### Docker
```bash
docker-compose up --build
```

### SDKs
```bash
# TypeScript SDK (Published)
npm install @hivellm/vectorizer-client-ts

# JavaScript SDK (Published)
npm install @hivellm/vectorizer-client-js

# Rust SDK (Published)
cargo add vectorizer-rust-sdk

# Python SDK (Coming Soon)
# pip install hivellm-vectorizer-client
```

## 🔧 MCP Integration

IDE integration via Model Context Protocol:

```json
{
  "mcpServers": {
    "vectorizer": {
      "url": "http://localhost:15002/sse",
      "type": "sse",
      "protocol": "http"
    }
  }
}
```

**Available Tools:** search_vectors, list_collections, embed_text, create_collection, insert_texts, delete_vectors, batch operations

## 📁 Workspace Configuration

Multi-project workspace support via `vectorize-workspace.yml`:

```yaml
workspace:
  name: "My Workspace"
  projects:
    - name: "project1"
      path: "../project1"
      collections:
        - name: "docs"
          dimension: 768
          embedding:
            model: "bm25"
```

## 🌐 REST API

Production-ready HTTP API:

```bash
# Health check
curl http://127.0.0.1:15001/api/v1/health

# List collections
curl http://127.0.0.1:15001/api/v1/collections

# Semantic search
curl -X POST http://127.0.0.1:15001/api/v1/collections/docs/search/text \
  -H "Content-Type: application/json" \
  -d '{"query": "machine learning algorithms", "limit": 5}'
```

## 🏗️ Technical Details

- **Architecture**: GRPC-based microservices with REST/MCP interfaces
- **Storage**: In-memory with binary persistence and smart caching
- **Indexing**: HNSW for ANN search with parallel processing
- **Performance**: 3x faster service communication with GRPC
- **Compression**: LZ4 for payloads >1KB

## 🧪 Testing

```bash
cargo test --all
cargo clippy
```

**Status**: 73+ tests passing, zero warnings

## ⚙️ Configuration

```yaml
server:
  host: "127.0.0.1"
  port: 15001

cuda:
  enabled: true
  device_id: 0
```

## 🚀 CUDA GPU Acceleration

High-performance GPU acceleration for vector operations:

```bash
# Build CUDA library
./scripts/build_cuda.sh
```

**Performance**: Up to 3.6x speedup for vector operations


## 📚 Documentation

- [Roadmap](docs/ROADMAP.md) - Implementation plan and status
- [Future Implementations](docs/FUTURE_IMPLEMENTATIONS.md) - Planned enhancements
- [Technical Documentation](docs/TECHNICAL_DOCUMENTATION_INDEX.md) - Complete overview


## 🤝 Contributing

1. Review documentation in `docs/`
2. Submit PRs with tests and documentation
3. Follow Rust best practices

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

## 📬 Contact

For questions or collaboration, open an issue at [hivellm/gov](https://github.com/hivellm/gov).

---

**Note**: This project is part of the HiveLLM ecosystem.