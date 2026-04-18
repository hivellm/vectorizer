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
