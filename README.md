# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ **Version 1.2.2 - Critical BM25 & Persistence Fixes**

### 🎉 **Latest Updates (v1.2.2)** 🔥
- **🔴 CRITICAL FIX**: BM25 vocabulary now preserved on CTRL+C shutdown
- **✅ Tokenizers Saved**: All tokenizer files included in `.vecdb` archive
- **✅ Checksums Saved**: File integrity data preserved across restarts
- **🚀 No More Errors**: "BM25 vocabulary is empty" error completely resolved
- **🎯 All Protocols Working**: MCP, REST, and UMICP search fully operational

### 🎯 **Key Features**
- **🔄 Master-Replica Replication (BETA)**: Replication system with automatic failover (currently in beta - see docs)
  - Full sync via snapshot with CRC32 checksum verification
  - Partial sync via incremental replication log
  - Circular replication log (1M operations buffer)
  - Auto-reconnect with exponential backoff
  - REST API endpoints for replication management
  - ⚠️ Known issues with snapshot synchronization - use with caution
- **🚀 GPU Acceleration**: Metal GPU support for macOS (Apple Silicon) with cross-platform compatibility
- **🎯 MCP Tools**: 19 focused individual tools for better model integration
- **🔄 UMICP v0.2.1**: Native JSON types + Tool Discovery endpoint
- **🔍 Tool Discovery**: GET `/umicp/discover` exposes all MCP tools with full schemas
- **🖥️ Desktop GUI**: Electron-based desktop application for visual database management
- **💾 Compact Storage (.vecdb)**: Unified compressed archives with 20-30% space savings and snapshot support
- **📄 Document Conversion**: Automatic conversion of PDF, DOCX, XLSX, PPTX, HTML, XML, and images to Markdown
- **📸 Snapshot System**: Automatic backups with configurable retention policies
- **Text Normalization System**: Content-aware normalization with 30-50% storage reduction
- **Real-time File Watcher**: Automatic file monitoring and indexing
- **Intelligent Search**: Advanced semantic search with multi-query generation
- **File Operations**: Complete file management with summaries and analysis
- **Multi-tier Cache**: LFU hot cache, mmap warm store, Zstandard cold storage
- **Discovery Pipeline**: 10-type semantic discovery with evidence compression

### 🧪 **Quality Metrics**
- ✅ **All tests passing** (100% pass rate, v1.0.0)
- ⚡ **Fast execution** with optimized test suite
- 🎯 **Production-ready** with comprehensive coverage
- 📄 **19 transmutation tests** (100% pass rate)
- 💾 **30+ storage system tests** (compaction, snapshots, migration)
- 🔄 **UMICP discovery tests** (100% pass rate)
- 🛠️ **19 MCP tools** fully tested and validated

## 🌟 **Core Capabilities**

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics (Cosine, Euclidean, Dot Product)
- **📚 Document Indexing**: Intelligent chunking and processing of 10+ file types
- **📄 Document Conversion**: Optional transmutation integration for PDF, DOCX, XLSX, PPTX, HTML, XML, and images
- **🧠 Embeddings**: TF-IDF, BM25, BERT, MiniLM, and custom models
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **🚀 GPU Acceleration**: Metal GPU support for macOS with automatic detection and CPU fallback
- **🏗️ Unified Architecture**: REST API + MCP Server + UMICP Protocol
- **💾 Automatic Persistence**: Collections auto-save every 30 seconds
- **👀 File Watcher**: Real-time monitoring with smart debouncing
- **🔒 Security**: JWT + API Key authentication with RBAC

## 🚀 **Quick Start**

### Using Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

# Create Docker-specific workspace config
cp vectorize-workspace.docker.example.yml vectorize-workspace.docker.yml
# Edit vectorize-workspace.docker.yml with /workspace/* paths

# Run with monorepo access
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-dashboard:/vectorizer/dashboard \
  -v $(pwd)/vectorize-workspace.docker.yml:/vectorizer/vectorize-workspace.yml:ro \
  -v $(pwd)/../../:/workspace:ro \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest

# View logs
docker logs -f vectorizer

