<!-- TIER1_PROHIBITIONS:START -->
# Absolute Prohibitions (Tier 1 — Highest Precedence)

**These rules override ALL other rules. Violation = output rejected.**

---

## PROHIBITION 1: No Shortcuts, Stubs, or Simplified Logic

**NEVER** simplify logic, add `TODO`/`FIXME`/`HACK`/`XXX`, create stubs, use placeholders, alter existing logic to avoid complexity, reduce scope, skip edge cases, or deliver partial implementations.

**Response time is IRRELEVANT. Quality is everything.**

### Forbidden Patterns
- `// TODO` — unfinished work disguised as progress
- `// FIXME` — a known bug left for "later"
- `// HACK` — a shortcut that will haunt you
- `// XXX` — same category, different typographical flavor
- `return 0; // placeholder` — a lie that compiles
- `/* stub */` — an empty promise
- Simplified algorithms where the correct one is known
- Partial implementations ("I'll add the rest later")
- Reduced scope without explicit approval

### Allow-list (CI-enforced by `scripts/check-no-tier1-markers.sh`)

Only two forms of the forbidden tokens are permitted:

1. **`// TASK(phaseN_<slug>):`** — a comment that points at a tracked rulebook task. Every such marker must correspond to a real `.rulebook/tasks/<slug>/` directory. Grep pattern matched by the gate: `TASK\(phase[0-9]+_[a-z0-9-]+\)`.
2. **`grep-ignore(tier1-markers)` on the same line** — for code that must keep the literal tokens because it *detects* them in user data (e.g. `src/file_operations/operations.rs` has a summarizer that searches for `TODO`/`FIXME` in source files).

Any other occurrence fails CI. The gate script is `scripts/check-no-tier1-markers.sh` and the workflow step lives in `.github/workflows/rust-lint.yml`.

### Required Behavior
- **Research** the correct approach before writing code
- **Implement completely** — every function, every edge case, every error path
- **Take as long as needed** — correct implementation > fast delivery
- **Ask if unsure** — propose a plan, never silently simplify

---

## PROHIBITION 2: No Destructive Git Operations Without Authorization

### Allowed (always safe)
- `git status`, `git diff`, `git log`, `git blame`
- `git add`, `git commit` (after quality checks)

### Forbidden (require explicit user authorization)
- `git stash` — can lose uncommitted work
- `git rebase` — rewrites history
- `git reset --hard` — destroys uncommitted changes
- `git checkout -- .` / `git restore .` — discards all changes
- `git revert` — creates new commits
- `git cherry-pick` — can cause conflicts
- `git merge` — can create conflicts
- `git branch -D` — deletes branch permanently
- `git push --force` — overwrites remote history
- `git clean -f` — deletes untracked files permanently
- `git checkout <branch>` / `git switch` — breaks concurrent sessions sharing the worktree

**Why**: Multiple AI sessions may share the same working tree. Destructive operations affect ALL concurrent sessions.

---

## PROHIBITION 3: No Deletion Without Authorization

**NEVER** run `rm`, `rm -rf`, `del`, or delete any file without explicit user authorization ("yes, delete it").

This includes:
- Cache files (they auto-invalidate — DO NOT manually delete)
- Backup files
- Temporary files (clean up YOUR temp files, never others')
- Build artifacts (use the build system's clean command)
- Lock files (investigate what holds the lock first)

**Why**: AI agents repeatedly delete important files during "cleanup." The cost of an unauthorized deletion (hours rebuilding caches, lost data) always exceeds the cost of asking first.

---

## PROHIBITION 4: Research Before Implementing — Never Guess

**NEVER** guess at:
- The cause of a bug
- How an API works
- What a function does based on its name
- What "correct" output looks like

### Required Process
1. **State what you KNOW** (from logs, debug output, code reading)
2. **State what you DON'T KNOW**
3. **Research** the unknown (read source, check docs, use diagnostic tools)
4. **Only then** implement the fix

**"I think this might be the problem" is NOT acceptable.**
**"Source X does Y at file:line, we do Z, the difference causes W" IS acceptable.**

---

## PROHIBITION 5: Sequential File Editing

**ALWAYS** edit files one at a time in sequence: Read file1 → Edit file1 → Read file2 → Edit file2.

**NEVER** batch-read multiple files then batch-edit them. By the time you edit file 3, the context from file 1 may be stale.

When a task touches 3+ files across subsystems:
1. **STOP** — do not start implementing
2. **Plan** the changes (list files, dependency order)
3. **Decompose** into sub-tasks of 1-2 files each
4. **Execute** sub-tasks in dependency order
5. **Build/test** after each sub-task

---

## PROHIBITION 6: No Deferred Tasks

If a task is in the checklist, **implement it**. No exceptions.

- **NEVER** mark tasks as "Deferred"
- **NEVER** write "Deferred — requires X"
- **NEVER** skip tasks with excuses
- **NEVER** deliver partial implementations with "will do later"

If a task has a genuine dependency:
1. Implement the dependency FIRST
2. Then implement the task
3. Mark BOTH as done

If you truly cannot implement something, explain WHY in concrete terms and propose an alternative — do NOT just write "Deferred."

---

## PROHIBITION 7: Follow Task Sequence — No Reordering, No Cherry-Picking

When a `tasks.md` checklist defines a sequence of items, **execute them in EXACTLY that order**.

### Forbidden

- **NEVER** skip ahead to "easier" or "more interesting" tasks
- **NEVER** reorder tasks because you think a different order is better
- **NEVER** cherry-pick tasks from the middle of a list
- **NEVER** decide which tasks are "important enough" to do — do ALL of them, in order
- **NEVER** group or batch tasks in a different sequence than listed
- **NEVER** start Phase N+1 before Phase N is 100% complete

### Required Behavior

1. Read `tasks.md` from top to bottom
2. Find the FIRST unchecked item (`- [ ]`)
3. Implement THAT item — not the one you prefer
4. Mark it `[x]` with what was done
5. Move to the NEXT unchecked item
6. Repeat until all items are checked

### Why

The human spent time defining the task sequence for a reason. The order reflects dependencies, priorities, and a deliberate implementation strategy. When AI agents skip around:
- Dependencies break because upstream work wasn't done first
- The human loses track of what's actually complete
- Work has to be redone because it was built on missing foundations
- Trust erodes — the human defined a plan and the AI ignored it

**The task list is an ORDER, not a MENU. Execute sequentially.**

<!-- TIER1_PROHIBITIONS:END -->