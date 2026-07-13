<!-- RULEBOOK:START v5.3.0 — DO NOT EDIT BY HAND. Regenerated on `rulebook update`.
     Put project-specific content in AGENTS.override.md or CLAUDE.local.md.
     Anything outside the RULEBOOK:START/END sentinels is preserved across updates. -->

# CLAUDE.md

This project is managed by [@hivehub/rulebook](https://github.com/hivellm/rulebook).
The authoritative rules come from the imports below. Claude Code loads all of them
automatically at session start (see [Anthropic memory docs](https://code.claude.com/docs/en/memory#claude-md-imports)).

## Project identity & live state
@.rulebook/STATE.md

## Core standards (team-shared, versioned)
@AGENTS.md

## Project-specific overrides (user-owned, survives `rulebook update`)
@AGENTS.override.md

## Session scratchpad (human notes)
@.rulebook/PLANS.md

## Critical rules (highest precedence — apply on every turn)

1. **Read `AGENTS.md` and `AGENTS.override.md`** before making changes. These contain project-specific conventions that override generic guidance.
2. **Never revert or discard uncommitted work** — fix forward. Treat the working tree as sacred; investigate before destructive operations.
3. **Edit files sequentially**, not in parallel. When a task touches 3+ files, decompose into 1–2 file sub-tasks.
4. **Run `check`/type-check before `test`** — diagnostic-first. Cheap diagnostics catch issues that expensive test suites miss or take longer to surface.
5. **If a fix fails twice, escalate** — stop, research, or open a team. Do not retry the same approach a third time.
6. **Prefer MCP tools** (`mcp__rulebook__*` and project-specific MCP servers) over shell commands when the equivalent tool exists.
7. **Capture learnings**: at the end of significant work, save patterns and anti-patterns to `.rulebook/knowledge/` and insights to `.rulebook/learnings/`.
8. **Never archive a task** without docs updated, tests written, and tests passing — the task tail enforces this structurally.

## Delegation & parallelism (highest precedence — apply on every turn)

**Default behavior: delegate, don't do it yourself. Parallelize, don't serialize. Create new agents/skills when the gap is real.**

1. **Delegate by default.** If a step matches an agent in the delegation table, dispatch it via `Agent` instead of doing it inline. Implementation → `implementer` (sonnet). Research / read-only exploration → `researcher` (haiku). Tests → `tester`. Docs → `docs-writer` (haiku). Architecture / cross-cutting → `architect` (opus). Reserve the main conversation for orchestration + decisions.
2. **Parallelize independent work.** When a turn requires multiple independent investigations or edits, dispatch every independent piece in **a single message with multiple `Agent` tool-use blocks**. Sequential `Agent` calls are a smell — every time you catch yourself writing "first X, then Y", check whether the two halves are independent.
3. **Use Teams for multi-specialist work.** Anything that needs ≥2 background agents to coordinate MUST go through a Team (`TeamCreate` + `team_name` on dispatch). Standalone background `Agent` calls without `team_name` are blocked by the enforcement hook.
4. **Create skills + agents when the gap is real.** If you write the same multi-step instructions twice in one session, lift it into a skill (`templates/skills/<category>/<name>/SKILL.md`). If a class of work repeats across projects, create an agent definition under `.claude/agents/`. Default to creating, not improvising.
5. **Foreground vs background.** Use foreground `Agent` when you need the result to inform your next step. Use background only with `team_name` so messages can flow.

## Editing discipline (Karpathy-inspired)

Behavioral guidelines that reduce common LLM coding mistakes. Adapted from [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills), grounded in [Andrej Karpathy's observations](https://x.com/karpathy/status/2015883857489522876).

1. **Think before coding.** State assumptions explicitly. If multiple interpretations exist, present them — don't pick silently. If a simpler approach exists, say so. If something is unclear, stop and ask. Don't hide confusion.
2. **Simplicity first.** Minimum code that solves the problem. No features beyond what was asked, no abstractions for single-use code, no "flexibility" that wasn't requested, no error handling for impossible scenarios. If you write 200 lines and 50 would do, rewrite.
3. **Surgical changes.** Touch only what you must. Don't "improve" adjacent code, comments, or formatting. Don't refactor things that aren't broken. Match existing style. If you notice unrelated dead code, mention it — don't delete it. Every changed line must trace directly to the user's request.
4. **Goal-driven execution.** Define verifiable success criteria upfront. "Add validation" → "write tests for invalid inputs, then make them pass." For multi-step tasks, state a brief plan: `[step] → verify: [check]`. Strong criteria let you loop independently; weak criteria require constant clarification.

## Session continuity

- **Start of session**: read `.rulebook/PLANS.md` and call `rulebook_session_start` to load prior context.
- **End of session**: `rulebook_session_end` writes a summary to `.rulebook/PLANS.md`.

## Knowledge base

Before implementing anything non-trivial:

- `rulebook_knowledge_list` — check existing patterns and anti-patterns.
- `rulebook_learn_list` — review past learnings.
- `rulebook_decision_list` — review architectural decisions.

After implementing, capture at least one entry per task:

- `rulebook_knowledge_add` for reusable patterns or anti-patterns to avoid.
- `rulebook_learn_capture` for implementation insights that don't belong in code comments.
- `rulebook_decision_create` for significant architectural choices.

## Task workflow

**MANDATORY: ALWAYS use the Rulebook MCP tools for task management.** Never create task directories or files manually — use `rulebook_task_create`, `rulebook_task_update`, `rulebook_task_archive`, `rulebook_task_list`, `rulebook_task_show`, `rulebook_task_validate`. These tools enforce naming conventions, mandatory tail items, phase structure, and metadata that manual file creation skips.

1. `rulebook_task_list` to see pending work.
2. `rulebook_task_create` to create new tasks — **never `mkdir` + `Write` manually**.
3. Pick the **first unchecked item from the lowest-numbered phase** — never reorder.
4. Read the task's `proposal.md` and `tasks.md` before touching code.
5. Implement step by step. Run lint + type-check after each significant change.
6. `rulebook_task_update` to change task status as you progress.
7. Mark items `[x]` in `tasks.md` as you finish them.
8. The mandatory tail (docs + tests + verify) is **not optional** — `rulebook_task_archive` will refuse to close the task otherwise.

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

