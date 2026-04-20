# Proposal: phase4_split-vectorizer-workspace

## Why

Vectorizer ships as a single 1989-line-Cargo.toml monolith with **44 public modules** in one crate. The sibling HiveLLM repos (`Synap`, `Nexus`) are both Cargo workspaces with named crates (`*-server`, `*-cli`, `*-core`, `*-protocol`). Three concrete pains today:

1. **Compile-time tax.** Touching anything in `src/` recompiles every dependent (the whole crate). With 27 features and a deep transitive graph (113 top-level deps), incremental rebuilds for a one-line change in, say, `src/embedding/` rebuild the gRPC layer, the auth handlers, the dashboard handlers, the CLI binary — none of which depend on the change. A workspace draws crate boundaries that the build system can respect, so a one-line change in `vectorizer-core` rebuilds `vectorizer-core` + `vectorizer-server` (which depends on it), not `vectorizer-cli` (which doesn't need to be touched).
2. **SDK type duplication.** [`sdks/rust/`](../../../sdks/rust/) re-derives every wire type with `serde_json::Value` because it cannot reach into the server crate. With a `vectorizer-protocol` workspace member that carries the wire types, the Rust SDK depends on it directly and the duplication disappears. This is the same trick `Synap` uses (`sdks/rust` is a workspace member); `Nexus` keeps it separate, and we follow `Synap` because the wire-type duplication is the bigger lever.
3. **Boundary enforcement.** "Don't import `crate::server` from `crate::db`" is a convention today. Under a workspace it becomes a Cargo dependency edge — the borrow checker enforces the layering instead of code review.

Cross-checked against:
- [`e:/HiveLLM/Synap/`](../../../../Synap/) — workspace with `synap-server`, `synap-cli`, `synap-migrate`, `sdks/rust` as a member
- [`e:/HiveLLM/Nexus/`](../../../../Nexus/) — workspace with `nexus-core`, `nexus-server`, `nexus-protocol`, `nexus-cli`

The previous task (`phase4_consolidate-repo-layout`) flagged the workspace split as out of scope precisely because of its size; this task picks it up.

## What Changes

Final layout target (matches the design diagram from the consolidation task, adapted to the existing module shape):

```
Vectorizer/
├── Cargo.toml                          ← [workspace] root, lists members
├── crates/
│   ├── vectorizer-core/                ← src/{db,embedding,quantization,models,error,cache,parallel,normalization,quantization}
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── vectorizer-protocol/            ← src/{protocol,grpc/types,codec,models/wire-shaped types}
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── vectorizer-server/              ← src/{server,api,auth,replication,cluster,hub,grpc/server-side}
│   │   ├── Cargo.toml
│   │   └── src/{lib.rs, bin/server.rs}
│   └── vectorizer-cli/                 ← src/{cli, bin/}
│       ├── Cargo.toml
│       └── src/{lib.rs, bin/cli.rs}
├── sdks/rust/                          ← workspace member, depends on vectorizer-protocol
│   └── Cargo.toml                      ← [package], no [workspace]
├── tests/                              ← workspace-level integration tests
├── benches/                            ← unchanged
└── docs/, scripts/, k8s/, ... unchanged
```

Approach is **incremental** — every sub-phase compiles cleanly on its own and gets its own commit. No big-bang refactor.

### Sub-phases

1. **Skeleton (no refactor).** Create `crates/`, move the existing monolithic `src/`, `Cargo.toml`, and `tests/` under `crates/vectorizer/` so it becomes the sole workspace member. Add a workspace root `Cargo.toml` with `[workspace] members = ["crates/*"]`. Verify `cargo build`, `cargo test --lib`, `cargo clippy -- -D warnings`. Zero behaviour change — every `use crate::X` keeps working because the crate is still one piece, only its location moved.
2. **Extract `vectorizer-protocol`.** Lift `src/protocol/`, `src/codec.rs`, the wire-shaped types from `src/models/` (`Vector`, `CollectionConfig`, `SearchRequest/Response`, etc.), and the `tonic` proto-generated modules from `src/grpc/{vectorizer,cluster,qdrant_proto}.rs` into a new `crates/vectorizer-protocol/`. Update `crates/vectorizer/` to depend on it. This is the biggest single extraction; the goal is to fix the seam the SDK Rust client needs.
3. **Extract `vectorizer-core`.** Lift the storage / indexing / embedding engine (`src/{db,embedding,quantization,cache,parallel,normalization,models}` minus what already moved to `protocol`) into `crates/vectorizer-core/`. Server now depends on `core` + `protocol`.
4. **Extract `vectorizer-server`.** Lift `src/{server,api,auth,replication,cluster,hub}` + the gRPC server-side handlers + the `vectorizer` binary into `crates/vectorizer-server/`. Now `crates/vectorizer/` is empty (or just a re-export shim) and gets removed.
5. **Extract `vectorizer-cli`.** Lift `src/cli/` + `src/bin/{vectorizer-cli,create_mcp_key}.rs` into `crates/vectorizer-cli/`. CLI depends on `core` + `protocol` (not `server` — CLI is offline).
6. **Wire `sdks/rust`.** Add `sdks/rust` to the root `[workspace] members`. Replace its `serde_json::Value` wire-type stubs with imports from `vectorizer-protocol`. SDK retains its `[package]` (publishable to crates.io); the workspace inheritance is for shared `[workspace.dependencies]` and tooling.
7. **Verification.** `cargo check --workspace --all-features` clean, `cargo clippy --workspace -- -D warnings` clean, `cargo test --workspace` passes, `cargo doc --workspace --no-deps -D warnings` clean, every existing CI job still green.

## Impact

- **Affected specs**: none directly. `docs/specs/RUST.md` and `AGENTS.md` get a section on workspace conventions.
- **Affected code**: every file under `src/` moves into a `crates/<name>/src/` location. Imports rewrite from `crate::X` → `crate::X` (within a crate) or `vectorizer_core::X` / `vectorizer_protocol::X` (across crates). All 1000+ Rust files touched, but mechanically.
- **Breaking change**: YES for downstream Rust consumers who imported from `vectorizer::*`. The `vectorizer` crate name is preserved as an umbrella re-export crate (`crates/vectorizer/` becomes a thin facade that re-exports from `vectorizer-core` / `vectorizer-protocol`) so `vectorizer::db::VectorStore` still resolves; new code can target the leaner crates directly. Migration note in CHANGELOG.
- **User benefit**:
  - Incremental compile times drop substantially (touching `src/embedding/` no longer rebuilds `src/grpc/`).
  - Rust SDK shares wire types with the server — no more stub re-derivation, no more wire-format drift between server and client.
  - `core ↔ server ↔ protocol` boundary becomes Cargo-enforced instead of convention-enforced.
  - New contributors get the standard HiveLLM workspace shape.

## Out of scope

- Splitting per-feature flag combinations into separate crates (e.g. `vectorizer-fastembed`). Features stay where they are today.
- Renaming the published crate name on crates.io. The `vectorizer` umbrella crate keeps the existing slug and version line.
- C-FFI / Python wheel layout changes — those keep targeting the umbrella crate.
- Migrating tests/integration/* into per-crate `tests/`. Those stay at workspace root for now.

## Reference

- `e:/HiveLLM/Synap/` — closest match (workspace + `sdks/rust` as member). Use as the structural reference for `Cargo.toml` `[workspace]` shape.
- `e:/HiveLLM/Nexus/` — alternative pattern (`nexus-protocol` extracted, `sdks/rust` *not* a member). We diverge from Nexus on the SDK-membership choice but follow on the named-crate naming.
- `phase4_consolidate-repo-layout` (archived 2026-04-20) — the prerequisite housekeeping (root cleanup + config consolidation + Docker collapse) that makes this split tractable.
