# Proposal: phase8_ci-all-features-linux-matrix

## Why

`cargo test --workspace --lib --all-features` fails at link time on
Windows with:

```
error: linking with `link.exe` failed: exit code: 1319
  |
  = note: ... <160 object files omitted>
```

Exit code 1319 on MSVC is the "path too long" signal — the combined
feature set (`real-models + onnx-models + arrow + parquet +
transmutation + fastembed + hive-gpu + simd + simd-avx2 + simd-neon +
simd-wasm + ...`) produces enough rlib paths + symbol files to
overflow the MSVC command line.

Consequence: the release-gate test pass
(`phase8_release-v3-runtime-verification` tail item 6.3) cannot be
verified from a Windows dev workstation with the strict all-features
flag. Current workaround is running without `--all-features` (1263
tests pass), which leaves the feature-gated paths unexercised on
every PR.

Source: `docs/releases/v3.0.0-verification.md` section 5 + commit
messages noting "MSVC path-length link failure".

## What Changes

Stand up a GitHub Actions job that runs the all-features suite on
Linux (where the command-line limit is ~2 MB vs Windows' ~32 kB) so
the release gate has an authoritative green signal regardless of
the contributor's local OS.

1. Add `.github/workflows/rust-all-features.yml`:
   - `runs-on: ubuntu-latest`
   - `cargo test --workspace --lib --all-features --no-fail-fast`
   - Cache the target dir via `Swatinem/rust-cache@v2`.
   - Upload the full test log as a build artifact on failure.
2. Gate merging to `release/v3.0.0` + `main` on this workflow.
3. Document the Windows workaround in `docs/specs/RUST.md` (or
   CONTRIBUTING.md): developers on Windows should run
   `cargo test --workspace --lib` (no `--all-features`); the
   all-features coverage is the CI responsibility.

Optional follow-up (not part of this task, tracked in
`docs/specs/RUST.md`): shorten compiled rlib paths via
`CARGO_TARGET_DIR=C:\t` and see if that brings Windows under the
limit. Unlikely to move the needle on a 160+ object-file link line
but a one-line try is cheap.

## Impact

- Affected specs: `docs/specs/RUST.md` (testing guide);
  `CONTRIBUTING.md` (Windows workflow note).
- Affected code: `.github/workflows/rust-all-features.yml` (new).
- Breaking change: NO (pure CI addition).
- User benefit: every PR gets an authoritative all-features green /
  red signal. The release gate (task 6.3 of the v3 verification)
  can cite a specific workflow run.
