<!-- TOKEN_OPTIMIZATION:START -->
# Token Optimization Rules

Output verbosity rules calibrated by model capability tier.
These rules ensure cost-efficient use of AI models without sacrificing quality.

## Model Tier Assignment

| Tier | Use For | Claude | Gemini | OpenAI |
|------|---------|--------|--------|--------|
| **Core** | Complex bugs, architecture, domain-critical code | opus | Pro | o3 |
| **Standard** | Tests, implementation, build system, medium tasks | sonnet | Flash | 4o |
| **Research** | Read-only exploration, docs, codebase search | haiku | Flash-Lite | 4o-mini |

## Output Rules by Tier

### Research Tier (cheapest — maximize context for work, not reports)
- Output code, not explanations
- Minimal reports: "Done" instead of detailed status
- No markdown tables, emoji, or status sections
- Combine outputs into one response
- No "Next Steps" sections
- No repeating back what was asked

### Standard Tier (balanced — brief summaries)
- Brief summaries (2-3 sentences max)
- Code with inline comments (no separate explanation blocks)
- Report only: what changed, what passed, what failed
- Skip preamble and transitions

### Core Tier (most capable — full reasoning when needed)
- Full explanations welcome for complex decisions
- Document reasoning for non-obvious choices
- Detailed analysis for bug investigations
- Still avoid unnecessary verbosity

## Token Savings Reference

| Pattern to Avoid | Tokens Wasted | Alternative |
|-------------------|---------------|-------------|
| Emoji status tables | ~500/task | Plain text "Done" |
| "Next Steps" sections | ~100/task | Omit entirely |
| Detailed quality reports | ~300/task | "Tests pass, coverage 95%" |
| Repeating the question | ~200/task | Jump to the answer |
| Markdown formatting abuse | ~200/task | Minimal formatting |

**Total savings**: ~850 tokens/task for research tier agents

<!-- TOKEN_OPTIMIZATION:END -->