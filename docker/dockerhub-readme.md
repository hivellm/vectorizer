# Vectorizer

[![Docker Pulls](https://img.shields.io/docker/pulls/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Stars](https://img.shields.io/docker/stars/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Image Size](https://img.shields.io/docker/image-size/hivehub/vectorizer/latest)](https://hub.docker.com/r/hivehub/vectorizer)

**🐳 Docker Hub**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications. v3.0.0 ships a binary RPC transport (MessagePack over TCP, port `15503`) as the recommended primary channel alongside REST + MCP on `15002`.

## 🚀 Quick Start

### Basic Usage

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -p 15503:15503 \
  --restart unless-stopped \
  hivehub/vectorizer:3.0.0
```

First boot creates an admin user and writes credentials to `/root/.local/share/vectorizer/.root_credentials` inside the container (read with `docker exec` + `cat` or `docker cp` — the image is distroless so there's no shell). Rotate via the dashboard or `/auth` API as soon as you've copied them.

### With Persistent Data

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -p 15503:15503 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  --restart unless-stopped \
  hivehub/vectorizer:3.0.0
```

The `/vectorizer/data` mount holds the `.vecdb` store, `config.yml`, `workspace.yml`, logs, and the root-credentials file. One mount is enough — the binary creates everything underneath on first boot.

### Docker Compose

```yaml
services:
  vectorizer:
    image: hivehub/vectorizer:3.0.0
    container_name: vectorizer
    # Distroless nonroot (UID 65532) refuses host-UID bind mounts on
    # Docker Desktop for Windows / macOS; flip to `user: root` if your
    # host can't align UIDs. On Linux with a named volume or a
    # UID-65532-owned bind mount, remove this.
    # user: root
    ports:
      - "15503:15503"   # VectorizerRPC (binary MessagePack over TCP — primary)
      - "15002:15002"   # REST + MCP + GraphQL + Dashboard
    volumes:
      - ./vectorizer-data:/vectorizer/data
    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - VECTORIZER_AUTH_ENABLED=true
      - VECTORIZER_ADMIN_USERNAME=admin
      - VECTORIZER_ADMIN_PASSWORD=change-me-in-production
      - VECTORIZER_JWT_SECRET=change-this-to-a-random-32-char-secret
    restart: unless-stopped
```

> ⚠️ **Healthcheck note.** The image is **distroless** — there's no `curl`, `wget`, or `sh`. `test: ["CMD", "curl", ...]` will always mark the container unhealthy. Use an external TCP probe, a reverse-proxy healthcheck against `/health`, or a Kubernetes `httpGet` liveness/readiness probe instead.

## ✨ Features (v3.0.0)

- **⚡ VectorizerRPC** — length-prefixed MessagePack over raw TCP on port `15503`, ~10× lower per-frame overhead than REST/JSON. Default binary transport across every SDK (Rust, TypeScript, Go, Python, C#).
- **🔍 Semantic Search** — Cosine / Euclidean / Dot Product, HNSW indexing, sub-3 ms typical search, hybrid dense + sparse (BM25) with rank fusion.
- **⚡ SIMD Acceleration** — AVX2 on x86_64, NEON on aarch64, scalar fallback. CPU-feature detection at boot.
- **🧠 Embeddings** — BM25 (default, 512-dim), TF-IDF, and **FastEmbed ONNX** models wired into the server bootstrap: `all-MiniLM-L6-v2`, `bge-small-en-v1.5`, `bge-base-en-v1.5`, `bge-large-en-v1.5`, and `-q` int8-quantized variants (selected via `embedding.model` in `config.yml`).
- **💾 Compact Storage** — unified `.vecdb` format with 20–30% space savings, MMap support for datasets larger than RAM, automatic snapshots.
- **📦 Quantization** — Scalar + Product Quantization (PQ) for up to 64× memory reduction with minimal accuracy loss.
- **🔄 Replication & Sharding** — master → replica TCP streaming (BETA), openraft-backed consensus for HA clusters.
- **📄 Document Conversion** — built-in pipelines for PDF, DOCX, XLSX, PPTX, HTML, XML, images.
- **🔄 Qdrant Compatibility** — drop-in `/qdrant/collections/{name}/points/*` surface for migrations.
- **🎯 MCP Integration** — focused tool-per-action MCP 2025-03-26 server on `POST /mcp` (streamable HTTP).
- **🕸️ Graph Relationships** — relationship discovery + traversal, GUI-backed.
- **🔒 Auth enforcement** — JWT + API Key with RBAC gating **every** data route when `auth.enabled: true`.
- **📊 Observability** — Prometheus metrics at `/prometheus/metrics`, OpenTelemetry OTLP export, structured tracing via `RUST_LOG`.

## 📦 Tags

| Tag | Points to | Notes |
|---|---|---|
| `3.0.0` | v3.0.0 release | Current stable. Workspace-split crates, RPC-default, Edition 2024. |
| `latest` | same as `3.0.0` | Updated on every stable tag. Pin to a specific version in production. |

Older `1.x` / `2.x` tags remain on Docker Hub for rollback but are no longer receiving updates.

## 🔌 Access Points

| Endpoint | Purpose |
|---|---|
| `tcp://localhost:15503` | **VectorizerRPC** (binary MessagePack — primary transport) |
| `http://localhost:15002` | REST API |
| `http://localhost:15002/dashboard/overview` | Web Dashboard (React + TS) |
| `http://localhost:15002/dashboard/setup` | First-run setup wizard |
| `http://localhost:15002/dashboard/docs` | Swagger/OpenAPI browser |
| `http://localhost:15002/graphql` | GraphQL endpoint |
| `http://localhost:15002/mcp` | MCP server (streamable HTTP, protocol `2025-03-26`) |
| `http://localhost:15002/umicp` | UMICP transport discovery |
| `http://localhost:15002/health` | Health check (anonymous, returns `{"status":"healthy","version":"3.0.0",...}`) |
| `http://localhost:15002/prometheus/metrics` | Prometheus scrape target |

## 🛠️ Configuration

### Environment Variables

| Variable | Default | Purpose |
|---|---|---|
| `VECTORIZER_HOST` | `0.0.0.0` | Bind address. |
| `VECTORIZER_PORT` | `15002` | REST + MCP + dashboard port. |
| `VECTORIZER_AUTH_ENABLED` | *(unset)* | Set to `true` to gate data routes behind JWT/API-key. |
| `VECTORIZER_ADMIN_USERNAME` | `admin` | Admin username seeded on first boot. |
| `VECTORIZER_ADMIN_PASSWORD` | *(prompted on boot)* | Admin password. Set this in production or the server writes a generated one to the `.root_credentials` file. |
| `VECTORIZER_JWT_SECRET` | *(generated)* | Minimum 32 chars for production; share across HA-cluster nodes so JWTs are portable. |
| `RUST_LOG` | `info` | Per-module tracing filter, e.g. `vectorizer=debug,hyper=info`. |
| `TZ` | `Etc/UTC` | Container timezone. |
| `RUN_MODE` | `production` | `production` or `development`. |

### Config Files

- Mount `config.yml` to `/vectorizer/config/config.yml` to override defaults (embedding model, quantization, HNSW params, auth mode, replication topology).
- Mount `workspace.yml` to `/vectorizer/workspace.yml` + bind the source tree as `/workspace:ro` for monorepo indexing (the file-watcher service re-indexes on change).

## 📝 Examples

### REST — create / insert / search

```bash
# Login to get a JWT
TOKEN=$(curl -sS -X POST http://localhost:15002/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin"}' \
  | jq -r .access_token)

# Create a collection (dim 512 matches the default BM25 embedder)
curl -X POST http://localhost:15002/collections \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"docs","dimension":512,"metric":"cosine"}'

# Insert text with automatic embedding generation
curl -X POST http://localhost:15002/insert_texts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "collection":"docs",
    "texts":[
      {"id":"doc_1","text":"Vectorizer is a high-performance vector database","metadata":{"tag":"intro"}},
      {"id":"doc_2","text":"Rust is a systems programming language focused on safety","metadata":{"tag":"rust"}}
    ]
  }'

# Semantic search by text
curl -X POST http://localhost:15002/collections/docs/search/text \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"query":"vector database","limit":10}'

# Direct vector search
curl -X POST http://localhost:15002/search \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"collection":"docs","vector":[0.1, 0.2, 0.3, ...], "limit":10}'
```

### API Key (preferred for non-interactive clients)

```bash
# Create a long-lived key
curl -X POST http://localhost:15002/auth/keys \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"my-service","permissions":["read","write","create_collection","delete_collection"]}'

# Use it via X-API-Key header
curl -H "X-API-Key: <api-key-from-response>" http://localhost:15002/collections
```

### MCP (Claude Code / Cursor / any MCP client)

```json
{
  "mcpServers": {
    "vectorizer": {
      "type": "http",
      "url": "http://localhost:15002/mcp",
      "headers": { "X-API-Key": "<api-key>" }
    }
  }
}
```

### Graph Relationships

```bash
curl -X POST http://localhost:15002/graph/discover/docs \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"similarity_threshold":0.7,"max_per_node":10}'
```

## 🏷️ Image Details

- **Base Image**: `gcr.io/distroless/cc-debian12:nonroot` (minimal attack surface, no shell, near-zero CVEs)
- **Default User**: nonroot (UID 65532)
- **Architectures**: `linux/amd64`, `linux/arm64` (multi-arch manifest)
- **Compressed Size**: ~66 MB (v3.0.0 default build, `--no-default-features` excludes ONNX/GPU)
- **Rust Edition**: 2024 (mandatory, pinned rustc ≥ 1.90 per async-graphql / asynk-strim floor)
- **Build Flags**: `--package vectorizer-server --bin vectorizer --no-default-features`
- **Supply Chain**: SPDX SBOM embedded at `/vectorizer/vectorizer.spdx.json`, OpenContainer labels for revision, source, and licenses
- **License**: Apache-2.0

## 🔧 Advanced Usage

### Monorepo Indexing

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 -p 15503:15503 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/workspace.yml:/vectorizer/workspace.yml \
  -v $(pwd):/workspace:ro \
  --restart unless-stopped \
  hivehub/vectorizer:3.0.0
```

### Debug Logging

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -e RUST_LOG="vectorizer=debug,vectorizer::replication=trace,hyper=info" \
  -e RUST_BACKTRACE=1 \
  hivehub/vectorizer:3.0.0
```

### Pinning by Digest (Production)

```bash
docker pull hivehub/vectorizer@sha256:f49699dbe49e6399b8d144bd64804a2388ea91f7dcd81990a62aea3424d2ca59
```

## 🤝 Support

- **🐳 Docker Hub**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)
- **GitHub**: https://github.com/hivellm/vectorizer
- **Issues**: https://github.com/hivellm/vectorizer/issues
- **Discussions**: https://github.com/hivellm/vectorizer/discussions

## 📄 License

Apache-2.0 License — see [LICENSE](https://github.com/hivellm/vectorizer/blob/main/LICENSE).

---

**📦 Pull:**
```bash
docker pull hivehub/vectorizer:3.0.0
docker pull hivehub/vectorizer:latest
```

**🔗 Repository**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)
