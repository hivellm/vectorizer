# Proposal: phase4_split-sdk-rust-client

## Why

`sdks/rust/src/client.rs` is **1,989 lines** with every API surface on one `VectorizerClient` struct. Rust users have the hardest time with this because `rust-analyzer`'s navigation is slower on large files and the trait-signature surface is dense. See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Split `sdks/rust/src/client/`:

- `transport.rs` — reqwest client, retry, auth header, error mapping.
- One module per surface: `collections`, `vectors`, `search`, `graph`, `admin`, `auth`.
- `mod.rs` — re-exports + a thin `VectorizerClient` facade struct that holds one inner `Arc<Transport>` and delegates to per-surface impls.

## Impact

- Affected specs: none.
- Affected code: `sdks/rust/src/client.rs` → `sdks/rust/src/client/`.
- Breaking change: NO — `use vectorizer_sdk::VectorizerClient` keeps working.
- User benefit: per-surface traits; each becomes independently reviewable.

## Cross-reference: RPC as the default transport

Plan the per-surface split so the upcoming RPC client
(`phase6_sdk-rust-rpc`) plugs into the same surface modules
(`collections`, `vectors`, `search`, `graph`, `admin`, `auth`) without
duplicating wrappers. The eventual constructor contract per
`phase6_make-rpc-default-transport`:

- `VectorizerClient::new("vectorizer://host:15503")` → RPC (default
  scheme; binary MessagePack, see `docs/specs/VECTORIZER_RPC.md`).
- `VectorizerClient::new("vectorizer://host")` → RPC on default port
  15503.
- `VectorizerClient::new("host:15503")` (no scheme) → RPC.
- `VectorizerClient::new("http://host:15002")` → REST (the legacy
  fallback; available for the lifetime of the v3.x line).

Keep `Transport` an enum behind the per-surface trait impls so the same
`collections::CollectionsApi` works against either backing transport.
