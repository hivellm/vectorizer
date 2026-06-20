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
- **Built-in providers** — TF-IDF, BM25, FastEmbed, BERT, MiniLM, custom models. **`embedding_provider` (on `POST /collections`) and `model` (on `POST /embed`) are honoured contracts as of v3.4.0** ([issue #306](https://github.com/hivellm/vectorizer/issues/306)) — unknown providers / models return `400 unsupported_provider` / `400 unsupported_model` with the available list; no more silent BM25-512 coercion. Discover registered providers via `GET /stats.providers` or the `list_providers` MCP tool. See [`docs/users/guides/EMBEDDINGS.md`](docs/users/guides/EMBEDDINGS.md#contract-post-collections-and-post-embed).
- **Document conversion** — PDF, DOCX, XLSX, PPTX, HTML, XML, images (14 formats).
- **Qdrant API compatibility** — Snapshots, Sharding, Cluster Management, Query (with prefetch), Search Groups, Matrix, Named Vectors (partial), PQ/Binary quantization config.
- **Summarization** — extractive, keyword, sentence, abstractive (OpenAI GPT).

### Security
- **JWT + API Key** authentication with RBAC, **scoped API keys** (per-collection permissions), **atomic key rotation** with grace window, **RFC 7662 token introspection**, **admin audit log** (in-memory ring + daily-rotated JSONL).
- **API key usage metrics** — per-key `usage_count` (atomic, lock-free) plus a 30-day per-day ring buffer surfaced via `GET /auth/keys/{id}/usage`.
- **Permission update without rotation** — `PUT /auth/keys/{id}/permissions` swaps `permissions`/`scopes` while keeping `key_hash`/`id`/`user_id`/`created_at` immutable.
- **Hardened dashboard cookies + CSRF** — login/refresh emit `vectorizer_session` (`HttpOnly; Secure; SameSite=Strict`) plus a sibling `XSRF-TOKEN` cookie; `require_csrf_middleware` guards every mutating `/auth/*` and `/admin/*` request. `auth.cookies.insecure_dev` opt-out for plain-HTTP loopback dev (boot rejects it on `0.0.0.0`).
- **Loopback dev-mode auth bypass** — `auth.dev_mode_skip_loopback` short-circuits credential validation as `local-dev-admin` and stamps `X-Vectorizer-Dev-Mode: true` on every response. Boot fails on any non-loopback host.
- **JWT secret is mandatory** — boot refuses to start with empty / default / <32 char secrets when auth is enabled.
- **First-run root credentials** written to `{data_dir}/.root_credentials` (0o600), never logged.
- **Payload encryption** — optional ECC-P256 + AES-256-GCM, zero-knowledge, per-collection policies ([docs](docs/features/encryption/README.md)).
- **TLS 1.2/1.3** with mTLS, configurable cipher suites, ALPN.
- **Per-API-key rate limiting** with tiers + overrides.
- **Path-traversal guard** on file discovery; canonicalized base, symlink-escape refusal.

### UI
- **Web Dashboard** — React + TypeScript; JWT login, graph CRUD (edges, neighbors, paths), collection management, API sandbox, setup wizard with glassmorphism design. Embedded in the binary (~26MB, no external assets needed).
- **Desktop GUI** — Electron + vis-network for visual database management.

## 🎉 Latest Release: v3.4.0

Highlights — see [CHANGELOG.md](./CHANGELOG.md) for the full breakdown.

**Fixed — Container deployments lose all collections on restart (phase32, [#300](https://github.com/hivellm/vectorizer/issues/300))**
- 3.3.0 image wrote persistent state to `/.local/share/vectorizer/` even though the README advertised `/data` as the volume mount. The XDG path lived on the container's writable layer, so `docker compose up -d --force-recreate vectorizer` silently wiped every collection.
- Image defaults `VECTORIZER_DATA_DIR=/data` and seeds the directory in the `writable-dirs` stage with the nonroot user as owner. A single `--volume vec-data:/data` mount now captures collections, auth keys, JWT secret, and snapshots.
- `vectorizer --data-dir <path>` is a first-class CLI flag; resolves through `vectorizer_core::paths::data_dir` so every persistence subpath (auth, vector store, snapshots, fastembed cache) picks up the override.
- Startup emits `WARN data dir at <path> is ephemeral; recommend mounting a volume` when the resolved data dir has no backing mount (Linux only, via `/proc/self/mountinfo`). Surfaces the trap on the first boot without a volume.
- Migration runbook in `docs/users/configuration/DATA_DIRECTORY.md` for operators who mounted the workaround `/.local/share/vectorizer` second volume.

**Added — Honour `embedding_provider` / `model` in REST contracts (phase33, [#306](https://github.com/hivellm/vectorizer/issues/306)) — _contract change_**
- 3.3.0 silently coerced every `embedding_provider` to BM25-512. Downstream consumers (e.g. hivellm/cortex) saw their hybrid pipeline degrade to keyword-only because the vector lane was returning lexical BM25 vectors regardless of what they posted.
- `POST /collections` honours `embedding_provider`. Unknown provider → `400 unsupported_provider { requested, available }`. Caller-requested `dimension` that conflicts with the provider's native dimension → `400 provider_dimension_mismatch`.
- `POST /embed` honours `model`. Unknown model → `400 unsupported_model { requested, available }`. Response echoes the resolved `model` so callers can confirm which provider produced the vector.
- `GET /stats` lists `providers[]` + `default_provider` so clients can discover the registered embedding surface without trial-and-error. Mirrored as the `list_providers` MCP tool.
- `CollectionConfig.embedding_provider: String` persists which provider the collection was created with. Legacy `.vecdb` files default to `"bm25"` via serde so reload stays lossless.
- Bootstrap registers every available provider at boot — without this, `POST /collections {embedding_provider: "bm25"}` would have returned 400 on any fastembed-default deployment.
- Optional FastEmbed Docker variant: `docker build --build-arg ENABLE_FASTEMBED=1 --build-arg NO_DEFAULT_FEATURES=0 --build-arg FEATURES=fastembed .` ships an image with `all-MiniLM-L6-v2` (384-dim dense) registered alongside `bm25`. Default published image stays slim (BM25-only).

**Fixed — Bumped vulnerable transitive deps**
- `axios` `<1.16.0` → `1.17.0` (gui pnpm override): closes 6 dependabot alerts (proxy-auth leak via redirects, prototype-pollution MITM via `config.proxy`, `shouldBypassProxy` IPv4-mapped IPv6 NO_PROXY bypass, header injection via merge gadgets, null-prototype patch bypass).
- `tmp` `<0.2.6` → `0.2.7` (gui pnpm override): closes path-traversal via unsanitized prefix/postfix.
- `react-router` `7.14.2` → `7.17.0` (dashboard top-level + override): closes DoS via unbounded path expansion in `__manifest`.
- `tar` `0.4.45` → `0.4.46` (root Cargo.lock): closes PAX header desynchronization.

**Build**
- Docker builder base bumped from `lukemathwalker/cargo-chef:rust-1.90-bookworm` to `rust:1.95-slim-trixie`. glibc 2.40 matches the runtime `dhi.io/debian-base:trixie`, clearing the `__isoc23_strtol`/`__isoc23_strtoull` link errors that surfaced when fastembed pulled in ORT prebuilt binaries linked against glibc 2.38+ symbols.
- `libstdc++.so.6` copied from the builder into the runtime stage so the fastembed Docker variant boots without `error while loading shared libraries: libstdc++.so.6`.

Server-side at **v3.4.0**. The Rust SDK tracks server versioning; TypeScript, Python, Go, and C# SDKs are also on v3.4.0 (no breaking server-contract changes beyond the embedding-provider error shapes documented above).

---

### v3.3.0 highlights (previous release)

**Security — Hardened dashboard cookies + CSRF**
- `POST /auth/login` and `POST /auth/refresh` now set a hardened `vectorizer_session` cookie (`HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=<jwt_exp>`) carrying the JWT, plus a sibling `XSRF-TOKEN` cookie carrying a 32-byte random CSRF token (non-`HttpOnly` so the SPA can echo it).
- New `require_csrf_middleware` rejects `POST/PUT/PATCH/DELETE` requests under `/auth/*` and `/admin/*` with HTTP 403 unless the `X-CSRF-Token` header matches the token bound to the caller's session JWT. `GET/HEAD/OPTIONS`, `/auth/login`, `/auth/validate-password`, and `X-API-Key` requests bypass the gate.
- `POST /auth/logout` emits expired `Set-Cookie` headers for both cookies and drops the CSRF binding.
- New `auth.cookies.insecure_dev` config flag (default `false`) drops only `Secure` for plain-HTTP `127.0.0.1` development. Boot rejects the flag on any non-loopback host.
- `dashboard/src/lib/api-middleware.ts` adds a `csrfMiddleware` that reads the `XSRF-TOKEN` cookie and echoes it on every mutating request. Legacy `access_token` JSON body preserved for SDK / `Authorization: Bearer` callers.

**Security — Loopback dev-mode auth bypass**
- New `auth.dev_mode_skip_loopback` config flag (default `false`) lets local devs run the dashboard / SDK against `127.0.0.1` without minting a JWT or echoing tokens on every cURL.
- When on: middleware short-circuits with a synthetic `local-dev-admin` principal (`Role::Admin`) and every response carries `X-Vectorizer-Dev-Mode: true`. CSRF middleware no-ops in the same mode.
- Boot fails fast when the flag is on and the bind host is anything other than `127.0.0.1`, `::1`, or `localhost`. Multi-line `WARN` banner emitted at boot when engaged on a loopback bind.
- See `docs/users/api/AUTHENTICATION.md` → "Local Development".

**Added — API key usage metrics + permission update**
- Per-key `usage_count` (atomic, lock-free on the validation hot path) plus a 30-day per-key per-day ring buffer. Every successful `validate_api_key` bumps both. Counter persists across restarts; old payloads deserialize with `usage_count: 0`.
- New `PUT /auth/keys/{id}/permissions` (admin-gated) replaces a key's permissions/scopes without rotating the credential. `key_hash`/`id`/`user_id`/`created_at` stay immutable.
- New `GET /auth/keys/{id}/usage?window=<n>` (admin-gated) returns the per-day counter ring (default 7, max 30) plus live key view + window total. Zero-count days included so consumers render gap-free sparklines.
- `GET /auth/keys` list response now carries `usage_count` + `usage_24h` per row — dashboard renders `Last 24h` + `Total` columns without an N+1 fetch.
- SDK parity (Rust / TypeScript / Python): `update_api_key_permissions`, `get_api_key_usage`, plus `ApiKeyView`, `ApiKeyUsageReport`, `ApiKeyUsageBucket`, `ApiKeyScope`, `UpdateApiKeyPermissionsRequest`. `ApiKey` gains `usage_count` (additive).
- Dashboard `ApiKeysPage.tsx` adds the new columns and a `Usage` button that opens a modal with a 14-day SVG sparkline (new dependency-free `Sparkline.tsx`) + per-day bucket table.

**Added — Cluster admin endpoints**
- `POST /cluster/failover` — promote a replica with a pre-flight WAL-lag check (HTTP 409 when lag exceeds `max_lag_segments`).
- `POST /cluster/replicas/{id}/resync` — force a full snapshot + WAL replay on a lagging replica.
- `POST /cluster/peers` — add a member or observer peer.
- `POST /cluster/rebalance` + `GET /cluster/rebalance/status` — async shard rebalance using insert-before-delete invariant; poll the job by id.

**Added — Auth / RBAC admin endpoints**
- `POST /auth/keys/{id}/rotate` — atomic key rotation with a 300 s grace window. Returns `{old_key_id, new_key_id, new_token, grace_until}`.
- `POST /auth/keys` (extended) — optional `scopes: [{collection, permissions}]` for collection-scoped keys; empty list = default-deny on scope-aware routes. Existing global-key callers unaffected.
- `POST /auth/introspect` — RFC 7662 token introspection for any JWT or API key.
- `GET /auth/audit` — admin-only audit log query (in-memory ring + daily-rotated JSONL files), filterable by `from`, `to`, `actor`, `action`.
- New `AuditLogger` ships record events through an unbounded `mpsc` channel — never blocks the handler hot-path.

**Added — Tier-demotion API ([#265](https://github.com/hivellm/vectorizer/issues/265))**
- `POST /collections/{src}/vectors/move` — cross-collection move with insert-before-delete invariant; per-id status (`ok | missing_in_src | dst_insert_failed | src_delete_failed`) so a mid-batch crash leaves a recoverable duplicate, never data loss. See [`docs/users/api/API_REFERENCE.md`](docs/users/api/API_REFERENCE.md).
- All three SDKs (Rust / TypeScript / Python): `delete_vector`, `delete_vectors` (batch with per-id `DeleteReport`), `move_to_collection` (cross-collection with `MoveReport`). TypeScript / Python `delete_vectors` now return the canonical `DeleteReport` shape — callers asserting on the old `{deleted: number}` / `bool` shapes need to read `report.deleted` / `report.results`.

**Build — Docker pipeline overhaul**
- Cuts cold local multi-arch build from ~30–45 min to ~15–20 min, warm to under 10 min via four changes:
  1. **Buildx registry cache** on `hivehub/vectorizer-cache:buildx` (opt out with `-NoCache`).
  2. **Dedicated `release-docker` Cargo profile** (`lto = false`, `codegen-units = 16`, `incremental = false`) — ~30 % faster compile, ~50 % lower peak rustc memory; ~10–15 % lower throughput on hot paths vs the host `release` profile, which is unchanged.
  3. **Drop in-image `cargo sbom`** — SBOM now exclusively from BuildKit's `--sbom=true` syft attestation.
  4. **Native arm64 in CI + Docker Hub publish** — per-arch matrix, manifest-list stitched and pushed to both `ghcr.io/hivellm/vectorizer` and `hivehub/vectorizer`. Eliminates QEMU emulation from the arm64 build.
- Operator runbook: [`docs/development/docker-builds.md`](docs/development/docker-builds.md).

For prior releases see [CHANGELOG.md](./CHANGELOG.md).

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
  -v vec-data:/data \
  -e VECTORIZER_AUTH_ENABLED=true \
  -e VECTORIZER_ADMIN_USERNAME=admin \
  -e VECTORIZER_ADMIN_PASSWORD=your-secure-password \
  -e VECTORIZER_JWT_SECRET=$(openssl rand -hex 64) \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

Starting in `hivehub/vectorizer:3.4.0` the image defaults
`VECTORIZER_DATA_DIR=/data`, so a **single `--volume vec-data:/data`
mount** captures every collection, auth key, JWT secret, and
snapshot. `docker compose up -d --force-recreate vectorizer` is now
safe; see `docs/users/configuration/DATA_DIRECTORY.md` for the
3.3.0 migration runbook (issue [#300](https://github.com/hivellm/vectorizer/issues/300)).

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

> **Keeping `target/` bounded.** Cargo never GCs `target/` — stale rlibs accumulate until you nuke them. This repo already pins `[profile.dev] debug = "line-tables-only"` + `[profile.release] strip = true` + `CARGO_INCREMENTAL=0` on CI; on a developer box, run `bash scripts/sweep-target.sh` (or `pwsh scripts/sweep-target.ps1` on Windows) weekly to drop stale artifacts without losing the incremental hot set. Full runbook + scheduler examples in [`docs/development/rust-target-hygiene.md`](docs/development/rust-target-hygiene.md) (issue [#320](https://github.com/hivellm/vectorizer/issues/320)).

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
- **Complete SDK coverage** — Rust, Python, TypeScript (+JS), Go, C# — all on v3.4.0.

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

Server-side at **v3.4.0**. The Rust SDK tracks server versioning; the TypeScript, Python, Go, and C# SDKs are also on **v3.4.0**. The TypeScript SDK ships compiled CJS + ESM — usable from plain JavaScript, no separate JS package needed.

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
- [Bulk-upsert Backpressure Runbook](./docs/deployment/backpressure.md) — `429` / `Retry-After`, vocab-build cap, ops metrics ([#263](https://github.com/hivellm/vectorizer/issues/263))
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
