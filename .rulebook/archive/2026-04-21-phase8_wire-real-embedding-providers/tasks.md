## 1. Implementation

- [x] 1.1 Add `FastEmbedProvider` adapter at
  `crates/vectorizer/src/embedding/providers/fastembed.rs` — wraps
  `fastembed::TextEmbedding`, lazy-init on first `embed()`, caches
  models under `vectorizer_core::paths::cache_dir().join("fastembed")`.
  Exposes the `EmbeddingProvider` trait.
- [x] 1.2 Expose `OnnxEmbedder::new_from_config(cfg)` and
  `RealModelEmbedder::new_from_config(cfg)` as public constructors the
  bootstrap can reach; wire their tracing so `ort::Session` init + the
  hf-hub download progress land in the structured log.
- [x] 1.3 Parse `config.embedding.model`, `default_model`, and the
  per-backend sub-sections in `bootstrap.rs` and dispatch:
  `bm25` → existing path; `fastembed:<id>` → FastEmbedProvider;
  `onnx:<path|id>` → OnnxEmbedder; `candle:<id>` → RealModelEmbedder.
- [x] 1.4 Register the resolved provider on all three embedding
  managers the bootstrap creates (server, file watcher, final), not
  just the first.
- [x] 1.5 Add boot-time validation: if the config names a backend
  whose Cargo feature is off, fail fast with a typed error (no silent
  fallback to BM25).

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  (new `docs/specs/EMBEDDING.md` model-selection matrix, `config.example.yml`
  comment block on the `embedding.model` shape, CHANGELOG entry under
  `3.0.0 > Added`).
- [x] 2.2 Write tests covering the new behavior (integration test
  under `tests/api/rest/real_embedding_real.rs` that boots the server
  with `embedding.model: "fastembed:all-MiniLM-L6-v2"`, POSTs `/embed`,
  and asserts the response vector dim matches the model's expected
  dim; gated `#[cfg(feature = "fastembed")]`).
- [x] 2.3 Run tests and confirm they pass (the new integration test
  plus `cargo test -p vectorizer embedding::`).
