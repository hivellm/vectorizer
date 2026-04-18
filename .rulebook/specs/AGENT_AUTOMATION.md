<!-- AGENT_AUTOMATION:START -->
# Agent Automation Rules

**CRITICAL**: Mandatory workflow that AI agents MUST execute after EVERY implementation.

## ⚠️ TOKEN OPTIMIZATION (MANDATORY FOR HAIKU)

**Claude Haiku has limited context. Every token counts.**

### Core Rules ✅
1. **Output code, not explanation** - Put logic in comments, not markdown
2. **Minimal reports** - Say "✅ Done" instead of detailed status reports
3. **No markdown abuse** - No unnecessary headings, tables, or emoji status lines
4. **Combine outputs** - One response instead of multiple small ones
5. **Comments > Documentation** - Use code comments for explaining logic

### Tokens Saved 💰
- Remove status emoji lines: **~500 tokens/task**
- Skip "Next Steps" sections: **~100 tokens/task**
- Eliminate markdown tables: **~200 tokens/task**
- Use concise commit messages: **~50 tokens/task**

**Total per task**: ~850 tokens saved = much more context for actual work

### Examples

❌ BAD (Wastes tokens):
```
✅ Implementation Complete

📝 Changes:
- Added UserService
- Added middleware
- Updated routes

🧪 Quality Checks:
- ✅ Type check: Passed
- ✅ Lint: 0 warnings
- ✅ Tests: 45/45 passed
- ✅ Coverage: 96%
```

✅ GOOD (Efficient):
```
✅ Done. UserService + middleware committed.
```

## Workflow Overview

After completing ANY feature, bug fix, or code change, execute this workflow in order:

### Step 1: Quality Checks (MANDATORY)

Run these checks in order - ALL must pass:

```bash
1. Type check (if applicable)
2. Lint (MUST pass with ZERO warnings)
3. Format code
4. Run ALL tests (MUST pass 100%)
5. Verify coverage meets threshold (default 95%)
```

**Language-specific commands**: See your language template (TYPESCRIPT, RUST, PYTHON, etc.) for exact commands.

**IF ANY CHECK FAILS:**
- ❌ STOP immediately
- ❌ DO NOT proceed
- ❌ DO NOT commit
- ✅ Fix the issue first
- ✅ Re-run ALL checks

**⚠️ TOKEN OPTIMIZATION**:
- Output only pass/fail status for each check
- Do NOT output detailed logs or test results
- Do NOT create status tables or emoji reports
- Use concise format: "✅ type-check pass" or "❌ tests fail: reason"

### Step 2: Capture to Persistent Memory

**IMPORTANT**: Save implementation insights and decisions to persistent memory for context across future sessions.

```bash
# 1. Identify key learnings from this implementation:
#    - Design decisions made
#    - Patterns discovered or applied
#    - Gotchas or edge cases encountered
#    - Performance insights
#    - Test coverage notes

# 2. Save to memory (via MCP or CLI):
rulebook memory save "<content>" --type feature --title "Brief title" --tags tag1,tag2

# 3. Example:
rulebook memory save "Implemented OAuth token refresh with 30-min expiry. Key gotcha: tokens expire silently without warning on API calls - must check headers before retry. Pattern: Use interceptor middleware for transparent refresh." --type feature --title "OAuth token refresh implementation" --tags auth,oauth,gotchas
```

**Memory Auto-Capture**: If memory auto-capture is enabled in `.rulebook`, significant implementation outputs are automatically captured. Review and augment with additional context as needed.

### Step 3: Security & Dependency Audits

```bash
# Check for vulnerabilities (language-specific)
# Check for outdated dependencies (informational)
# Find unused dependencies (optional)
```

**Language-specific commands**: See your language template for audit commands.

**IF VULNERABILITIES FOUND:**
- ✅ Attempt automatic fix
- ✅ Document if auto-fix fails
- ✅ Include in Step 5 report
- ❌ Never ignore critical/high vulnerabilities without user approval

### Step 4: Update Rulebook Tasks

If using rulebook task management:

```bash
# Mark implemented tasks as completed
rulebook task update <task-id> --status completed

# Update any blocked/pending tasks
rulebook task update <task-id> --status blocked --reason "explanation"
```

**⚠️ CRITICAL: Follow the task sequence**

When working through a `tasks.md` checklist:
- Execute items in the EXACT order listed — top to bottom
- NEVER skip ahead, reorder, or cherry-pick "easier" tasks
- NEVER start Phase N+1 before Phase N is 100% complete
- The task list is an ORDER, not a MENU

### Step 5: Update OpenSpec Tasks

