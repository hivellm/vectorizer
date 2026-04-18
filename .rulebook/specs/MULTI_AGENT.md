<!-- MULTI_AGENT:START -->
# Multi-Agent Development Directives

## Overview

When `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` is enabled, Claude Code can spawn specialized sub-agents that work in parallel on complex tasks. This template defines coordination rules and team patterns for effective multi-agent development.

## When to Use Teams

Use agent teams when:
- Tasks have clear parallel work streams (e.g., frontend + backend simultaneously)
- Research, implementation, and testing can run concurrently
- Multiple independent modules need changes at the same time
- Complex refactoring spans many files across different subsystems

Do NOT use teams when:
- The task is simple and sequential
- Changes are tightly coupled and require serial execution
- The task can be completed in under 5 minutes by a single agent

## Team Structure Patterns

### Standard Team (Recommended)

| Agent | Role | Responsibilities |
|-------|------|-----------------|
| **team-lead** | Orchestrator | Breaks down tasks, assigns work, integrates results |
| **researcher** | Information Gatherer | Reads code, searches docs, analyzes patterns |
| **implementer** | Code Writer | Writes production code following established patterns |
| **tester** | Quality Assurance | Writes tests, validates coverage, runs quality checks |

### Minimal Team (Simple Tasks)

| Agent | Role |
|-------|------|
| **team-lead** | Orchestrator + integrator |
| **implementer** | Code writer + researcher |

## Coordination Rules

1. **Team lead assigns tasks explicitly** -- agents do not self-assign work
2. **Agents report completion via SendMessage** -- not by writing to shared files
3. **No two agents modify the same file simultaneously** -- team lead coordinates file ownership
4. **Use git-friendly workflows** -- each agent works on distinct files to avoid merge conflicts
5. **Quality gates are mandatory** -- tester agent validates all changes before completion

## Communication Protocol

- **Task Assignment**: Team lead sends specific, scoped instructions to each agent
- **Progress Updates**: Agents report status via messages, not file-based communication
- **Blocking Issues**: Agents immediately notify team lead of blockers
- **Completion**: Agents send explicit "done" messages with summary of changes

## File Ownership

To prevent conflicts, the team lead must:
1. Identify all files that need modification
2. Assign exclusive ownership of each file to one agent
3. If multiple agents need the same file, serialize their access

## Agent Definitions

Pre-configured agent definitions are available in `.claude/agents/`. These provide specialized instructions for each agent role, ensuring consistent behavior and expertise.

## Quality Enforcement

All agent teams must adhere to the project's quality gates:
- Type checking must pass
- Linter must pass with zero warnings
- All tests must pass
- Coverage thresholds must be met

The tester agent is responsible for running these checks before the team lead marks a task as complete.
<!-- MULTI_AGENT:END -->