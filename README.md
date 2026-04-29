# Vectorizer

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![Rust Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/vectorizer.svg)](https://crates.io/crates/vectorizer)
[![GitHub release](https://img.shields.io/github/release/hivellm/vectorizer.svg)](https://github.com/hivellm/vectorizer/releases)
[![Production Ready](https://img.shields.io/badge/status-production%20ready-success.svg)](https://github.com/hivellm/vectorizer)

High-performance vector database and search engine in Rust for semantic search, document indexing, and AI applications. Ships as a Cargo workspace (5 crates) with binary RPC + HTTP transports, a React dashboard, and native SDKs for Rust, Python, TypeScript, Go, and C#.

## ✨ Key Features

### Transport & API
- **VectorizerRPC** (default, port `15503`) — binary MessagePack over TCP, multiplexed connection pool. See [wire spec](docs/specs/VECTORIZER_RPC.md).
- **REST API** (port `15002`) — universal HTTP fallback, powers the dashboard and any caller that doesn't speak raw TCP.
- **gRPC** — Qdrant-compatible service.
- **GraphQL** — full REST parity with async-graphql + GraphiQL playground.
- **MCP** — 31 focused tools for AI model integration (Cursor, Claude Desktop, etc.).
- **UMICP Protocol** — native JSON types + tool discovery endpoint.

### Performance
- **SIMD acceleration** — AVX2-optimized vector ops with runtime CPU detection (5-10x faster).
- **Metal GPU** — macOS Apple Silicon via [`hive-gpu`](https://github.com/hivellm/hive-gpu) 0.2; logs render real device name, driver, VRAM.
- **Sub-3ms search** (CPU) / **<1ms** (GPU) via HNSW indexing.
- **4-5x faster than Qdrant** in head-to-head benchmarks (0.16-0.23ms vs 0.80-0.87ms avg latency).

### Storage
- **`.vecdb` unified format** — 20-30% space savings, automatic snapshots.
- **Memory-mapped storage** — datasets larger than RAM, efficient OS paging.
- **Product Quantization** — 64x memory reduction with minimal accuracy loss.
- **Scalar Quantization** + cache hit ratio metrics.

### High Availability & Scaling
- **Raft consensus** via openraft (pinned `=0.10.0-alpha.17`) — automatic leader election in 1-5s, write-redirect via HTTP 307, WAL-backed durable replication, DNS discovery for Kubernetes headless services.
- **Master-Replica** — TCP streaming replication with full/partial sync, exponential reconnect backoff (5s→60s).
- **Distributed sharding** — horizontal scaling with automatic routing; distributed hybrid search via `RemoteHybridSearch` RPC with dense-only fallback for mixed-version clusters.
- **HiveHub cluster mode** — multi-tenant with quotas, usage tracking, tenant isolation, mandatory MMap storage, 1GB cache cap.

### Search
- **Semantic similarity** — Cosine, Euclidean, Dot Product.
- **Hybrid search** — Dense + Sparse with Reciprocal Rank Fusion (RRF).
- **Intelligent search** — query expansion, semantic reranking.
- **Multi-collection search** across projects.
- **Graph relationships** — automatic edge discovery, neighbor exploration, shortest-path finding.

### Embeddings & Docs
- **Built-in providers** — TF-IDF, BM25, FastEmbed, BERT, MiniLM, custom models.
- **Document conversion** — PDF, DOCX, XLSX, PPTX, HTML, XML, images (14 formats).
- **Qdrant API compatibility** — Snapshots, Sharding, Cluster Management, Query (with prefetch), Search Groups, Matrix, Named Vectors (partial), PQ/Binary quantization config.
- **Summarization** — extractive, keyword, sentence, abstractive (OpenAI GPT).

### Security
- **JWT + API Key** authentication with RBAC.
- **JWT secret is mandatory** — boot refuses to start with empty / default / <32 char secrets when auth is enabled.
- **First-run root credentials** written to `{data_dir}/.root_credentials` (0o600), never logged.
- **Payload encryption** — optional ECC-P256 + AES-256-GCM, zero-knowledge, per-collection policies ([docs](docs/features/encryption/README.md)).
- **TLS 1.2/1.3** with mTLS, configurable cipher suites, ALPN.
- **Per-API-key rate limiting** with tiers + overrides.
- **Path-traversal guard** on file discovery; canonicalized base, symlink-escape refusal.

### UI
- **Web Dashboard** — React + TypeScript; JWT login, graph CRUD (edges, neighbors, paths), collection management, API sandbox, setup wizard with glassmorphism design. Embedded in the binary (~26MB, no external assets needed).
- **Desktop GUI** — Electron + vis-network for visual database management.

## 🎉 Latest Release: v3.1.0

Highlights — see [CHANGELOG.md](./CHANGELOG.md) for the full breakdown.

**Added**
- **`POST /insert_vectors`** — bulk-insert pre-computed embeddings with caller-supplied vector ids. Skips the embedding pipeline; the request body carries the vectors as raw `Vec<f32>`. For clients with their own embedder, idempotent re-ingest by client id, or upsert without auto-chunking. See [`docs/users/api/BATCH.md`](docs/users/api/BATCH.md).
- **Client `id` honored on `/insert` and `/insert_texts`** — the `id` field on each text entry is now used as the resulting `Vector.id` (non-chunked) or as the prefix for `<id>#<chunk_index>` chunk ids. Re-ingesting the same id upserts in place instead of duplicating; delete-by-doc and citation round-trips no longer need a UUID lookup.
- **`payload.parent_id`** on chunked vectors — links every chunk back to its source document (the request's `id`, or a single shared UUID v4 when omitted). Lets clients group, count, or delete every chunk of a logical document without re-deriving membership from a defensive `_id` duplicate.

**Changed**
- **`/insert_texts` chunked payload layout flipped from nested to flat — BREAKING for clients that read `payload.metadata.<field>` directly.** Pre-3.1.0 chunks landed as `{content, metadata: {file_path, chunk_index, _id, casa, parlamentar, ...}}` — Qdrant payload filters `payload.parlamentar = "X"` silently missed every chunked row. 3.1.0 emits `{content, file_path, chunk_index, parent_id, _id, casa, parlamentar, ...}` with every key at the root. Server-side readers (`FileOperations`, `file_watcher`, MCP `search_semantic`) tolerate both shapes during the deprecation window. Migration guide: [CHANGELOG `[3.1.0]`](./CHANGELOG.md#migrating-from-30x-chunked-payloads).

## Previous Release: v3.0.0

Highlights — see [CHANGELOG.md](./CHANGELOG.md) for the full breakdown.

**Breaking**
- **RPC is default transport** (`rpc.enabled: true`, port `15503`). REST stays on `15002`. Migration guide: [`docs/migration/rpc-default.md`](docs/migration/rpc-default.md). Opt out with `rpc.enabled: false`.
- **gRPC `SearchResult.score` narrowed `double` → `float`**. Clients on the pre-v3 proto must regenerate.
- **JWT secret must be explicitly configured** — no more insecure default. Generate via `openssl rand -hex 64` and inject via `VECTORIZER_JWT_SECRET`.
- **Configs moved under `config/`** — `config.yml` → `config/config.yml`, presets under `config/presets/`. Legacy `./config.yml` still works with a deprecation warning (removed in v3.1).
- **Cargo workspace split** — `vectorizer-core`, `vectorizer-protocol`, `vectorizer`, `vectorizer-server`, `vectorizer-cli`. Callers reaching into the server layer need to switch from `vectorizer::{server,api,grpc,logging,umicp}::*` to `vectorizer_server::*`.

**Removed**
- **Standalone JavaScript SDK dropped** — TypeScript SDK ships compiled CJS + ESM, usable from plain JS. Migrate `@hivehub/vectorizer-sdk-js` → `@hivehub/vectorizer-sdk`.
- **TypeScript SDK scope is `@hivehub`**, not `@hivellm` (docs corrected).
- **Framework integration packages dropped** — `langchain`, `langchain-js`, `langflow`, `n8n`, `tensorflow`, `pytorch` adapters. Published versions stay installable; integrate against native SDKs directly.

**Added**
- **Layered config loader** — `VECTORIZER_MODE=dev|production` merges `config/modes/<mode>.yml` over base. Deep YAML merge with null-clear semantics. See [`docs/deployment/configuration.md`](docs/deployment/configuration.md).
- **Docker collapsed to one compose** with profiles — `docker compose --profile <default|dev|ha|hub> up -d`.
- **C# SDK RPC transport** (`Vectorizer.Sdk.Rpc` 3.0.0) — TCP + MessagePack framing, connection pool, ASP.NET Core DI.
- **`#![deny(missing_docs)]` + `cargo doc -D warnings` CI gate** — cleared 2,219 missing-docs warnings to 0.
- **`unwrap_used` / `expect_used` denied workspace-wide** — every production `.unwrap()` either returns `Result` or sits behind a documented `#[allow]`.

**Changed**
- **`rmcp` 0.10 → 1.5** — MCP SDK major rewrite; builder-based construction across every handler.
- **Second-pass dep migrations** — reqwest 0.13, arrow/parquet 58, zip 8, tantivy 0.26, hmac 0.13 + sha2 0.11, hf-hub 0.5, sysinfo 0.38, candle 0.10.2, bcrypt 0.19, openraft pinned `=0.10.0-alpha.17`.
- **Frontend majors** — React 19, react-router 7, TypeScript 6 (dashboard), vitest 4, eslint 10, Electron 41, Vue-router 5 (GUI).
- **`parking_lot` migration complete** — all `std::sync::{Mutex,RwLock}` off the hot path; CI grep gate prevents regression.
- **Hot-path `rand` / `hmac` / `tonic 0.14` / `prost 0.14` / `bincode 2.0`** upgraded.

## 🚀 Quick Start

### Install Script (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

Installs CLI + systemd service. Commands: `sudo systemctl {status|restart|stop} vectorizer`, `sudo journalctl -u vectorizer -f`.

### Install Script (Windows)

```powershell
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

Installs CLI + Windows Service (requires Admin). Commands: `Get-Service Vectorizer`, `{Start|Stop|Restart}-Service Vectorizer`.

### Docker

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 -p 15503:15503 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -e VECTORIZER_AUTH_ENABLED=true \
  -e VECTORIZER_ADMIN_USERNAME=admin \
  -e VECTORIZER_ADMIN_PASSWORD=your-secure-password \
  -e VECTORIZER_JWT_SECRET=$(openssl rand -hex 64) \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

**Docker Compose with profiles:**

```bash
cp .env.example .env
# Edit .env with your credentials
docker compose --profile default up -d          # standalone
docker compose --profile dev up -d              # dev overlay
docker compose --profile ha up -d               # Raft cluster
docker compose --profile hub up -d              # multi-tenant
```

Profiles are mutually exclusive on host port `15002`.

Images: [Docker Hub](https://hub.docker.com/r/hivehub/vectorizer) · [GHCR](https://github.com/hivellm/vectorizer/pkgs/container/vectorizer)

### Build from Source

```bash
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

cargo build --release                          # Basic
cargo build --release --features hive-gpu      # macOS Metal
cargo build --release --features full          # All features
./target/release/vectorizer
```

### Access Points

| Surface | URL | Notes |
|---|---|---|
| **VectorizerRPC** (primary) | `vectorizer://localhost:15503` | Binary MessagePack over TCP — see [operator guide](docs/deployment/rpc.md) |
| **REST API** | `http://localhost:15002` | Universal HTTP fallback |
| **Web Dashboard** | `http://localhost:15002/dashboard/` | React UI, embedded in binary |
| **MCP Server** | `http://localhost:15002/mcp` | 31 tools for AI agents |
| **GraphQL** | `http://localhost:15002/graphql` | GraphiQL at `/graphql` |
| **UMICP Discovery** | `http://localhost:15002/umicp/discover` | |
| **Health Check** | `http://localhost:15002/health` | |

> **Upgrading from v2.x?** RPC is now on by default on port `15503`. REST is unchanged. If you can't expose the new port, set `rpc.enabled: false`. See [v3.x migration guide](docs/migration/rpc-default.md).

### Configuration

Configs live under `config/`:

```
config/
├── config.yml             # Base config (your deployment)
├── config.example.yml     # Reference
├── modes/
│   ├── dev.yml            # Layered override: verbose logs, loopback, watcher on
│   └── production.yml     # Layered override: warn logs, larger threads/cache, zstd, scheduled snapshots
└── presets/               # Standalone full configs (legacy style)
    ├── production.yml
    ├── cluster.yml
    ├── hub.yml
    └── development.yml
```

**Layered loader (recommended):**

```bash
VECTORIZER_MODE=production ./target/release/vectorizer
```

Merges `config/modes/production.yml` over `config/config.yml`. Typos in the mode override fail fast at boot.

### Authentication

Auth is **enabled by default in Docker**. Default creds — **change in production**.

```bash
# Login
curl -X POST http://localhost:15002/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'

# JWT in requests
curl http://localhost:15002/collections \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Create API key (JWT required)
curl -X POST http://localhost:15002/auth/keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"name":"Production","permissions":["read","write"],"expires_in_days":90}'

# API key in requests (NO Bearer prefix)
curl http://localhost:15002/collections \
  -H "Authorization: YOUR_API_KEY"
```

| Method | Header | Use case |
|---|---|---|
| JWT | `Authorization: Bearer <token>` | Dashboard, short-lived sessions |
| API Key | `Authorization: <key>` | MCP, CLI, long-lived integrations |

**Production must set:**
- `VECTORIZER_JWT_SECRET` — ≥32 chars, not the historical default. Boot aborts otherwise.
- `VECTORIZER_ADMIN_PASSWORD` — strong, ≥32 chars.

First-run root credentials are written to `{data_dir}/.root_credentials` (0o600), never printed to stdout. Read and delete after first login.

See [Docker Authentication Guide](docs/users/getting-started/DOCKER_AUTHENTICATION.md) and [Security Policy](SECURITY.md).

## 📊 Performance

| Metric | Value |
|---|---|
| Search latency (CPU) | < 3ms |
| Search latency (Metal GPU) | < 1ms |
| Throughput | 4,400-6,000 QPS (vs Qdrant 1,100-1,300) |
| Storage reduction | 20-30% (`.vecdb`) + PQ 64x |
| MCP tools | 31 |
| Document formats | 14 |

### Benchmark vs Qdrant

- **Search**: 4-5x faster (0.16-0.23ms vs 0.80-0.87ms avg latency).
- **Insert**: Fire-and-forget pattern, configurable batch / body limits, background processing.
- **Scenarios**: Small (1K) / Medium (5K) / Large (10K) vectors × dimensions 384 / 512 / 768.

See [Benchmark Documentation](./docs/specs/BENCHMARKING.md).

## 🔄 Feature Comparison

| Feature | Vectorizer | Qdrant | pgvector | Pinecone | Weaviate | Milvus | Chroma |
|---|---|---|---|---|---|---|---|
| **Core** |
| Language | Rust | Rust | C | C++/Go | Go | C++/Go | Python |
| License | Apache 2.0 | Apache 2.0 | PostgreSQL | Proprietary | BSD | Apache 2.0 | Apache 2.0 |
| **APIs** |
| REST | ✅ | ✅ | via PG | ✅ | ✅ | ✅ | ✅ |
| gRPC (Qdrant-compat) | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ |
| GraphQL | ✅ + GraphiQL | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| MCP | ✅ 31 tools | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Binary RPC | ✅ MessagePack | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **SDKs** | Rust, Python, TS, Go, C# | All | All | Most | Most | Most | Python |
| **Performance** |
| Search latency | < 3ms CPU / < 1ms GPU | 1-5ms | 5-50ms | 50-100ms | 10-50ms | 5-20ms | 10-100ms |
| SIMD | ✅ AVX2 | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| GPU | ✅ Metal | ✅ CUDA | ❌ | ✅ Cloud | ❌ | ✅ CUDA | ❌ |
| **Storage** |
| HNSW | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| PQ (64x) | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Scalar Quantization | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| MMap | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ |
| **Advanced** |
| Graph Relationships | ✅ auto + GUI | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Document Processing | ✅ 14 formats | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Hybrid Search | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Query Expansion | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Qdrant API compat | ✅ + migration | N/A | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Scaling** |
| Sharding | ✅ | ✅ | via PG | ✅ Cloud | ✅ | ✅ | ❌ |
| Replication | ✅ Raft + Master-Replica | ✅ | via PG | ✅ Cloud | ✅ | ✅ | ❌ |
| **Management** |
| Dashboard | ✅ React + graph GUI | ✅ basic | pgAdmin | ✅ Cloud | ✅ | ✅ | ✅ basic |
| Desktop GUI | ✅ Electron | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Security** |
| JWT + API Keys | ✅ | ✅ | via PG | ✅ Cloud | ✅ | ✅ | ✅ |
| Payload Encryption | ✅ ECC-P256 + AES-GCM | ❌ | via PG | ✅ Cloud | ❌ | ❌ | ❌ |

### Key Differentiators

- **MCP integration** (31 tools) — native AI-agent protocol.
- **Graph relationships** — auto-discovery + full GUI (edges, path-finding, neighbor exploration).
- **GraphQL** — full REST parity + GraphiQL.
- **Document processing** — 14 formats built in.
- **Qdrant compatibility** — full API + migration tools.
- **Performance** — 4-5x faster than Qdrant in benchmarks.
- **Binary RPC default** — MessagePack over TCP on port 15503 for low-overhead client traffic.
- **Complete SDK coverage** — Rust, Python, TypeScript (+JS), Go, C# — all on v3.0.0.

**Best fit:** AI apps needing MCP, document ingestion, graph relationships, and sub-ms search with an embedded dashboard.

## 🎯 Use Cases

- **RAG systems** — semantic search with automatic document conversion.
- **Document search** — PDFs, Office, web content.
- **Code analysis** — semantic code navigation.
- **Knowledge bases** — enterprise multi-format search.

## 🔧 MCP Integration

Cursor / Claude Desktop config:

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

### Available Tools (31)

**Core operations (9)**
`list_collections` · `create_collection` · `get_collection_info` · `insert_text` · `get_vector` · `update_vector` · `delete_vector` · `search` · `multi_collection_search`

**Advanced search (4)**
`search_intelligent` (query expansion) · `search_semantic` (reranking) · `search_extra` (combined) · `search_hybrid` (dense + sparse RRF)

**Discovery & files (7)**
`filter_collections` · `expand_queries` · `get_file_content` · `list_files` · `get_file_chunks` · `get_project_outline` · `get_related_files`

**Graph (8)**
`graph_list_nodes` · `graph_get_neighbors` · `graph_find_related` · `graph_find_path` · `graph_create_edge` · `graph_delete_edge` · `graph_discover_edges` · `graph_discover_status`

**Maintenance (3)**
`list_empty_collections` · `cleanup_empty_collections` · `get_collection_stats`

> Cluster-management operations are REST-only for security.

## 📦 Client SDKs

Server-side at **v3.1.0**. The Rust SDK tracks server versioning and is also at v3.1.0; the TypeScript, Python, Go, and C# SDKs are on v3.0.x and bump when they need a breaking server contract. The TypeScript SDK ships compiled CJS + ESM — usable from plain JavaScript, no separate JS package needed.

| SDK | Install |
|---|---|
| Python | `pip install vectorizer-sdk` |
| TypeScript / JS | `npm install @hivehub/vectorizer-sdk` |
| Rust | `cargo add vectorizer-sdk` |
| C# | `dotnet add package Vectorizer.Sdk` (REST) · `Vectorizer.Sdk.Rpc` (RPC) |
| Go | `go get github.com/hivellm/vectorizer-sdk-go` |

Every SDK accepts both `vectorizer://host[:port]` (RPC, default port 15503) and `http(s)://host[:port]` (REST) URLs through the same endpoint parser.

## 🔄 Qdrant Migration

- **Config migration** — parse Qdrant YAML/JSON → Vectorizer format.
- **Data migration** — export from Qdrant, import into Vectorizer.
- **Validation** — integrity + compatibility checks.
- **REST compatibility** — full Qdrant API at `/qdrant/*`.

```rust
use vectorizer::migration::qdrant::{QdrantDataExporter, QdrantDataImporter};

let exported = QdrantDataExporter::export_collection(
    "http://localhost:6333",
    "my_collection"
).await?;

let result = QdrantDataImporter::import_collection(&store, &exported).await?;
```

See [Qdrant Migration Guide](./docs/specs/QDRANT_MIGRATION.md).

## ☁️ HiveHub Cloud

Multi-tenant cluster mode integration with [HiveHub.Cloud](https://hivehub.cloud).

- **Tenant isolation** — owner-scoped collections.
- **Quota enforcement** — collections / vectors / storage per tenant.
- **Usage tracking** — automatic reporting.
- **User-scoped backups**.

```yaml
hub:
  enabled: true
  api_url: "https://api.hivehub.cloud"
  tenant_isolation: "collection"
  usage_report_interval: 300
```

```bash
export HIVEHUB_SERVICE_API_KEY="your-service-api-key"
```

**Cluster-mode requirements** (enforced at boot):

| Requirement | Default |
|---|---|
| MMap storage (Memory storage rejected) | Enforced |
| Max cache memory across all caches | 1 GB |
| File watcher | Disabled |
| Strict config validation | Enabled |

```yaml
cluster:
  enabled: true
  node_id: "node-1"
  memory:
    max_cache_memory_bytes: 1073741824
    enforce_mmap_storage: true
    disable_file_watcher: true
    strict_validation: true
```

See [HiveHub Integration](./docs/features/HUB_INTEGRATION.md) and [Cluster Memory Limits](./docs/specs/CLUSTER_MEMORY.md).

## 🏗️ Workspace Layout

```
crates/
├── vectorizer-core/       # Foundation: error, codec, quantization, simd, compression, paths
├── vectorizer-protocol/   # RPC wire types + tonic-generated gRPC
├── vectorizer/            # Engine (umbrella): db, embedding, models, cache, persistence, search, ...
├── vectorizer-server/     # Transport: HTTP / gRPC / MCP / RPC + binary
└── vectorizer-cli/        # CLI binaries
sdks/rust/                 # Rust SDK — re-exports vectorizer-protocol wire types
```

Runtime directories resolve to platform-standard locations (`~/.local/share/vectorizer/` on Linux, `~/Library/Application Support/vectorizer/` on macOS, `%APPDATA%\vectorizer\` on Windows), overridable via `VECTORIZER_DATA_DIR` / `VECTORIZER_LOGS_DIR`.

## 📚 Documentation

- [User Documentation](./docs/users/) — install + tutorials
- [API Reference](./docs/specs/API_REFERENCE.md) — REST
- [VectorizerRPC Spec](./docs/specs/VECTORIZER_RPC.md) — wire protocol
- [RPC Operator Guide](./docs/deployment/rpc.md)
- [Configuration](./docs/deployment/configuration.md) — layered loader
- [v3.x Migration](./docs/migration/rpc-default.md) — RPC-default rollout
- [Dashboard Integration](./docs/features/DASHBOARD_INTEGRATION.md)
- [Qdrant Compatibility](./docs/users/qdrant/)
- [HiveHub Integration](./docs/features/HUB_INTEGRATION.md)
- [Cluster Memory Limits](./docs/specs/CLUSTER_MEMORY.md)
- [MCP Guide](./docs/specs/MCP.md)
- [Encryption](./docs/features/encryption/README.md)
- [Technical Specs](./docs/specs/) — architecture, performance, implementation

## 📄 License

Apache License 2.0 — see [LICENSE](./LICENSE).

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).
