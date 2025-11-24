# Vectorizer

[![Docker Pulls](https://img.shields.io/docker/pulls/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Stars](https://img.shields.io/docker/stars/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Image Size](https://img.shields.io/docker/image-size/hivehub/vectorizer/latest)](https://hub.docker.com/r/hivehub/vectorizer)

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
    image: USERNAME/vectorizer:latest
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

- **🔍 Semantic Search**: Advanced vector similarity with multiple distance metrics
- **⚡ High Performance**: Sub-3ms search times with HNSW indexing
- **💾 Memory-Mapped Storage**: MMap support for datasets larger than RAM
- **🚀 GPU Acceleration**: Metal GPU support for macOS (Apple Silicon)
- **📦 Product Quantization**: PQ compression for 64x memory reduction
- **🖥️ Web Dashboard**: Modern React + TypeScript dashboard included
- **🕸️ Graph Relationships**: Automatic relationship discovery and traversal
- **🔒 Security**: JWT + API Key authentication with RBAC

## 📖 Documentation

- **GitHub**: https://github.com/hivellm/vectorizer
- **Documentation**: https://github.com/hivellm/vectorizer/docs
- **API Reference**: http://localhost:15002/docs (when running)

## 🔌 Access Points

When running, Vectorizer provides:

- **REST API**: http://localhost:15002
- **Web Dashboard**: http://localhost:15002/dashboard
- **MCP Server**: ws://localhost:15002/mcp
- **Health Check**: http://localhost:15002/health

## 📦 Tags

- `latest` - Latest stable release
- `1.5.0` - Specific version tag
- `1.5` - Minor version tag

## 🛠️ Configuration

Vectorizer can be configured via:

1. **Environment Variables**:
   - `VECTORIZER_HOST` - Bind address (default: `0.0.0.0`)
   - `VECTORIZER_PORT` - Port number (default: `15002`)
   - `VECTORIZER_LOG_LEVEL` - Log level (default: `info`)

2. **Config File**: Mount `config.yml` to `/vectorizer/config.yml`

## 📝 Examples

### Basic Search

```bash
# Create a collection
curl -X POST http://localhost:15002/api/v1/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "docs", "dimension": 128, "metric": "cosine"}'

# Insert vectors
curl -X POST http://localhost:15002/api/v1/collections/docs/vectors \
  -H "Content-Type: application/json" \
  -d '{"vectors": [{"id": "1", "data": [0.1, 0.2, ...]}]}'

# Search
curl -X POST http://localhost:15002/api/v1/collections/docs/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, ...], "limit": 10}'
```

## 🏷️ Image Details

- **Base Image**: `debian:13-slim`
- **Architecture**: `linux/amd64`, `linux/arm64`
- **Size**: ~100MB (compressed)
- **License**: Apache-2.0

## 🤝 Support

- **Issues**: https://github.com/hivellm/vectorizer/issues
- **Discussions**: https://github.com/hivellm/vectorizer/discussions

## 📄 License

Apache-2.0 License - see [LICENSE](https://github.com/hivellm/vectorizer/blob/main/LICENSE) file for details.

