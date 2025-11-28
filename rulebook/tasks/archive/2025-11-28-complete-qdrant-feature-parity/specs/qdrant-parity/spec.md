# Qdrant Feature Parity Specification

## ADDED Requirements

### Requirement: Qdrant gRPC Protocol Support
The system SHALL provide complete Qdrant gRPC API compatibility.

#### Scenario: Qdrant Collections gRPC Service
Given a gRPC client using Qdrant protocol
When the client calls Collections service methods
Then the system MUST respond with Qdrant-compatible gRPC messages
And the system MUST support: List, Get, Create, Update, Delete, UpdateAliases, ListCollectionAliases, ListAliases, CollectionExists, CollectionClusterInfo, UpdateCollectionClusterSetup, CreateShardKey, DeleteShardKey

#### Scenario: Qdrant Points gRPC Service
Given a gRPC client using Qdrant protocol
When the client calls Points service methods
Then the system MUST respond with Qdrant-compatible gRPC messages
And the system MUST support: Upsert, Delete, Get, UpdateVectors, DeleteVectors, SetPayload, OverwritePayload, DeletePayload, ClearPayload, UpdateBatch, Search, SearchBatch, SearchGroups, Scroll, Recommend, RecommendBatch, RecommendGroups, Discover, DiscoverBatch, Count, Query, QueryBatch, QueryGroups, Facet, SearchMatrixPairs, SearchMatrixOffsets

#### Scenario: Qdrant Snapshots gRPC Service
Given a gRPC client using Qdrant protocol
When the client calls Snapshots service methods
Then the system MUST respond with Qdrant-compatible gRPC messages
And the system MUST support: Create, List, Delete, CreateFull, ListFull, DeleteFull

### Requirement: Snapshots via Qdrant API
The system SHALL expose snapshot operations through Qdrant-compatible REST endpoints.

**Note**: Snapshot functionality already exists in Vectorizer but is not exposed via Qdrant API.

#### Scenario: List Collection Snapshots
Given a Qdrant API request GET `/qdrant/collections/{name}/snapshots`
When the request is received
Then the system MUST return list of snapshots in Qdrant format

#### Scenario: Create Collection Snapshot
Given a Qdrant API request POST `/qdrant/collections/{name}/snapshots`
When the request is received
Then the system MUST create collection snapshot
And the system MUST return snapshot information in Qdrant format

#### Scenario: Delete Collection Snapshot
Given a Qdrant API request DELETE `/qdrant/collections/{name}/snapshots/{snapshot_name}`
When the request is received
Then the system MUST delete the snapshot
And the system MUST return success in Qdrant format

#### Scenario: List All Snapshots
Given a Qdrant API request GET `/qdrant/snapshots`
When the request is received
Then the system MUST return all snapshots in Qdrant format

#### Scenario: Create Full Snapshot
Given a Qdrant API request POST `/qdrant/snapshots`
When the request is received
Then the system MUST create full snapshot
And the system MUST return snapshot information in Qdrant format

#### Scenario: Upload Snapshot
Given a Qdrant API request POST `/qdrant/collections/{name}/snapshots/upload`
When the request is received with snapshot file
Then the system MUST save the uploaded snapshot
And the system MUST return success in Qdrant format

#### Scenario: Recover from Snapshot
Given a Qdrant API request POST `/qdrant/collections/{name}/snapshots/recover`
When the request is received with snapshot name
Then the system MUST restore collection from snapshot
And the system MUST return success in Qdrant format

### Requirement: Sharding via Qdrant API
The system SHALL expose sharding operations through Qdrant-compatible REST endpoints.

**Note**: Sharding implementation already exists in Vectorizer but is not exposed via Qdrant API.

#### Scenario: Create Shard Key
Given a Qdrant API request PUT `/qdrant/collections/{name}/shards`
When the request specifies shard key configuration
Then the system MUST create shard key
And the system MUST return success in Qdrant format

#### Scenario: Delete Shard Key
Given a Qdrant API request POST `/qdrant/collections/{name}/shards/delete`
When the request specifies shard key to delete
Then the system MUST delete shard key
And the system MUST return success in Qdrant format

### Requirement: Cluster Management via Qdrant API
The system SHALL provide Qdrant-compatible cluster management REST endpoints.

#### Scenario: Get Cluster Status
Given a Qdrant API request GET `/qdrant/cluster`
When the request is received
Then the system MUST return cluster information in Qdrant format
And the system MUST include cluster nodes and status

#### Scenario: Recover Current Peer
Given a Qdrant API request POST `/qdrant/cluster/recover`
When the request is received
Then the system MUST request snapshot for current peer
And the system MUST return success in Qdrant format

