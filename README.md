# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ **Version 0.5.0 - Text Normalization & Performance Release**

### 🎯 **Key Features**
- **Text Normalization System**: Content-aware normalization with 30-50% storage reduction
- **Real-time File Watcher**: Automatic file monitoring and indexing
- **Intelligent Search**: Advanced semantic search with multi-query generation
- **File Operations**: 6 MCP tools for AI-powered file analysis
- **Multi-tier Cache**: LFU hot cache, mmap warm store, Zstandard cold storage
- **Discovery Pipeline**: 9-stage semantic discovery with evidence compression

### 🧪 **Quality Metrics**
- ✅ **282 tests passing** (100% pass rate)
- ⚡ **2.01s execution time**
- 🎯 **Production-ready** with comprehensive coverage

## 🌟 **Core Capabilities**

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics (Cosine, Euclidean, Dot Product)
- **📚 Document Indexing**: Intelligent chunking and processing of 10+ file types
- **🧠 Embeddings**: TF-IDF, BM25, BERT, MiniLM, and custom models
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **🏗️ Unified Architecture**: REST API + MCP Server
- **💾 Automatic Persistence**: Collections auto-save every 30 seconds
- **👀 File Watcher**: Real-time monitoring with smart debouncing
- **🔒 Security**: JWT + API Key authentication with RBAC

## 🚀 **Quick Start**

```bash
# Build and run
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer
cargo build --release
./target/release/vectorizer

# Or use the CLI
./target/release/vzr start --workspace vectorize-workspace.yml
```

### **Access Points**
- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp/sse
- **Health Check**: http://localhost:15002/health

### **Basic Usage**
```bash
# Create collection
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "docs", "dimension": 512, "metric": "cosine"}'

# Insert text
curl -X POST http://localhost:15002/insert \
  -H "Content-Type: application/json" \
  -d '{"collection": "docs", "text": "Your content", "metadata": {}}'

# Search
curl -X POST http://localhost:15002/collections/docs/search \
  -H "Content-Type: application/json" \
  -d '{"query": "search term", "limit": 10}'
```

## 🧠 **Advanced Search Capabilities**

### **Intelligent Search**
- Multi-query generation (4-8 variations)
- Domain expansion with technical terms
- MMR diversification for diverse results
- Cross-collection search with reranking

### **Search Methods**
- `intelligent_search`: Multi-query with domain expansion
- `semantic_search`: High-precision with similarity thresholds
- `multi_collection_search`: Cross-collection with deduplication
- `contextual_search`: Metadata filtering with context-aware ranking

### **Discovery Pipeline**
- 9-stage pipeline: Filtering → Expansion → Search → Ranking → Compression
- README promotion for documentation
- Evidence compression with citations
- LLM-ready prompt generation

## 📚 **Configuration**

```yaml
# config.yml - Main configuration
vectorizer:
  host: "localhost"
  port: 15002
  default_dimension: 512
  default_metric: "cosine"

# Text normalization (v0.5.0)
normalization:
  enabled: true
  level: "conservative"  # conservative/moderate/aggressive
  line_endings:
    normalize_crlf: true
    collapse_multiple_newlines: true
    trim_trailing_whitespace: true

# Multi-tier cache
cache:
  enabled: true
  max_entries: 10000
  ttl_seconds: 3600
```

## 📊 **Performance**

| Metric | Value |
|--------|-------|
| **Search Speed** | < 3ms |
| **Startup Time** | Non-blocking |
| **Storage Reduction** | 30-50% with normalization |
| **Test Coverage** | 282 tests, 100% pass rate |
| **Collections** | 107+ tested |

## 🎯 **Use Cases**

- **RAG Systems**: Semantic search for AI applications
- **Document Search**: Intelligent indexing and retrieval
- **Code Analysis**: Semantic code search and navigation
- **Knowledge Bases**: Enterprise knowledge management

## 📚 **Documentation**

- **[API Reference](./docs/api/)** - REST API documentation
- **[MCP Integration](./docs/specs/MCP_INTEGRATION.md)** - Model Context Protocol guide
- **[Technical Specs](./docs/specs/)** - Complete technical documentation
- **[Roadmap](./docs/specs/ROADMAP.md)** - Development roadmap

## 🔧 **MCP Integration**

Cursor IDE configuration:

```json
{
  "mcpServers": {
    "vectorizer": {
      "url": "http://localhost:15002/sse",
      "type": "sse"
    }
  }
}
```

**Available MCP Tools** (40+ tools):
- **Core**: search_vectors, list_collections, embed_text, create_collection
- **Intelligent**: intelligent_search, semantic_search, contextual_search
- **File Ops**: get_file_content, list_files, get_file_summary
- **Discovery**: discover, filter_collections, expand_queries
- **Batch**: batch_insert, batch_search, batch_update, batch_delete

## 📦 **Client SDKs**

- **Python**: `pip install vectorizer-client`
- **TypeScript**: `npm install @hivellm/vectorizer-client-ts`
- **JavaScript**: `npm install @hivellm/vectorizer-client-js`
- **Rust**: `cargo add vectorizer-rust-sdk`

## 📄 **License**

MIT License - See [LICENSE](./LICENSE) for details