# Proposal: phase4_enforce-public-api-docs

## Why

Spot-checks across the codebase found that public APIs regularly ship without doc comments:

- `src/db/mod.rs` — declares 30+ public re-exports with **zero `///` documentation**; only trait bounds are present.
- `src/embedding/mod.rs` — trait `EmbeddingProvider` methods are documented, but the concrete provider types (`TfIdfEmbedding`, `Bm25Embedding`, `SvdEmbedding`, `BertEmbedding`, `FastEmbed`) have no doc comments.
- `src/api/mod.rs` — no `//!` module-level documentation; submodules (`cluster.rs`, `graph.rs`, `graphql/`) lack item-level docs.
- README markets "1701 passing tests, 95% coverage" but `cargo doc` output is thin.

Rust ecosystem convention expects `#![warn(missing_docs)]` on the crate root. Enforcing it now catches undocumented items mechanically, and future contributions can't regress.

## What Changes

1. Add `#![warn(missing_docs)]` (or `#![deny(...)]` if we're confident) to `src/lib.rs`.
2. Walk every public item flagged by the compiler and add a `///` comment describing what it is and what invariants it preserves.
3. Add `//!` module-level headers explaining the role of each module: `db`, `api`, `embedding`, `server`, `grpc`, `cluster`, `auth`, `cache`, `discovery`, `persistence`, `quantization`, `replication`.
4. Add crate-level `#![doc = include_str!("../README.md")]` or a curated crate-doc section at the top of `lib.rs` so `cargo doc` renders a landing page.
5. Configure CI to run `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"` — doc warnings become errors.

## Impact

- Affected specs: documentation spec
- Affected code: every public item in `src/` without current docs (100s of items)
- Breaking change: NO
- User benefit: consumers of the crate (and internal contributors) get reliable API docs; drift is prevented.
