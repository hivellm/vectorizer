## 1. Root-cause investigation

- [ ] 1.1 Trace `POST /collections` from handler to `EmbeddingManager` and log every provider-resolution branch
- [ ] 1.2 Reproduce the coercion in a local `cargo run` build against `hivehub/vectorizer:3.3.0` config
- [ ] 1.3 Verify which `--features` the published Docker image was built with (compare `Dockerfile` vs `crates/vectorizer/Cargo.toml` defaults)
- [ ] 1.4 Determine whether `fastembed`/`onnx` providers are registered at all in the running binary
- [ ] 1.5 Write findings to `design.md` (root cause: code bug vs config gap vs missing assets)

## 2. Honour `embedding_provider` on create-collection

- [ ] 2.1 Replace silent fall-through with a `Result<ProviderHandle, EmbeddingError::UnsupportedProvider { requested, available: Vec<String> }>`
- [ ] 2.2 Map `EmbeddingError::UnsupportedProvider` to `400 Bad Request` with body `{ "error": "unsupported_provider", "requested": "...", "available": [...] }`
- [ ] 2.3 Reject dimension mismatch with `400 dimension_mismatch` when caller-requested `dimension` differs from the provider's native dimension
- [ ] 2.4 Update REST tests + MCP parity tests for the new error shapes

## 3. Honour `/embed` `model`

- [ ] 3.1 Resolve `model` against the registry; same `UnsupportedModel` error shape as §2
- [ ] 3.2 Default-model selection MUST be deterministic (config / first-registered / explicit default)
- [ ] 3.3 Log the default model at startup and expose it via `GET /stats`

## 4. Provider discovery endpoint

- [ ] 4.1 Add `GET /providers` returning `[ { name, kind: "dense"|"sparse", dimension, default: bool }, ... ]`
- [ ] 4.2 Or extend `GET /stats` with a `providers` array (decide in §1's design.md)
- [ ] 4.3 Mirror through MCP (`list_providers` tool)

## 5. Default dense provider in Docker

- [ ] 5.1 Decide bundled model (proposal: FastEmbed `all-MiniLM-L6-v2`, 384-dim) — record in `design.md`
- [ ] 5.2 Pre-fetch the model in the `Dockerfile` so the image is self-contained (no first-boot download)
- [ ] 5.3 Register the dense provider as the default when the image is built with `--features fastembed`
- [ ] 5.4 Keep BM25 registered as the sparse provider; `dense+bm25` hybrid stays available

## 6. Documentation + samples

- [ ] 6.1 Write `docs/embedding/providers.md`: supported providers, dimensions, how to enable each (env / config / model mount)
- [ ] 6.2 Update REST API docs for the new `400 unsupported_provider` / `400 unsupported_model` shapes
- [ ] 6.3 README "Embedding" section advertises the bundled dense default + how to swap models
- [ ] 6.4 CHANGELOG entry under v3.4.0 calling out the contract change

## 7. Integration tests

- [ ] 7.1 Test: create a collection with `embedding_provider: "fastembed"` on an image that bundles fastembed — collection round-trips with correct provider + dimension
- [ ] 7.2 Test: create a collection with `embedding_provider: "fastembed"` on a build without fastembed feature — `400 unsupported_provider` with the available list
- [ ] 7.3 Test: `/embed` with `model: "bge-small"` returns the requested model's vector when registered, `400 unsupported_model` otherwise
- [ ] 7.4 Test: `GET /providers` (or `/stats.providers`) lists every registered provider with correct dimension
- [ ] 7.5 Docker smoke test: pull image, `POST /collections {embedding_provider: "fastembed", dimension: 384}`, insert + query a vector, assert non-zero recall on a paraphrase

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update or create documentation covering the implementation
- [ ] 8.2 Write tests covering the new behavior
- [ ] 8.3 Run tests and confirm they pass
