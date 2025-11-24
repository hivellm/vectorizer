# Add Graph Edge Discovery Endpoints - Proposal

## Why

Graph nodes are automatically created when vectors are inserted, but SIMILAR_TO edges are skipped during insertion to avoid timeout. While edges based on metadata (REFERENCES, CONTAINS, DERIVED_FROM) are created automatically, similarity-based edges require manual creation via `graph_create_edge`. This limits the usefulness of automatic relationship discovery. Adding endpoints and MCP tools to discover and create SIMILAR_TO edges will enable users to populate the graph with similarity relationships after insertion, either for specific nodes or for entire collections.

## What Changes

This task adds endpoints and tools to discover and create graph edges:

1. **REST API Endpoints**:
   - `POST /api/v1/graph/discover/{collection}` - Discover and create SIMILAR_TO edges for all nodes in a collection
   - `POST /api/v1/graph/discover/{collection}/{node_id}` - Discover and create SIMILAR_TO edges for a specific node
   - `GET /api/v1/graph/discover/{collection}/status` - Get discovery status/progress

2. **MCP Tools**:
   - `graph_discover_edges` - Discover and create SIMILAR_TO edges for a collection or specific node
   - `graph_discover_status` - Get discovery status

3. **Background Processing**:
   - Optional background job support for large collections
   - Progress tracking for batch discovery operations

## Impact

- **Affected specs**: 
  - `specs/db/spec.md` - Update graph discovery requirements
  - `specs/api-rest/spec.md` - Add graph discovery endpoints
- **Affected code**: 
  - `src/api/graph.rs` - Add discovery endpoints
  - `src/server/graph_handlers.rs` - Add MCP discovery handlers
  - `src/server/mcp_tools.rs` - Add discovery tool definitions
  - `src/db/graph_relationship_discovery.rs` - Expose discovery functions
- **Breaking change**: NO (additive only)
- **User benefit**: 
  - Ability to populate graph with similarity relationships after insertion
  - Batch discovery for entire collections
  - Better graph relationship coverage
