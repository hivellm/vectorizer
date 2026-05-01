<!-- RULEBOOK:START v5.3.0 — DO NOT EDIT BY HAND. Regenerated on `rulebook update`.
     Put project-specific content in AGENTS.override.md or CLAUDE.local.md.
     Anything outside the RULEBOOK:START/END sentinels is preserved across updates. -->

# CLAUDE.md

This project is managed by [@hivehub/rulebook](https://github.com/hivellm/rulebook).
The authoritative rules come from the imports below. Claude Code loads all of them
automatically at session start (see [Anthropic memory docs](https://code.claude.com/docs/en/memory#claude-md-imports)).

## Project identity & live state
<!-- @.rulebook/STATE.md (skipped — target file not present) -->

## Core standards (team-shared, versioned)
@AGENTS.md

## Project-specific overrides (user-owned, survives `rulebook update`)
@AGENTS.override.md

## Session scratchpad (human notes)
<!-- @.rulebook/PLANS.md (skipped — target file not present) -->

## Critical rules (highest precedence — apply on every turn)

1. **Read `AGENTS.md` and `AGENTS.override.md`** before making changes. These contain project-specific conventions that override generic guidance.
2. **Never revert or discard uncommitted work** — fix forward. Treat the working tree as sacred; investigate before destructive operations.
3. **Edit files sequentially**, not in parallel. When a task touches 3+ files, decompose into 1–2 file sub-tasks.
4. **Run `check`/type-check before `test`** — diagnostic-first. Cheap diagnostics catch issues that expensive test suites miss or take longer to surface.
5. **If a fix fails twice, escalate** — stop, research, or open a team. Do not retry the same approach a third time.
6. **Prefer MCP tools** (`mcp__rulebook__*` and project-specific MCP servers) over shell commands when the equivalent tool exists.
7. **Capture learnings**: at the end of significant work, save patterns and anti-patterns to `.rulebook/knowledge/` and insights to `.rulebook/learnings/`.
8. **Never archive a task** without docs updated, tests written, and tests passing — the task tail enforces this structurally.

## Editing discipline (Karpathy-inspired)

Behavioral guidelines that reduce common LLM coding mistakes. Adapted from [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills), grounded in [Andrej Karpathy's observations](https://x.com/karpathy/status/2015883857489522876).

1. **Think before coding.** State assumptions explicitly. If multiple interpretations exist, present them — don't pick silently. If a simpler approach exists, say so. If something is unclear, stop and ask. Don't hide confusion.
2. **Simplicity first.** Minimum code that solves the problem. No features beyond what was asked, no abstractions for single-use code, no "flexibility" that wasn't requested, no error handling for impossible scenarios. If you write 200 lines and 50 would do, rewrite.
3. **Surgical changes.** Touch only what you must. Don't "improve" adjacent code, comments, or formatting. Don't refactor things that aren't broken. Match existing style. If you notice unrelated dead code, mention it — don't delete it. Every changed line must trace directly to the user's request.
4. **Goal-driven execution.** Define verifiable success criteria upfront. "Add validation" → "write tests for invalid inputs, then make them pass." For multi-step tasks, state a brief plan: `[step] → verify: [check]`. Strong criteria let you loop independently; weak criteria require constant clarification.

## Persistent memory

This project uses the Rulebook MCP server for persistent memory across sessions.

- **Start of session**: `rulebook_memory_search` for relevant prior context.
- **During work**: `rulebook_memory_save` for decisions, bugs, discoveries, user preferences.
- **End of session**: `rulebook_session_end` to write a session summary.

Memory is auto-captured for tool interactions (task create/update/archive, skill enable/disable). Manual saves are required for everything else worth remembering.

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
