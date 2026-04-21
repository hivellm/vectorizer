## 1. Discovery tests

- [x] 1.1 Reusable helper — turned out inlining the BM25-bootstrap pattern from `MCPToolHandler::new_with_store` was simpler than creating a shared `test_support` module, because only two call sites (`discovery/tests.rs` + `intelligent_search/examples.rs`) need it. Each test file owns its own 5-line `test_embedding_manager()` helper. If a third caller appears, the extraction becomes natural.
- [x] 1.2 Re-enable `src/discovery/tests.rs` integration test — new `discovery_pipeline_runs_against_empty_store` exercises the real `Discovery::new(config, Arc<VectorStore>, Arc<EmbeddingManager>)` signature and asserts the pipeline returns an empty result set with populated metrics.
- [x] 1.3 Delete commented-out tests — the old `test_full_pipeline` body was replaced by the new integration test; no commented code remains.

## 2. Intelligent-search examples

- [x] 2.1 Re-enable `examples.rs:311` (`test_example_usage`) — constructs a real `MCPToolHandler` via `MCPToolHandler::new(store, embedding_manager)` plus `RESTAPIHandler::new()` and `MCPServerIntegration::new()`. Verifies all three type constructors work end-to-end.
- [x] 2.2 Re-enable `examples.rs:325` and `examples.rs:344` (`test_tool_schemas`, `test_rest_endpoints`) — both now call `MCPServerIntegration::new()` (which takes no arguments, despite the original `TASK()` marker's claim) and validate the tool/endpoint schema shape.
- [x] 2.3 Delete commented-out tests — the three previously commented test bodies are fully restored with working assertions; no commented code remains.

## 3. Tail (mandatory)

- [x] 3.1 Inline helpers documented via doc comments in each test file; the 5-line `test_embedding_manager()` function in `intelligent_search/examples.rs::tests` carries the rationale. Captured in the follow-up task `phase4_extract-shared-test-support` if a third caller emerges.
- [x] 3.2 Tests are the deliverable — 4 new / restored tests (1 discovery + 3 intelligent_search).
- [x] 3.3 `cargo test --all-features` pass — 1122/1122 lib (+2 from restored bodies), 780/780 integration.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
