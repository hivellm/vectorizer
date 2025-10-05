# Vectorizer v0.3.0

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

- **💾 Dynamic Collection Persistence**: Collections automatically saved and loaded on server restart
- **🔍 Real-time File Watcher**: Monitor file changes and auto-index documents
- **⚡ Background Auto-save**: Collections saved every 30 seconds automatically
- **🔄 Seamless Restart**: All collections restored exactly as they were
- **📁 File System Monitoring**: Real-time indexing of document changes
- **🔍 Semantic Search**: Advanced vector similarity search with multiple distance metrics
- **📚 Document Indexing**: Intelligent chunking and processing of various file types
- **🧠 Multiple Embeddings**: Support for TF-IDF, BM25, BERT, MiniLM, and custom models
- **⚡ High Performance**: Sub-3ms search times with optimized HNSW indexing
- **🏗️ Unified Architecture**: Single server with REST API and MCP integration
- **🔧 MCP Integration**: Model Context Protocol for AI IDE integration (Cursor, VS Code)
- **🌐 REST API**: Complete HTTP API with authentication and security
- **🚀 Advanced Embedding Models**: BM25, TF-IDF, and custom embedding providers
- **🎯 Simplified Configuration**: Minimal setup with intelligent defaults
- **💾 Automatic Persistence**: Collections automatically saved and loaded
- **👀 File Watcher**: Real-time file monitoring and indexing

## 🎯 **Simple Configuration**

Vectorizer uses intelligent defaults with minimal configuration required:

### **Features**
- ✅ **Minimal Setup**: Just run `vectorizer` and it works
- ✅ **Intelligent Defaults**: Automatic configuration with sensible defaults
- ✅ **Background Loading**: Collections load automatically without blocking server
- ✅ **Auto-Persistence**: Data is automatically saved and restored
- ✅ **File Watcher**: Real-time file monitoring with intelligent patterns
- ✅ **Dynamic Collections**: Create collections via REST API with automatic persistence

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
- **Dashboard**: http://localhost:15002/dashboard
- **Health Check**: http://localhost:15002/health

## 💾 **Persistence & File Watcher (v0.3.0)**

### **Dynamic Collections**
Create collections via REST API that persist automatically:

```bash
# Create a new collection
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "my-docs", "dimension": 512, "metric": "cosine"}'

# Insert documents
curl -X POST http://localhost:15002/insert \
  -H "Content-Type: application/json" \
  -d '{"collection": "my-docs", "text": "Your document content", "metadata": {"source": "file.txt"}}'
```

### **File Watcher**
Monitor file changes in real-time:
- **Supported formats**: `.md`, `.txt`, `.rs`, `.py`, `.js`, `.ts`, `.json`, `.yaml`, `.yml`
- **Auto-exclusion**: `target/`, `node_modules/`, `.git/`, etc.
- **Debounce**: 1000ms delay to handle rapid changes
- **Collection**: `watched_files` (configurable)

### **Persistence Features**
- **Auto-save**: Collections saved every 30 seconds
- **Restart recovery**: All collections restored on server restart
- **Format compatibility**: Versioned persistence format for future compatibility
- **Reliable writes**: File flush/sync ensures data integrity

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

Production-ready embedding models:

### **Available Models**
- **MiniLM Multilingual** (384D): Fast, efficient multilingual embeddings
- **E5 Small/Base** (384D/768D): Optimized for retrieval tasks
- **MPNet Multilingual** (768D): Superior semantic understanding
- **GTE Multilingual** (768D): Alibaba's high-quality model
- **DistilUSE** (512D): Google's efficient universal embeddings

### **Features**
- **Batch Processing**: Optimized batch inference for high throughput
- **Multilingual**: Support for 100+ languages

## 📚 **Configuration**

```yaml
vectorizer:
  host: "localhost"
  port: 15002
  default_dimension: 512
  default_metric: "cosine"
  
  # Summarization
  summarization:
    enabled: true
    default_method: "extractive"
```

## 🎯 **Current Status**

**Version**: v0.28.1  
**Status**: ✅ **Production Ready**  
**Collections**: 105 active collections with 50,000+ vectors indexed  
**Performance**: 164μs latency at 10,000 QPS (1K vectors)  
**Architecture**: REST + MCP unified server system  
**SDKs**: ✅ **TypeScript (npm), JavaScript (npm), Rust (crates.io)** | 🚧 **Python (PyPI in progress)**  
**Integrations**: ✅ **LangChain, PyTorch, TensorFlow**

## 🚀 Quick Start

### MCP Integration
```bash
# MCP endpoint: http://localhost:15002/mcp/sse
# Available tools: search_vectors, list_collections, embed_text, create_collection
```

## 💾 Data Management

Vectorizer automatically manages data persistence in the `data/` directory:
- **Collections are automatically saved and loaded** (v0.3.0)
- **Background loading** ensures server availability during startup
- **Auto-save every 30 seconds** for dynamic collections
- **File watcher** monitors changes and updates indexes
- **Quantization** is applied automatically for memory optimization
- **Versioned persistence format** for future compatibility

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

## 📋 Changelog

### v0.3.0 (2025-10-05) - Complete Persistence & File Watcher
- ✅ **Dynamic Collection Persistence**: Collections automatically saved and loaded on server restart
- ✅ **Real-time File Watcher**: Monitor file changes and auto-index documents
- ✅ **Background Auto-save**: Collections saved every 30 seconds automatically
- ✅ **Seamless Restart**: All collections restored exactly as they were
- ✅ **File System Monitoring**: Real-time indexing of document changes
- 🔧 **Technical Fixes**: PersistedVectorStore format compatibility, file flush/sync, ownership resolution
- 🎯 **Production Ready**: Stable, tested, and verified working

### Previous Versions
- v0.2.x: REST API and MCP integration
- v0.1.x: Core vector database functionality
