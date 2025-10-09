# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ **Version 0.4.0 - File Watcher System Release**

### 🚀 **Major Feature: Real-time File Monitoring**
- ✅ **Complete File Watcher System**: Real-time file monitoring with automatic indexing and reindexing
- ✅ **File Discovery**: Automatic discovery and indexing of files in workspace directories
- ✅ **Smart Debouncing**: Intelligent event debouncing to prevent excessive processing
- ✅ **Hash Validation**: Content-based change detection using file hashing
- ✅ **Pattern Filtering**: Configurable include/exclude patterns for file types and directories
- ✅ **31 Comprehensive Tests**: Complete test suite with 100% success rate
- ✅ **Zero External Dependencies**: Pure Rust implementation with no external tool dependencies

### 🚀 **File Operations Module** (v0.3.2+)
- ✅ **6 Production-Ready MCP Tools**: Complete file-level operations for AI assistants
  - `get_file_content` - Retrieve complete files with metadata
  - `list_files_in_collection` - Advanced file listing and filtering
  - `get_file_summary` - Extractive and structural summaries
  - `get_project_outline` - Hierarchical project visualization
  - `get_related_files` - Semantic file similarity search
  - `search_by_file_type` - File type-specific search
- ⚡ **Smart Caching**: Multi-tier LRU caching (10min, 5min, 30min TTLs)
- 🔒 **Security**: Path validation preventing directory traversal
- 📊 **Rich Metadata**: File types, sizes, language detection

### 🔍 **Discovery System**
- ✅ **9-Stage Pipeline**: Collection filtering → Query expansion → Broad search → Focus search → README promotion → Evidence compression → Answer planning → Prompt rendering
- 🧠 **Intelligent Query Expansion**: Automatic variations (definition, features, architecture, API)
- 🎯 **MMR Diversification**: Maximal Marginal Relevance for diverse results
- 📚 **Evidence Compression**: Key sentences (8-30 words) with citations
- 🔄 **Hybrid Search**: RRF combining sparse and dense retrieval

### 🧪 **Test Suite**
- ✅ **282 tests passing** (100% pass rate)
- ⚡ **2.01s execution time** (optimized from >60s)
- 🎯 **Production-ready** with comprehensive coverage
- ✅ **File Watcher Tests**: 31 dedicated tests for real-time monitoring system

## 🌟 **Key Features**

- **🔍 Real-time File Watcher**: Complete file monitoring system with automatic discovery, indexing, and reindexing
- **📁 File System Monitoring**: Live detection of file changes (create, modify, delete, move) with smart debouncing
- **🎯 Pattern-based Filtering**: Configurable include/exclude patterns for file types and directories
- **🔐 Hash Validation**: Content-based change detection using SHA-256 hashing to avoid unnecessary reindexing
- **💾 Dynamic Collection Persistence**: Collections automatically saved and loaded on server restart
- **⚡ Background Auto-save**: Collections saved every 30 seconds automatically
- **🔄 Seamless Restart**: All collections restored exactly as they were
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
- **🧠 Intelligent Search**: Advanced semantic search with multi-query generation (v0.3.1)
- **🔬 Semantic Reranking**: High-precision search with similarity thresholds
- **🌐 Multi-Collection Search**: Cross-collection search with intelligent reranking
- **🎯 Contextual Search**: Context-aware search with metadata filtering

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

## 💾 **Persistence & File Watcher**

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

## 🧠 **Intelligent Search Features (v0.3.1)**

Vectorizer now includes advanced intelligent search capabilities that provide 3-4x better coverage than traditional search methods:

### **🔍 intelligent_search**
- **Multi-query generation**: Automatically generates 4-8 related queries
- **Domain expansion**: Expands queries with technical terms and synonyms
- **MMR diversification**: Ensures diverse, high-quality results
- **Technical focus**: Boosts scores for technical content
- **Collection bonuses**: Prioritizes relevant collections

### **🔬 semantic_search**
- **Semantic reranking**: Advanced relevance scoring
- **Similarity thresholds**: Configurable quality filters (0.1-0.5)
- **Cross-encoder support**: Maximum precision matching

