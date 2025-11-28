# Vectorizer

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![Rust Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/vectorizer.svg)](https://crates.io/crates/vectorizer)
[![GitHub release](https://img.shields.io/github/release/hivellm/vectorizer.svg)](https://github.com/hivellm/vectorizer/releases)
[![Tests](https://img.shields.io/badge/tests-703%20passing-brightgreen.svg)](https://github.com/hivellm/vectorizer/actions)
[![Coverage](https://img.shields.io/badge/coverage-95%25%2B-success.svg)](https://github.com/hivellm/vectorizer)

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ Key Features

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics (Cosine, Euclidean, Dot Product)
- **⚡ SIMD Acceleration**: AVX2-optimized vector operations (5-10x faster) with automatic CPU feature detection
- **💾 Memory-Mapped Storage**: MMap support for datasets larger than RAM with efficient OS paging
- **🚀 GPU Acceleration**: Metal GPU support for macOS (Apple Silicon) with cross-platform compatibility
- **📦 Product Quantization**: PQ compression for 64x memory reduction with minimal accuracy loss
- **💾 Compact Storage**: Unified `.vecdb` format with 20-30% space savings and automatic snapshots
- **🔄 Master-Replica Replication**: High availability with automatic failover (BETA)
- **🔗 Distributed Sharding**: Horizontal scaling across multiple servers with automatic shard routing (BETA)
- **📄 Document Conversion**: Automatic conversion of PDF, DOCX, XLSX, PPTX, HTML, XML, and images
- **🔄 Qdrant Migration**: Complete migration tools and full Qdrant 1.14.x API compatibility
  - Snapshots API (create, list, delete, recover)
  - Sharding API (create shard keys, distribute data)
  - Cluster Management API (status, recovery, peer management, metadata)
  - Query API (query, batch query, grouped queries with prefetch)
  - Search Groups and Matrix API (grouped results, similarity matrices)
  - Named Vectors support (partial)
  - Quantization configuration (PQ and Binary)
- **🎯 MCP Integration**: 20 focused individual tools for AI model integration
- **🔄 UMICP Protocol**: Native JSON types + Tool Discovery endpoint
- **🖥️ Web Dashboard**: Modern React + TypeScript dashboard with complete graph management interface
  - Create/delete edges with relationship types and weights
  - Explore node neighbors and related nodes
  - Find shortest paths between nodes
  - Node-specific edge discovery with configurable parameters
  - Real-time graph visualization with vis-network
- **🖥️ Desktop GUI**: Electron-based desktop application with vis-network graph visualization for visual database management
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **🧠 Multiple Embeddings**: TF-IDF, BM25, BERT, MiniLM, and custom models
- **🕸️ Graph Relationships**: Automatic relationship discovery and graph traversal with full GUI support for edge management, node exploration, and path finding
- **🔗 n8n Integration**: Official n8n community node for no-code workflow automation (400+ node integrations)
- **🎨 Langflow Integration**: LangChain-compatible components for visual LLM app building
- **🔒 Security**: JWT + API Key authentication with RBAC

## 🚀 Quick Start

### Install Script (Linux/macOS)

Installs Vectorizer CLI and configures it as a system service that starts automatically on boot:

```bash
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

**After installation:**

- ✅ CLI available: `vectorizer --help`
- ✅ Service running: `sudo systemctl status vectorizer`
- ✅ Auto-starts on boot
- ✅ Service commands:
  - `sudo systemctl restart vectorizer` - Restart service
  - `sudo systemctl stop vectorizer` - Stop service
  - `sudo journalctl -u vectorizer -f` - View logs

### Install Script (Windows PowerShell)

Installs Vectorizer CLI and configures it as a Windows Service that starts automatically on boot:

```powershell
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

**Note:** Service installation requires Administrator privileges. If not running as admin, the script will provide instructions.

**After installation:**

- ✅ CLI available: `vectorizer --help`
- ✅ Service running: `Get-Service Vectorizer`
- ✅ Auto-starts on boot
- ✅ Service commands:
  - `Restart-Service Vectorizer` - Restart service
  - `Stop-Service Vectorizer` - Stop service
  - `Start-Service Vectorizer` - Start service

### Using Docker (Recommended)

**Docker Hub:**
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

**GitHub Container Registry:**
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest
```

**Available at:**
- 🐳 [Docker Hub](https://hub.docker.com/r/hivehub/vectorizer) - `hivehub/vectorizer:latest`
- 📦 [GitHub Container Registry](https://github.com/hivellm/vectorizer/pkgs/container/vectorizer) - `ghcr.io/hivellm/vectorizer:latest`

### Building from Source

```bash
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

# Basic build
cargo build --release
./target/release/vectorizer

# With GPU acceleration (macOS Metal)
cargo build --release --features hive-gpu

# With all features
cargo build --release --features full
```

### Access Points

- **Web Dashboard**: http://localhost:15002/dashboard/ - Modern React dashboard with complete graph management interface (create/delete edges, explore neighbors, find paths, discover relationships)
- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp
- **UMICP Discovery**: http://localhost:15002/umicp/discover
- **Health Check**: http://localhost:15002/health

## 📊 Performance

| Metric                | Value                          |
| --------------------- | ------------------------------ |
| **Search Speed**      | < 3ms (CPU), < 1ms (Metal GPU) |
| **Storage Reduction** | 30-50% with normalization      |
| **Test Coverage**     | 95%+ coverage                  |
| **Test Suite**        | 703 passing, 6 ignored        |
| **MCP Tools**         | 20 focused individual tools    |
| **Document Formats**  | 14 formats supported           |

### Benchmark Results (vs Qdrant)

Comprehensive benchmark comparing Vectorizer with Qdrant across multiple scenarios:

- **Search Performance**: Vectorizer is **4-5x faster** than Qdrant in all test scenarios
  - Average latency: 0.16-0.23ms (Vectorizer) vs 0.80-0.87ms (Qdrant)
  - Throughput: 4,400-6,000 queries/sec (Vectorizer) vs 1,100-1,300 queries/sec (Qdrant)
- **Insert Performance**: Optimized with fire-and-forget pattern for non-blocking operations
  - Configurable batch sizes and request body limits
  - Background processing prevents API blocking
- **Test Scenarios**: 5 comprehensive scenarios tested
  - Small (1K vectors), Medium (5K vectors), Large (10K vectors) datasets
  - Multiple dimensions: 384, 512, 768
  - Full benchmark reports available in `docs/` directory

See [Benchmark Documentation](./docs/specs/BENCHMARKING.md) for detailed performance metrics and how to run benchmarks.

## 🔄 Feature Comparison

Comprehensive feature comparison with major vector database solutions:

| Feature | Vectorizer | Qdrant | pgvector | Pinecone | Weaviate | Milvus | Chroma |
|---------|------------|-------|----------|----------|----------|--------|--------|
| **Core** |
| Language | Rust | Rust | C (PostgreSQL) | C++/Go | Go | C++/Go | Python |
| License | Apache 2.0 | Apache 2.0 | PostgreSQL | Proprietary | BSD 3-Clause | Apache 2.0 | Apache 2.0 |
| Deployment | Standalone/Embedded | Standalone | PostgreSQL Extension | Cloud/Self-hosted | Standalone | Standalone | Standalone |
| **APIs & Integration** |
| REST API | ✅ Full | ✅ Full | ❌ (via PostgreSQL) | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| gRPC API | ✅ Qdrant-compatible | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ |
| GraphQL API | ✅ Full with GraphiQL | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| MCP Integration | ✅ 20 tools | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| n8n Integration | ✅ Official node | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Langflow Integration | ✅ LangChain components | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Python SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TypeScript SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| JavaScript SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Rust SDK | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ |
| C# SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Go SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Performance** |
| Search Latency | < 3ms (CPU)<br>< 1ms (GPU) | ~1-5ms | ~5-50ms | ~50-100ms | ~10-50ms | ~5-20ms | ~10-100ms |
| SIMD Acceleration | ✅ AVX2 | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| GPU Support | ✅ Metal (macOS) | ✅ CUDA | ❌ | ✅ Cloud GPU | ❌ | ✅ CUDA | ❌ |
| **Storage & Indexing** |
| HNSW Index | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Product Quantization | ✅ 64x compression | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Scalar Quantization | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Memory-Mapped Storage | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ |
| Persistent Storage | ✅ .vecdb format | ✅ | ✅ | ✅ Cloud | ✅ | ✅ | ✅ |
| **Distance Metrics** |
| Cosine Similarity | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Euclidean Distance | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Dot Product | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Advanced Features** |
| Graph Relationships | ✅ Auto-discovery | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Document Processing | ✅ 14 formats | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Multi-Collection Search | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Hybrid Search | ✅ Dense + Sparse | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Semantic Reranking | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ |
| Query Expansion | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Embedding Providers** |
| Built-in Embeddings | ✅ TF-IDF, BM25, BERT, MiniLM | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Custom Models | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Scalability** |
| Horizontal Sharding | ✅ (BETA) | ✅ | ✅ (PostgreSQL) | ✅ Cloud | ✅ | ✅ | ❌ |
| Replication | ✅ Master-Replica (BETA) | ✅ | ✅ (PostgreSQL) | ✅ Cloud | ✅ | ✅ | ❌ |
| Auto-scaling | ❌ | ❌ | ❌ | ✅ Cloud | ❌ | ✅ | ❌ |
| **Management & UI** |
| Web Dashboard | ✅ React + Full Graph UI | ✅ Basic | ❌ (pgAdmin) | ✅ Cloud | ✅ | ✅ | ✅ Basic |
| Desktop GUI | ✅ Electron + vis-network | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Graph Visualization | ✅ vis-network + Full Controls | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Graph Management | ✅ Create/Delete Edges, Path Finding | ❌ | ❌ | ❌ | ✅ Basic | ❌ | ❌ |
| CLI Tools | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| **Migration & Compatibility** |
| Qdrant Compatibility | ✅ Full API | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Migration Tools | ✅ Qdrant → Vectorizer | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Security** |
| Authentication | ✅ JWT + API Keys | ✅ | ✅ (PostgreSQL) | ✅ Cloud | ✅ | ✅ | ✅ |
| RBAC | ✅ | ✅ | ✅ (PostgreSQL) | ✅ Cloud | ✅ | ✅ | ❌ |
| Encryption at Rest | ✅ | ✅ | ✅ (PostgreSQL) | ✅ Cloud | ✅ | ✅ | ❌ |
| **Cost & Licensing** |
| Open Source | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Self-Hosted | ✅ | ✅ | ✅ | ✅ (Enterprise) | ✅ | ✅ | ✅ |
| Cloud Hosted | ❌ | ✅ (Qdrant Cloud) | ✅ (Various) | ✅ | ✅ (Weaviate Cloud) | ✅ (Zilliz Cloud) | ✅ |
| Free Tier | ✅ Unlimited | ✅ | ✅ | ✅ Limited | ✅ | ✅ | ✅ |

### Key Differentiators

**Vectorizer Advantages:**
- ✅ **MCP Integration**: Native Model Context Protocol support with 20 focused tools
- ✅ **Graph Relationships**: Automatic relationship discovery with complete GUI management (create/delete edges, path finding, neighbor exploration)
- ✅ **No-Code Integrations**: Official n8n node and Langflow components for visual workflow/LLM app building
- ✅ **GraphQL API**: Full GraphQL API with GraphiQL playground and complete REST parity
- ✅ **Document Processing**: Built-in support for 14 document formats (PDF, Office, images)
- ✅ **Desktop GUI**: Electron-based desktop application with vis-network graph visualization
- ✅ **Qdrant Compatibility**: Full API compatibility + migration tools + gRPC support
- ✅ **Performance**: 4-5x faster search than Qdrant in benchmarks
- ✅ **Unified Storage**: Compact `.vecdb` format with 20-30% space savings
- ✅ **Complete SDK Coverage**: 6 official SDKs (Python, TypeScript, JavaScript, Rust, C#, Go)

**Best Use Cases:**
- **Vectorizer**: AI applications requiring MCP integration, no-code workflows, document processing, graph relationships, and high-performance search
- **Qdrant**: Production-ready vector search with good performance and cloud options
- **pgvector**: PostgreSQL-based applications needing vector search alongside relational data
- **Pinecone**: Managed cloud solution with minimal infrastructure management
- **Weaviate**: Applications requiring GraphQL and built-in ML models
- **Milvus**: Large-scale deployments requiring advanced scalability features
- **Chroma**: Python-first applications with simple setup requirements

## 🔧 Recent Improvements (v1.6.0)

### New Features

- **✅ Graph Dashboard Enhancements**: Complete graph management interface
  - Create/delete edges with relationship types and weights
  - View node neighbors and find related nodes
  - Find shortest paths between nodes
  - Node-specific edge discovery with configurable parameters
  - Enhanced node details panel with inline actions
- **✅ n8n Integration**: Official community node for workflow automation
  - Collection, Vector, and Search resources
  - 12 operations across all resources
  - Visual workflow builder integration
- **✅ Langflow Integration**: LangChain-compatible components
  - VectorizerVectorStore for document storage
  - VectorizerRetriever for RAG pipelines
  - VectorizerLoader for existing vectors
- **✅ GraphQL API**: Full GraphQL API with async-graphql
  - Complete REST API parity with flexible queries
  - GraphiQL playground for interactive exploration
  - 37 unit and integration tests

### Quality Improvements (v1.5.0)

- **✅ All core tests passing**: 703+ tests with comprehensive coverage
- **✅ Better error handling**: Improved dimension validation and error messages
- **✅ Storage reliability**: MMap storage now properly persists vector counts
- **✅ Test stability**: Timeout protection prevents hanging tests
- **✅ BM25 search quality**: Fixed document frequency calculation for correct IDF values and improved BM25 scores

## 🎯 Use Cases

- **RAG Systems**: Semantic search for AI applications with automatic document conversion
- **Document Search**: Intelligent indexing and retrieval of PDFs, Office files, and web content
- **Code Analysis**: Semantic code search and navigation
- **Knowledge Bases**: Enterprise knowledge management with multi-format support

## 🔧 MCP Integration

Cursor IDE configuration:

```json
{
  "mcpServers": {
    "vectorizer": {
      "url": "http://localhost:15002/mcp",
      "type": "streamablehttp"
    }
  }
}
```

**Available MCP Tools** (20 tools):

### Core Operations

- `list_collections`, `create_collection`, `get_collection_info`
- `insert_text`, `get_vector`, `update_vector`, `delete_vector`
- `search`, `multi_collection_search`

### Advanced Search

- `search_intelligent` - AI-powered search with query expansion
- `search_semantic` - Semantic search with reranking
- `search_extra` - Combined search using multiple strategies
- `search_hybrid` - Hybrid dense + sparse vector search

### Discovery & Files

- `filter_collections`, `expand_queries`
- `get_file_content`, `list_files`, `get_file_chunks`
- `get_project_outline`, `get_related_files`

## 📦 Client SDKs

All SDKs are synchronized with server version **1.6.0**:

- **Python**: `pip install vectorizer-sdk` (v1.6.0)
- **TypeScript**: `npm install @hivellm/vectorizer-sdk` (v1.6.0)
- **Rust**: `cargo add vectorizer-sdk` (v1.6.0)
- **JavaScript**: `npm install @hivellm/vectorizer-sdk-js` (v1.6.0)
- **C#**: `dotnet add package Vectorizer.SDK` (v1.6.0)
- **Go**: `go get github.com/hivellm/vectorizer/sdks/go` (v1.6.0)

## 🔗 Workflow & LLM Integrations

### n8n Integration

Official n8n community node for no-code workflow automation.

**Installation:**
```bash
npm install @vectorizer/n8n-nodes-vectorizer
```

**Features:**
- Collection management (create, delete, get, list)
- Vector operations (insert, batch insert, delete, get)
- Search operations (vector, semantic, hybrid)
- 400+ n8n node integrations available
- Visual workflow builder

**Example Workflow:**
```
Document Loader → Vectorizer (Insert) → Trigger → Vectorizer (Search) → Response
```

See [n8n Integration Guide](./sdks/n8n/README.md) for detailed usage.

### Langflow Integration

LangChain-compatible components for visual LLM application building.

**Installation:**
```bash
pip install vectorizer-langflow
```

**Components:**
- `VectorizerVectorStore` - Full LangChain VectorStore implementation
- `VectorizerRetriever` - RAG pipeline retriever
- `VectorizerLoader` - Document loader for existing vectors

**Example:**
```python
from vectorizer_langflow import VectorizerVectorStore
from langchain.embeddings import OpenAIEmbeddings

vectorstore = VectorizerVectorStore(
    host="http://localhost:15002",
    collection_name="docs",
    embedding=OpenAIEmbeddings()
)

# Add documents
vectorstore.add_texts(["Document 1", "Document 2"])

# Search
results = vectorstore.similarity_search("query", k=5)
```

See [Langflow Integration Guide](./sdks/langflow/README.md) for detailed usage.

## 🔄 Qdrant Migration

Vectorizer provides comprehensive migration tools to help you migrate from Qdrant:

- **Configuration Migration**: Parse and convert Qdrant config files (YAML/JSON) to Vectorizer format
- **Data Migration**: Export collections from Qdrant and import into Vectorizer
- **Validation**: Validate exported data, check compatibility, and verify integrity after migration
- **REST API Compatibility**: Full Qdrant REST API compatibility at `/qdrant/*` endpoints

**Quick Migration Example:**

```rust
use vectorizer::migration::qdrant::{QdrantDataExporter, QdrantDataImporter};

// Export from Qdrant
let exported = QdrantDataExporter::export_collection(
    "http://localhost:6333",
    "my_collection"
).await?;

// Import into Vectorizer
let result = QdrantDataImporter::import_collection(&store, &exported).await?;
```

See [Qdrant Migration Guide](./docs/specs/QDRANT_MIGRATION.md) for detailed instructions.

## 📚 Documentation

- **[User Documentation](./docs/users/)** - Installation guides and user tutorials
- **[API Reference](./docs/specs/API_REFERENCE.md)** - Complete REST API documentation
- **[Dashboard Integration](./docs/DASHBOARD_INTEGRATION.md)** - Web dashboard setup and integration guide
- **[Qdrant Compatibility](./docs/users/qdrant/)** - Qdrant API compatibility and migration guide
- **[Technical Specifications](./docs/specs/)** - Architecture, performance, and implementation guides
- **[MCP Integration](./docs/specs/MCP.md)** - Model Context Protocol guide

## 📄 License

Apache License 2.0 - See [LICENSE](./LICENSE) for details

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.
