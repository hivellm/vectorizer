# Vectorizer Improvement Analysis — 2026-07-11

> Five parallel read-only audits over the v3.5.0 codebase
> (`release/3.5.0` @ post-phase36 dependency refresh): core engine,
> API surface, test coverage, performance, operations/DX. Every
> finding carries a `file:line` reference verified at analysis time.
>
> Companion task set: `phase37`–`phase41` in `.rulebook/tasks/`
> (one task per theme, ordered by severity).
>
> Prior related analyses: `docs/specs/analysis/DOC_GAP_ANALYSIS_2026-04-24.md`
> (API/doc drift — partially superseded, see §2) and
> `docs/specs/analysis/IMPROVEMENT_ANALYSIS.md` (pre-v3 era).

## Executive summary — top findings by impact

| # | Severity | Theme | Finding | Where |
|---|---|---|---|---|
| 1 | **CRITICAL** | Durability | WAL never fsyncs (only `flush()`) and has no per-record checksum — power loss silently drops "durable" writes; a torn final JSON line aborts recovery of every later entry | `persistence/wal.rs:198,217,222-226` |
| 2 | **CRITICAL** | Correctness | BM25 vocabulary is NOT persisted by auto-save (stub writes `vocab_size: 0`); after restart, query embedding falls back to a hash space disjoint from stored vectors → search returns nothing until full re-index. Reproduced manually during the v3.4.0 Docker validation | `db/vector_store/autosave.rs:398-426,508-534`, `embedding/providers/bm25.rs:143,385-425` |
| 3 | **HIGH** | Performance | `insert_batch` holds the HNSW `index.write()` for the entire batch — payload indexing, sparse indexing, quantization, and graph discovery all run under the write lock, collapsing search p99 under mixed load | `db/collection/data.rs:61-226` |
| 4 | **HIGH** | Testing | 152 `#[ignore]`d tests (docs claim ~40); ~60 REST tests require a live server; ~30 REST handlers have zero non-ignored coverage; 5 replication test files are orphaned (never compiled) | see §3 |
| 5 | **HIGH** | API | Capability registry (`capabilities.rs`) claims source-of-truth but omits live REST endpoints, mismatches HTTP methods (graph.find_related GET vs POST), and doesn't model RPC/gRPC at all | `capabilities.rs:90-424,328` |
| 6 | **HIGH** | Security-adjacent | REST/MCP `search` accept unbounded `limit` (schema says max 100, handler ignores it) — memory-DoS vector | `search.rs:59,467`, `mcp/handlers.rs:175` |
| 7 | **MED** | Multi-tenancy bug | GraphQL tenant prefix inconsistent: `create_collection` uses `user_{id}:{name}`, `upload_file` uses `user_{id}_{name}` — uploads land in a different collection than created | `graphql/schema/mutation.rs:92,767` |
| 8 | **MED** | Architecture | 9 upward back-references (db→cluster, cache→monitoring, config→auth/hub/cluster, …) block the umbrella-crate split; `monitoring::METRICS` as a trait would kill 4 of 9 | see §1 |
| 9 | **MED** | Performance | PQ + Binary quantization fully implemented (900+ lines) but never wired into HNSW — only Scalar is accepted | `quantization/hnsw_integration.rs:74-83` |
| 10 | **MED** | SIMD | `quantize_f32_to_u8`/`dequantize` scalar-only everywhere; int8 dot product has no AVX2 path; SIMD correctness no longer CI-verified since simd-matrix removal | `vectorizer-core/src/simd/backend.rs:156-194` |

Sections: [§1 Core engine](01-core-engine.md) · [§2 API surface](02-api-surface.md)
· [§3 Test coverage](03-test-coverage.md) · [§4 Performance](04-performance.md)
· [§5 Operations & DX](05-ops-dx.md)

## Task mapping

| Task | Theme | Scope |
|---|---|---|
| `phase37_wal-durability-and-bm25-persistence` | Findings 1, 2 | fsync + CRC framing in WAL; wire BM25 vocab save/load into auto-save |
| `phase38_hot-path-lock-contention` | Findings 3, 9, 10 | narrow HNSW write-lock scope; clone churn; PQ/Binary wiring; SIMD quantize kernels |
| `phase39_test-debt-in-process-harness` | Finding 4 | in-process axum harness; un-orphan replication tests; handler coverage; SDK integration CI |
| `phase40_api-parity-and-hardening` | Findings 5, 6, 7 | capability registry completeness; limit clamps; error-shape unification; GraphQL tenant fix |
| `phase41_architecture-decoupling` | Finding 8 | MetricsSink trait; config sub-struct inversion; ShardRouter trait |

## Method

Five `Explore` agents ran in parallel with read-only access, each
scoped to one dimension and instructed to verify every claim with
`file:line` evidence. Their raw reports are reproduced (lightly
edited for layout) in the per-section files. Nothing in this
analysis modified code.
