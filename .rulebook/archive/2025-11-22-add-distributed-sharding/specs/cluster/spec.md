# Distributed Horizontal Sharding Specification

## Purpose

This specification defines the requirements for distributed horizontal sharding that enables Vectorizer to operate as a cluster of multiple server instances, with collections and vectors distributed across them for horizontal scalability, load distribution, and fault tolerance.

## ADDED Requirements

### Requirement: Cluster Management
The system SHALL maintain a cluster of multiple server instances with automatic membership management, server discovery, and cluster state coordination.

#### Scenario: Server joins cluster
Given a new server instance starts with cluster configuration
When the server connects to the cluster
Then it registers itself with other cluster members and receives cluster state

#### Scenario: Server leaves cluster
Given a server instance is shutting down or fails
When the server becomes unavailable
Then other cluster members detect the failure and update cluster state, redistributing shards if needed

#### Scenario: Cluster state synchronization
Given multiple servers are part of a cluster
When cluster state changes (server added/removed, shard moved)
Then all servers receive updated cluster state within a configurable timeout

#### Scenario: Static cluster configuration
Given a cluster configuration file with server addresses
When servers start
Then they connect to the specified servers and form a cluster

### Requirement: Distributed Shard Routing
The system SHALL route operations to the correct server instance based on shard location, using consistent hashing to determine shard-to-server assignment.

#### Scenario: Route insert to correct server
Given a vector is inserted into a distributed sharded collection
When the shard for the vector is determined
Then the insert operation is routed to the server that owns that shard

#### Scenario: Route search across multiple servers
Given a search operation is performed on a distributed sharded collection
When the search requires querying multiple shards
Then the search is executed on all relevant servers and results are merged

#### Scenario: Shard assignment with consistent hashing
Given a cluster with multiple servers
When shards are assigned to servers
Then shard-to-server mapping uses consistent hashing to ensure even distribution and minimal rebalancing when servers are added/removed

#### Scenario: Handle server failure during operation
Given an operation is routed to a server
When that server is unavailable
Then the operation fails gracefully with appropriate error, and cluster state is updated

### Requirement: Cross-Server Communication
The system SHALL provide gRPC-based communication between servers for distributed operations, with connection pooling, retries, and error handling.

#### Scenario: Remote vector insert
Given a server needs to insert a vector on another server
When the insert operation is initiated
Then a gRPC call is made to the target server with the vector data, and the result is returned

#### Scenario: Remote vector search
Given a server needs to search vectors on another server
When the search operation is initiated
Then a gRPC call is made with search parameters, and search results are returned

#### Scenario: Connection pooling
Given multiple operations target the same remote server
When operations are executed
Then connections are pooled and reused for efficiency

#### Scenario: Retry on transient failure
Given a remote operation fails with a transient error
When retry is configured
Then the operation is retried up to the configured number of times with exponential backoff

#### Scenario: Timeout handling
Given a remote operation takes too long
When the operation exceeds the configured timeout
Then the operation is cancelled and returns a timeout error

### Requirement: Shard Placement and Load Balancing
The system SHALL automatically assign shards to servers with load balancing, and support manual shard placement and rebalancing.

#### Scenario: Automatic shard placement
Given a cluster with multiple servers
When a new sharded collection is created
Then shards are automatically assigned to servers using consistent hashing, ensuring even distribution

#### Scenario: Load-based rebalancing
Given shards are unevenly distributed across servers
When rebalancing is triggered
Then shards are moved from overloaded servers to underloaded servers to balance load

#### Scenario: Manual shard placement
Given a cluster administrator wants to control shard placement
When shard placement is configured manually
Then shards are placed on the specified servers as configured

#### Scenario: Shard migration
Given a shard needs to be moved to a different server
When shard migration is initiated
Then vectors in the shard are transferred to the target server, and routing is updated

### Requirement: Distributed Collection Operations
The system SHALL support all collection operations (insert, search, update, delete) across distributed shards, with proper result merging and error handling.

#### Scenario: Distributed insert
Given a vector is inserted into a distributed sharded collection
When the insert operation is performed
Then the vector is routed to the correct server based on shard assignment, and the operation succeeds or fails based on that server's response

#### Scenario: Distributed search with result merging
Given a search is performed on a distributed sharded collection
When the search queries multiple shards on different servers
Then searches are executed in parallel on all relevant servers, results are merged and sorted by score, and the top K results are returned

#### Scenario: Distributed update
Given a vector is updated in a distributed sharded collection
When the update operation is performed
Then the update is routed to the correct server, and the operation succeeds or fails based on that server's response

#### Scenario: Distributed delete
Given a vector is deleted from a distributed sharded collection
When the delete operation is performed
Then the delete is routed to the correct server, and the operation succeeds or fails based on that server's response

