## 1. REST API Endpoints
- [x] 1.1 Implement POST `/api/v1/graph/discover/{collection}` endpoint
- [x] 1.2 Implement POST `/api/v1/graph/discover/{collection}/{node_id}` endpoint
- [x] 1.3 Implement GET `/api/v1/graph/discover/{collection}/status` endpoint
- [x] 1.4 Add request/response types for discovery endpoints
- [x] 1.5 Add error handling for discovery operations

## 2. MCP Tools
- [x] 2.1 Add `graph_discover_edges` MCP tool
- [x] 2.2 Add `graph_discover_status` MCP tool
- [x] 2.3 Implement handlers for discovery tools
- [x] 2.4 Add tool definitions to mcp_tools.rs

## 3. Discovery Implementation
- [x] 3.1 Expose `discover_similarity_relationships` function publicly
- [x] 3.2 Add batch discovery function for entire collection
- [x] 3.3 Add progress tracking for batch operations
- [x] 3.4 Add configuration options (similarity threshold, max_per_node)

## 4. Testing
- [x] 4.1 Add unit tests for discovery functions
- [x] 4.2 Add integration tests for REST endpoints
- [x] 4.3 Add integration tests for MCP tools
- [x] 4.4 Test with large collections (performance)

## 5. Documentation
- [x] 5.1 Update API documentation with new endpoints
- [x] 5.2 Add examples for discovery usage
- [x] 5.3 Update CHANGELOG.md

## 6. SDK Implementation Phase
- [x] 6.1 Add discovery models to Rust SDK (`sdks/rust/src/models/mod.rs`)
- [x] 6.2 Implement `discover_edges` method in Rust SDK client (collection-level)
- [x] 6.3 Implement `discover_edges` method in Rust SDK client (node-level)
- [x] 6.4 Implement `get_discovery_status` method in Rust SDK client
- [x] 6.5 Add discovery models to Python SDK (`sdks/python/models.py`)
- [x] 6.6 Implement discovery methods in Python SDK client
- [x] 6.7 Add discovery models to TypeScript SDK (`sdks/typescript/src/models/`)
- [x] 6.8 Implement discovery methods in TypeScript SDK client
- [x] 6.9 Add discovery models to JavaScript SDK (`sdks/javascript/src/models/`)
- [x] 6.10 Implement discovery methods in JavaScript SDK client
- [x] 6.11 Add SDK tests for discovery operations (Rust, Python, TypeScript, JavaScript)
- [x] 6.12 Update SDK documentation with discovery usage examples