# Access the services
# - MCP Server: http://localhost:15002/mcp
# - REST API: http://localhost:15002
# - Dashboard: http://localhost:15002/
# - UMICP Discovery: http://localhost:15002/umicp/discover
```

**Alternative: Docker Compose**
```bash
docker-compose up -d
docker-compose logs -f
```

See [docs/DOCKER.md](docs/DOCKER.md) for detailed Docker documentation.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

# Build and run (basic - CPU only)
cargo build --release
./target/release/vectorizer

# Build with GPU acceleration (macOS Metal)
cargo build --release --features hive-gpu
./target/release/vectorizer

# Build with transmutation support for document conversion
cargo build --release --features transmutation
./target/release/vectorizer

# Build with all features (GPU + Transmutation)
cargo build --release --features full
./target/release/vectorizer
```

**Platform Notes:**
- **macOS (Apple Silicon)**: Full Metal GPU acceleration available with `hive-gpu` feature
- **Linux/Windows**: Compiles successfully with graceful CPU fallback messages
- **Cross-Platform**: All features work on all platforms with appropriate fallbacks

### **Access Points**
- **Desktop GUI**: `./gui/` - Electron desktop application (NEW in v0.8.2)
- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp (StreamableHTTP)
- **UMICP**: http://localhost:15002/umicp (Protocol v0.2.1)
- **UMICP Discovery**: http://localhost:15002/umicp/discover (Tool discovery endpoint)
- **Health Check**: http://localhost:15002/health

## 🖥️ **Desktop GUI (v0.8.2)**

Modern Electron-based desktop application for managing your Vectorizer database:

**Features:**
- 🎨 Beautiful Vue 3 + TailwindCSS interface
- 📊 Real-time collection management and monitoring
- 🔍 Visual search and vector browsing
- ⚙️ Configuration editor with live preview
- 📁 File watcher and workspace management
- 💾 Backup/restore operations
- 📈 System metrics and performance monitoring

**Installation:**
```bash
cd gui
pnpm install
pnpm electron:build:win    # Windows MSI installer
pnpm electron:build:mac    # macOS DMG installer
pnpm electron:build:linux  # Linux DEB package
```

**Development:**
```bash
cd gui
pnpm install
pnpm dev  # Hot-reload development mode
```

**Note:** Requires Node.js 64-bit (x64 architecture) for building

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

### **MCP Search Tools (v1.0.0)**

#### **Basic Search** (`search`)
- Simple vector similarity search
- Configurable similarity threshold (default: 0.1)
- Fast and efficient for direct queries
- Single collection focus

#### **Intelligent Search** (`search_intelligent`)
- AI-powered query expansion
- Automatic deduplication across results
- Domain-specific term expansion
- Cross-collection search support
- Optimized for MCP (MMR disabled for speed)

#### **Semantic Search** (`search_semantic`)
- Advanced semantic reranking
- Precision-focused results
- Configurable similarity thresholds
- Optimized for MCP (cross-encoder disabled for speed)

#### **Combined Search** (`search_extra`) - NEW in v1.0.0
- Concatenates results from multiple strategies
- Combines: basic + semantic + intelligent
- Automatic deduplication
- Best of all search methods in one call

#### **Multi-Collection Search** (`multi_collection_search`)
- Search across multiple collections simultaneously
- Results grouped by collection
- Configurable limits per collection
- Simplified for MCP (no cross-collection reranking)

### **Discovery Tools (Simplified)**
- `filter_collections`: Filter collections by name patterns
- `expand_queries`: Generate query variations (definition, features, architecture)

### **REST API Only (Advanced Features)**
For complex operations requiring MMR, cross-encoder reranking, batch processing, or full discovery pipeline, use the REST API which provides all advanced features without MCP limitations.

## 📚 **Configuration**

