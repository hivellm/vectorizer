## 1. Planning & Design Phase
- [x] 1.1 Research lightweight graph implementations in Rust (petgraph, graphlib, etc.)
- [x] 1.2 Design graph data structure (nodes, edges, relationship types)
- [x] 1.3 Design automatic relationship discovery algorithm
- [ ] 1.4 Design graph persistence format (if needed)
- [x] 1.5 Create technical specification document

## 2. Core Graph Implementation Phase
- [x] 2.1 Create `src/db/graph.rs` module with basic graph structure
- [x] 2.2 Implement Node struct (id, type, metadata)
- [x] 2.3 Implement Edge struct (source, target, relationship_type, weight, metadata)
- [x] 2.4 Implement RelationshipType enum (SIMILAR_TO, REFERENCES, CONTAINS, DERIVED_FROM)
- [x] 2.5 Implement Graph struct with adjacency list or edge list storage
- [x] 2.6 Implement graph operations: add_node, add_edge, remove_node, remove_edge
- [x] 2.7 Implement graph queries: get_neighbors, find_path, get_connected_components
- [ ] 2.8 Add graph persistence (optional, to disk or WAL)

## 3. Automatic Relationship Discovery Phase
- [x] 3.1 Integrate graph relationship creation into `Collection::insert_vectors`
- [x] 3.2 Implement automatic SIMILAR_TO relationship creation based on similarity threshold
- [x] 3.3 Implement relationship discovery from payload metadata (REFERENCES, CONTAINS)
- [x] 3.4 Add configurable relationship creation rules in CollectionConfig
- [x] 3.5 Implement relationship weight calculation based on similarity score

## 4. Graph Models Phase
- [x] 4.1 Add graph models to `src/models/mod.rs`
- [x] 4.2 Implement Node model with serialization
- [x] 4.3 Implement Edge model with serialization
- [x] 4.4 Implement RelationshipType enum with serialization
- [x] 4.5 Implement GraphQuery models (FindRelatedRequest, FindPathRequest, etc.)

## 5. REST API Phase
- [x] 5.1 Create `src/api/graph.rs` module for graph REST endpoints
- [x] 5.2 Implement GET `/api/v1/graph/nodes/:collection` - List all nodes in collection
- [x] 5.3 Implement GET `/api/v1/graph/nodes/:collection/:node_id/neighbors` - Get neighbors
- [x] 5.4 Implement POST `/api/v1/graph/nodes/:collection/:node_id/related` - Find related nodes
- [x] 5.5 Implement POST `/api/v1/graph/path` - Find shortest path between nodes
- [x] 5.6 Implement POST `/api/v1/graph/edges` - Create explicit edge
- [x] 5.7 Implement DELETE `/api/v1/graph/edges/:edge_id` - Delete edge
- [x] 5.8 Integrate graph routes into main REST router

## 6. MCP Tools Phase
- [x] 6.1 Add graph MCP tools to `src/mcp/tools.rs`
- [x] 6.2 Implement `graph_find_related` tool - Find related documents/files
- [x] 6.3 Implement `graph_find_path` tool - Find path between two nodes
- [x] 6.4 Implement `graph_get_neighbors` tool - Get neighbors of a node
- [ ] 6.5 Implement `graph_create_relationship` tool - Create explicit relationship
- [x] 6.6 Register graph tools in MCP server

## 7. Testing Phase
- [x] 7.1 Write unit tests for graph structure (add_node, add_edge, remove_node)
- [x] 7.2 Write unit tests for graph queries (get_neighbors, find_path)
- [x] 7.3 Write integration tests for automatic relationship creation
- [ ] 7.4 Write integration tests for graph REST endpoints
- [ ] 7.5 Write integration tests for graph MCP tools
- [ ] 7.6 Test graph persistence and recovery
- [ ] 7.7 Verify test coverage meets 95% threshold

## 8. Documentation Phase
- [ ] 8.1 Update README.md with graph functionality overview
- [ ] 8.2 Update CHANGELOG.md with graph features
- [ ] 8.3 Add graph API documentation to `docs/api/`
- [ ] 8.4 Add graph usage examples
- [x] 8.5 Update code documentation with module and function docs

## 9. Integration & Cleanup Phase
- [x] 9.1 Verify graph integration with existing vector operations
- [ ] 9.2 Test graph performance with large numbers of nodes and edges
- [ ] 9.3 Remove debug code and temporary files
- [ ] 9.4 Run linter and fix warnings
- [ ] 9.5 Run type checker and fix errors
- [ ] 9.6 Final code review

