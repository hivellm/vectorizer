# Proposal: phase27_phase25-multi-sdk-propagation

## Why

Phase25 §7.1 shipped the Rust SDK changes (`RuntimeMetrics`,
`RouteStats`, `WalSnapshot`, `VectorCountSample` models, plus
`get_runtime_metrics()` and the new `Stats` / `Collection` fields).
Phase25 §7.2 calls for the same surface in the TypeScript, Python,
Go, and C# SDKs.

Each SDK lives in its own crate / package with its own model layer,
transport plumbing, test setup, and changelog. Doing all four inside
phase25 would balloon the task without a clean review boundary, so
this is split out as a focused follow-up. The dashboard already
consumes `/metrics/runtime` directly via fetch, so multi-SDK parity
is a follower-of-record not a critical path.

## What Changes

For each of the four SDKs (TS, Python, Go, C#):

1. Mirror the Rust models:
   - `RuntimeMetrics` (cpu/memory/connections/uptime/QPS/5xx-rate +
     `throughput_by_route` + `wal`).
   - `RouteStats` (per-route p50/p99 + qps).
   - `WalSnapshot` (current_seq / size_bytes / last_checkpoint_at /
     last_checkpoint_seq).
   - `VectorCountSample { at, count }`.
   - Extend the existing `Stats` model with `default_quantization`
     and `compression_ratio` (default to `("none", 1.0)`).
   - Extend the `Collection` / `CollectionInfo` model with
     `vector_count_history: VectorCountSample[]` (default `[]`).
2. Add a `getRuntimeMetrics()` (camelCase per SDK convention) client
   method targeting `GET /metrics/runtime`.
3. Defaults must keep older servers working without runtime errors —
   missing fields decode to zero / empty.
4. Unit tests covering full + partial payloads.
5. CHANGELOG entry under `[Unreleased]` per SDK.

## Impact

- Affected code:
  - `sdks/typescript/src/models/*` + `src/client/*`
  - `sdks/python/vectorizer/*` (models + client + tests)
  - `sdks/go/*` (models + client + tests)
  - `sdks/dotnet/*` (models + client + tests)
- Affected specs: none (additive, REST surface unchanged)
- Breaking change: NO
- User benefit: every supported SDK can introspect the live
  `/metrics/runtime` endpoint without falling back to raw HTTP, and
  surface the new `Stats` quantization fields + per-collection
  `vector_count_history` to downstream apps.
