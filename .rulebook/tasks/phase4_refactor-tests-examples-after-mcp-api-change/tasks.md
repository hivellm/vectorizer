## 1. Discovery tests

- [ ] 1.1 Build a reusable `test_support::in_memory_vector_store()` + `test_support::null_embedding_manager()` if they don't already exist.
- [ ] 1.2 Re-enable `src/discovery/tests.rs` integration tests, passing the new constructor arguments.
- [ ] 1.3 Delete any sub-test whose feature has been removed entirely — do NOT keep it commented out.

## 2. Intelligent-search examples

- [ ] 2.1 Re-enable the test at `src/intelligent_search/examples.rs:311` by constructing a real `MCPToolHandler` with the test `VectorStore` / `EmbeddingManager`.
- [ ] 2.2 Re-enable the tests at `examples.rs:325` and `examples.rs:344` with proper `MCPServerIntegration::new` arguments.
- [ ] 2.3 Delete obsolete sub-tests entirely instead of commenting.

## 3. Tail (mandatory)

- [ ] 3.1 Document the new `test_support` helpers in the testing section of the module README.
- [ ] 3.2 Tests are the deliverable — no separate tests needed.
- [ ] 3.3 Run `cargo test --all-features` and confirm pass.