### **🌐 multi_collection_search**
- **Cross-collection search**: Simultaneous search across multiple collections
- **Intelligent reranking**: Balanced results from different sources
- **Deduplication**: Removes duplicate content across collections

### **🎯 contextual_search**
- **Metadata filtering**: Filter by file type, chunk index, etc.
- **Context reranking**: Reorder based on contextual relevance
- **Configurable weights**: Balance between relevance and context

### **Usage Examples**
```bash
# Intelligent search with domain expansion
curl -X POST http://localhost:15002/intelligent_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "CMMV framework architecture",
    "collections": ["cmmv-core-docs"],
    "max_results": 10,
    "domain_expansion": true,
    "technical_focus": true,
    "mmr_enabled": true
  }'

# Semantic search with high precision
curl -X POST http://localhost:15002/semantic_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "authentication system",
    "collection": "cmmv-core-docs",
    "similarity_threshold": 0.15,
    "semantic_reranking": true
  }'
```

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

## 💾 Data Management

Vectorizer automatically manages data persistence in the `data/` directory:
- **Collections are automatically saved and loaded** (v0.3.0)
- **Background loading** ensures server availability during startup
- **Auto-save every 30 seconds** for dynamic collections
- **File watcher** monitors changes and updates indexes
- **Quantization** is applied automatically for memory optimization
- **Versioned persistence format** for future compatibility

## 📊 **Performance Metrics**

### **Intelligent Search Performance (v0.3.1)**
Based on comprehensive testing with 107 collections:

| Metric | Traditional Search | Intelligent Search | Improvement |
|--------|-------------------|-------------------|-------------|
| **Coverage** | 4 results | 18 results (5 final) | 3-4x more |
| **Query Generation** | 1 query | 4-8 queries | Automatic |
| **Deduplication** | None | 18→9→5 results | Smart filtering |
| **Relevance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Superior |
| **Diversity** | ⭐⭐ | ⭐⭐⭐⭐⭐ | Much better |

### **System Performance**
- **🚀 Server Startup**: Non-blocking with background collection loading
- **⚡ Search Speed**: Sub-3ms search times with optimized HNSW indexing
- **💾 Memory Usage**: Automatic quantization for memory optimization
- **📈 Scalability**: Tested with 107+ collections
- **🔄 Real-time**: Live file watching and indexing

## 🎯 Use Cases

- **RAG Systems**: Large knowledge bases with semantic search
- **AI Applications**: Real-time context sharing and retrieval
- **Document Search**: Intelligent document indexing and search
- **Production Workflows**: Enterprise-grade vector operations

## 🔍 Embedding Methods

**Sparse Embeddings**: TF-IDF, BM25 with SVD dimensionality reduction  
**Dense Embeddings**: BERT, MiniLM with contextual understanding  
**Hybrid Search**: Sparse retrieval + dense re-ranking for optimal results

## 📚 **Documentation**

- **[API Documentation](./docs/api/)** - Complete API reference
- **[Intelligent Search Guide](./docs/specs/INTELLIGENT_SEARCH_TOOLS.md)** - Complete guide to intelligent search tools
- **[Quality Report](./docs/specs/INTELLIGENT_SEARCH_QUALITY_REPORT.md)** - Performance analysis and recommendations
- **[MCP Integration](./docs/specs/MCP_INTEGRATION.md)** - Model Context Protocol guide
- **[Performance Guide](./docs/specs/PERFORMANCE_GUIDE.md)** - Optimization and tuning
- **[Technical Documentation](./docs/specs/TECHNICAL_DOCUMENTATION_INDEX.md)** - Complete technical reference
- **[Roadmap](./docs/specs/ROADMAP.md)** - Development roadmap and milestones

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

**Available Tools:** 
- **Traditional**: search_vectors, list_collections, embed_text, create_collection, insert_texts, delete_vectors
- **Intelligent Search**: intelligent_search, semantic_search, multi_collection_search, contextual_search
- **Batch Operations**: batch_insert_texts, batch_search_vectors, batch_update_vectors, batch_delete_vectors