## 1. Analysis

- [x] 1.1 Enumerate every `as_any().downcast_ref()` site in `src/` — found 4 sites, all inside the single `EmbeddingManager::save_vocabulary_json` method in `src/embedding/mod.rs`. The "7-branch match at `src/server/mcp_handlers.rs:30-87`" that the proposal flagged turned out to be a STRING→CONSTRUCTOR factory (`"bm25" => Bm25Embedding::new(...)`), not a runtime downcast dispatch — that pattern is idiomatic and unchanged.
- [x] 1.2 Designed a single trait method `save_vocabulary_json(&self, path: &Path) -> Result<()>` with a default impl that returns an error. Concrete providers override to call their inherent method.

## 2. Trait extension

- [x] 2.1 Added `fn save_vocabulary_json(&self, _path: &Path) -> Result<()>` to `EmbeddingProvider` in `src/embedding/mod.rs` with a default `Err(...)` body.
- [x] 2.2 Overrode on the four vocabulary-bearing providers: `TfIdfEmbedding`, `Bm25Embedding`, `BagOfWordsEmbedding`, `CharNGramEmbedding`. Each override delegates to the existing inherent `save_vocabulary_json<P: AsRef<Path>>` via UFCS (`Type::save_vocabulary_json(self, path)`), so the inherent method keeps its generic signature for direct callers.

## 3. Dispatch replacement

- [x] 3.1 Replaced the 4-branch `downcast_ref` if-chain at the end of `EmbeddingManager::save_vocabulary_json` with a single `provider.save_vocabulary_json(path.as_ref())` call, wrapping the error with the provider name for actionable HTTP/MCP responses.
- [x] 3.2 No change to `src/server/mcp_handlers.rs` — the "match" cited in the original proposal is a string→constructor factory (legitimate) and has no downcast.
- [x] 3.3 `as_any` and `as_any_mut` kept on the trait — still used by `real_models` dispatch code and by a few integration-test paths. A full removal would need a separate audit, tracked as a potential follow-up.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 CHANGELOG `[Unreleased] > Changed` entry added describing the trait-method contract and what it unlocks for new providers.
- [x] 4.2 Two new unit tests in `embedding::tests`: `save_vocabulary_dispatches_through_trait_for_bm25` (success path, asserts the JSON body contains `"type": "bm25"`), and `save_vocabulary_errors_for_provider_without_override` (SVD path, asserts the provider-aware error message).
- [x] 4.3 `cargo test --lib -p vectorizer -- embedding::tests::save_vocabulary` — 2/2 passing. `cargo clippy --all-targets -- -D warnings` — green.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `Changed` entry)
- [x] Write tests covering the new behavior (2 new unit tests, both passing)
- [x] Run tests and confirm they pass (`cargo test --lib -- embedding::tests::save_vocabulary` 2/2)
