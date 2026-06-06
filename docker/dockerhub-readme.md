# Vectorizer

[![Docker Pulls](https://img.shields.io/docker/pulls/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Stars](https://img.shields.io/docker/stars/hivehub/vectorizer.svg)](https://hub.docker.com/r/hivehub/vectorizer)
[![Docker Image Size](https://img.shields.io/docker/image-size/hivehub/vectorizer/latest)](https://hub.docker.com/r/hivehub/vectorizer)

**🐳 Docker Hub**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)

A high-performance vector database and search engine built in Rust, designed for semantic search, document indexing, and AI-powered applications. The v3.x line ships a binary RPC transport (MessagePack over TCP, port `15503`) as the recommended primary channel alongside REST + MCP on `15002`.

**v3.4.0 highlights**:

- **Fixed — container deployments lose all collections on restart** ([#300](https://github.com/hivellm/vectorizer/issues/300)). 3.3.0 wrote persistent state to `/.local/share/vectorizer/` even though the README advertised `/data`; the XDG path lived on the container's writable layer, so `docker compose up -d --force-recreate` silently wiped every collection. 3.4.0 defaults `VECTORIZER_DATA_DIR=/data` and the canonical mount is a single `--volume vec-data:/data`. New `--data-dir <path>` CLI flag + a Linux startup warning (`WARN data dir at <path> is ephemeral; recommend mounting a volume`) when the resolved dir has no backing mount. Migration runbook lives in [`docs/users/configuration/DATA_DIRECTORY.md`](https://github.com/hivellm/vectorizer/blob/main/docs/users/configuration/DATA_DIRECTORY.md).
- **Added — `embedding_provider` / `model` honoured contracts (CONTRACT CHANGE)** ([#306](https://github.com/hivellm/vectorizer/issues/306)). 3.3.0 silently coerced every collection to BM25-512 regardless of the requested provider. 3.4.0:
  - `POST /collections` honours `embedding_provider`. Unknown provider → `400 unsupported_provider { requested, available }`. Caller-requested `dimension` that conflicts with the provider's native dimension → `400 provider_dimension_mismatch`.
  - `POST /embed` honours `model`. Unknown model → `400 unsupported_model { requested, available }`. Response echoes the resolved `model`.
  - `GET /stats` carries `providers[]` and `default_provider` so callers can discover the registered embedding surface (mirrored as the `list_providers` MCP tool).
  - `CollectionConfig.embedding_provider: String` persists which provider the collection was created with. Legacy `.vecdb` files default to `"bm25"` via serde so reload stays lossless.
  - Bootstrap registers every available provider (configured default + always-on `bm25` sparse fallback) — `POST /collections {embedding_provider: "bm25"}` works on every build.
  - Optional FastEmbed Docker variant: `docker build --build-arg ENABLE_FASTEMBED=1 --build-arg NO_DEFAULT_FEATURES=0 --build-arg FEATURES=fastembed .` ships a 205 MB image with FastEmbed `all-MiniLM-L6-v2` (384-dim dense) registered alongside `bm25`. Default published image stays slim (BM25-only, ~91 MB).
- **Security — bumped vulnerable transitive deps**: `axios` 1.x → 1.17.0 (gui — closes 6 dependabot alerts incl. proxy-auth redirect leak, prototype-pollution MITM); `tmp` <0.2.6 → 0.2.7 (path traversal); `react-router` 7.14.2 → 7.17.0 (DoS via unbounded path expansion in `__manifest`); `tar` 0.4.45 → 0.4.46 (PAX header desync).
- **Build** — Docker builder base moved from `lukemathwalker/cargo-chef:rust-1.90-bookworm` to `rust:1.95-slim-trixie`. glibc 2.40 matches the runtime DHI trixie base, clearing the `__isoc23_strtol` / `__isoc23_strtoull` link errors that surfaced when fastembed pulled in ORT prebuilt binaries linked against glibc 2.38+ symbols. `libstdc++.so.6` is now copied from the builder so the fastembed variant boots cleanly.

## 🚀 Quick Start

### Basic Usage

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -p 15503:15503 \
  --restart unless-stopped \
  hivehub/vectorizer:3.4.0
```

First boot creates an admin user and writes credentials to `/data/.root_credentials` inside the container (read with `docker exec` + `cat` or `docker cp` — the image is distroless so there's no shell). Rotate via the dashboard or `/auth` API as soon as you've copied them.

### With Persistent Data (v3.4.0+ canonical layout)

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -p 15503:15503 \
  -v vec-data:/data \
  --restart unless-stopped \
  hivehub/vectorizer:3.4.0
```

The image defaults `VECTORIZER_DATA_DIR=/data`, so a single `-v vec-data:/data` mount captures the entire persistent state (`.vecdb` store, auth keys, JWT secret, snapshots, fastembed cache when enabled). `docker compose up -d --force-recreate vectorizer` is now safe — collections survive the recreate.

> **Upgrading from 3.3.0?** 3.3.0 wrote state to `/.local/share/vectorizer/` (the XDG path) on the container's writable layer despite the README advertising `/data`. If your existing setup mounted both `/data` and `/.local/share/vectorizer` as a workaround, copy state from the workaround volume into `/data` and drop the second mount. Full runbook: [`docs/users/configuration/DATA_DIRECTORY.md`](https://github.com/hivellm/vectorizer/blob/main/docs/users/configuration/DATA_DIRECTORY.md) (issue [#300](https://github.com/hivellm/vectorizer/issues/300)).

### Docker Compose

```yaml
services:
  vectorizer:
    image: hivehub/vectorizer:3.4.0
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
      # v3.4.0+ canonical mount — VECTORIZER_DATA_DIR defaults to /data.
      - vec-data:/data
    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - VECTORIZER_AUTH_ENABLED=true
      - VECTORIZER_ADMIN_USERNAME=admin
      - VECTORIZER_ADMIN_PASSWORD=change-me-in-production
      - VECTORIZER_JWT_SECRET=change-this-to-a-random-32-char-secret
    restart: unless-stopped

volumes:
  vec-data:
```

> ✅ **Healthcheck note.** Since v3.0.1 the image ships a static `busybox` at `/busybox` and a built-in `HEALTHCHECK` (`/busybox wget -q --spider http://127.0.0.1:15002/health`). `docker compose ps` reports `(healthy)` once the server is up — no overrides needed. If you customize the healthcheck on Compose / Kubernetes, point it at the same `/busybox wget` command or use a TCP probe; `curl` and `sh` are still absent from the runtime image.

## ✨ Features (v3.4.0)

- **💾 Single-volume container persistence** — the image defaults `VECTORIZER_DATA_DIR=/data` so a single `-v vec-data:/data` mount captures collections, auth keys, JWT secret, snapshots, and the fastembed cache. Linux startup emits `WARN data dir at <path> is ephemeral; recommend mounting a volume` when the resolved dir has no backing mount, so a forgotten `--volume` flag fails loud instead of silently wiping collections on `--force-recreate`. New `--data-dir <path>` CLI flag for non-`/data` deployments.
- **🧠 Honoured `embedding_provider` / `model` contract** — `POST /collections` and `POST /embed` reject unknown providers with `400 unsupported_provider { requested, available }` / `400 unsupported_model { requested, available }` instead of silently coercing to BM25. `GET /stats` lists every registered provider with dimension + default flag; mirrored as the `list_providers` MCP tool. `CollectionConfig.embedding_provider` persists which provider the collection was created with.
- **🛡️ Backpressure-aware ingest** — bounded-resource bulk-upsert: per-collection admission with `429 Too Many Requests` + `Retry-After` on overload, BM25 vocab-build semaphore, structured `queue_full` MCP error, gRPC `RESOURCE_EXHAUSTED`. Configured via `backpressure.{max_concurrent_vocab_builds,upsert_queue_high_water,upsert_queue_hard_limit}` in `config.yml`. All five first-party SDKs honor `Retry-After` (1 s default, 30 s cap, 3 retries).
- **🆔 Stable client-id upserts** — `POST /insert_texts` and `POST /insert` use the request `id` verbatim as `Vector.id` (or `<id>#<chunkIndex>` for chunked entries). Re-running the same payload upserts in place. Bulk `POST /insert_vectors` ingests pre-computed embeddings without going through the embedding pipeline.
- **⚡ VectorizerRPC** — length-prefixed MessagePack over raw TCP on port `15503`, ~10× lower per-frame overhead than REST/JSON. Default binary transport across every SDK (Rust, TypeScript, Go, Python, C#).
- **🔍 Semantic Search** — Cosine / Euclidean / Dot Product, HNSW indexing, sub-3 ms typical search, hybrid dense + sparse (BM25) with rank fusion.
- **⚡ SIMD Acceleration** — AVX2 on x86_64, NEON on aarch64, scalar fallback. CPU-feature detection at boot.
- **🧠 Embeddings** — BM25 (default, 512-dim), TF-IDF, and **FastEmbed ONNX** models: `all-MiniLM-L6-v2` (384-dim), `bge-small-en-v1.5` (384), `bge-base-en-v1.5` (768), `bge-large-en-v1.5` (1024), plus `-q` int8-quantized variants (selected via `embedding.model: fastembed:<id>` in `config.yml`). Default published image is BM25-only (~91 MB); operators wanting fastembed out of the box build with `--build-arg ENABLE_FASTEMBED=1 --build-arg NO_DEFAULT_FEATURES=0 --build-arg FEATURES=fastembed` (~205 MB, model pre-fetched).
- **💾 Compact Storage** — unified `.vecdb` format with 20–30% space savings, MMap support for datasets larger than RAM, automatic snapshots.
- **📦 Quantization** — Scalar + Product Quantization (PQ) for up to 64× memory reduction with minimal accuracy loss.
- **🔄 Replication & Sharding** — master → replica TCP streaming (BETA), openraft-backed consensus for HA clusters.
- **📄 Document Conversion** — built-in pipelines for PDF, DOCX, XLSX, PPTX, HTML, XML, images.
- **🔄 Qdrant Compatibility** — drop-in `/qdrant/collections/{name}/points/*` surface for migrations.
- **🎯 MCP Integration** — focused tool-per-action MCP 2025-03-26 server on `POST /mcp` (streamable HTTP).
- **🕸️ Graph Relationships** — relationship discovery + traversal, GUI-backed.
- **🔒 Auth enforcement** — JWT + API Key with RBAC gating **every** data route when `auth.enabled: true`.
- **📊 Observability** — Prometheus metrics at `/prometheus/metrics` (now including `upsert_queue_depth`, `upsert_in_flight`, `vocab_build_permits_available`, `upsert_rejected_total{reason}`, `bm25_empty_vocab_fallback_total{collection}`), OpenTelemetry OTLP export, structured tracing via `RUST_LOG`. Operator runbook at [`docs/deployment/backpressure.md`](https://github.com/hivellm/vectorizer/blob/main/docs/deployment/backpressure.md) and importable Grafana panels at [`docs/grafana/backpressure-panels.json`](https://github.com/hivellm/vectorizer/blob/main/docs/grafana/backpressure-panels.json).

## 📦 Tags

| Tag | Points to | Notes |
|---|---|---|
| `3.4.0` | v3.4.0 release | **Current stable.** `/data` is the canonical volume mount (#300); `embedding_provider` / `model` honoured contracts (#306); `list_providers` MCP tool; optional FastEmbed Docker variant via `ENABLE_FASTEMBED=1` build arg. |
| `3.3.0` | v3.3.0 release | Hardened dashboard cookies + CSRF; API key usage metrics + permission update; cluster + auth admin endpoints. ⚠️ has the persistence trap fixed in 3.4.0 — upgrade or follow the migration runbook. |
| `3.2.0` | v3.2.0 release | Bulk-upsert backpressure, `Retry-After` SDKs, new Prometheus metrics. |
| `3.1.0` | v3.1.0 release | `/insert_vectors`, stable client-id upserts, flat chunked-payload layout. |
| `3.0.2` | v3.0.2 release | Docker Hardened Image base (`dhi.io/debian-base:trixie`); ~88 MB compressed; Scout-compliant. |
| `3.0.1` | v3.0.1 release | Built-in `HEALTHCHECK` via static busybox; SDK CI / build fixes. |
| `3.0.0` | v3.0.0 release | First v3 cut: workspace-split crates, RPC-default, Edition 2024. |
| `latest` | same as `3.4.0` | Updated on every stable tag. Pin to a specific version in production. |

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
| `http://localhost:15002/health` | Health check (anonymous, returns `{"status":"healthy","version":"3.4.0",...}`) |
| `http://localhost:15002/prometheus/metrics` | Prometheus scrape target |

## 🛠️ Configuration

### Environment Variables

| Variable | Default | Purpose |
|---|---|---|
| `VECTORIZER_HOST` | `0.0.0.0` | Bind address. |
| `VECTORIZER_PORT` | `15002` | REST + MCP + dashboard port. |
| `VECTORIZER_DATA_DIR` | `/data` | v3.4.0+ — persistent state dir (collections, auth keys, JWT secret, snapshots, fastembed cache). Set to a non-`/data` path if your mount targets a different location; the resolver propagates to every persistence subsystem. |
| `VECTORIZER_AUTH_ENABLED` | *(unset)* | Set to `true` to gate data routes behind JWT/API-key. |
| `VECTORIZER_ADMIN_USERNAME` | `admin` | Admin username seeded on first boot. |
| `VECTORIZER_ADMIN_PASSWORD` | *(prompted on boot)* | Admin password. Set this in production or the server writes a generated one to the `.root_credentials` file. |
| `VECTORIZER_JWT_SECRET` | *(generated)* | Minimum 32 chars for production; share across HA-cluster nodes so JWTs are portable. |
| `CORTEX_VECTORIZER_BACKPRESSURE_ENABLED` | `true` | v3.2.0+ — global enable for the bulk-upsert backpressure layer. |
| `CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS` | `num_cpus` | v3.2.0+ — semaphore for the BM25 vocab-build hot path. |
| `CORTEX_VECTORIZER_UPSERT_HIGH_WATER` | `256` | v3.2.0+ — per-collection in-flight-depth at which the server emits a structured warn + bumps `upsert_rejected_total{reason="queue_high_water_warn"}` (still admits the request). |
| `CORTEX_VECTORIZER_UPSERT_HARD_LIMIT` | `1024` | v3.2.0+ — per-collection in-flight-depth at which new upserts are refused with `429 Too Many Requests` + `Retry-After`. |
| `RUST_LOG` | `info` | Per-module tracing filter, e.g. `vectorizer=debug,hyper=info`. |
| `TZ` | `Etc/UTC` | Container timezone. |
| `RUN_MODE` | `production` | `production` or `development`. |

### Config Files

- Mount `config.yml` to `/vectorizer/config.yml` to override defaults (embedding model, quantization, HNSW params, auth mode, replication topology). Set `embedding.model: fastembed:all-MiniLM-L6-v2` (or any other fastembed id) to register a dense provider as the default — note that the default published image is BM25-only; build with `--build-arg ENABLE_FASTEMBED=1 --build-arg NO_DEFAULT_FEATURES=0 --build-arg FEATURES=fastembed` to get fastembed compiled in.
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

- **Base Image**: `dhi.io/debian-base:trixie` (Docker Hardened Image — Docker-signed, Scout-native, weekly rebuilds, CIS-compliant). v3.0.0 shipped on `gcr.io/distroless/cc-debian12:nonroot`; v3.0.2 swapped to DHI to make Scout's "Approved Base Images" + "Up-to-Date Base Images" policies flip to `Compliant`.
- **Default User**: nonroot (UID 65532). Every `COPY` in the runtime stage is `--chown=65532:65532` so the binary writes `config.yml` / `workspace.yml` on first boot without `--user root`.
- **Architectures**: `linux/amd64`, `linux/arm64` (multi-arch manifest)
- **Compressed Size**: ~88 MB (v3.0.2+ on DHI; +21 MB vs the original distroless build because debian-base ships `bash` + full `libssl`/`libcrypto`/`libsystemd`/`libreadline`).
- **Healthcheck**: built-in `HEALTHCHECK ... CMD ["/busybox", "wget", "-q", "--spider", "http://127.0.0.1:15002/health"]` (a static `busybox:stable-musl` is COPY-ed into `/busybox` and used **only** as the healthcheck entrypoint).
- **Rust Edition**: 2024 (mandatory, pinned rustc ≥ 1.90 per async-graphql / asynk-strim floor)
- **Build Flags**: `--package vectorizer-server --bin vectorizer --no-default-features` (excludes ONNX / FastEmbed / GPU / Transmutation from the default image to keep the dependency surface small). The container binary is compiled with the dedicated `release-docker` Cargo profile (`lto = false`, `codegen-units = 16`, inherits `release` otherwise) — same opt-level=3 + strip behavior as the host `cargo build --release`, but ~30% faster to compile inside BuildKit at the cost of ~10–15% lower throughput on hot paths versus a host-built `release` binary. Operators chasing peak per-op throughput should rebuild from source with the workspace `release` profile.
- **Supply Chain**: per-arch SBOM and SLSA `mode=max` provenance attached as in-toto attestations to the multi-arch manifest list (Docker Scout reads from there). Inspect with `docker buildx imagetools inspect hivehub/vectorizer:<tag>`. OpenContainer labels carry revision, source, and license metadata.
- **License**: Apache-2.0

## 🔧 Advanced Usage

### Monorepo Indexing

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 -p 15503:15503 \
  -v vec-data:/data \
  -v $(pwd)/workspace.yml:/vectorizer/workspace.yml \
  -v $(pwd):/workspace:ro \
  --restart unless-stopped \
  hivehub/vectorizer:3.4.0
```

### Fastembed Variant (semantic search out of the box)

```bash
docker build \
  --build-arg ENABLE_FASTEMBED=1 \
  --build-arg NO_DEFAULT_FEATURES=0 \
  --build-arg FEATURES=fastembed \
  -t my/vectorizer:3.4.0-fastembed .

docker run -d \
  --name vectorizer \
  -p 15002:15002 -p 15503:15503 \
  -v vec-data:/data \
  -v $(pwd)/fastembed-config.yml:/vectorizer/config.yml:ro \
  my/vectorizer:3.4.0-fastembed
```

Where `fastembed-config.yml` carries:

```yaml
embedding:
  model: fastembed:all-MiniLM-L6-v2

auth:
  enabled: true
```

`GET /stats.providers` now lists both `fastembed:all-MiniLM-L6-v2` (384-dim, default) and `bm25` (512-dim, sparse fallback). Hybrid retrieval works out of the box.

### Debug Logging

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -e RUST_LOG="vectorizer=debug,vectorizer::replication=trace,hyper=info" \
  -e RUST_BACKTRACE=1 \
  hivehub/vectorizer:3.4.0
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
docker pull hivehub/vectorizer:3.4.0
docker pull hivehub/vectorizer:latest
# Or pin earlier v3.x stable points:
docker pull hivehub/vectorizer:3.3.0
docker pull hivehub/vectorizer:3.2.0
docker pull hivehub/vectorizer:3.1.0
docker pull hivehub/vectorizer:3.0.2
```

**🔗 Repository**: [https://hub.docker.com/r/hivehub/vectorizer](https://hub.docker.com/r/hivehub/vectorizer)
