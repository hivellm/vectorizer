# Add Basic Graph Support - Proposal

## Why

Currently, Vectorizer can find semantically related files through vector similarity search, but it lacks a persistent graph structure to represent and navigate relationships between documents and files. This limitation prevents efficient relationship traversal, automatic relationship discovery, and basic graph-based queries without requiring an external graph database or complex query languages like Cypher. A lightweight internal graph implementation will enable automatic relationship tracking between files based on semantic similarity, shared entities, and explicit metadata relationships, making it easier to navigate document relationships and understand file dependencies for use cases like codebase analysis, document management, and knowledge graph construction.

## What Changes

This task introduces a lightweight internal graph database module within Vectorizer that provides:

1. **Graph Storage**: In-memory graph structure with optional persistence for nodes (documents/files) and edges (relationships)
2. **Automatic Relationship Discovery**: Automatically create relationships based on semantic similarity thresholds when vectors are inserted
3. **Simple Graph Queries**: Provide simple graph traversal APIs (e.g., "find all files related to X", "find shortest path between A and B") without requiring Cypher or complex query languages
4. **Relationship Types**: Support basic relationship types (SIMILAR_TO, REFERENCES, CONTAINS, DERIVED_FROM) with configurable weights
5. **Graph Metadata**: Store relationship metadata (strength, creation time, source) for analysis
6. **REST and MCP APIs**: Expose graph functionality through both REST endpoints and MCP tools, following the REST-first architecture
7. **Integration with Vector Store**: Seamlessly integrate with existing vector collections, automatically maintaining graph as vectors are added/updated/deleted

This is simpler than Neo4j integration - no Cypher queries, no external dependencies, focused on basic relationship tracking and traversal for document/file relationships.

## Impact

- **Affected specs**: 
  - `specs/db/spec.md` - Add graph database specification
  - `specs/api-rest/spec.md` - Add graph REST endpoints
  - `specs/search/spec.md` - Extend search to include graph traversal
- **Affected code**: 
  - New module: `src/db/graph.rs` - Core graph implementation
  - New module: `src/api/graph.rs` - Graph REST endpoints
  - Modified: `src/db/collection.rs` - Integrate graph relationship creation on vector insert
  - Modified: `src/mcp/tools.rs` - Add graph MCP tools
  - Modified: `src/models/mod.rs` - Add graph models (Node, Edge, RelationshipType)
- **Breaking change**: NO
- **User benefit**: 
  - Automatic relationship discovery between files/documents
  - Simple graph traversal queries without external dependencies
  - Better understanding of document dependencies and relationships
  - Foundation for advanced features like impact analysis and relationship visualization

