Each item is a runtime probe against a live server. Expected result
and acceptance criterion recorded inline so the pass/fail is
unambiguous. Server is assumed booted at `127.0.0.1:15002` unless the
probe requires a different topology.

## 1. Environment

- [x] 1.1 Clean data dir (`vectorizer_core::paths::data_dir()` under
  `%APPDATA%\vectorizer\data` on Windows, or `$XDG_DATA_HOME/vectorizer/data`
  on Linux, or `~/Library/Application Support/vectorizer/data` on macOS).
  Accept: `list_collections` returns `{collections:[],total:0}` at boot.
- [x] 1.2 `cargo build --release --workspace` clean. Accept: every
  `target/release/*.exe` present + links (re-verify on top of the last
  commit).
- [x] 1.3 `./target/release/vectorizer --host 127.0.0.1` boots without
  panics. Accept: `/health` → `{"status":"healthy","version":"3.0.0"}`.
- [x] 1.4 Log inspection: `grep -iE "(panic|error)" <logfile>` returns
  only the expected `auth.jwt_secret empty` warning + validation errors
  from intentional bad requests. No unexpected ERROR lines.

## 2. Dep-sensitive hot paths

- [x] 2.1 **Snapshot round-trip** (`hmac 0.13` + `sha2 0.11` + `lz4_flex 0.13`
  + `bincode 2` + `dirs 6`). Steps: create collection → insert 100 vectors
  → `curl -X POST /admin/snapshot` (or wait 5min for auto-save) → kill
  server → restart → `list_collections` → `search` same query. Accept:
  snapshot file appears under `data_dir()/snapshots/`, collection comes
  back after restart with same vector_count, search returns same results.
- [ ] 2.2 **Real embedding model** (`fastembed 5.13` + `ort rc.11` +
  `hf-hub 0.5`). Enable a real ONNX model in `config.yml` under
  `embedding.default_model` (e.g. `all-MiniLM-L6-v2`). Restart server.
  Accept: `POST /embed` returns a vector that is NOT the uniform `0.1`
  placeholder; model download logged from hf-hub; `ort::Session`
  initialisation logged.
- [ ] 2.3 **gRPC** (`tonic 0.14` + `grpc_conversions.rs`). Use `grpcurl`
  (or `tests/grpc` integration tests pointed at the live server) to call
  `list_collections`, `create_collection`, `upsert_points`, `search_points`
  over the Qdrant-compatible service on `:15100`. Accept: each RPC returns
  200 with the expected proto shape.
- [ ] 2.4 **UMICP** (`umicp-core 0.2`). `curl /umicp/discover` returns the
  operation manifest; call at least one operation via POST `/umicp`.
  Accept: both endpoints return 200 + structured JSON matching
  `docs/specs/UMICP.md`.
- [ ] 2.5 **VectorizerRPC** (`vectorizer-protocol` wire types) on
  `:15503`. Use the `sdks/rust` RPC client to issue one `ListCollections`
  + one `Search`. Accept: the binary length-prefixed msgpack frames
  decode cleanly + return correct results.
- [ ] 2.6 **Prometheus exposition** (`opentelemetry-prometheus 0.31`).
  Find the actual endpoint (not the JSON `/metrics`) via source grep
  of `PrometheusBuilder` / `with_registry`. Accept: GET that endpoint
  returns `# HELP ...` / `# TYPE ...` prometheus text format with at
  least `vectorizer_*` metrics.
- [ ] 2.7 **Query cache** (`lru 0.17`). Send the same `search` query
  twice in a row. Accept: second response has cache-hit indicator
  (header, metadata, or log line) and measurably lower latency than
  the first.

## 3. Broader v3 surface

- [ ] 3.1 **Auth flow**. Enable `auth.enabled: true` + set a
  `jwt_secret` in `config.yml`. Restart. POST `/auth/login` with
  admin credentials. Accept: returns JWT; subsequent authenticated
  REST calls (list_collections) work with `Authorization: Bearer
  <jwt>`; unauthenticated calls return 401.
- [ ] 3.2 **Batch REST**. `POST /batch_insert` with 50 texts,
  `/batch_search` with 5 queries, `/batch_update` renaming 3 vector
  IDs, `/batch_delete` with 10 IDs. Accept: all four return 200 with
  structured per-item results.
- [ ] 3.3 **File upload** via `POST /files/upload` with a small .md
  file. Accept: 200, file appears in `list_files` for the target
  collection + chunks retrievable via `get_file_chunks`.
- [ ] 3.4 **File watcher**. Drop a file into a watched directory (fix
  `workspace.yml` schema if needed so the watcher picks up paths).
  Wait 2s for debounce. Accept: watcher logs the event, file is
  indexed + searchable.
- [ ] 3.5 **GraphQL**. `POST /graphql` with `{ collections { name
  vectorCount } }`. Accept: returns the same list `list_collections`
  does. Visit `/graphiql` and confirm playground loads.
