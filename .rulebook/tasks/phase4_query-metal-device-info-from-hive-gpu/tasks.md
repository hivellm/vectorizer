## 1. Upstream dependency

- [ ] 1.1 Confirm (or land, if we own it) the `hive-gpu` API for Metal device introspection.
- [ ] 1.2 Bump the `hive-gpu` version in `Cargo.toml` if the new API requires it.

## 2. Implementation

- [ ] 2.1 Replace the placeholder block at `src/db/gpu_detection.rs:133` with real `hive-gpu` queries.
- [ ] 2.2 Add an `info!` log line at startup showing the detected device.

## 3. Tests

- [ ] 3.1 Smoke test on a macOS runner (CI) that the detection returns a non-default name when `hive-gpu` is enabled.
- [ ] 3.2 Gated behind `#[cfg(all(target_os = "macos", feature = "hive-gpu"))]`.

## 4. Tail (mandatory)

- [ ] 4.1 Update the GPU setup doc with what users see in the startup log.
- [ ] 4.2 Tests above cover the new behavior.
- [ ] 4.3 Run `cargo test --all-features` on macOS and confirm pass.
