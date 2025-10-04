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
- **🌍 Universal Multi-GPU Support**: Metal (macOS), Vulkan (Linux), DirectX 12 (Windows), CUDA (NVIDIA)

## 🌍 **Universal Multi-GPU Backend Detection** (NEW in v0.27.0)

Cross-platform GPU acceleration with automatic backend selection and intelligent fallback:

### **Supported Backends**
- 🍎 **Metal**: Apple Silicon (M1/M2/M3/M4) - Native macOS GPU
- 🔥 **Vulkan**: AMD/NVIDIA/Intel GPUs - Cross-platform (Linux/Windows)
- 🪟 **DirectX 12**: NVIDIA/AMD/Intel - Native Windows GPU
- ⚡ **CUDA**: NVIDIA only - Maximum performance (optional)
- 💻 **CPU**: Universal fallback - Always available

### **Key Features**
- ✅ **Auto-Detection**: Automatically selects the best GPU backend for your system
- ✅ **Smart Fallback**: Graceful degradation to CPU for small workloads or GPU failure
- ✅ **Backend Priority**: Metal > Vulkan > DirectX12 > CUDA > CPU
- ✅ **CLI Control**: `--gpu-backend` flag for explicit backend selection
- ✅ **High Performance**: **6-10× speedup** over CPU for typical workloads

### **GPU Operations**
- Cosine Similarity (vec4 SIMD optimized)
- Euclidean Distance
- Dot Product
- Batch Search (parallel processing)

### **Performance Benchmarks** (Apple M3 Pro - Metal)
| Operation | Throughput | Latency | Speedup |
|-----------|------------|---------|---------|
| **Vector Insertion** | 1,373 ops/sec | 0.728 ms | ~8× |
| **Single Search** | 1,151 QPS | 0.869 ms | ~7× |
| **Batch Search (100)** | 1,129 QPS | 0.886 ms | ~8× |
| **Large Set (10K)** | 1,213 ops/sec | 8.24 s | ~6× |
| **Sustained Load** | 395 QPS | - | ~7× |

### **Build with Multi-GPU Support**
```bash
# Build with GPU support (Metal/Vulkan/DirectX12)
cargo build --release --features wgpu-gpu

# Build CPU-only (no GPU)
cargo build --release
```

### **Usage Examples**

**Auto-Detection (Recommended)**:
```bash
# Automatically detects best GPU backend
./target/release/vzr start --workspace vectorize-workspace.yml
```

**Explicit Backend Selection**:
```bash
# Force Vulkan (Linux/Windows)
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend vulkan

# Force DirectX 12 (Windows)
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend dx12

# Force Metal (macOS)
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend metal

# Force CPU (debugging)
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend cpu
```

**Rust API**:
```rust
use vectorizer::db::VectorStore;

#[tokio::main]
async fn main() -> Result<()> {
    // Auto-detect best GPU backend
    let store = VectorStore::new_auto_universal();
    
    // Or use specific backend
    let config = GpuConfig::for_vulkan();
    let store = VectorStore::new_with_vulkan_config(config);
    
    // Create collection and use GPU-accelerated operations
    store.create_collection("docs", config)?;
    store.insert("docs", vectors)?;
    let results = store.search("docs", &query, 10)?;
    
    Ok(())
}
```

### **Platform Support**
| Platform | Auto-Detected Backend | Fallback |
|----------|----------------------|----------|
| **macOS (Apple Silicon)** | 🍎 Metal | CPU |
| **Linux (AMD GPU)** | 🔥 Vulkan | CPU |
| **Linux (NVIDIA GPU)** | 🔥 Vulkan → ⚡ CUDA | CPU |
| **Windows (NVIDIA)** | 🪟 DirectX 12 → 🔥 Vulkan | CPU |
| **Windows (AMD)** | 🪟 DirectX 12 → 🔥 Vulkan | CPU |
| **Windows (Intel)** | 🪟 DirectX 12 → 🔥 Vulkan | CPU |

### **System Requirements**
- **macOS**: macOS 12+ with Apple Silicon or Metal-compatible GPU
- **Linux**: Vulkan SDK and compatible GPU drivers (AMD/NVIDIA/Intel)
- **Windows**: Windows 10 1709+ with DirectX 12 compatible GPU
- **Memory**: 8GB+ recommended for large datasets
- **GPU Memory**: 2GB+ VRAM recommended

