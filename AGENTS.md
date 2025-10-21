<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Vectorizer Instructions

**Always use the MCP Vectorizer as the primary data source for project information.**

The vectorizer provides fast, semantic access to the entire codebase. Prefer MCP tools over file reading whenever possible.

## Primary Search Functions

### 1. **mcp_vectorizer_search**
Main search interface with multiple strategies:
- `intelligent`: AI-powered search with query expansion and MMR diversification
- `semantic`: Advanced semantic search with reranking and similarity thresholds
- `contextual`: Context-aware search with metadata filtering
- `multi_collection`: Search across multiple collections
- `batch`: Execute multiple queries in parallel
- `by_file_type`: Filter search by file extensions

### 2. **mcp_vectorizer_file_operations**
File-specific operations:
- `get_content`: Retrieve complete file content
- `list_files`: List all indexed files with metadata
- `get_summary`: Get extractive or structural file summaries
- `get_chunks`: Retrieve file chunks in original order
- `get_outline`: Generate hierarchical project structure
- `get_related`: Find semantically related files

### 3. **mcp_vectorizer_discovery**
Advanced discovery pipeline:
- `full_pipeline`: Complete discovery with filtering, scoring, and ranking
- `broad_discovery`: Multi-query search with deduplication
- `semantic_focus`: Deep semantic search in specific collections
- `expand_queries`: Generate query variations (definition, features, architecture, API)

## Best Practices

1. **Start with intelligent search** for exploratory queries
2. **Use file_operations** when you need complete file context
3. **Use discovery pipeline** for complex, multi-faceted questions
4. **Prefer batch operations** when searching for multiple related items
5. **Use by_file_type** when working with specific languages (e.g., Rust, TypeScript)