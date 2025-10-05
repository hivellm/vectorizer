# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## 🚀 **Key Features**

- **🔍 Semantic Search**: Advanced vector similarity search with multiple distance metrics
- **📚 Document Indexing**: Intelligent chunking and processing of various file types
- **🧠 Multiple Embeddings**: Support for TF-IDF, BM25, BERT, MiniLM, and custom models
- **⚡ High Performance**: Sub-3ms search times with optimized HNSW indexing
- **🏗️ Unified Architecture**: Single server with REST API and MCP integration
- **🔧 MCP Integration**: Model Context Protocol for AI IDE integration (Cursor, VS Code)
- **🌐 REST API**: Complete HTTP API with authentication and security
- **📱 TypeScript SDK**: ✅ Published on npm - Complete TypeScript client for web applications
- **🟨 JavaScript SDK**: ✅ Published on npm - Modern JavaScript client with multiple build formats
- **🦀 Rust SDK**: ✅ Published on crates.io - High-performance native client with memory safety and MCP support
- **🐍 Python SDK**: 🚧 In development - PyPI publishing in progress
- **🔗 LangChain Integration**: Complete VectorStore for Python and JavaScript/TypeScript
- **🚀 Advanced Embedding Models**: BM25, TF-IDF, and custom embedding providers
- **🎯 Simplified Configuration**: Minimal setup with intelligent defaults
- **💾 Automatic Persistence**: Collections automatically saved and loaded

## 🎯 **Simple Configuration**

Vectorizer uses intelligent defaults with minimal configuration required:

### **Features**
- ✅ **Minimal Setup**: Just run `vectorizer` and it works
- ✅ **Intelligent Defaults**: Automatic configuration with sensible defaults
- ✅ **Background Loading**: Collections load automatically without blocking server
- ✅ **Auto-Persistence**: Data is automatically saved and restored

## 🚀 **Quick Start**

Get Vectorizer running in minutes:

### **1. Build and Run**
```bash
# Clone and build
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer
cargo build --release

# Start the server
./target/release/vectorizer
```

### **2. Access Services**
- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp/sse  
- **Dashboard**: http://localhost:15002/
- **Health Check**: http://localhost:15002/health

### **3. Basic Usage**
```bash
# Check server status
curl http://localhost:15002/health

# List collections
curl http://localhost:15002/collections

# Search vectors (after adding some data)
curl -X POST http://localhost:15002/collections/my-collection/search \
  -H "Content-Type: application/json" \
  -d '{"query": "example text", "limit": 10}'
```

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

### MCP Integration
```bash
# MCP endpoint: http://localhost:15002/mcp/sse
# Available tools: search_vectors, list_collections, embed_text, create_collection
```

## 💾 Data Management

Vectorizer automatically manages data persistence in the `.vectorizer/` directory:
- Collections are automatically saved and loaded
- Background loading ensures server availability during startup
- Quantization is applied automatically for memory optimization

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

## 📁 Configuration

Vectorizer uses intelligent defaults and minimal configuration:

```yaml
# Optional configuration file (config.yml)
server:
  host: "0.0.0.0"
  port: 15002

logging:
  level: "info"
```
