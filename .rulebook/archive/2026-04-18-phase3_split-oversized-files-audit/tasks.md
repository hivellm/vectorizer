## 1. Audit document

- [x] 1.1 Create `docs/refactoring/oversized-files-audit.md` with full table (file, lines, severity, structural seams, proposed module layout) for all 14 oversized files
- [x] 1.2 Verify line counts are current (re-run `wc -l`) and update the audit if drift is found
- [x] 1.3 Add cross-link from the audit to each of the 3 existing `phase3_split-*-monolith` tasks
- [x] 1.4 Add cross-link from each of the 3 existing `phase3_split-*-monolith` proposals back to the audit

## 2. Create follow-up split tasks (phase3 — core engine)

- [x] 2.1 `rulebook_task_create` → `phase3_split-collection-monolith` (`src/db/collection.rs`, 2649) — split `impl Collection` + extract tests file
- [x] 2.2 `rulebook_task_create` → `phase3_split-qdrant-grpc` (`src/grpc/qdrant_grpc.rs`, 2109) — split by gRPC trait: collections / points / snapshots
- [x] 2.3 `rulebook_task_create` → `phase3_split-auth-handlers` (`src/server/auth_handlers.rs`, 1749) — split types / state / handlers / admin / middleware / tests

## 3. Create follow-up split tasks (phase4 — providers, API surfaces, SDKs)

- [x] 3.1 `rulebook_task_create` → `phase4_split-embedding-providers` (`src/embedding/mod.rs`, 1724) — one file per provider (bm25, bert, minilm, tfidf, svd, bow, char_ngram) + manager
- [x] 3.2 `rulebook_task_create` → `phase4_split-graphql-schema` (`src/api/graphql/schema.rs`, 1698) — split `QueryRoot` / `MutationRoot`; investigate possible dead code at L1610–1698
- [x] 3.3 `rulebook_task_create` → `phase4_split-advanced-search` (`src/search/advanced_search.rs`, 1508) — extract 638L of config/structs to `config.rs`; one file per engine component
- [x] 3.4 `rulebook_task_create` → `phase4_split-qdrant-api-integration-tests` (`tests/integration/qdrant_api.rs`, 1595) — split by capability tested
- [x] 3.5 `rulebook_task_create` → `phase4_split-sdk-python-client` (`sdks/python/client.py`, 2907)
- [x] 3.6 `rulebook_task_create` → `phase4_split-sdk-javascript-client` (`sdks/javascript/src/client.js`, 2002)
- [x] 3.7 `rulebook_task_create` → `phase4_split-sdk-rust-client` (`sdks/rust/src/client.rs`, 1989)
- [x] 3.8 `rulebook_task_create` → `phase4_split-sdk-typescript-client` (`sdks/typescript/src/client.ts`, 1879)

## 4. Knowledge capture

- [x] 4.1 `rulebook_knowledge_add` anti-pattern: "Files > 1500 lines without impl-level boundaries" referencing this audit
- [x] 4.2 `rulebook_memory_save` summary of the audit outcome (14 files surveyed, 3 in progress, 11 newly tracked)

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 5.1 Update or create documentation covering the implementation (audit doc in `docs/refactoring/`)
- [x] 5.2 Write tests covering the new behavior (N/A — meta task; record rationale in task notes instead of skipping: no production code changed, so no test is possible. A lint rule / CI check that flags files > 1500 lines is the equivalent verification and is proposed in §1 of the audit doc.)
- [x] 5.3 Run tests and confirm they pass (run full `cargo check` + `cargo test` to confirm no drift from adjacent branches before archiving; attach output to archive notes)
