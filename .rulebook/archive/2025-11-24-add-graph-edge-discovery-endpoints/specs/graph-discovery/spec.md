# Graph Edge Discovery Specification

## ADDED Requirements

### Requirement: Graph Edge Discovery REST Endpoints
The system SHALL provide REST API endpoints to discover and create SIMILAR_TO edges for graph nodes.

#### Scenario: Discover Edges for Collection
Given a collection with graph enabled
When a POST request is made to `/api/v1/graph/discover/{collection}`
Then the system MUST discover SIMILAR_TO edges for all nodes in the collection
And the system MUST create edges based on similarity threshold
And the system MUST return discovery results with count of edges created

#### Scenario: Discover Edges for Specific Node
Given a collection with graph enabled and a specific node
When a POST request is made to `/api/v1/graph/discover/{collection}/{node_id}`
Then the system MUST discover SIMILAR_TO edges for the specified node
And the system MUST create edges to similar nodes based on similarity threshold
And the system MUST return discovery results with count of edges created

#### Scenario: Get Discovery Status
Given a collection with graph enabled
When a GET request is made to `/api/v1/graph/discover/{collection}/status`
Then the system MUST return discovery status including:
- Total nodes in collection
- Nodes with edges discovered
- Total edges created
- Discovery progress percentage

### Requirement: Graph Edge Discovery MCP Tools
The system SHALL provide MCP tools to discover and create SIMILAR_TO edges.

#### Scenario: Discover Edges via MCP
Given a collection with graph enabled
When the `graph_discover_edges` tool is called
Then the system MUST discover SIMILAR_TO edges for specified nodes or entire collection
And the system MUST create edges based on similarity threshold
And the system MUST return discovery results

#### Scenario: Get Discovery Status via MCP
Given a collection with graph enabled
When the `graph_discover_status` tool is called
Then the system MUST return discovery status information

### Requirement: Discovery Configuration
The system SHALL support configurable discovery parameters.

#### Scenario: Configure Discovery Threshold
Given a discovery request
When similarity_threshold parameter is provided
Then the system MUST use the specified threshold instead of default
And the system MUST only create edges above the threshold

#### Scenario: Configure Max Edges Per Node
Given a discovery request
When max_per_node parameter is provided
Then the system MUST limit edges created per node to the specified value
And the system MUST prioritize edges with highest similarity scores

## MODIFIED Requirements

### Requirement: Graph Relationship Discovery
The graph relationship discovery functionality SHALL be exposed via REST and MCP APIs.

#### Scenario: Expose Discovery Function
Given the existing `discover_similarity_relationships` function
When discovery endpoints are called
Then the system MUST use the existing discovery logic
And the system MUST expose it via REST and MCP interfaces

## Quality Requirements

### Performance
- Discovery for single node MUST complete within 5 seconds
- Discovery for entire collection SHOULD support background processing for large collections
- Discovery MUST respect similarity threshold to avoid creating too many edges

### Configuration
- Default similarity threshold: 0.7
- Default max edges per node: 10
- Configuration MUST be overridable per request

### Testing
- Test coverage MUST be 95%+ for new code
- Integration tests MUST verify edge creation
- Performance tests MUST verify discovery completes within acceptable time