```yaml
# config.yml - Main configuration
vectorizer:
  host: "localhost"
  port: 15002
  default_dimension: 512
  default_metric: "cosine"

# Replication configuration (NEW in v1.1.0)
replication:
  enabled: false
  mode: "master"  # or "replica"
  master:
    host: "0.0.0.0"
    port: 6380
    repl_backlog_size: 1048576  # 1MB circular buffer
  replica:
    master_host: "localhost"
    master_port: 6380
    read_only: true

# Text normalization (v0.5.0)
normalization:
  enabled: true
  level: "conservative"  # conservative/moderate/aggressive
  line_endings:
    normalize_crlf: true
    collapse_multiple_newlines: true
    trim_trailing_whitespace: true

# Transmutation document conversion (optional feature)
transmutation:
  enabled: true
  max_file_size_mb: 50
  conversion_timeout_secs: 300
  preserve_images: false

# Multi-tier cache
cache:
  enabled: true
  max_entries: 10000
  ttl_seconds: 3600

# Compact storage with snapshots (NEW in v0.8.0)
storage:
  compression:
    enabled: true          # Enable .vecdb format
    format: "zstd"
    level: 3               # Balanced compression
  snapshots:
    enabled: true          # Automatic backups
    interval_hours: 1      # Hourly snapshots
    retention_days: 2      # Keep for 2 days
    max_snapshots: 48
```

## 💾 **Storage System**

### Compact Format (.vecdb)

New unified storage format with compression and snapshots:

**Benefits:**
- ✅ 20-30% disk space reduction
- ✅ Automatic snapshots with retention policies
- ✅ Single-file backups (easy portability)
- ✅ Atomic updates (corruption-safe)
- ✅ Faster backups (copy vs full backup)

**CLI Commands:**
```bash
# View storage stats
vectorizer storage info --detailed

# Manage snapshots
vectorizer snapshot list
vectorizer snapshot create
vectorizer snapshot restore --id 20241014_120000 --force

# Verify integrity
vectorizer storage verify --fix

# Manual migration (if needed)
vectorizer storage migrate
```

**Format Support:**
- **Legacy:** Individual files (automatic migration offered on startup)
- **Compact:** Single `.vecdb` archive (recommended for production)

**Migration:**
- ✅ **Automatic detection and prompt on startup**
- ✅ **Interactive migration** - asks user confirmation (Y/n)
- ✅ Safe migration with timestamped backup
- ✅ Rollback support if needed
- ✅ Zero data loss guarantee

See [STORAGE.md](docs/STORAGE.md) and [MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md) for details.

## 📊 **Performance**

| Metric | Value |
|--------|-------|
| **Search Speed** | < 3ms (CPU), < 1ms (Metal GPU) |
| **Startup Time** | Non-blocking |
| **Storage Reduction** | 30-50% with normalization |
| **Test Coverage** | All tests passing, 100% pass rate |
| **MCP Tools** | 19 focused individual tools |
| **Collections** | 107+ tested |
| **PDF Conversion** | 98x faster than Docling |
| **Document Formats** | 14 formats supported |
| **GPU Acceleration** | Metal (macOS), graceful CPU fallback |
| **Cross-Platform** | Linux, macOS, Windows |

## 🎯 **Use Cases**

- **RAG Systems**: Semantic search for AI applications with automatic PDF/DOCX conversion
- **Document Search**: Intelligent indexing and retrieval of PDFs, Office files, and web content
- **Code Analysis**: Semantic code search and navigation
- **Knowledge Bases**: Enterprise knowledge management with multi-format support
- **Research Papers**: Automatic PDF indexing with page-level metadata
- **Legal Documents**: DOCX/PDF processing with precise page tracking

## 🔄 **Master-Replica Replication** ⚠️ **BETA**