#### Scenario: Remove Peer
Given a Qdrant API request DELETE `/qdrant/cluster/peer/{peer_id}`
When the request is received
Then the system MUST remove peer from cluster
And the system MUST return success in Qdrant format

#### Scenario: Get Cluster Metadata Keys
Given a Qdrant API request GET `/qdrant/cluster/metadata/keys`
When the request is received
Then the system MUST return list of metadata keys in Qdrant format

#### Scenario: Get Cluster Metadata Key
Given a Qdrant API request GET `/qdrant/cluster/metadata/keys/{key}`
When the request is received
Then the system MUST return metadata value in Qdrant format

#### Scenario: Update Cluster Metadata Key
Given a Qdrant API request PUT `/qdrant/cluster/metadata/keys/{key}`
When the request is received with metadata value
Then the system MUST update metadata key
And the system MUST return success in Qdrant format

### Requirement: Search Groups
The system SHALL support search groups endpoint.

#### Scenario: Search Point Groups
Given a Qdrant API request POST `/qdrant/collections/{name}/points/search/groups`
When the request specifies group_by field and search parameters
Then the system MUST return grouped search results in Qdrant format

### Requirement: Search Matrix
The system SHALL support search matrix endpoints.

#### Scenario: Search Matrix Pairs
Given a Qdrant API request POST `/qdrant/collections/{name}/points/search/matrix/pairs`
When the request specifies matrix search parameters
Then the system MUST return distance matrix for pairs in Qdrant format

#### Scenario: Search Matrix Offsets
Given a Qdrant API request POST `/qdrant/collections/{name}/points/search/matrix/offsets`
When the request specifies matrix search parameters
Then the system MUST return distance matrix for offsets in Qdrant format

### Requirement: Query API
The system SHALL support Qdrant Query API endpoints.

#### Scenario: Query Points
Given a Qdrant API request POST `/qdrant/collections/{name}/points/query`
When the request specifies query parameters
Then the system MUST return query results in Qdrant format

#### Scenario: Query Batch
Given a Qdrant API request POST `/qdrant/collections/{name}/points/query/batch`
When the request specifies multiple queries
Then the system MUST return batch query results in Qdrant format

#### Scenario: Query Groups
Given a Qdrant API request POST `/qdrant/collections/{name}/points/query/groups`
When the request specifies group_by field and query parameters
Then the system MUST return grouped query results in Qdrant format

### Requirement: Named Vectors Support
The system SHALL support named vectors with `using` parameter for vector selection.

#### Scenario: Search with Named Vector
Given a collection with multiple named vectors
When a search request specifies `using` parameter
Then the system MUST search using the specified named vector
And the system MUST return results based on selected vector

#### Scenario: Query with Named Vector
Given a collection with multiple named vectors
When a query request specifies `using` parameter
Then the system MUST query using the specified named vector
And the system MUST return results based on selected vector

#### Scenario: Upsert with Named Vectors
Given a collection with named vectors support
When an upsert request specifies named vectors
Then the system MUST store vectors with their names
And the system MUST return success in Qdrant format

### Requirement: Prefetch Operations
The system SHALL support prefetch operations in search and query requests.

#### Scenario: Search with Prefetch
Given a search request with prefetch parameter
When the request is processed
Then the system MUST prefetch specified vectors
And subsequent operations MUST benefit from cached vectors

#### Scenario: Query with Prefetch
Given a query request with prefetch parameter
When the request is processed
Then the system MUST prefetch specified vectors
And subsequent operations MUST benefit from cached vectors

### Requirement: Quantization via Qdrant API
The system SHALL expose Product Quantization and Binary Quantization through Qdrant API endpoints.

**Note**: PQ and Binary quantization implementations already exist in Vectorizer but are not exposed via Qdrant API.

#### Scenario: PQ Configuration via Qdrant API
Given a Qdrant API request to create collection with PQ quantization
When the request specifies `quantization_config.product`
Then the system MUST enable Product Quantization
And the system MUST return success response compatible with Qdrant format

#### Scenario: Binary Quantization via Qdrant API
Given a Qdrant API request to create collection with binary quantization
When the request specifies `quantization_config.binary`
Then the system MUST enable Binary Quantization
And the system MUST return success response compatible with Qdrant format

## Quality Requirements

### Performance
- gRPC operations MUST have latency within 10% of REST API
- Prefetch operations MUST improve subsequent query performance by at least 20%
- Search groups and matrix operations MUST complete within acceptable time limits

### Compatibility
- All Qdrant API operations MUST be compatible with Qdrant 1.14.x
- gRPC protocol MUST match Qdrant gRPC service definitions exactly
- Response formats MUST match Qdrant JSON/Protobuf schemas

### Testing
- Test coverage MUST be 95%+ for all new code
- Integration tests MUST validate against real Qdrant server
- Compatibility tests MUST cover all Qdrant API endpoints
