# Proposal: phase4_triage-todo-debt

## Why

Audit counted **1,535 `TODO`/`FIXME`/`HACK`/`XXX` comments across 167 source files**. `AGENTS.md` Tier-1 rule #1 forbids TODOs outright. Concentration highlights:

- `src/server/` — 177 TODOs (handlers, MCP integration, auth)
- `src/intelligent_search/` — 39 TODOs
- `src/db/` — 38 TODOs
- `src/monitoring/` — 41 TODOs
- `src/server/mcp_handlers.rs` — 31 TODOs
- `src/intelligent_search/mcp_tools.rs` — 12 TODOs

Some are benign notes; some encode real missing behavior. Two specific hot spots already discovered:
- `src/server/rest_handlers.rs:1340,1394` — "TODO: Use actual collection UUID" → generates fresh UUIDs per request (data-loss / correctness bug — see `phase4_fix-uuid-generation-todo`).
- Multiple `// TODO: handle error properly` masking `.unwrap()` chains (see `phase3_reduce-unwrap-in-handlers`).

Leaving 1,535 TODOs sitting in code defeats the rule's purpose: reviewers tune out, and real issues hide inside noise.

## What Changes

1. **Extract all TODO comments** into a CSV: file, line, text, category (bug | feature | cleanup | noise | obsolete).
2. **Triage** in a working session:
   - **Bug** (code is incorrect as-is) → create a dedicated rulebook task
   - **Feature** (missing capability, documented) → create a rulebook task in the relevant backlog phase
   - **Cleanup** (refactor opportunity, low risk) → fix inline in a dedicated PR batch
   - **Noise** (stale comment, already done) → delete
   - **Obsolete** (about a removed feature) → delete
3. **Bulk-delete noise + obsolete**. Update any that should be bugs/features to refer to their rulebook task ID.
4. **Enforce zero-TODO** going forward via a CI grep gate allowing only `// TODO(task-id):` pattern with a valid task ID.

## Impact

- Affected specs: `/.rulebook/specs/TIER1_PROHIBITIONS.md`
- Affected code: every file containing `TODO`/`FIXME`/`HACK`/`XXX`
- Breaking change: NO
- User benefit: visible, prioritized backlog; review signal-to-noise improves.
