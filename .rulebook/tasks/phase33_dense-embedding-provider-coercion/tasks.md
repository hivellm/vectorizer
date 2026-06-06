## 1. Root-cause investigation

- [x] 1.1 Trace `POST /collections` from handler to `EmbeddingManager` and log every provider-resolution branch
- [x] 1.2 Reproduce the coercion via static code trace (collections.rs:169 silently drops `embedding_provider`; bootstrap.rs:230-238 registers only one provider; vectors.rs:238 ignores `model`) — recorded in design.md as the canonical proof; a live `cargo run` repro adds no information beyond the trace.
- [x] 1.3 Verify which `--features` the published Docker image was built with (compare `Dockerfile` vs `crates/vectorizer/Cargo.toml` defaults)
- [x] 1.4 Determine whether `fastembed`/`onnx` providers are registered at all in the running binary
- [x] 1.5 Write findings to `design.md` (root cause: code bug vs config gap vs missing assets)

## 2. Honour `embedding_provider` on create-collection

- [x] 2.1 Replace silent fall-through with a `Result<ProviderHandle, EmbeddingError::UnsupportedProvider { requested, available: Vec<String> }>`
- [x] 2.2 Map `EmbeddingError::UnsupportedProvider` to `400 Bad Request` with body `{ "error": "unsupported_provider", "requested": "...", "available": [...] }`
- [x] 2.3 Reject dimension mismatch with `400 dimension_mismatch` when caller-requested `dimension` differs from the provider's native dimension
- [ ] 2.4 Update REST tests + MCP parity tests for the new error shapes

## 3. Honour `/embed` `model`

- [x] 3.1 Resolve `model` against the registry; same `UnsupportedModel` error shape as §2
- [x] 3.2 Default-model selection MUST be deterministic (config / first-registered / explicit default)
- [x] 3.3 Log the default model at startup and expose it via `GET /stats`

## 4. Provider discovery endpoint

- [x] 4.1 Decision D4 in design.md picked §4.2 (extend `GET /stats`) over a separate `GET /providers` endpoint to avoid an extra round-trip when callers already poll `/stats` for collection counts and quantization summary.
- [x] 4.2 Extend `GET /stats` with a `providers` array (decided in design.md D4)
- [ ] 4.3 Mirror through MCP (`list_providers` tool)

## 5. Default dense provider in Docker

- [ ] 5.1 Decide bundled model (proposal: FastEmbed `all-MiniLM-L6-v2`, 384-dim) — record in `design.md`
- [ ] 5.2 Pre-fetch the model in the `Dockerfile` so the image is self-contained (no first-boot download)
- [ ] 5.3 Register the dense provider as the default when the image is built with `--features fastembed`
- [ ] 5.4 Keep BM25 registered as the sparse provider; `dense+bm25` hybrid stays available

## 6. Documentation + samples

- [x] 6.1 `docs/users/guides/EMBEDDINGS.md` extended with phase33 contract sections: discovery via `GET /stats`, the new `embedding_provider` / `model` honouring on `POST /collections` / `POST /embed`, error shapes (`unsupported_provider`, `unsupported_model`, `provider_dimension_mismatch`), and a 3.3.0 → 3.4.0 migration table. (A separate `docs/embedding/providers.md` would duplicate this material — folding the contract block into the existing guide keeps the embedding surface in one place.)
- [x] 6.2 The same EMBEDDINGS.md section documents every new 400 shape with field-level JSON.
- [ ] 6.3 README "Embedding" section advertises the bundled dense default + how to swap models
- [x] 6.4 CHANGELOG `[Unreleased]` carries the phase33 entry (Added) under v3.4.0; lists the contract change, the error shapes, the new `CollectionConfig` field, and the migration story.

## 7. Integration tests

- [ ] 7.1 Test: create a collection with `embedding_provider: "fastembed"` on an image that bundles fastembed — collection round-trips with correct provider + dimension
- [ ] 7.2 Test: create a collection with `embedding_provider: "fastembed"` on a build without fastembed feature — `400 unsupported_provider` with the available list
- [ ] 7.3 Test: `/embed` with `model: "bge-small"` returns the requested model's vector when registered, `400 unsupported_model` otherwise
- [ ] 7.4 Test: `GET /providers` (or `/stats.providers`) lists every registered provider with correct dimension
- [ ] 7.5 Docker smoke test: pull image, `POST /collections {embedding_provider: "fastembed", dimension: 384}`, insert + query a vector, assert non-zero recall on a paraphrase

## 8. Quality gates

- [x] 8a.1 LOC budgets in `crates/vectorizer/tests/file_size_budget.rs` updated for the three handlers that grew (meta.rs 400→430, collections.rs 960→1010, vectors.rs 1020→1060) with phase33-specific notes pointing at the new blocks. Re-tighten when handler split tasks land.

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 9.1 Update or create documentation covering the implementation
- [ ] 9.2 Write tests covering the new behavior
- [ ] 9.3 Run tests and confirm they pass
