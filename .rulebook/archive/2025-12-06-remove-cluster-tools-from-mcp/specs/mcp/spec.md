# Remove Cluster Tools from MCP Specification (Vectorizer)

## REMOVED Requirements

### Requirement: Cluster Management Tools in MCP
The system SHALL NOT expose cluster management operations via MCP interface.

#### Scenario: Cluster tools removed from MCP
Given the MCP server is started
When listing available MCP tools
Then the system SHALL NOT include cluster_list_nodes
And the system SHALL NOT include cluster_get_shard_distribution
And the system SHALL NOT include cluster_rebalance
And the system SHALL NOT include cluster_add_node
And the system SHALL NOT include cluster_remove_node
And the system SHALL NOT include cluster_get_node_info

#### Scenario: Attempting to call removed cluster tool
Given a client calls a removed cluster tool via MCP
When the tool is invoked
Then the system SHALL return an error indicating the tool does not exist
And the system SHALL return appropriate error message

### Requirement: Cluster Operations Remain in REST API
The system SHALL maintain cluster management operations in REST API only.

#### Scenario: Cluster operations via REST API
Given cluster management is needed
When using REST API endpoints
Then the system SHALL provide GET /api/v1/cluster/nodes
And the system SHALL provide GET /api/v1/cluster/nodes/:node_id
And the system SHALL provide GET /api/v1/cluster/shard-distribution
And the system SHALL provide POST /api/v1/cluster/rebalance
And the system SHALL provide POST /api/v1/cluster/nodes
And the system SHALL provide DELETE /api/v1/cluster/nodes/:node_id

#### Scenario: REST API authentication
Given cluster management endpoints are accessed
When a request is made
Then the system SHALL require proper authentication
And the system SHALL enforce authorization for administrative operations

### Requirement: MCP Tool Count Update
The system SHALL reflect the correct number of MCP tools after removal.

#### Scenario: Tool count in discovery service
Given the discovery service is queried
When tool count is requested
Then the system SHALL report 26 tools (down from 32)
And the system SHALL NOT include cluster tools in the count

#### Scenario: Tool list accuracy
Given MCP tools are listed
When the list is retrieved
Then the system SHALL include only data operation tools
And the system SHALL NOT include administrative cluster tools

## MODIFIED Requirements

### Requirement: MCP Interface Scope
The MCP interface SHALL focus on end-user data operations, not administrative tasks.

#### Scenario: MCP tools are data-focused
Given the MCP interface is used
When tools are listed
Then the system SHALL include collection operations
And the system SHALL include vector operations
And the system SHALL include search operations
And the system SHALL include file operations
And the system SHALL include graph operations
And the system SHALL NOT include cluster management operations

