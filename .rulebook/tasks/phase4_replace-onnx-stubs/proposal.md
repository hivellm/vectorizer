# Proposal: phase4_replace-onnx-stubs

## Why

`src/embedding/onnx_models.rs` contains three `#[allow(dead_code)]` stub functions that hit `unreachable!()`:

- Line 160 — `get_or_download_model()`
- Line 204 — `infer_batch()`
- Line 210 — `apply_pooling()`

The file header describes them as a "compat layer generating placeholder embeddings." These are implementation stubs left from an incomplete ONNX integration. Problems:

- If the function is called under an unexpected code path, the server panics (`unreachable!()` aborts the process).
- Users relying on what looks like ONNX-backed embeddings may silently get placeholders.
- The stubs are masked by `#[allow(dead_code)]` so the compiler doesn't warn anyone.

## What Changes

Two paths, choose via decision record:

**Option A — Complete the ONNX integration.** Implement real `get_or_download_model`, `infer_batch`, `apply_pooling` using the `ort` crate already listed in `Cargo.toml` under `fastembed` feature. Gate the code behind a `onnx-models` feature that pulls in `ort` explicitly.

**Option B — Delete the stubs.** Remove the three functions, their `#[allow]`, and any callers. Document in CHANGELOG that ONNX models are only available via the `fastembed` provider (which already works).

## Impact

- Affected specs: embedding spec
- Affected code: `src/embedding/onnx_models.rs`, possibly `src/embedding/mod.rs` dispatcher, Cargo.toml features
- Breaking change: Option B is a feature removal if any consumer depended on the stubs
- User benefit: no more silent placeholder embeddings and no more panic paths; clear embedding-provider surface.
