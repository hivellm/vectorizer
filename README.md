# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ **Version 0.9.6 - Docker Multi-Platform & Monorepo Support**

### 🎯 **Key Features**
- **🔄 UMICP v0.2.1**: Native JSON types + Tool Discovery endpoint (NEW in v0.9.1)
- **🔍 Tool Discovery**: GET `/umicp/discover` exposes all 38+ MCP tools with full schemas
- **🖥️ Desktop GUI**: Electron-based desktop application for visual database management
- **💾 Compact Storage (.vecdb)**: Unified compressed archives with 20-30% space savings and snapshot support
- **📄 Document Conversion**: Automatic conversion of PDF, DOCX, XLSX, PPTX, HTML, XML, and images to Markdown
- **📸 Snapshot System**: Automatic backups with configurable retention policies
- **Text Normalization System**: Content-aware normalization with 30-50% storage reduction
- **Real-time File Watcher**: Automatic file monitoring and indexing
- **Intelligent Search**: Advanced semantic search with multi-query generation
- **File Operations**: 6 MCP tools for AI-powered file analysis
- **Multi-tier Cache**: LFU hot cache, mmap warm store, Zstandard cold storage
- **Discovery Pipeline**: 9-stage semantic discovery with evidence compression

### 🧪 **Quality Metrics**
- ✅ **403 tests passing** (100% pass rate)
- ⚡ **2.01s execution time**
- 🎯 **Production-ready** with comprehensive coverage
- 📄 **19 transmutation tests** (100% pass rate)
- 💾 **30+ storage system tests** (compaction, snapshots, migration)
- 🔄 **6 UMICP discovery tests** (100% pass rate)

## 🌟 **Core Capabilities**

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics (Cosine, Euclidean, Dot Product)
- **📚 Document Indexing**: Intelligent chunking and processing of 10+ file types
- **📄 Document Conversion**: Optional transmutation integration for PDF, DOCX, XLSX, PPTX, HTML, XML, and images
- **🧠 Embeddings**: TF-IDF, BM25, BERT, MiniLM, and custom models
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
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
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorizer-snapshots:/vectorizer/snapshots \
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

# Build and run (basic)
cargo build --release
./target/release/vectorizer

# Build with transmutation support for document conversion
cargo build --release --features transmutation
./target/release/vectorizer

# Build with all features
cargo build --release --features full
```

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
| **Search Speed** | < 3ms |
| **Startup Time** | Non-blocking |
| **Storage Reduction** | 30-50% with normalization |
| **Test Coverage** | 366 tests, 100% pass rate |
| **Collections** | 107+ tested |
| **PDF Conversion** | 98x faster than Docling |
| **Document Formats** | 14 formats supported |

## 🎯 **Use Cases**

- **RAG Systems**: Semantic search for AI applications with automatic PDF/DOCX conversion
- **Document Search**: Intelligent indexing and retrieval of PDFs, Office files, and web content
- **Code Analysis**: Semantic code search and navigation
- **Knowledge Bases**: Enterprise knowledge management with multi-format support
- **Research Papers**: Automatic PDF indexing with page-level metadata
- **Legal Documents**: DOCX/PDF processing with precise page tracking

## 📚 **Documentation**

- **[API Reference](./docs/api/)** - REST API documentation
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