- [ ] 3.6 **Qdrant-compat**. Call at least `PUT /qdrant/collections/{name}`,
  `PUT /qdrant/collections/{name}/points`, `POST /qdrant/collections/{name}/points/search`.
  Accept: responses match the Qdrant REST spec shape (result, status,
  time).
- [ ] 3.7 **Dashboard UI**. GET `/dashboard/` in a browser (or `curl -I`
  for the HTML). Accept: 200 + HTML body renders React app shell
  referencing the bundled JS/CSS under `/dashboard/assets/`.
- [ ] 3.8 **Payload encryption**. Create a collection with `encryption:
  {enabled: true}` in the config. Insert text. Accept: `insert_text`
  response has `"encrypted": true`; `get_vector` returns the original
  payload (round-trip works); raw `.vecdb` on disk doesn't contain the
  plaintext.
- [ ] 3.9 **Cross-encoder reranking**. `search_semantic` with
  `cross_encoder: true` (or whatever the config knob is). Accept: the
  response's `tool_metadata.cross_encoder_reranking` flips to `true`
  and score ordering differs from the non-reranked baseline.
- [ ] 3.10 **Tantivy 0.26 index**. Exercise the BM25 path — find the
  entry point that actually builds a tantivy index (grep
  `IndexWriter` / `Schema::builder()` outside the test modules).
  Accept: index builds + search returns results with BM25 scores.
- [ ] 3.11 **Arrow/Parquet 58**. Build with `--features arrow,parquet`.
  Trigger the Parquet export path (check
  `crates/vectorizer/src/embedding/cache.rs` for an endpoint or CLI
  command). Accept: produces a valid `.parquet` file readable by
  `pyarrow.parquet.read_table()`.
- [ ] 3.12 **Zip 8** snapshot export / import. Find the CLI command or
  API that wraps snapshots in zip. Accept: produced `.zip` opens + a
  fresh server can import it back and recover collections.
- [ ] 3.13 **Cluster / Raft** — 2-node local setup (port 15002 + 15003).
  If the config boilerplate is too large for release gating, tag this
  item as "smoke-only" and just verify single-node server still boots
  with `cluster.enabled: false`. Accept either result, but document
  which path was exercised.

## 4. SDK integration against the live server

- [ ] 4.1 **TypeScript SDK** — set `VECTORIZER_BASE_URL=http://localhost:15002`
  and run `pnpm test` under `sdks/typescript/` in a mode that flips the
  live-server tests on (remove the env-guard marker that currently
  excludes them). Accept: the 46 previously-excluded integration tests
  now run and pass.
- [ ] 4.2 **Python SDK** — `cd sdks/python && python -m pytest
  --ignore=tests/test_file_upload.py --ignore=tests/test_routing.py`
  against the running server. Accept: the 46 currently-failing
  server-integration tests pass.
- [ ] 4.3 **Go SDK** — `cd sdks/go && go test ./... -run Integration`
  against the running server. Accept: the integration-tagged tests
  pass.
- [ ] 4.4 **Rust SDK** — `cd sdks/rust && cargo test --features rpc --
  --ignored` (or whatever gate the live-server tests use). Accept:
  live-server tests pass against both HTTP and RPC transports.
- [ ] 4.5 **C# SDK** — first resolve the pre-existing
  `Vectorizer.Tests/FileUploadTests.cs` CS1503 compile errors; then
  `dotnet test` against the running server. Accept: tests compile +
  pass.

## 5. Stress / performance sanity

- [ ] 5.1 Insert 10k synthetic vectors (dim=512, random payload) via
  `/batch_insert` in chunks of 500. Accept: all 10k inserted without
  errors, vector_count = 10k, HNSW graph built (collection metadata
  shows non-zero graph size).
- [ ] 5.2 Search latency: 1000 random-query `search` calls, measure
  p50 + p95. Accept: p50 ≤ 5ms, p95 ≤ 15ms on this hardware
  (matches or beats README claims for a 10k collection).
- [ ] 5.3 Confirm SIMD path — build with default features, inspect
  logs for `SIMD enabled: avx2` (or neon / wasm depending on platform)
  on VectorStore init. Accept: a SIMD provider is loaded; search
  latency above is consistent with SIMD-enabled paths.

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
  (release notes in CHANGELOG.md under `3.0.0`; a new
  `docs/releases/v3.0.0-verification.md` capturing each probe's result
  + evidence; any regression found gets its own follow-up task
  linked from this one).
- [ ] 6.2 Write tests covering the new behavior (every probe whose
  path wasn't already covered by the existing unit/integration suite
  earns a new regression test so the next release doesn't have to
  rediscover the probe).
- [ ] 6.3 Run tests and confirm they pass (`cargo test --workspace
  --lib --all-features`, plus each SDK's own test command with the
  live-server gate flipped on).
