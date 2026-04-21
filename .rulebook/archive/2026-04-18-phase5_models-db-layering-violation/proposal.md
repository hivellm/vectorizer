# Proposal: phase5_models-db-layering-violation

## Why

`CLAUDE.md` claims strict layering: **Foundation → Core → Features → Presentation**, where models sit at Foundation. `src/models/mod.rs` currently imports and re-exports db-layer types:

```rust
pub use crate::db::graph::{Edge, Node, RelationshipType};
```

This is a **reverse-direction dependency**: Foundation (`models`) depends on Core (`db`). The architecture audit flagged this as the clearest layering violation in the codebase. Consequences:

- Circular-dep risk: any type `models` needs from `db` could eventually need something from `models` back.
- Confuses reviewers about which layer owns graph types.
- Breaks the "can I use `models` in isolation" contract that Foundation layers must preserve.

## What Changes

Two options, pick via decision record:

**Option A — Move graph types to models.** Relocate `Edge`, `Node`, `RelationshipType` definitions from `src/db/graph/` to `src/models/graph.rs`. The `db` layer then *imports* them from models (correct direction). Any db-specific graph operations (indexing, traversal implementations) stay in `db`.

**Option B — Remove the re-export.** If callers need graph types, import them directly from `crate::db::graph`. Delete the re-export line from `models`. This acknowledges graph as a Core concept, not Foundation.

Option A is preferred if `Edge`/`Node` are pure data structures; Option B if they carry db-state (indices, handles).

## Impact

- Affected specs: architecture/layering spec
- Affected code: `src/models/mod.rs`, `src/db/graph/*`, any consumer that imported via the re-export
- Breaking change: NO for external API, YES for internal import paths
- User benefit: restores clean layering; unblocks future `models`-only usage (e.g., client crates).
