## 1. Upstream dependency

- [x] 1.1 Confirm (or land, if we own it) the `hive-gpu` API for Metal device introspection.
- [x] 1.2 Bump the `hive-gpu` version in `Cargo.toml` if the new API requires it.

## 2. Implementation

- [x] 2.1 Replace the placeholder block at `src/db/gpu_detection.rs:133` with real `hive-gpu` queries.
- [x] 2.2 Add an `info!` log line at startup showing the detected device.

## 3. Tests

- [x] 3.1 Smoke test on a macOS runner (CI) that the detection returns a non-default name when `hive-gpu` is enabled.
- [x] 3.2 Gated behind `#[cfg(all(target_os = "macos", feature = "hive-gpu"))]`.

## 4. Tail (mandatory)

- [x] 4.1 Update the GPU setup doc with what users see in the startup log.
- [x] 4.2 Tests above cover the new behavior.
- [x] 4.3 Run `cargo test --all-features` on macOS and confirm pass.
