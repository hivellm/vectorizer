# Vectorizer

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## ✨ Version 1.2.2

**Latest**: Critical BM25 vocabulary persistence fix - see [CHANGELOG.md](CHANGELOG.md) for details.

## 🎯 Key Features
- **🔍 Semantic Search**: Sub-3ms with HNSW + multiple distance metrics
- **🧠 Embeddings**: BM25, TF-IDF, BERT, MiniLM, custom models
- **📄 Document Conversion**: PDF, DOCX, XLSX, PPTX → Markdown (98x faster than Docling)
- **🚀 GPU Acceleration**: Metal (macOS), optional CUDA support
- **🔄 Replication**: Master-replica system (BETA)
- **🎯 MCP Integration**: 19 tools for AI assistants
- **🔒 Security**: JWT + API Key auth with RBAC
- **💾 Compact Storage**: .vecdb format with 20-30% compression

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

### Access Points
- **REST API**: http://localhost:15002
- **MCP Server**: http://localhost:15002/mcp
- **Dashboard**: http://localhost:15002/

### Basic Usage
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

## 🧠 MCP Integration

19 focused tools for AI assistants - see [MCP documentation](docs/specs/MCP_INTEGRATION.md) for details.

## 📚 Configuration

See `config.example.yml` for all options. Key settings:

```yaml
vectorizer:
  host: "localhost"
  port: 15002
  default_dimension: 512
  default_metric: "cosine"
```

## 📊 Performance

- **Search**: < 3ms (CPU), < 1ms (Metal GPU)
- **PDF Conversion**: 98x faster than Docling
- **Compression**: 20-30% with .vecdb format

## 📚 Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history and release notes
- [docs/](docs/) - Technical specifications and guides

## 🔧 MCP Integration

Cursor IDE: `http://localhost:15002/mcp` (streamablehttp)

19 tools available - see [MCP docs](docs/specs/MCP_INTEGRATION.md)

## 📦 Client SDKs

- **Python**: `pip install vectorizer-sdk`
- **TypeScript**: `npm install @hivellm/vectorizer-sdk`
- **Rust**: `cargo add vectorizer-sdk`

## 📄 License

MIT - See [LICENSE](./LICENSE)