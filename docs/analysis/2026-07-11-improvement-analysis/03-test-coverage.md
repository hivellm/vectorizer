# §3 — Test Coverage Gaps

> Scope: `crates/vectorizer/tests/`, `crates/vectorizer-server/tests/`,
> colocated unit tests, `docs/development/testing.md`,
> `.github/workflows/sdk-*.yml`.

## 3.1 Ignored-test inventory

**152 `#[ignore]` across 43 files** — not "~40" as
`docs/development/testing.md` claims; the doc is badly stale.

Breakdown:

- **Live-server REST (`*_real.rs`, "requires 127.0.0.1:15002")** —
  the bulk: `batch_insert_real` (13), `vector_search_real` (5),
  `batch_ops_real` (5), `embedding_provider_real` (5),
  `auth_enforcement_real` (4), `move_vectors_real` (4),
  `file_upload_real` (3) + `file_upload` (6),
  `qdrant_compat_minimal_real` (3), `force_save_real` (2).
- **Live hub/GraphQL**: `hub_integration_live` (10),
  `graphql/hub_integration` (8), `hub/failover_tests` (7),
  `graphql/encryption` (4), `encryption` (3), `encryption_complete`
  (5, "Flaky on macOS CI").
- **gRPC "Update operation fails in CI"**: `grpc_integration`,
  `grpc_comprehensive` (2), `grpc/vectors`, `grpc_s2s`, `grpc_live` —
  the same symptom in 5+ places.
- **Replication**: `failover` (5), `comprehensive` (5).
- **integration/**: `payload_index` (6), `sparse_vector` (5),
  `binary_quantization` (4), `graph` (4), `query_cache` (3),
  `cluster_performance` (3), `hybrid_search` (1).
- **performance/**: `multi_collection` (3), `multi_tenant_load` (3).
- **src unit tests**: `candle_models` (5), `storage/snapshot` (2),
  `quantization/product` (1), `async_indexing` (1),
  `batch/parallel` (1), `replication/{sync,tests}` (2).

**Stale / bad reasons:**

1. `failover.rs` (5), `comprehensive.rs` (4), `gpu/hive_gpu` (1),
   `product.rs` (1) use **bare `#[ignore]`** — the doc's own policy
   calls this a regression.
2. The gRPC-update cluster labeled "flaky/CI env" is **identical
   everywhere** → almost certainly a real update bug mislabeled as
   flakiness.
3. `encryption_complete` "flaky on macOS" was never revisited.

## 3.2 Untested public surface (REST handlers)

Grepping all ~90 handler fn names across `tests/`: **zero name
references in the integration tree** — only two `src/*/tests.rs`
unit files match. Handlers with **no non-ignored coverage**:

`batch_update_vectors`, `batch_delete_vectors`, `explain_search`,
`bulk_update_metadata`, `copy_vectors`, `set_vector_expiry`,
`delete_by_filter`, `reencode_collection`, `set_collection_ttl`,
`rename_collection`, `reindex_collection`,
`create/list/restore_native_snapshot`, `get_project_outline`,
`get_related_files`, `search_by_file_type`, `get_file_summary`,
`list_slow_queries` / `set_slow_query_config`, `restart_server`,
`update_workspace_config`, `get_config`, `broad_discovery`,
`semantic_focus`, `promote_readme`, `build_answer_plan`,
`render_llm_prompt`, `get_backup_directory`,
`list_empty_collections`.

Their only exercise is via the ignored `*_real.rs` HTTP suites →
effectively **0 CI coverage**.

## 3.3 Untested failure paths

Handler modules return rich `ErrorResponse` variants but the only
compiled tests are happy-path. `handler_robustness.rs` tests a toy
2-field router, not real handlers. Error branches in `search.rs`,
`vectors.rs`, `collections.rs`, `backups.rs` (dim mismatch, missing
collection, empty IDs, quota) are asserted **only inside ignored
live suites**.

## 3.4 In-process harness feasibility

**Feasible and partially bootstrapped.** The codebase already uses
`tower::ServiceExt::oneshot` in-process in `handler_robustness.rs`,
`csrf_bearer_exemption.rs`, and `auth_handlers_tests.rs`. There is no
`axum_test`/`TestServer` dep and **no shared harness that builds the
full `VectorizerServer` router**. Adding one (Router from real app
state + `oneshot`) lets the ~60 `*_real.rs`/`*_live.rs` ignored tests
run in CI without a live binary.

## 3.5 SDK test posture

All five SDK workflows run **unit tests only; every integration/s2s
test is env-skipped**: `VITEST_SKIP_S2S` (TS),
`SKIP_INTEGRATION_TESTS`/`SKIP_S2S_TESTS` (Py/C#/Go/Rust), Go
`-short`, Rust explicitly excludes `integration_tests.rs`.
`sdk-all-tests.yml` is a no-op summary aggregator. **No SDK-to-server
integration runs in CI at all.**

## 3.6 Replication / cluster test reality

- ~~**`replication/mod.rs` wires only `failover, handlers, integration,
  qdrant`.** Five files — `comprehensive.rs`, `integration_basic.rs`,
  `api.rs`, `qdrant_migration.rs`, `qdrant_api.rs` — are **orphaned:
  never compiled or run**.~~ **CORRECTED during phase39
  implementation:** those five files ARE compiled — `integration.rs`
  and `qdrant.rs` pull them in via `#[path]` includes, which the
  mod.rs-only scan missed. They run in the normal suite (13 + 26
  tests pass), and `integration_basic`'s "Category C known bug" tests
  pass too, i.e. the tracked snapshot-sync bug has since been fixed.
  The stale part of the claim was the testing doc, not the wiring.
- `failover.rs`'s 5 tests are bare-ignored (TCP).
- **Cluster is healthier**: `integration/mod.rs` wires all
  `cluster_*` + `raft`/`sharding`; only `cluster_performance` (3) is
  ignored — the rest run in-process in CI.

## 3.7 Findings table

| Sev | Location | Description | Fix |
|---|---|---|---|
| **HIGH** | `tests/replication/{comprehensive,integration_basic,api,qdrant_migration,qdrant_api}.rs` | Not declared in `mod.rs`; never compiled/run | Wire into `replication/mod.rs` or delete |
| **HIGH** | `tests/api/rest/*_real.rs`, `hub_integration_live.rs` (~60 tests) | Entire REST/hub surface gated behind a live server | Build in-process axum `oneshot` harness over the `VectorizerServer` router |
| **HIGH** | `rest_handlers/{search,vectors,collections,backups,files,slow_queries,admin}.rs` | ~30 handlers with zero non-ignored coverage | Router-level tests once the harness exists |
| **MED** | gRPC `test_update_vector` etc. | "Flaky/CI env" identical everywhere = likely real update bug | Triage as bug, not flaky |
| **MED** | `docs/development/testing.md` | Claims ~40 ignores (actual 152); lists dead `integration_basic.rs` as active | Regenerate from grep; add CI count gate |
| **MED** | `.github/workflows/sdk-*.yml` | All SDK integration skipped; no server-backed CI | Gated job spinning up the server (docker) |
| **LOW** | `failover.rs`, `comprehensive.rs`, `gpu/hive_gpu.rs`, `product.rs` | Bare `#[ignore]` without reason strings | Add `#[ignore = "…"]` reasons |

(→ phase39)
