# Proposal: phase8_release-v3-runtime-verification

## Why

The v3.0.0 release bundles a large number of structurally-invasive
changes landed across phases 4–7:

- workspace split into 5 crates + `sdks/rust` member
- OS-canonical data / log directory resolution via `vectorizer-core::paths`
- rmcp 0.10 → 1.5 (MCP protocol major rewrite)
- reqwest 0.12 → 0.13 (TLS feature rename + `blocking` moved out of default)
- hmac 0.12 → 0.13 + sha2 0.10 → 0.11 (output type changed; every hex-render
  call migrated from `format!("{:x}", ...)` to `hex::encode(...)`)
- arrow/parquet 57 → 58, zip 6 → 8, tantivy 0.25 → 0.26
- fastembed 5.4 → 5.13 + ort 2.0.0-rc.10 → rc.11
- hf-hub 0.4 → 0.5, sysinfo 0.37 → 0.38
- opentelemetry-prometheus 0.29 → 0.31 (upstream final before the crate
  was marked discontinued)
- tokio-tungstenite 0.28 → 0.29, lru 0.16 → 0.17, lz4_flex 0.12 → 0.13,
  dirs 5 → 6
- TypeScript SDK: eslint 8 → 9 flat-config + vitest 2 → 4 + @types/node
  24 → 25 + tsconfig `target` + `lib` ES2020 → ES2022
- Dashboard: React 18 → 19 family, react-router 6 → 7, typescript 5 → 6
  (`baseUrl` removed), jsdom 27 → 29, tailwind-merge 2 → 3
- GUI: typescript 5 → 6, vite 7 → 8, vue-router 4 → 5, uuid 13 → 14,
  electron 39 → 41 (manifest-only pending the
  `@hivehub/vectorizer-sdk@3.0.0` publish)

Unit + integration tests (1262 passing) and a first-pass smoke test
(server boot + REST CRUD + 31/31 MCP tools exercised) both passed. But
several runtime paths that the bumps materially touch have **not** been
exercised end-to-end yet. For a "major version" release we want every
code path the bumps influence to have at least one real live call before
we ship.

## What Changes

No code changes unless a probe surfaces a regression. This task is a
release-gate verification pass: run each probe, record the result, open
a follow-up task for anything that fails.

### Hot paths the dep bumps actually touch
1. **Snapshot round-trip** — exercises `hmac 0.13` (checksum HMAC) +
   `sha2 0.11` (hasher output type) + `lz4_flex 0.13` (compression) +
   `bincode 2` (serialization) + `dirs 6` (data_dir resolution).
2. **Real embedding model load** — exercises `fastembed 5.13` + `ort
   2.0.0-rc.11` (pinned by fastembed) + `hf-hub 0.5` (model download) +
   `candle 0.10` (tokenizer / model runtime).
3. **gRPC real client call** — exercises `tonic 0.14` + the
   `grpc_conversions` module that stayed in the umbrella specifically
   for the orphan rule.
4. **UMICP endpoint** — exercises `umicp-core 0.2` runtime.
5. **VectorizerRPC binary transport** on `:15503` — exercises
   `vectorizer-protocol` wire types + length-prefixed msgpack codec
   that `sdks/rust` also depends on.
6. **Prometheus exposition format** — exercises `opentelemetry-prometheus
   0.31`. Currently `/metrics` returns JSON file-watcher stats; the real
   prometheus endpoint may live elsewhere.
7. **Query cache** miss → hit via repeated search — exercises `lru 0.17`
   on the query-cache path.

### Broader release surface (not strictly migration-touching but worth
exercising at v3.0)
8. Auth flow: JWT login + token refresh + admin-user bootstrap +
   `/auth/*` endpoints.
9. REST batch endpoints: `/batch_insert`, `/batch_search`,
   `/batch_update`, `/batch_delete`.
10. File upload via `/files/upload` + watcher picking it up.
11. GraphQL `/graphql` query + `/graphiql` playground.
12. Qdrant-compat API in `/qdrant/*`.
13. Dashboard UI served at `/dashboard/`.
14. Payload encryption enabled at collection-create time.
15. Cross-encoder reranking flag on semantic search.
16. Real tantivy 0.26 index (the current code only touches tantivy from
    test-only modules).
17. Cluster / Raft paths with a 2-node local setup (if feasible).
18. Arrow/Parquet 58 via the optional feature.
19. Zip 8 snapshot export / import.

### SDK integration against the running server
20. Each SDK (`sdks/typescript`, `sdks/python`, `sdks/go`, `sdks/rust`,
    `sdks/csharp`) runs its integration tests pointed at the live
    `localhost:15002` server instead of the usual mocked baseURL.

### Mild stress
21. Insert 10k synthetic vectors to force HNSW graph growth, measure
    search p50/p95 latency, confirm SIMD-accelerated paths kick in.

## Impact

- Affected specs: none (verification only).
- Affected code: zero changes unless a probe fails.
- Breaking change: NO.
- User benefit: every hot path the v3 dep bumps touch gets at least one
  real-traffic confirmation before we cut the release; any regression
  surfaces now instead of in a user's production.
