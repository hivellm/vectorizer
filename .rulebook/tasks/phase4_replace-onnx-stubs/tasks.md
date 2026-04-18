## 1. Decision

- [x] 1.1 Audit callers of the three stub methods — grep confirmed only the definition sites reference them; no caller anywhere in `src/`. The stubs were purely shape placeholders for a future real `ort` integration.
- [x] 1.2 Chose Option B (delete). Rationale: the real embedding path in `OnnxEmbedder` (`embed`, `embed_batch`, `embed_parallel`) uses a deterministic xxh3-hash-derived vector and never touches the stubbed helpers. Keeping the stubs creates a "if anyone ever wires a call, the process aborts via `unreachable!()`" foot-gun for zero benefit today. Option A (implement real ONNX) is a separate, multi-day feature that belongs in its own task when there's demand.

## 2. Option B — Delete

- [x] 2.1 Delete the three stub methods and their `#[allow(dead_code)]` attributes from `src/embedding/onnx_models.rs`: `get_or_download_model` (line 159), `infer_batch` (line 203), `apply_pooling` (line 209).
- [x] 2.2 No references in `src/embedding/mod.rs` dispatcher or anywhere else needed removal — the stubs were private and unreferenced.
- [x] 2.3 README / docs already list `OnnxEmbedder` as a deterministic-hash compat layer (see the file's module header). No copy change required.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 CHANGELOG `[Unreleased] > Removed` entry added explaining what was deleted and why.
- [x] 3.2 No new positive test — deleting dead code doesn't need one; the compile-time check (no caller can even name the functions anymore) is the guard. Adding a "negative" test that the functions don't exist would exercise the compiler, not the program.
- [x] 3.3 Run `cargo check --lib` and `cargo clippy --all-targets -- -D warnings` — both green in under 1s each (incremental rebuild after the trivial delete).

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `Removed` entry)
- [x] Write tests covering the new behavior (no behavior to test — code deletion; compile is the contract)
- [x] Run tests and confirm they pass (cargo check + clippy green)
