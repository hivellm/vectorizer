# Add GraphQL API Support - Proposal

## Why

Currently, Vectorizer provides a comprehensive REST API for all operations, but lacks GraphQL support. GraphQL offers several advantages over REST: clients can request exactly the data they need in a single query, reducing over-fetching and under-fetching; it provides a strongly-typed schema that enables better tooling and client code generation; and it allows for more efficient data fetching with nested queries. Many modern applications and tools expect GraphQL support, and adding it will make Vectorizer more accessible to developers using GraphQL-based frameworks and tools. The route `/graphql` is already mentioned in the codebase but not implemented, indicating this was planned but not yet completed.

## What Changes

This task implements a complete GraphQL API layer for Vectorizer that mirrors the functionality of the existing REST API:

1. **GraphQL Schema Definition**:
   - Define GraphQL schema covering all major operations (collections, vectors, search, graph operations)
   - Create type definitions for all entities (Collection, Vector, Node, Edge, etc.)
   - Define Query and Mutation operations

2. **GraphQL Server Integration**:
   - Integrate async-graphql library (or similar Rust GraphQL implementation)
   - Add `/graphql` endpoint to the existing Axum router
   - Implement GraphQL handler that processes queries and mutations
   - Add GraphQL playground/IDE endpoint for development and testing

3. **Query Operations**:
   - Implement queries for collections (list, get by name)
   - Implement queries for vectors (get by ID, search, scroll)
   - Implement queries for graph operations (nodes, edges, neighbors, paths)
   - Implement queries for workspace and configuration

4. **Mutation Operations**:
   - Implement mutations for collection management (create, update, delete)
   - Implement mutations for vector operations (upsert, delete, update payload)
   - Implement mutations for graph operations (create/delete edges)
   - Implement mutations for workspace operations

5. **Integration**:
   - Reuse existing business logic from REST handlers
   - Share state (VectorStore, EmbeddingManager) between REST and GraphQL
   - Ensure GraphQL and REST have feature parity
   - Add proper error handling and validation

6. **Documentation and Testing**:
   - Add GraphQL schema documentation
   - Create example queries and mutations
   - Add integration tests for GraphQL endpoints
   - Update API documentation

## Impact

- **Affected specs**: 
  - `specs/api/spec.md` - Add GraphQL API requirements
- **Affected code**: 
  - **NEW**: `src/api/graphql/` - GraphQL schema, resolvers, and handlers
  - **MODIFIED**: `src/server/mod.rs` - Add GraphQL route to router
  - **MODIFIED**: `Cargo.toml` - Add GraphQL dependencies (async-graphql or similar)
- **Breaking change**: NO (additive feature, REST API remains unchanged)
- **User benefit**: 
  - More flexible API access with GraphQL queries
  - Better developer experience with GraphQL tooling
  - Reduced network overhead with precise data fetching
  - Compatibility with GraphQL-based tools and frameworks
  - Single endpoint for all operations instead of multiple REST endpoints
