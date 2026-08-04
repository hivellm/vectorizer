<!-- RULEBOOK:START v7.0.0 — DO NOT EDIT BY HAND. Regenerated on `rulebook update`.
     Put project-specific content in AGENTS.override.md or CLAUDE.local.md.
     Anything outside the RULEBOOK:START/END sentinels is preserved across updates. -->

# CLAUDE.md

Managed by [@hivehub/rulebook](https://github.com/hivellm/rulebook) — few rules, all deliberate.

## Project-specific overrides (user-owned, survives `rulebook update`, wins on conflict)
@AGENTS.override.md

## Commands
- Before each commit: type-check + lint + the tests covering what changed.
- Before push / PR / task archive: the FULL quality gate (type-check → lint →
  full test suite), all green. Never bypass hooks — what the project wired
  into pre-commit/pre-push is the floor.
- Diagnostic-first: run the type-checker before the test suite; it is the faster signal.

## Values
1. Complete implementations — no stubs, no TODO markers left behind; finish, or say concretely why you can't.
2. Root causes, not workarounds — diagnose before changing code; never guess at bug causes.
3. Surgical diffs — touch only what the task needs; match existing style.
4. Simplicity first — the least code that solves the problem; no unrequested abstractions or features.
5. Fix forward — never discard uncommitted work.
6. State assumptions — if interpretations diverge, say so instead of picking silently.

## Git safety (requires explicit user authorization)
`reset --hard` · `checkout -- .` / `restore .` · `clean -f` · `push --force` ·
`rebase` on shared branches · `stash` · `branch -D` · switching a shared checkout
with changes you did not author. Yours autonomously: status/diff/log/add/commit,
branches you create (create/switch/merge), `git worktree`, PRs via `gh`.

## Orchestration
Subagents, parallel dispatch, and teams are your call — fan out freely when work is
parallel or context-heavy; work directly when it isn't. Rulebook never blocks or
mandates orchestration.

## Rulebook (on demand — no ceremony for small fixes)
- Multi-session or multi-phase work: track via the `rulebook` MCP (`rulebook_task`).
  Checklist order = dependencies; independent items may run in parallel.
- Optional session context: `rulebook_session`. Learned something non-obvious?
  `rulebook_memory`.
- Project specs live in `.rulebook/specs/` — read a spec when the work touches its area.
- Analyses live in `docs/analysis/<slug>/` — numbered files, one theme per file.
- Long session? `/compact <focus>` at a task boundary (~60% context). After
  `rulebook_task {action:"archive"}`, `/clear` is free — state lives in `.rulebook/`.

<!-- RULEBOOK:END -->

<!-- ──────────────────────────────────────────────────────────────────
     CONTENT BELOW IS PRESERVED ACROSS `rulebook update`.
     Project-specific behavioral overrides live here.
     ────────────────────────────────────────────────────────────────── -->

## Delegation, parallelism, and skill creation (highest precedence — apply on EVERY turn)

Recent sessions have under-used Claude Code's agent / Team / parallelism surface even when tasks clearly warranted it. Reverse that. The defaults below override any tendency to single-thread work.

### 1. Default to delegating non-trivial work to specialist agents

If a task fits a registered subagent's description (`implementer`, `tester`, `researcher`, `code-reviewer`, `docs-writer`, `architect`, `build-engineer`, `security-reviewer`, `Explore`, `Plan`, `feature-dev:*`, `cortex:*`, etc.), **delegate to that agent rather than doing the work in the main conversation**. Reasons:

- Specialist agents have task-tuned prompts and tooling.
- Subagent context is isolated, so the main thread stays uncluttered for orchestration.
- Parallel subagents finish in wall-clock time bounded by the slowest, not the sum.

Concrete heuristics (do these without being asked):

| Situation | Action |
|-----------|--------|
| User asks for any non-trivial implementation across 1+ files | Spawn `implementer` (sonnet) — don't write the code in the main thread |
| Tests need to be written | Spawn `tester` (sonnet) in parallel with `implementer` once the contract is clear |
| Research / "where is X" / "how does Y work" spanning >3 file reads | Spawn `Explore` (haiku) instead of grepping inline |
| Code review pass after implementation | Spawn `code-reviewer` (sonnet) — even on your own changes |
| Architecture / ADR / scalability question | Spawn `architect` (opus) |
| Build failure or CI break | Spawn `build-engineer` (sonnet) |
| Security audit, dependency review | Spawn `security-reviewer` (haiku) |
| Documentation refresh after a feature lands | Spawn `docs-writer` (haiku) in parallel with the merge |
| Need historical context on a decision | Spawn `cortex:cortex-historian` |
| About to take a destructive / governance-sensitive action | Spawn `cortex:cortex-lawkeeper` BEFORE the action |

The bar for "should I delegate?" is low. If a step would consume >100 lines of context or take >2 tool calls, it probably belongs in a subagent.

### 2. Parallelize aggressively

When two or more steps have **no data dependency on each other**, send them in a single message with multiple tool/agent calls — never serially. This includes:

- Multiple `Read` / `Grep` / `Glob` lookups on independent files.
- Multiple subagent spawns whose outputs you'll merge later.
- Independent `Bash` commands (e.g. `cargo check` + `cargo clippy` + `cargo fmt --check`).
- A `code-reviewer` agent on diff A and a `tester` agent on diff B.

A serial chain of 5 single-file reads is a regression vs one parallel batch.

### 3. Use Teams for coordinated multi-agent work

When a task needs >1 background agent that must coordinate (hand-offs, shared intermediate results, consensus), spawn a **Team**, not standalone background agents. Standalone background agents cannot exchange `SendMessage` and will silently produce conflicting edits.

Two valid patterns:

- **`team-lead` orchestrator**: spawn `Agent(subagent_type: "team-lead", run_in_background: true, ...)` and let it `TeamCreate` + dispatch members. The lead handles consensus.
- **Pre-create the Team**: `TeamCreate(name: "...", members: [...])` then spawn each member with `team_name` set so they share the message bus.

Background `Agent` calls without `team_name` (and not `team-lead`) are blocked by a project hook. That block is intentional — heed it.

`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` is already enabled for this project.

### 4. Create new skills + agents when a pattern repeats

If you find yourself doing the same multi-step ritual twice — or you anticipate it will be done again — **create a skill or subagent for it instead of re-running the prompt**. Specifically:

- **Repeat command sequence** (lint + format + test + commit + push pattern) → propose a slash command in `.claude/commands/`.
- **Repeated specialist persona** (e.g. "audit our gRPC schemas for Qdrant compatibility") → propose a new subagent in `.claude/agents/` with a tight `description` so future sessions auto-route to it.
- **Repeat investigation pattern** (e.g. "trace this error through the indexing pipeline") → propose a skill that codifies the runbook.

Don't ask permission for trivial helpers; do ask before adding anything that changes behavior outside the current task. When proposing, include: name, when-to-use trigger, files it would create, and a one-line value justification.

### 5. When NOT to delegate

These genuinely belong in the main thread:

- Single-file, sub-50-line edits where the change is obvious.
- Direct answers to user questions that don't require code.
- The orchestration / merging step itself (synthesizing subagent outputs is your job — don't delegate that).
- Tool calls that need access to the live conversation context (e.g. resolving an `<ide_selection>` reference).

### 6. Self-check at every turn

Before responding, ask: *"Did I parallelize what could be parallelized? Did I delegate what a specialist could do better?"* If the answer is no without a justifiable reason, restructure before sending.

