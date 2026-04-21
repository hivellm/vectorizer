# Vectorizer

[![Docker Pulls](https://img.shields.io/docker/pulls/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Stars](https://img.shields.io/docker/stars/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Image Size](https://img.shields.io/docker/image-size/hivehub/vectorizer/latest)](https://hub.docker.com/r/hivehub/vectorizer)

**🐳 Docker Hub**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications.

## 🚀 Quick Start

### Basic Usage

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

### With Persistent Data

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorizer-snapshots:/vectorizer/snapshots \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

### Docker Compose

```yaml
version: "3.8"

services:
  vectorizer:
    image: hivehub/vectorizer:latest
    container_name: vectorizer
    ports:
      - "15002:15002"
    volumes:
      - ./vectorizer-data:/vectorizer/data
      - ./vectorizer-storage:/vectorizer/storage
      - ./vectorizer-snapshots:/vectorizer/snapshots
    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:15002/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## ✨ Features

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics (Cosine, Euclidean, Dot Product)
- **⚡ SIMD Acceleration**: AVX2-optimized vector operations (5-10x faster) with automatic CPU feature detection
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **💾 Memory-Mapped Storage**: MMap support for datasets larger than RAM with efficient OS paging
- **🚀 GPU Acceleration**: Metal GPU support for macOS (Apple Silicon) with cross-platform compatibility
- **📦 Product Quantization**: PQ compression for 64x memory reduction with minimal accuracy loss
- **💾 Compact Storage**: Unified `.vecdb` format with 20-30% space savings and automatic snapshots
- **🔄 Master-Replica Replication**: High availability with automatic failover (BETA)
- **🔗 Distributed Sharding**: Horizontal scaling across multiple servers with automatic shard routing (BETA)
- **📄 Document Conversion**: Automatic conversion of PDF, DOCX, XLSX, PPTX, HTML, XML, and images
- **🔄 Qdrant Migration**: Complete migration tools for seamless transition from Qdrant
- **🎯 MCP Integration**: 20 focused individual tools for AI model integration
- **🔄 UMICP Protocol**: Native JSON types + Tool Discovery endpoint
- **🖥️ Web Dashboard**: Modern React + TypeScript dashboard with complete graph management interface
- **🖥️ Desktop GUI**: Electron-based desktop application with vis-network graph visualization
- **🕸️ Graph Relationships**: Automatic relationship discovery and graph traversal with full GUI support
- **🧠 Multiple Embeddings**: TF-IDF, BM25, BERT, MiniLM, and custom models
- **📊 Structured Logging**: Built-in tracing support for observability and debugging
- **🔒 Security**: JWT + API Key authentication with RBAC

## 📖 Documentation

- **GitHub**: https://github.com/hivellm/vectorizer
- **Documentation**: https://github.com/hivellm/vectorizer/docs
- **API Reference**: http://localhost:15002/docs (when running)

## 🔌 Access Points

When running, Vectorizer provides:

- **REST API**: http://localhost:15002
- **Web Dashboard**: http://localhost:15002/dashboard
- **MCP Server**: ws://localhost:15002/mcp (Model Context Protocol for AI integration)
- **UMICP Server**: ws://localhost:15002/umicp (Universal Model Interface Context Protocol)
- **Health Check**: http://localhost:15002/health
- **API Documentation**: http://localhost:15002/docs (Swagger/OpenAPI)

## 📦 Tags

- `latest` - Latest stable release (currently v1.5.0)
- `1.5.0` - Specific version tag
- `1.5` - Minor version tag

## 🛠️ Configuration

Vectorizer can be configured via:

1. **Environment Variables**:
   - `VECTORIZER_HOST` - Bind address (default: `0.0.0.0`)
   - `VECTORIZER_PORT` - Port number (default: `15002`)
   - `VECTORIZER_LOG_LEVEL` - Log level: `trace`, `debug`, `info`, `warn`, `error` (default: `info`)
   - `RUST_LOG` - Advanced tracing configuration (e.g., `vectorizer=debug,hyper=info`)

2. **Config File**: Mount `config.yml` to `/vectorizer/config.yml`

3. **Workspace Configuration**: Mount `workspace.yml` to `/vectorizer/workspace.yml` for monorepo indexing

## 📝 Examples

### Basic Search

```bash
# Create a collection
curl -X POST http://localhost:15002/api/v1/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "docs", "dimension": 128, "similarity_metric": "cosine"}'

# Insert text (automatic embedding generation)
curl -X POST http://localhost:15002/api/v1/collections/docs/texts \
  -H "Content-Type: application/json" \
  -d '{"texts": [{"id": "1", "text": "Vectorizer is a high-performance vector database"}]}'

# Search by text (semantic search)
curl -X POST http://localhost:15002/api/v1/collections/docs/search/text \
  -H "Content-Type: application/json" \
  -d '{"query": "vector database", "limit": 10}'

# Or insert vectors directly
curl -X POST http://localhost:15002/api/v1/collections/docs/vectors \
  -H "Content-Type: application/json" \
  -d '{"vectors": [{"id": "1", "data": [0.1, 0.2, 0.3, ...]}]}'

# Search by vector
curl -X POST http://localhost:15002/api/v1/collections/docs/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, 0.3, ...], "limit": 10}'
```

### Graph Relationships

```bash
# Discover relationships in a collection
curl -X POST http://localhost:15002/api/v1/graph/discover/docs \
  -H "Content-Type: application/json" \
  -d '{"similarity_threshold": 0.7, "max_per_node": 10}'

# List all nodes
curl http://localhost:15002/api/v1/graph/nodes/docs

# Find related nodes
curl -X POST http://localhost:15002/api/v1/graph/nodes/docs/node1/related \
  -H "Content-Type: application/json" \
  -d '{"max_hops": 2}'
```

## 🏷️ Image Details

- **Base Image**: `debian:13-slim`
- **Architecture**: `linux/amd64`, `linux/arm64`
- **Size**: ~100MB (compressed)
- **Rust Edition**: 2024
- **Rust Version**: 1.92+
- **License**: Apache-2.0

## 🔧 Advanced Usage

### With Workspace Configuration (Monorepo Indexing)

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/workspace.yml:/vectorizer/workspace.yml:ro \
  -v $(pwd)/workspace:/workspace:ro \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

### Custom Logging Configuration

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -e RUST_LOG="vectorizer=debug,hyper=info" \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

## 🤝 Support

- **🐳 Docker Hub**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)
- **GitHub**: https://github.com/hivellm/vectorizer
- **Issues**: https://github.com/hivellm/vectorizer/issues
- **Discussions**: https://github.com/hivellm/vectorizer/discussions

## 📄 License

Apache-2.0 License - see [LICENSE](https://github.com/hivellm/vectorizer/blob/main/LICENSE) file for details.

---

**📦 Pull the image:**
```bash
docker pull hivehub/vectorizer:latest
```

**🔗 Repository**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)