#### Scenario: Handle partial failures in distributed search
Given a distributed search queries multiple servers
When some servers fail during the search
Then results from available servers are returned, and the response indicates which servers failed

### Requirement: Cluster Configuration
The system SHALL support configuration of cluster topology, server addresses, discovery method, and shard distribution settings.

#### Scenario: Configure cluster with server list
Given a cluster configuration file
When servers start
Then they use the configured server addresses to join the cluster

#### Scenario: Configure cluster discovery method
Given cluster configuration supports multiple discovery methods
When a discovery method is configured (static, DNS, service registry)
Then servers use that method to discover and join the cluster

#### Scenario: Configure shard distribution
Given cluster configuration includes shard distribution settings
When shards are assigned to servers
Then distribution follows the configured strategy (automatic, manual, or custom)

#### Scenario: Configure cluster timeouts and retries
Given cluster configuration includes timeout and retry settings
When cross-server operations are performed
Then operations use the configured timeouts and retry policies

### Requirement: Cluster REST API
The system SHALL expose cluster management functionality through REST endpoints that allow querying cluster state, managing nodes, and controlling shard distribution.

#### Scenario: List cluster nodes
Given a cluster is running
When a GET request is made to `/api/v1/cluster/nodes`
Then the system returns a list of all cluster nodes with their status, addresses, and assigned shards

#### Scenario: Get node information
Given a cluster node exists
When a GET request is made to `/api/v1/cluster/nodes/:node_id`
Then the system returns detailed information about that node including status, shards, and metrics

#### Scenario: Get shard distribution
Given a cluster has shards distributed across servers
When a GET request is made to `/api/v1/cluster/shard-distribution`
Then the system returns the mapping of shards to servers with load information

#### Scenario: Trigger shard rebalancing
Given a cluster has uneven shard distribution
When a POST request is made to `/api/v1/cluster/rebalance`
Then the system initiates shard rebalancing to distribute load evenly

#### Scenario: Add node to cluster
Given a new server is available
When a POST request is made to `/api/v1/cluster/nodes` with server information
Then the server is added to the cluster and shards may be redistributed

#### Scenario: Remove node from cluster
Given a server is part of the cluster
When a DELETE request is made to `/api/v1/cluster/nodes/:node_id`
Then the server is removed from the cluster and its shards are redistributed to other servers

### Requirement: Cluster MCP Tools
The system SHALL provide MCP tools for cluster operations that enable AI assistants to query cluster state, manage nodes, and control shard distribution.

#### Scenario: Cluster list nodes tool
Given a cluster is running
When the `cluster_list_nodes` MCP tool is called
Then the tool returns a list of all cluster nodes with their status and information

#### Scenario: Cluster get shard distribution tool
Given a cluster has shards distributed across servers
When the `cluster_get_shard_distribution` MCP tool is called
Then the tool returns the shard-to-server mapping with load information

#### Scenario: Cluster rebalance tool
Given a cluster has uneven shard distribution
When the `cluster_rebalance` MCP tool is called
Then the tool triggers shard rebalancing and returns the rebalancing status

#### Scenario: Cluster add node tool
Given a new server is available
When the `cluster_add_node` MCP tool is called with server information
Then the tool adds the server to the cluster and returns the updated cluster state

#### Scenario: Cluster remove node tool
Given a server is part of the cluster
When the `cluster_remove_node` MCP tool is called with a node ID
Then the tool removes the server from the cluster and returns the updated cluster state

### Requirement: Fault Tolerance
The system SHALL handle server failures gracefully, redistributing shards when servers become unavailable, and continuing operations with available servers.

#### Scenario: Server failure detection
Given a server in the cluster becomes unavailable
When other servers attempt to communicate with it
Then the failure is detected within a configurable timeout, and cluster state is updated

#### Scenario: Shard redistribution on server failure
Given a server fails and has assigned shards
When the failure is detected
Then shards from the failed server are redistributed to other available servers

#### Scenario: Continue operations with available servers
Given some servers in the cluster are unavailable
When operations are performed
Then operations succeed on available servers, and errors are returned only for operations targeting unavailable servers

#### Scenario: Server recovery
Given a failed server recovers and rejoins the cluster
When the server rejoins
Then it receives updated cluster state, and shards may be reassigned based on current cluster state

### Requirement: Integration with Existing Sharding
The system SHALL integrate with the existing `ShardedCollection` implementation, extending it to support distributed mode while maintaining backward compatibility with single-server sharding.

#### Scenario: Single-server sharding mode
Given a collection is configured with sharding but no cluster
When the collection is created
Then it uses the existing single-server sharding implementation

#### Scenario: Distributed sharding mode
Given a collection is configured with sharding and cluster is enabled
When the collection is created
Then it uses distributed sharding with shards distributed across cluster servers

#### Scenario: Backward compatibility
Given existing code uses `ShardedCollection` without cluster
When the code runs
Then it continues to work with single-server sharding without changes

