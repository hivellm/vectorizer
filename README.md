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
- **🎯 Simplified Workspace**: Minimal configuration with intelligent defaults (NEW in v0.26.0)
- **💾 Backup & Restore**: CLI commands for data directory backup/restore (NEW in v0.28.1)

## 🎯 **Simplified Workspace Configuration** (NEW in v0.26.0)

Dramatically reduce workspace configuration verbosity with intelligent defaults:

### **Features**
- ✅ **Minimal Collections**: Only `name`, `description`, `include_patterns`, `exclude_patterns` required
- ✅ **Intelligent Defaults**: Centralized configuration inheritance system
- ✅ **Backward Compatible**: Existing configurations continue to work
- ✅ **Override Support**: Still override any default when needed

### **Before vs After**
**Before (Complex)** - ~50 lines per collection:
```yaml
collections:
  - name: "docs"
    description: "Documentation"
    dimension: 512
    metric: "cosine"
    embedding:
      model: "bm25"
      dimension: 512
      parameters: { k1: 1.5, b: 0.75 }
    indexing:
      index_type: "hnsw"
      parameters: { m: 16, ef_construction: 200, ef_search: 64 }
    processing:
      chunk_size: 2048
      chunk_overlap: 256
      include_patterns: ["docs/**/*.md"]
      exclude_patterns: ["docs/draft/**"]
```

**After (Ultra-Simplified)** - ~3 lines per collection:
```yaml
workspace:
  name: "My Workspace"
  version: "1.0.0"

projects:
  - name: "my-project"
    path: "../my-project"
    collections:
      - name: "docs"
        description: "Documentation"
        include_patterns: ["docs/**/*.md"]
        exclude_patterns: ["docs/draft/**"]
```

### **Usage**
```bash
# Use simplified workspace configuration
vzr start --workspace vectorize-workspace-simplified.yml

# Automatic detection - works with both formats
vzr workspace validate --config your-workspace.yml
```

## 🌍 **Universal Multi-GPU Backend Detection**

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

### **Performance Benchmarks** (Real-world Testing)
| Collection Size | Search Latency | QPS | Memory Usage | Quality (MAP) |
|-----------------|----------------|-----|--------------|---------------|
| **1K vectors** | 164μs | 10,000 QPS | 2.0MB | 0.268 |
| **5K vectors** | 377μs | 3,333 QPS | 9.8MB | 0.176 |
| **10K vectors** | 588μs | 1,667 QPS | 19.5MB | 0.050 |
| **25K vectors** | 3.1ms | 333 QPS | 48.8MB | 0.044 |
| **50K vectors** | 5.3ms | 189 QPS | 97.7MB | 0.044 |
| **100K vectors** | 17.4ms | 57 QPS | 195.3MB | 0.024 |

### **GPU Acceleration** (Vulkan/DirectX 12)
| Collection Size | GPU QPS | CPU QPS | Speedup | GPU Memory |
|-----------------|---------|---------|---------|------------|
| **1K vectors** | 500 QPS | 10,000 QPS | 0.05× | 277.2MB |
| **5K vectors** | 500 QPS | 3,333 QPS | 0.15× | 277.5MB |
| **10K vectors** | 435 QPS | 1,667 QPS | 0.26× | 277.8MB |
| **25K vectors** | 370 QPS | 333 QPS | 1.11× | 278.8MB |
| **50K vectors** | 303 QPS | 189 QPS | 1.60× | 280.5MB |
| **100K vectors** | 175 QPS | 57 QPS | 3.07× | 283.8MB |

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

### **Performance Recommendations**
Based on real-world benchmarks:

- **Small Collections** (< 5K): **CPU recommended** - 10,000 QPS vs 500 QPS GPU
- **Medium Collections** (5K-25K): **GPU recommended** - 1.11× speedup at 25K
- **Large Collections** (25K+): **GPU strongly recommended** - Up to 3× speedup
- **Optimal Size**: **1K vectors** for best performance (164μs latency, 10K QPS)
- **Maximum Recommended**: **5K vectors** before performance degradation

### **Benchmark Insights**
- **CPU excels** at small datasets due to lower overhead
- **GPU advantage** increases with collection size (3× speedup at 100K vectors)
- **Memory efficiency**: GPU uses consistent ~280MB regardless of collection size
- **Quality trade-off**: Larger collections show lower MAP scores but maintain reasonable recall
- **Build time scales linearly**: 0.1s for 1K vectors → 12.8s for 100K vectors

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
    gpu_threshold_operations: 5000  # Enable GPU for collections > 5K vectors
  
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

**Version**: v0.28.1  
**Status**: ✅ **Production Ready**  
**Collections**: 105 active collections with 50,000+ vectors indexed  
**Performance**: 164μs latency at 10,000 QPS (1K vectors), 3× GPU speedup for large collections  
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

## 💾 Backup & Restore (CLI)

Use the `vzr` CLI to criar e restaurar backups do diretório `data/` em um único arquivo `.tar.gz`:

```bash
# Backup (gera backups/vectorizer_data_<timestamp>.tar.gz por padrão)
./target/release/vzr backup --data-dir data

# Backup com caminho de saída customizado
./target/release/vzr backup --data-dir data --output backups/meu_backup.tar.gz

# Restore para o diretório data (cria se não existir)
./target/release/vzr restore --archive backups/meu_backup.tar.gz --data-dir data

# Restore limpando o destino antes
./target/release/vzr restore --archive backups/meu_backup.tar.gz --data-dir data --clean
```

Notas:
- O arquivo inclui todos os conteúdos de `data/` (por coleção: `_vector_store.bin`, `*_metadata.json`, `*_tokenizer.json`, etc.).
- O restore respeita o diretório de destino informado e pode limpar antes com `--clean`.

## 🎯 Use Cases

- **RAG Systems**: Large knowledge bases with semantic search
- **AI Applications**: Real-time context sharing and retrieval
- **Document Search**: Intelligent document indexing and search
- **Production Workflows**: Enterprise-grade vector operations

## 🔍 Embedding Methods

**Sparse Embeddings**: TF-IDF, BM25 with SVD dimensionality reduction  
**Dense Embeddings**: BERT, MiniLM with contextual understanding  
**Hybrid Search**: Sparse retrieval + dense re-ranking for optimal results

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