If `openspec/` directory exists:

```bash
# Mark completed tasks as [DONE]
# Update in-progress tasks
# Add new tasks if discovered
# Update progress percentages
# Document deviations or blockers
```

### Step 6: Update Documentation

```bash
# Update ROADMAP.md (if feature is milestone)
# Update CHANGELOG.md (conventional commits format)
# Update feature specs (if implementation differs)
# Update README.md (if public API changed)
```

### Step 7: Git Commit

**ONLY after ALL above steps pass:**

**⚠️ CRITICAL: All commit messages MUST be in English**

```bash
git add .
git commit -m "<type>(<scope>): <description>

- Detailed change 1
- Detailed change 2
- Tests: [describe coverage]
- Coverage: X% (threshold: 95%)
- Memory: [saved key learnings to persistent memory]

Closes #<issue> (if applicable)"
```

**Commit Types**: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

**Language Requirement**: Commit messages must be written in English. Never use Portuguese, Spanish, or any other language.

### Step 8: Report to User (Minimal Token Output)

**⚠️ CRITICAL: Minimize report to save tokens**

**✅ MINIMAL REPORT (Preferred - Saves ~500 tokens):**
```
✅ Done. Committed: <hash>
```

**✅ SHORT REPORT (If needed - Only ~100 tokens):**
```
✅ Implementation complete
- Files changed: X
- Tests: pass
- Coverage: X%
- Committed
```

**❌ NEVER OUTPUT LONG REPORTS (Wastes tokens):**
- ❌ Don't create "Implementation Complete" sections
- ❌ Don't list all quality checks with emojis
- ❌ Don't show commit messages verbatim
- ❌ Don't add "Next Steps" sections
- ❌ Don't create status tables or boxes

**Why**: Long reports add 500-1000+ tokens per task. For Haiku (limited context), use minimal format.

## Automation Exceptions

Skip steps ONLY when:

1. **Exploratory Code**: User says "experimental", "draft", "try"
   - Still run quality checks
   - Don't commit

2. **User Explicitly Requests**: User says "skip tests", "no commit"
   - Only skip requested step
   - Warn about skipped steps

3. **Emergency Hotfix**: Critical production bug
   - Run minimal checks
   - Document technical debt

**In ALL other cases: Execute complete workflow**

## Error Recovery

If workflow fails 3+ times:

```bash
1. Create backup branch
2. Reset to last stable commit
3. Report to user with error details
4. Request guidance or try alternative approach
```

## Best Practices

### DO's ✅
- ALWAYS run complete workflow
- ALWAYS update OpenSpec and documentation
- ALWAYS use conventional commits
- ALWAYS report summary to user
- ASK before skipping steps

### DON'Ts ❌
- NEVER skip quality checks without permission
- NEVER commit failing tests
- NEVER commit linting errors
- NEVER skip documentation updates
- NEVER assume user wants to skip automation
- NEVER commit debug code or secrets

## Summary

**Complete workflow after EVERY implementation:**

1. ✅ Quality checks (type, lint, format, test, coverage)
2. ✅ Capture to persistent memory (save learnings and decisions)
3. ✅ Security audit
4. ✅ Update rulebook tasks
5. ✅ Update OpenSpec tasks (if applicable)
6. ✅ Update documentation
7. ✅ Git commit (conventional format)
8. ✅ Report summary to user

**Only skip with explicit user permission and document why.**

## Persistent Memory Best Practices

### What to Capture
- **Design decisions**: Why a particular approach was chosen
- **Patterns**: Reusable solutions discovered during implementation
- **Gotchas**: Edge cases, limitations, or surprising behaviors
- **Performance insights**: Optimization lessons or bottleneck discoveries
- **Bug fixes**: Root cause and resolution for future reference

### How to Capture
```bash
# Via CLI
rulebook memory save "<detailed content>" --type <type> --title "Short title" --tags tag1,tag2

# Memory types: bugfix, feature, refactor, decision, discovery, change, observation

# Example: Feature capture
rulebook memory save "Implemented async batch processing for large datasets. Pattern: use queue with worker threads (maxWorkers=4). Gotcha: Queue memory grows unbounded - added max size limit with drop strategy. Tests: Added batch-size edge cases." --type feature --title "Async batch processing implementation" --tags performance,queues,patterns
```

### Search Previous Context
Before implementing similar features, search memory for past learnings:
```bash
rulebook memory search "batch processing" --mode hybrid
```

This surfaces past decisions and patterns to avoid redundant work and preserve institutional knowledge.

<!-- AGENT_AUTOMATION:END -->