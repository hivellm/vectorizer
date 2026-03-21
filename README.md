# Vectorizer

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![Rust Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/vectorizer.svg)](https://crates.io/crates/vectorizer)
[![GitHub release](https://img.shields.io/github/release/hivellm/vectorizer.svg)](https://github.com/hivellm/vectorizer/releases)
[![Tests](https://img.shields.io/badge/tests-1701%20passing-brightgreen.svg)](https://github.com/hivellm/vectorizer/actions)
[![Coverage](https://img.shields.io/badge/coverage-95%25%2B-success.svg)](https://github.com/hivellm/vectorizer)
[![Production Ready](https://img.shields.io/badge/status-production%20ready-success.svg)](https://github.com/hivellm/vectorizer)

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ Key Features

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics (Cosine, Euclidean, Dot Product)
- **⚡ SIMD Acceleration**: AVX2-optimized vector operations (5-10x faster) with automatic CPU feature detection
- **💾 Memory-Mapped Storage**: MMap support for datasets larger than RAM with efficient OS paging
- **🚀 GPU Acceleration**: Metal GPU support for macOS (Apple Silicon) with cross-platform compatibility
- **📦 Product Quantization**: PQ compression for 64x memory reduction with minimal accuracy loss
- **💾 Compact Storage**: Unified `.vecdb` format with 20-30% space savings and automatic snapshots
- **🗳️ Raft Consensus (HA)**: Production-grade high availability with automatic leader election via openraft
  - Hybrid architecture: Raft for metadata consensus, TCP streaming for vector data
  - Automatic failover: replicas detect leader failure and elect new leader in 1-5 seconds
  - Write-redirect: follower nodes return HTTP 307 redirecting writes to the current leader
  - Read scaling: any node can serve read requests locally
  - WAL-backed durable replication with configurable write concern
  - Epoch-based conflict resolution for shard assignments
  - DNS discovery for Kubernetes headless services
  - Docker Compose and Helm chart for HA deployment
- **🔄 Master-Replica Replication**: TCP streaming replication with full/partial sync and auto-reconnect
- **🔗 Distributed Sharding**: Horizontal scaling across multiple servers with automatic shard routing
- **☁️ HiveHub Cluster Mode**: Multi-tenant cluster deployment with HiveHub.Cloud
  - Tenant isolation with user-scoped collections
  - Quota enforcement (collections, vectors, storage)
  - Usage tracking and reporting
  - Memory limits and MMap storage enforcement
  - Operation logging with cloud integration
  - Comprehensive audit trail and analytics
  - Tenant migration API (export, transfer, clone, cleanup)
- **📄 Document Conversion**: Automatic conversion of PDF, DOCX, XLSX, PPTX, HTML, XML, and images
- **🔄 Qdrant Migration**: Complete migration tools and full Qdrant 1.14.x API compatibility
  - Snapshots API (create, list, delete, recover)
  - Sharding API (create shard keys, distribute data)
  - Cluster Management API (status, recovery, peer management, metadata)
  - Query API (query, batch query, grouped queries with prefetch)
  - Search Groups and Matrix API (grouped results, similarity matrices)
  - Named Vectors support (partial)
  - Quantization configuration (PQ and Binary)
- **🎯 MCP Integration**: 26 focused individual tools for AI model integration
- **🔄 UMICP Protocol**: Native JSON types + Tool Discovery endpoint
- **📊 GraphQL API**: Full GraphQL API with async-graphql
  - Complete REST API parity with flexible queries
  - GraphiQL playground for interactive exploration
  - Mutations for collections, vectors, and search
- **🖥️ Web Dashboard**: Modern React + TypeScript dashboard with complete graph management interface
  - JWT-based authentication with login page and session management
  - Create/delete edges with relationship types and weights
  - Explore node neighbors and related nodes
  - Find shortest paths between nodes
  - Node-specific edge discovery with configurable parameters
  - Real-time graph visualization with vis-network
- **🖥️ Desktop GUI**: Electron-based desktop application with vis-network graph visualization for visual database management
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **🧠 Multiple Embeddings**: TF-IDF, BM25, FastEmbed (production), BERT/MiniLM (real or placeholder), and custom models
- **🔀 Hybrid Search**: Dense + Sparse search with Reciprocal Rank Fusion (RRF)
- **📝 Smart Summarization**: Extractive, keyword, sentence, and abstractive (OpenAI GPT) methods
- **🔐 TLS/SSL Security**: Full TLS 1.2/1.3 support with mTLS, configurable cipher suites, and ALPN
- **⚡ Rate Limiting**: Per-API-key rate limiting with configurable tiers and overrides
- **📊 Quantization Cache**: Cache hit ratio tracking with comprehensive metrics
- **🕸️ Graph Relationships**: Automatic relationship discovery and graph traversal with full GUI support for edge management, node exploration, and path finding
- **🔗 n8n Integration**: Official n8n community node for no-code workflow automation (400+ node integrations)
- **🎨 Langflow Integration**: LangChain-compatible components for visual LLM app building
- **🔒 Security**: JWT + API Key authentication with RBAC
- **🔐 Payload Encryption**: Optional ECC-P256 + AES-256-GCM payload encryption with zero-knowledge architecture ([docs](docs/features/encryption/README.md))

## 🎉 Latest Release: v2.4.0 - Transmutation Default & Enhanced File Upload

**New in v2.4.0:**
- **Transmutation enabled by default**: Document conversion (PDF, DOCX, XLSX, PPTX, images) now included in default build
- **Increased file upload limits**: Support for files up to 200MB (previously 100MB)
- **Enhanced file upload configuration**: Improved config loading with better error handling and logging
- **Extended file format support**: Added PDF, DOCX, XLSX, PPTX, and image formats to allowed extensions
- **Improved upload validation**: Better error messages and config path detection

**Previous Release (v2.3.0):**
- **Embedded Dashboard**: All dashboard assets now embedded in binary (single executable, ~26MB)
  - No external `dashboard/dist` folder required for distribution
  - Zero dependencies: binary can be copied anywhere and run immediately
  - Perfect for containerized deployments
- **Setup Wizard Visual Improvements**: Modern glassmorphism design with animated progress indicators
  - Dark gradient background with animated color orbs
  - Frosted glass cards with backdrop blur effects
  - Enhanced step progression visualization
- **Setup Wizard UX Enhancements**: Skip setup option and GraphRAG toggle per collection
  - Allow users to bypass wizard and configure later
  - Enable graph relationships per collection for semantic relationship discovery
- **API Sandbox**: Test API endpoints directly from dashboard with code examples generator

**Previous Release (v2.2.0):**
- Synchronized all SDKs and server to version 2.2.0
- Previous improvements and SDK synchronization
- Updated package names: TypeScript/JavaScript SDKs now use `@hivehub` scope
- Comprehensive documentation updates across all SDKs
- All SDKs fully synchronized and tested

**Previous Release (v2.1.0):**
- Added optional ECC-AES payload encryption with zero-knowledge architecture
- ECC-P256 + AES-256-GCM for end-to-end encrypted vector payloads
- Collection-level encryption policies (optional, required, mixed)
- Full support across all APIs (REST, GraphQL, MCP, Qdrant-compatible)
- Complete SDK support for all 6 official SDKs
- See [encryption documentation](docs/features/encryption/README.md) for details

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

**Basic Docker Run (with authentication):**
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -e VECTORIZER_AUTH_ENABLED=true \
  -e VECTORIZER_ADMIN_USERNAME=admin \
  -e VECTORIZER_ADMIN_PASSWORD=admin \
  -e VECTORIZER_JWT_SECRET=change-this-secret-in-production \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

**Production Docker Run (with custom credentials):**
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -e VECTORIZER_AUTH_ENABLED=true \
  -e VECTORIZER_ADMIN_USERNAME=admin \
  -e VECTORIZER_ADMIN_PASSWORD=your-secure-password \
  -e VECTORIZER_JWT_SECRET=your-jwt-secret-key \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

**Using Docker Compose:**
```bash
# Copy .env.example to .env and customize
cp .env.example .env
# Edit .env with your credentials

# Start with docker-compose
docker-compose up -d
```

**Default Credentials (CHANGE IN PRODUCTION!):**
- **Username:** `admin`
- **Password:** `admin`
- **JWT Secret:** `change-this-secret-in-production`

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

### 🔒 Authentication (Docker)

When using Docker, authentication is **enabled by default**:

**Default Credentials** (⚠️ CHANGE IN PRODUCTION!):
- **Username:** `admin`
- **Password:** `admin`
- **Login Endpoint:** `POST http://localhost:15002/auth/login`

**Authentication Example:**
```bash
# Login to get JWT token
curl -X POST http://localhost:15002/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'

# Use JWT token in requests
curl -X GET http://localhost:15002/collections \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

**API Key Authentication:**

API Keys can be created in the dashboard (`/api-keys`) or via REST API for programmatic access.

⚠️ **IMPORTANT:** API Keys do NOT use the `Bearer` prefix. Use them directly in the `Authorization` header:

```bash
# Create API key (requires JWT authentication)
curl -X POST http://localhost:15002/auth/keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "name": "Production API Key",
    "permissions": ["read", "write"],
    "expires_in_days": 90
  }'

# Use API key in requests (NO Bearer prefix!)
curl -X GET http://localhost:15002/collections \
  -H "Authorization: YOUR_API_KEY"

# MCP Configuration (mcp.json)
{
  "mcpServers": {
    "vectorizer": {
      "command": "npx",
      "args": ["-y", "@hivellm/mcp-vectorizer"],
      "env": {
        "VECTORIZER_API_URL": "http://localhost:15002",
        "VECTORIZER_API_KEY": "YOUR_API_KEY"
      }
    }
  }
}
```

**Authentication Methods Comparison:**

| Method | Header Format | Use Case |
|--------|--------------|----------|
| JWT Token | `Authorization: Bearer YOUR_JWT_TOKEN` | Dashboard, short-lived sessions |
| API Key | `Authorization: YOUR_API_KEY` | MCP, CLI, long-lived integrations |

**Production Security:**
- Change default credentials using environment variables
- Use strong passwords (minimum 32 characters)
- Generate secure JWT secret (minimum 48 characters)
- See [Docker Authentication Guide](docs/users/getting-started/DOCKER_AUTHENTICATION.md) for details
- Review [Security Policy](SECURITY.md) for best practices

## 📊 Performance

| Metric                | Value                          |
| --------------------- | ------------------------------ |
| **Search Speed**      | < 3ms (CPU), < 1ms (Metal GPU) |
| **Storage Reduction** | 30-50% with normalization      |
| **Test Coverage**     | 95%+ coverage                  |
| **Test Suite**        | 1514 passing, 101 ignored     |
| **MCP Tools**         | 26 focused individual tools    |
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
| MCP Integration | ✅ 26 tools | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
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
| Cloud Hosted | ✅ (HiveHub.Cloud) | ✅ (Qdrant Cloud) | ✅ (Various) | ✅ | ✅ (Weaviate Cloud) | ✅ (Zilliz Cloud) | ✅ |
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

## 🔧 Recent Improvements (v2.0.0 - v2.4.0)

### New Features (v2.0.0+)

- **✅ Dashboard Authentication**: Complete authentication system for the dashboard
  - Login page with username/password form and modern UI
  - JWT token-based authentication via `/auth/login` endpoint
  - Session persistence with localStorage and automatic route protection
- **✅ HiveHub Cluster Integration**: Multi-tenant cluster mode support
  - `HubManager` for HiveHub API integration with tenant isolation
  - API key validation, quota enforcement, and usage tracking
  - Request signing and IP whitelist support for security
- **✅ Cluster Memory Limits**: Enforce predictable memory usage in cluster mode
  - Global cache memory limit (default: 1GB)
  - MMap storage enforcement and file watcher auto-disable
  - Comprehensive configuration validator at startup
- **✅ MMap Storage Deadlock Fix**: Fixed deadlock during concurrent vector insertions
  - Removed internal `Arc<RwLock<>>` wrapper for proper lock management
  - Stable concurrent insert operations without blocking

### Quality Improvements (v2.0.0+)

- **✅ Dashboard SPA Routing Fix**: Browser refresh now works on all dashboard routes
- **✅ File Watcher Improvements**: Uses default collection instead of creating empty collections
- **✅ Empty Collection Management**: New endpoints to list and cleanup empty collections
- **✅ Dashboard Cache Headers**: Proper caching for faster dashboard loading

### Previous Features (v1.6.0 - v1.7.0)

- **✅ Graph Dashboard Enhancements**: Complete graph management interface
- **✅ n8n Integration**: Official community node for workflow automation
- **✅ Langflow Integration**: LangChain-compatible components for RAG pipelines
- **✅ GraphQL API**: Full GraphQL API with async-graphql and GraphiQL playground
- **✅ SDK Master/Replica Routing**: Automatic read/write routing for high availability
- **✅ All core tests passing**: 1514+ tests with comprehensive coverage

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

**Available MCP Tools** (26 tools):

### Core Operations (9 tools)
- `list_collections`, `create_collection`, `get_collection_info`
- `insert_text`, `get_vector`, `update_vector`, `delete_vector`
- `search`, `multi_collection_search`

### Advanced Search (4 tools)
- `search_intelligent` - AI-powered search with query expansion
- `search_semantic` - Semantic search with reranking
- `search_extra` - Combined search using multiple strategies
- `search_hybrid` - Hybrid dense + sparse vector search

### Discovery & Files (7 tools)
- `filter_collections`, `expand_queries`
- `get_file_content`, `list_files`, `get_file_chunks`
- `get_project_outline`, `get_related_files`

### Graph Operations (6 tools)
- `graph_list_nodes`, `graph_list_edges`, `graph_find_related`
- `graph_create_edge`, `graph_delete_edge`
- `graph_discover_edges`, `graph_discover_status`

> **Note:** Cluster management operations are available via REST API only for security reasons.

## 📦 Client SDKs

All SDKs are synchronized with server version **2.4.0**:

- **Python**: `pip install vectorizer-sdk` (v2.4.0)
- **TypeScript**: `npm install @hivehub/vectorizer-sdk` (v2.4.0)
- **Rust**: `cargo add vectorizer-sdk` (v2.4.0)
- **JavaScript**: `npm install @hivehub/vectorizer-sdk-js` (v2.4.0)
- **C#**: `dotnet add package Vectorizer.Sdk` (v2.4.0)
- **Go**: `go get github.com/hivellm/vectorizer-sdk-go` (v2.4.0)

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

## ☁️ HiveHub Cloud Integration

Vectorizer supports multi-tenant cluster mode integration with [HiveHub.Cloud](https://hivehub.cloud) for managed deployment:

### Features

- **Multi-Tenant Isolation**: Each user's collections are isolated with owner-based filtering
- **Quota Management**: Collection count, vector count, and storage quotas enforced per tenant
- **Usage Tracking**: Automatic tracking and reporting of resource usage
- **User-Scoped Backups**: Create, download, and restore backups per user

### Configuration

Enable HiveHub integration in `config.yml`:

```yaml
hub:
  enabled: true
  api_url: "https://api.hivehub.cloud"
  tenant_isolation: "collection"
  usage_report_interval: 300
```

Set the service API key:

```bash
export HIVEHUB_SERVICE_API_KEY="your-service-api-key"
```

### Internal Request Headers

For internal HiveHub requests:

```bash
# Bypass authentication
curl -H "x-hivehub-service: true" \
     http://localhost:15002/collections

# With user context (tenant scoping)
curl -H "x-hivehub-service: true" \
     -H "x-hivehub-user-id: <user-uuid>" \
     http://localhost:15002/collections
```

See [HiveHub Integration Guide](./docs/HUB_INTEGRATION.md) for detailed documentation.

### Cluster Mode Requirements

When running Vectorizer in cluster mode, the following requirements are enforced:

| Requirement | Description | Default |
|-------------|-------------|---------|
| **MMap Storage** | Memory storage is not allowed; MMap is required | Enforced |
| **Cache Limit** | Maximum cache memory across all caches | 1GB |
| **File Watcher** | Automatically disabled in cluster mode | Disabled |
| **Strict Validation** | Server fails to start on config violations | Enabled |

Example cluster configuration:

```yaml
cluster:
  enabled: true
  node_id: "node-1"
  memory:
    max_cache_memory_bytes: 1073741824  # 1GB
    enforce_mmap_storage: true
    disable_file_watcher: true
    strict_validation: true
```

See [Cluster Memory Limits](./docs/specs/CLUSTER_MEMORY.md) for detailed configuration and troubleshooting.

## 📚 Documentation

- **[User Documentation](./docs/users/)** - Installation guides and user tutorials
- **[API Reference](./docs/specs/API_REFERENCE.md)** - Complete REST API documentation
- **[Dashboard Integration](./docs/DASHBOARD_INTEGRATION.md)** - Web dashboard setup and integration guide
- **[Qdrant Compatibility](./docs/users/qdrant/)** - Qdrant API compatibility and migration guide
- **[HiveHub Integration](./docs/HUB_INTEGRATION.md)** - Multi-tenant cluster mode with HiveHub.Cloud
- **[Cluster Memory Limits](./docs/specs/CLUSTER_MEMORY.md)** - Cluster mode memory management and validation
- **[Technical Specifications](./docs/specs/)** - Architecture, performance, and implementation guides
- **[MCP Integration](./docs/specs/MCP.md)** - Model Context Protocol guide

## 📄 License

Apache License 2.0 - See [LICENSE](./LICENSE) for details

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.