### **Benchmarks & Testing**
```bash
# Run comprehensive GPU benchmark
cargo run --example multi_gpu_benchmark --features wgpu-gpu --release

# Run GPU stress test
cargo run --example gpu_stress_benchmark --features wgpu-gpu --release

# Test GPU detection
cargo run --example test_multi_gpu_detection --features wgpu-gpu --release
```

📚 **Full Documentation**: 
- Setup: `docs/VULKAN_SETUP.md`, `docs/DIRECTX12_SETUP.md`
- Benchmarks: `docs/GPU_BENCHMARKS.md`
- Comparison: `docs/GPU_COMPARISON.md`

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
  
  # Multi-GPU Configuration
  gpu:
    enabled: true
    backend: auto  # auto, metal, vulkan, dx12, cuda, cpu
    device_id: 0
    power_preference: high_performance
    gpu_threshold_operations: 500
  
  # Legacy CUDA support (optional)
  cuda:
    enabled: false
    device_id: 0
  
  # Summarization
  summarization:
    enabled: true
    default_method: "extractive"
```

## 🎯 **Current Status**

**Version**: v0.27.0  
**Status**: ✅ **Production Ready**  
**Collections**: 99 active collections with 47,000+ vectors indexed  
**Performance**: Sub-1ms search with multi-GPU acceleration  
**GPU Backends**: 🍎 Metal, 🔥 Vulkan, 🪟 DirectX 12, ⚡ CUDA, 💻 CPU  
**Architecture**: GRPC + REST + MCP unified server system  
**SDKs**: ✅ **TypeScript (npm), JavaScript (npm), Rust (crates.io)** | 🚧 **Python (PyPI in progress)**  
**Integrations**: ✅ **LangChain, PyTorch, TensorFlow**


## 🚀 Quick Start

```bash
# Build the project first (required for vzr to find executables)
cargo build --release

# Start all services using vzr CLI
./target/release/vzr start --workspace vectorize-workspace.yml

# Or use the start script (builds and starts)
./scripts/start.sh

# Check status
./scripts/status.sh
```

**Services:**
- **vectorizer-server** (port 15001) - HTTP API and dashboard
- **vectorizer-mcp-server** (port 15002) - Model Context Protocol integration  
- **vzr** (port 15003) - GRPC orchestrator and indexing engine
- **vectorizer-cli** - Command-line interface for management

**Note**: The `vzr` CLI now executes pre-built binaries directly instead of compiling on each run, providing faster startup and better reliability.

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
cargo build --release

# The vzr CLI will automatically find executables in ./target/release/
./target/release/vzr start --workspace vectorize-workspace.yml

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

# Multi-GPU configuration
gpu:
  enabled: true
  backend: auto  # Detects best available: metal, vulkan, dx12, cuda, cpu
  device_id: 0
  power_preference: high_performance
  gpu_threshold_operations: 500  # Minimum operations for GPU (CPU fallback)
```

## 🚀 Multi-GPU Acceleration

Universal GPU acceleration across platforms:

```bash
# Build with multi-GPU support
cargo build --release --features wgpu-gpu

# Auto-detect best GPU backend
./target/release/vzr start --workspace vectorize-workspace.yml

# Or use specific backend
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend vulkan
```

**Performance**: 
- **Metal (M3 Pro)**: 1,373 ops/sec, <1ms latency
- **Expected Vulkan**: ~1,200-1,500 ops/sec
- **Expected DirectX 12**: ~1,400-1,600 ops/sec
- **Speedup**: 6-10× faster than CPU


## 📚 Documentation

### GPU Acceleration
- [GPU Benchmarks](docs/GPU_BENCHMARKS.md) - Complete performance analysis
- [GPU Backend Comparison](docs/GPU_COMPARISON.md) - Backend selection guide
- [Vulkan Setup](docs/VULKAN_SETUP.md) - Linux/Windows Vulkan installation
- [DirectX 12 Setup](docs/DIRECTX12_SETUP.md) - Windows DirectX 12 setup

### General Documentation
- [Roadmap](docs/ROADMAP.md) - Implementation plan and status
- [Future Implementations](docs/FUTURE_IMPLEMENTATIONS.md) - Planned enhancements
- [Technical Documentation](docs/TECHNICAL_DOCUMENTATION_INDEX.md) - Complete overview
- [Changelog](CHANGELOG.md) - Version history and changes


## 🤝 Contributing

1. Review documentation in `docs/`
2. Submit PRs with tests and documentation
3. Follow Rust best practices

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

## 📬 Contact

For questions or collaboration, open an issue at [hivellm/gov](https://github.com/hivellm/gov).

---

