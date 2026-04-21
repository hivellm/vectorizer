## 1. Dependencies and Setup
- [x] 1.1 Research and select GraphQL library for Rust (async-graphql recommended)
- [x] 1.2 Add GraphQL dependencies to Cargo.toml (async-graphql, async-graphql-axum)
- [x] 1.3 Create GraphQL module structure (`src/api/graphql/`)
- [x] 1.4 Set up GraphQL schema foundation

## 2. Schema Definition
- [x] 2.1 Define Collection GraphQL type (id, name, config, stats, etc.)
- [x] 2.2 Define Vector GraphQL type (id, vector, payload, metadata)
- [x] 2.3 Define Graph Node GraphQL type (id, node_type, metadata)
- [x] 2.4 Define Graph Edge GraphQL type (id, source, target, relationship_type, weight)
- [x] 2.5 Define SearchResult GraphQL type (vector, score, payload)
- [x] 2.6 Define input types for mutations (CreateCollectionInput, UpsertVectorInput, etc.)
- [x] 2.7 Define filter and pagination input types
- [x] 2.8 Define Workspace types (GqlWorkspace, GqlWorkspaceConfig, AddWorkspaceInput)

## 3. Query Resolvers
- [x] 3.1 Implement collections query (list all collections)
- [x] 3.2 Implement collection query (get collection by name)
- [x] 3.3 Implement vectors query (list vectors in collection with pagination)
- [x] 3.4 Implement vector query (get vector by ID)
- [x] 3.5 Implement search query (semantic search with filters)
- [x] 3.6 Implement scroll query (pagination for large result sets) - merged with vectors query
- [x] 3.7 Implement graph nodes query (list nodes in collection)
- [x] 3.8 Implement graph edges query (list edges in collection)
- [x] 3.9 Implement graph neighbors query (get neighbors of a node)
- [x] 3.10 Implement graph path query (find path between nodes)
- [x] 3.11 Implement workspaces query (list workspaces)
- [x] 3.12 Implement workspaceConfig query (get workspace config)
- [x] 3.13 Implement stats query (server statistics)
- [x] 3.14 Implement graphStats query (graph statistics for a collection)
- [x] 3.15 Implement graphNode query (get a specific node by ID)
- [x] 3.16 Implement graphRelated query (find related nodes within N hops)

## 4. Mutation Resolvers
- [x] 4.1 Implement createCollection mutation
- [ ] 4.2 Implement updateCollection mutation (deferred - not in REST API)
- [x] 4.3 Implement deleteCollection mutation
- [x] 4.4 Implement upsertVector mutation (single vector)
- [x] 4.5 Implement upsertVectors mutation (batch)
- [x] 4.6 Implement deleteVector mutation
- [x] 4.7 Implement updateVectorPayload mutation (updatePayload)
- [x] 4.8 Implement createEdge mutation (graph) - createGraphEdge
- [x] 4.9 Implement deleteEdge mutation (graph) - deleteGraphEdge
- [x] 4.10 Implement enableGraph mutation
- [x] 4.11 Implement addGraphNode mutation
- [x] 4.12 Implement removeGraphNode mutation
- [x] 4.13 Implement addWorkspace mutation
- [x] 4.14 Implement removeWorkspace mutation
- [x] 4.15 Implement updateWorkspaceConfig mutation

## 5. GraphQL Server Integration
- [x] 5.1 Create GraphQL schema root (Query and Mutation objects)
- [x] 5.2 Create GraphQL handler function
- [x] 5.3 Add `/graphql` POST route to Axum router
- [x] 5.4 Add GraphQL state (VectorStore, EmbeddingManager)
- [x] 5.5 Integrate GraphQL handler with existing server state
- [x] 5.6 Add GraphQL playground endpoint (`/graphiql`)
- [x] 5.7 Add CORS support for GraphQL endpoint (via existing middleware)
- [x] 5.8 Add error handling and error formatting

## 6. Business Logic Integration
- [x] 6.1 Reuse collection operations from VectorStore
- [x] 6.2 Reuse vector operations from VectorStore
- [x] 6.3 Reuse search operations from VectorStore
- [x] 6.4 Reuse graph operations from Graph module
- [x] 6.5 Ensure GraphQL and REST share same business logic (via VectorStore)
- [x] 6.6 Add proper error conversion (VectorizerError to GraphQL errors)

## 7. Testing
- [x] 7.1 Write unit tests for GraphQL schema (37 tests in tests.rs)
- [x] 7.2 Write integration tests for GraphQL queries
- [x] 7.3 Write integration tests for GraphQL mutations
- [x] 7.4 Test error handling in GraphQL operations
- [x] 7.5 Test GraphQL playground/IDE endpoint (manual testing)
- [x] 7.6 Test GraphQL with complex nested queries (manual testing)
- [x] 7.7 Test GraphQL pagination and filtering (manual testing)
- [x] 7.8 Verify GraphQL and REST feature parity

## 8. Documentation
- [x] 8.1 Document GraphQL schema in code comments
- [x] 8.2 Create GraphQL API documentation (docs/users/api/GRAPHQL.md)
- [x] 8.3 Add example queries to documentation
- [x] 8.4 Add example mutations to documentation
- [x] 8.5 Update API README with GraphQL endpoint information
- [x] 8.6 Add GraphQL usage examples
- [x] 8.7 Update CHANGELOG.md

## 9. Performance and Optimization
- [x] 9.1 Optimize GraphQL query execution (via direct VectorStore access)
- [x] 9.2 Add query complexity analysis (limit_complexity: 1000)
- [x] 9.3 Add query depth limiting (limit_depth: 10)
- [ ] 9.4 Add query cost analysis (if needed)
- [ ] 9.5 Benchmark GraphQL vs REST performance
- [ ] 9.6 Add query caching if beneficial
