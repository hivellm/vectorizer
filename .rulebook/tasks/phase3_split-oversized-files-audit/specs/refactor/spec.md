# Oversized Files Split Audit — Spec

## ADDED Requirements

### Requirement: Oversized-files audit document

The repository SHALL contain a canonical audit document at `docs/refactoring/oversized-files-audit.md` that lists every project-owned source file exceeding 1500 lines, its severity, structural seams (impl/mod/section boundaries with line numbers), and a proposed module layout for splitting it.

#### Scenario: Audit lists every oversized project file

Given the audit is generated from a fresh `wc -l` run
When a reviewer opens `docs/refactoring/oversized-files-audit.md`
Then every file in `src/`, `tests/`, and `sdks/` with more than 1500 lines MUST appear in the table, except files whose first line contains `@generated` or that live under the vendored `qdrant/` submodule.

#### Scenario: Audit cross-references rulebook tasks

Given three split tasks already exist in phase3 and new tasks are created in §2/§3
When a reader follows any row in the audit table
Then the row MUST link to the rulebook task that owns the split, and the task's `proposal.md` MUST contain a reverse link to the audit section.

### Requirement: Every oversized file has an owning rulebook task

Every file identified in the audit SHALL have a dedicated rulebook task whose id matches `phase{3,4}_split-<description>`. No oversized file may remain without an owning task after this audit task is archived.

#### Scenario: All 14 oversized files have owning tasks

Given the audit identifies 14 oversized files
When `rulebook_task_list` is run after this task completes §2 and §3
Then it MUST contain at least 14 task ids whose `proposal.md` names the corresponding source file as the split target (3 pre-existing + 11 created here).

### Requirement: No production-code edits from the audit task

This audit task MUST NOT modify production code. Only `docs/**`, `.rulebook/**`, and knowledge-base entries may be written.

#### Scenario: Production code is untouched on audit branch

Given the audit task branch has completed all items
When `git diff --name-only main...HEAD` is run
Then no path under `src/` or `sdks/**/src/` SHALL appear in the diff.

### Requirement: Anti-pattern captured in knowledge base

An anti-pattern entry SHALL be added to the rulebook knowledge base describing "files over 1500 lines without impl-level boundaries" as a reviewability and maintenance risk, citing this audit as evidence.

#### Scenario: Anti-pattern queryable via knowledge list

Given the audit task has reached §4
When `rulebook_knowledge_list --type anti-pattern` is run
Then one entry MUST match the title "Files > 1500 lines without impl-level boundaries" and reference the audit document path.
