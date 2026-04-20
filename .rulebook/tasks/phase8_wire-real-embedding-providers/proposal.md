# Proposal: phase8_wire-real-embedding-providers

## Why

`fastembed 5.13`, `ort 2.0.0-rc.11`, and `hf-hub 0.5` are declared as
optional deps in `crates/vectorizer/Cargo.toml` but have **no active
call sites** outside of the `real-models` cfg-gated module. The server
bootstrap in
`crates/vectorizer-server/src/server/core/bootstrap.rs:156-159`
hard-wires `Bm25Embedding::new(512)` as the only registered provider
and never consults `config.yml > embedding.model`, so:

- `embedding.model: "bm25"` in `config/config.yml:193` is ignored — the
  server can only run BM25 whether the config says so or not.
- Building with `--features fastembed,ort,hf-hub,real-models` produces
  a binary that still cannot load any real model because the
  `RealModelEmbedder` path is unreachable from the REST / MCP / RPC
  surfaces.
- Probe 2.2 of `phase8_release-v3-runtime-verification` — "`ort::Session`
  initialisation logged + model download logged from hf-hub" — fails
  because neither ever fires.

The three deps were bumped during the v3 migrations
(`phase5_refresh-all-dependencies` +
`phase6_major-dep-migrations`), and the existing code compiles against
the new versions (otherwise the workspace build would fail), but no
production path exercises them. That is a release-readiness risk:
operators who expect the documented ONNX support to work will see
BM25 only, with no clear error message.

Found during probe 2.2 of `phase8_release-v3-runtime-verification`;
evidence at `docs/releases/v3.0.0-verification.md` probe 2.2 row.

## What Changes

Wire the real embedding providers end-to-end:

1. **Bootstrap reads `embedding` from `VectorizerConfig`.** Parse
   `embedding.model`, `embedding.default_model`, `embedding.fastembed.*`,
   `embedding.bert.*` keys in `bootstrap.rs` and dispatch on the
   selected model:
   - `"bm25"` — current path (already works).
   - `"fastembed:<model-id>"` (e.g. `fastembed:all-MiniLM-L6-v2`) —
     register a `FastEmbedProvider` that wraps `fastembed::TextEmbedding`.
   - `"onnx:<path>"` / `"onnx:<model-id>"` — register an `OnnxEmbedder`
     (`onnx_models.rs`) that loads via `ort::Session`.
   - `"candle:<model-id>"` — register `RealModelEmbedder` (already
     implemented behind `real-models` feature).
2. **Implement the `FastEmbedProvider` adapter** in
   `crates/vectorizer/src/embedding/providers/fastembed.rs`
   (new file) — takes the fastembed model id + cache dir, runs
   `TextEmbedding::try_new(...)` on first use, exposes the
   `EmbeddingProvider` trait.
3. **Log at INFO** the provider name + model id + `ort::Session`
   initialization + hf-hub download events (fastembed already logs to
   stderr by default; wire its log output through `tracing`).
4. **Default behavior stays BM25** when `embedding.model` is `"bm25"`
   or missing. Falling back to BM25 must be logged at WARN only when
   the operator explicitly asked for a real model that failed to load
   (unknown model id, download failed, ort init failed).
5. **Config validation** at boot: if `embedding.model` names a
   fastembed / onnx / candle provider but the corresponding Cargo
   feature wasn't enabled at compile time, exit with a typed error at
   bootstrap (do not silently fall back).

## Impact

- Affected specs: `docs/specs/EMBEDDING.md` (document the model
  selection matrix), `CHANGELOG.md` under `3.0.0 > Added` /
  `3.0.0 > Fixed`.
- Affected code:
  - `crates/vectorizer-server/src/server/core/bootstrap.rs`
    (~80 LOC of parse + register + log)
  - `crates/vectorizer/src/embedding/providers/fastembed.rs` (new)
  - `crates/vectorizer/src/embedding/providers/mod.rs` (register module)
  - `crates/vectorizer/src/embedding/real_models.rs` (expose constructor
    that the bootstrap can reach)
  - `crates/vectorizer/src/embedding/onnx_models.rs` (same)
  - `config/config.example.yml` (document the new model selector shape)
- Breaking change: NO — `embedding.model: "bm25"` stays the default.
- User benefit: the documented ONNX / fastembed / candle support
  actually works, and the v3 dep bumps (`fastembed 5.13`, `ort rc.11`,
  `hf-hub 0.5`, `candle 0.10`) have live runtime coverage before the
  release ships.