> **⚠️ BETA WARNING**: The replication system is currently in beta. While core functionality works, there are known issues with snapshot synchronization. Use in production with caution and monitor closely. See [GitHub Issues](https://github.com/hivellm/vectorizer/issues) for current status.

### Overview

Vectorizer v1.1.0 introduces a master-replica replication system inspired by Redis, enabling high availability and horizontal read scaling.

### Features

- **Full Sync**: Complete data synchronization via snapshots with CRC32 verification
- **Partial Sync**: Incremental updates via circular replication log (1M operations)
- **Automatic Failover**: Auto-reconnect with exponential backoff (1s → 60s max)
- **Real-time Replication**: Sub-10ms typical replication lag
- **REST API Management**: Complete replication control via HTTP endpoints

### Quick Start

**Master Node**:
```yaml
# config.production.yml
replication:
  enabled: true
  mode: "master"
  master:
    host: "0.0.0.0"
    port: 6380
```

**Replica Node**:
```yaml
# config.production.yml
replication:
  enabled: true
  mode: "replica"
  replica:
    master_host: "master.example.com"
    master_port: 6380
    read_only: true
```

### REST API Endpoints

```bash
# Get replication status
GET /api/v1/replication/status

# Trigger manual sync (replica only)
POST /api/v1/replication/sync

# Promote replica to master
POST /api/v1/replication/promote

# Get replication metrics
GET /api/v1/replication/metrics
```

### Performance Metrics

- **Replication Log Append**: 4-12M operations/second
- **Snapshot Creation**: ~250ms for 10K vectors (128D)
- **Snapshot Application**: ~400ms for 10K vectors
- **Typical Replication Lag**: <10ms

### Documentation

- **[Replication Guide](./docs/REPLICATION.md)** - Complete architecture and deployment guide
- **[Test Suite](./docs/REPLICATION_TESTS.md)** - 38 comprehensive tests with benchmarks
- **[Coverage Report](./docs/REPLICATION_COVERAGE.md)** - 95%+ coverage on testable logic
- **[Production Config](./config.production.yml)** - Production-optimized settings
- **[Development Config](./config.development.yml)** - Development-optimized settings

## 📚 **Documentation**

- **[API Reference](./docs/api/)** - REST API documentation
- **[Replication Guide](./docs/REPLICATION.md)** - Master-replica replication system
  - [Test Suite](./docs/REPLICATION_TESTS.md) - 38 comprehensive tests
  - [Coverage Report](./docs/REPLICATION_COVERAGE.md) - 95%+ coverage
- **[Transmutation Integration](./docs/specs/transmutation_integration.md)** - Document conversion guide
- **[MCP Integration](./docs/specs/MCP_INTEGRATION.md)** - Model Context Protocol guide
- **[Technical Specs](./docs/specs/)** - Complete technical documentation
- **[Roadmap](./docs/specs/ROADMAP.md)** - Development roadmap

## 🔧 **MCP Integration**

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

**Available MCP Tools** (19 individual tools):

### **Core Collection/Vector Operations (9 tools)**
1. `list_collections` - List all collections with metadata
2. `create_collection` - Create new collection (name, dimension, metric)
3. `get_collection_info` - Get detailed collection information
4. `insert_text` - Insert single text with automatic embedding
5. `get_vector` - Retrieve vector by ID
6. `update_vector` - Update vector text/metadata
7. `delete_vector` - Delete vectors by ID
8. `multi_collection_search` - Search across multiple collections
9. `search` - Basic vector similarity search

### **Search Operations (3 tools)**
10. `search_intelligent` - AI-powered search with query expansion and deduplication
11. `search_semantic` - Semantic search with basic reranking
12. `search_extra` - Combined search using multiple strategies (basic, semantic, intelligent)

### **Discovery Operations (2 tools)**
13. `filter_collections` - Filter collections by name patterns
14. `expand_queries` - Generate query variations for broader coverage

### **File Operations (5 tools)**
15. `get_file_content` - Retrieve complete file content
16. `list_files` - List indexed files with filtering and sorting
17. `get_file_chunks` - Retrieve file chunks in original order
18. `get_project_outline` - Generate hierarchical project structure
19. `get_related_files` - Find semantically related files

### **Key Improvements in v1.0.0**
- ✅ **No enum parameters** - Direct tool selection by name
- ✅ **Simplified parameters** - Only relevant parameters per tool
- ✅ **Better model accuracy** - Reduced entropy improves tool calling
- ✅ **Faster execution** - Disabled slow features (MMR, cross-encoder) in MCP
- ✅ **Enhanced security** - Dangerous operations restricted to REST API

## 📦 **Client SDKs**

All SDKs now follow standardized naming convention:

- **Python**: `pip install vectorizer-sdk` ✅ **Published to PyPI**
- **TypeScript**: `npm install @hivellm/vectorizer-sdk`
- **Rust**: `cargo add vectorizer-sdk`
- **JavaScript**: `npm install @hivellm/vectorizer-sdk-js`

### Installation Examples

```bash
# Python (Published to PyPI)
pip install vectorizer-sdk

# TypeScript
npm install @hivellm/vectorizer-sdk

# Rust
cargo add vectorizer-sdk

# JavaScript
npm install @hivellm/vectorizer-sdk-js
```

## 📄 **License**

MIT License - See [LICENSE](./LICENSE) for details