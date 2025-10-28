# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ **Version 1.1.2 - Qdrant Compatibility IMPLEMENTED**

### 🎉 **Latest Updates**

- **v1.1.2**: Qdrant REST API compatibility, MCP search fixes, performance monitoring
- **v1.1.0**: Master-replica replication (BETA), SDK standardization
- **v1.0.0**: MCP tools refactoring (19 focused tools)

See [CHANGELOG.md](CHANGELOG.md) for complete version history.

### 🎯 **Key Features**

- **🔍 Semantic Search**: Sub-3ms search with HNSW indexing
- **🧠 Multiple Embeddings**: TF-IDF, BM25, BERT, MiniLM, custom models
- **📄 Document Conversion**: PDF, DOCX, XLSX, PPTX → Markdown (98x faster than Docling)
- **🚀 GPU Acceleration**: Metal (macOS), optional CUDA support
- **🔄 Replication**: Master-replica system (BETA)
- **🎯 MCP Integration**: 22 tools for AI assistants
- **🔒 Security**: JWT + API Key auth with RBAC
- **💾 Compact Storage**: .vecdb format with 20-30% compression
- **🖥️ Desktop GUI**: Electron-based visual management


## 🛡️ **Guardrails System - BSOD Protection**

### Windows Users: Read This First!

If you're building on **Windows**, the vectorizer can cause **Blue Screen of Death (BSOD)** due to GPU drivers and heavy parallelism. We've implemented a **comprehensive guardrails system** to prevent this:

#### Quick Start (Windows-Safe)

```powershell
# 1. Run safety checks
.\scripts\pre-build-check.ps1

# 2. Build safely (NO GPU, prevents BSODs)
.\scripts\build-windows-safe.ps1

# 3. Test safely
.\scripts\build-windows-safe.ps1 test
```

#### Protection Layers

1. **Compile-Time Protection** (`build.rs`)
   - Detects Windows + GPU features
   - Warns about BSOD risks
   - Recommends safe build commands

2. **Runtime Protection** (`guardrails.rs`)
   - Monitors memory/CPU usage
   - Limits concurrent operations
   - Auto-throttles under load

3. **Safe Build Profiles**
   - `--profile=safe` - Single-threaded, no GPU
   - `--profile=test-safe` - Safe testing
   - `--no-default-features` - CPU-only build

4. **Configuration**
   - `config.windows.yml` - Windows-optimized settings
   - Limited parallelism (max 2 threads)
   - Disabled intensive features

#### Why BSODs Happen

- **GPU Drivers**: `hive-gpu` + `fastembed` load kernel-mode drivers
- **ONNX Runtime**: DirectML can crash on Windows
- **Heavy Parallelism**: Thread explosion exhausts resources
- **Memory Pressure**: Large embeddings + parallel builds

#### Safe Build Options

```bash
# ✅ SAFEST - No GPU, minimal features
cargo build --profile=safe --no-default-features

# ✅ SAFE - Fastembed only (ONNX on CPU)
cargo build --profile=safe --no-default-features --features "fastembed"

# ⚠️ RISKY - GPU enabled (requires latest drivers)
cargo build --profile=safe --no-default-features --features "hive-gpu"

# ❌ DANGER - Will likely cause BSOD on Windows
cargo build --release --all-features
```

#### Documentation

- **[BSOD Analysis](docs/BSOD_ANALYSIS.md)** - Root cause analysis
- **[Windows Build Guide](docs/WINDOWS_BUILD_GUIDE.md)** - Complete guide
- **[Guardrails](docs/GUARDRAILS.md)** - Protection system details
- **[Guardrails README](README_GUARDRAILS.md)** - Quick reference

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

#### Windows (Safe Build)

```powershell
# Clone the repository
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

# RECOMMENDED: Use safe build script
.\scripts\build-windows-safe.ps1

# Or manual safe build
cargo +nightly build --profile=safe --no-default-features

# Run tests safely
.\scripts\build-windows-safe.ps1 test
```

**⚠️ CRITICAL:** On Windows, **NEVER** use `--all-features` or `--release` without `--no-default-features`. This will likely cause BSOD!

#### Linux/macOS (Normal Build)

```bash
# Clone the repository
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

# Build and run (basic - CPU only)
cargo build --release --no-default-features
./target/release/vectorizer

# Build with GPU acceleration (macOS Metal)
cargo build --release --no-default-features --features "hive-gpu"
./target/release/vectorizer

# Build with fastembed (ONNX on CPU - safe)
cargo build --release --no-default-features --features "fastembed"
./target/release/vectorizer

# Build with transmutation support for document conversion
cargo build --release --no-default-features --features "transmutation"
./target/release/vectorizer
```

**⚠️ Windows:** Use `.\scripts\build-windows-safe.ps1` to prevent BSODs  
**💡 Features:** GPU/ML features require `--features` flag (default = none for safety)

### **Access Points**
- **Desktop GUI**: `./gui/` - Electron desktop application (NEW in v0.8.2)
- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp (StreamableHTTP)
- **UMICP**: http://localhost:15002/umicp (Protocol v0.2.1)
- **UMICP Discovery**: http://localhost:15002/umicp/discover (Tool discovery endpoint)
- **Health Check**: http://localhost:15002/health

## 📖 **Basic Usage**

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


## ⚙️ **Configuration**

Main settings in `config.yml` - see [config.example.yml](config.example.yml) for complete options.


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


## 📚 **Documentation**

### Core Documentation
- **[API Reference](./docs/api/)** - REST API documentation
- **[Replication Guide](./docs/REPLICATION.md)** - Master-replica replication system
  - [Test Suite](./docs/REPLICATION_TESTS.md) - 38 comprehensive tests
  - [Coverage Report](./docs/REPLICATION_COVERAGE.md) - 95%+ coverage
- **[Transmutation Integration](./docs/specs/transmutation_integration.md)** - Document conversion guide
- **[MCP Integration](./docs/specs/MCP_INTEGRATION.md)** - Model Context Protocol guide
- **[Technical Specs](./docs/specs/)** - Complete technical documentation
- **[Roadmap](./docs/specs/ROADMAP.md)** - Development roadmap

### Windows Build Safety
- **[Windows Build Guide](./docs/WINDOWS_BUILD_GUIDE.md)** - Safe building on Windows
- **[Guardrails System](./docs/GUARDRAILS.md)** - BSOD protection details

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

**Available MCP Tools** (22 individual tools):

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

### **Performance Monitoring (3 tools)**
20. `get_performance_metrics` - Get detailed performance metrics and cache statistics
21. `clear_cache` - Clear query and collection caches (all, queries, or collections)
22. `health_check` - Comprehensive system health check with collection accessibility

### **Key Improvements in v1.1.2**
- ✅ **No enum parameters** - Direct tool selection by name
- ✅ **Simplified parameters** - Only relevant parameters per tool
- ✅ **Better model accuracy** - Reduced entropy improves tool calling
- ✅ **Faster execution** - Disabled slow features (MMR, cross-encoder) in MCP
- ✅ **Enhanced security** - Dangerous operations restricted to REST API
- ✅ **Performance monitoring** - Built-in metrics collection and health checks
- ✅ **Enhanced error handling** - Specific error types with detailed validation
- ✅ **Cache management** - Query result caching with TTL support

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