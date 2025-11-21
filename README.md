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
- **📄 Document Conversion**: Automatic conversion of PDF, DOCX, XLSX, PPTX, HTML, XML, and images
- **🔄 Qdrant Migration**: Complete migration tools for seamless transition from Qdrant
- **🎯 MCP Integration**: 20 focused individual tools for AI model integration
- **🔄 UMICP Protocol**: Native JSON types + Tool Discovery endpoint
- **🖥️ Desktop GUI**: Electron-based desktop application for visual database management
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **🧠 Multiple Embeddings**: TF-IDF, BM25, BERT, MiniLM, and custom models
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

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest
```

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

## 🔧 Recent Improvements (v1.4.0)

### Test Suite Enhancements

- **✅ Fixed**: SIMD vector operations - Improved precision handling for large vectors
- **✅ Fixed**: Product Quantization - Corrected compression ratio calculations
- **✅ Fixed**: MMap storage - Added header persistence for reliable data recovery
- **✅ Fixed**: WAL tests - Improved test reliability with proper metric handling
- **✅ Improved**: Test execution time - Slow tests marked as optional (run with `--ignored`)

### Quality Improvements

- **✅ All core tests passing**: 703+ tests with comprehensive coverage
- **✅ Better error handling**: Improved dimension validation and error messages
- **✅ Storage reliability**: MMap storage now properly persists vector counts
- **✅ Test stability**: Timeout protection prevents hanging tests

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

- **Python**: `pip install vectorizer-sdk`
- **TypeScript**: `npm install @hivellm/vectorizer-sdk`
- **Rust**: `cargo add vectorizer-sdk`
- **JavaScript**: `npm install @hivellm/vectorizer-sdk-js`

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
- **[Qdrant Compatibility](./docs/users/qdrant/)** - Qdrant API compatibility and migration guide
- **[Technical Specifications](./docs/specs/)** - Architecture, performance, and implementation guides
- **[MCP Integration](./docs/specs/MCP.md)** - Model Context Protocol guide

## 📄 License

Apache License 2.0 - See [LICENSE](./LICENSE) for details

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.
