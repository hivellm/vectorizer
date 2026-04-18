## 1. Decision

- [ ] 1.1 Audit callers of `onnx_models.rs` functions; if zero callers, Option B is preferred. Record in `design.md`
- [ ] 1.2 Create a rulebook decision for Option A vs Option B

## 2. Option A — Complete ONNX

- [ ] 2.1 Implement `get_or_download_model` using the `ort` crate and a model cache dir under `data/onnx/`
- [ ] 2.2 Implement `infer_batch` using session.run with tensor creation
- [ ] 2.3 Implement `apply_pooling` (mean/cls/max) as the ONNX output post-step
- [ ] 2.4 Add a `onnx-models` feature flag in `Cargo.toml`; gate the code
- [ ] 2.5 Remove `#[allow(dead_code)]` and `unreachable!()` bodies

## 3. Option B — Delete

- [ ] 3.1 Delete the three stub functions and their `#[allow]` attributes
- [ ] 3.2 Delete any references in `src/embedding/mod.rs` dispatcher
- [ ] 3.3 Update README / docs to list only working embedding providers

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Document the chosen path in CHANGELOG and `docs/features/embeddings.md`
- [ ] 4.2 Under Option A: add integration test loading a small ONNX model and verifying embedding dimensions; under Option B: add a negative test confirming ONNX is unavailable except via fastembed
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
