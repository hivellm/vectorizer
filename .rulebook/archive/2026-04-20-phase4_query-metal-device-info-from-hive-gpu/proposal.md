# Proposal: phase4_query-metal-device-info-from-hive-gpu

## Why

`src/db/gpu_detection.rs` at L133 uses placeholder values for Metal device information (device name, memory, unified/discrete, etc.) because the upstream `hive-gpu` crate does not yet expose a public API for this. The project ships with GPU-tier logic that branches on these fields, so running on the wrong accelerator silently falls back to a conservative default — users have no diagnostic showing which GPU was actually picked.

## What Changes

1. Wait for (or contribute) a `hive-gpu` API surface that exposes: device name, adapter kind (integrated/discrete), total VRAM, Metal family, Apple Silicon generation.
2. Replace the placeholder block in `gpu_detection.rs` with real queries against that API.
3. Log the detected device at startup at `info!` so operators can confirm which accelerator is in use.

## Impact

- Affected specs: none.
- Affected code: `src/db/gpu_detection.rs`; possibly a `hive-gpu` version bump in `Cargo.toml`.
- Breaking change: NO — replaces placeholders with real data.
- User benefit: operators get accurate GPU diagnostics instead of default stubs.
