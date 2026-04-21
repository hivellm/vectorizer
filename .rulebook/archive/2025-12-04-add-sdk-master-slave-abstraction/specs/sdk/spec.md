# SDK Master/Slave Abstraction Specification

## ADDED Requirements

### Requirement: Host Configuration
The SDK SHALL support a `hosts` configuration object that accepts a master URL and an array of replica URLs for automatic read/write routing.

#### Scenario: Configure with master and replicas
Given a client initialization with hosts configuration
When the client is created with master "http://master:15001" and replicas ["http://replica1:15001", "http://replica2:15001"]
Then the client SHALL internally maintain connections to all specified hosts
And the client SHALL be ready to route operations automatically

#### Scenario: Configure with single node (backward compatibility)
Given a client initialization with only baseURL
When the client is created with baseURL "http://localhost:15001"
Then the client SHALL use the single URL for all operations
And no automatic routing SHALL occur

### Requirement: Read Preference
The SDK SHALL support a `readPreference` setting that determines where read operations are routed.

#### Scenario: Read preference replica
Given a client configured with readPreference "replica"
When a read operation is performed (search, get, list)
Then the operation SHALL be routed to one of the replica nodes
And round-robin load balancing SHALL be applied

#### Scenario: Read preference master
Given a client configured with readPreference "master"
When a read operation is performed
Then the operation SHALL be routed to the master node

#### Scenario: Read preference nearest
Given a client configured with readPreference "nearest"
When a read operation is performed
Then the operation SHALL be routed to the node with lowest measured latency

### Requirement: Automatic Write Routing
The SDK SHALL automatically route all write operations to the master node regardless of read preference.

#### Scenario: Insert operation routed to master
Given a client configured with any readPreference
When insertTexts or insertVectors is called
Then the operation MUST be sent to the master node

#### Scenario: Update operation routed to master
Given a client configured with any readPreference
When updateVector is called
Then the operation MUST be sent to the master node

#### Scenario: Delete operation routed to master
Given a client configured with any readPreference
When deleteVector or deleteCollection is called
Then the operation MUST be sent to the master node

#### Scenario: Create collection routed to master
Given a client configured with any readPreference
When createCollection is called
Then the operation MUST be sent to the master node

### Requirement: Read Preference Override
The SDK SHALL support per-operation override of the read preference for read-your-writes scenarios.

#### Scenario: Override to master for single operation
Given a client configured with readPreference "replica"
When getVector is called with readPreference override "master"
Then that specific operation SHALL be routed to the master node
And subsequent operations SHALL continue using the default preference

#### Scenario: With master context
Given a client configured with readPreference "replica"
When a block of operations is executed within withMaster context
Then all operations in that block SHALL be routed to the master node
And operations outside the block SHALL use the default preference

### Requirement: Round-Robin Load Balancing
The SDK SHALL implement round-robin load balancing when routing read operations to replicas.

#### Scenario: Distribute reads across replicas
Given a client with 3 replica nodes configured
When 6 consecutive read operations are performed
Then each replica SHALL receive exactly 2 requests
And the distribution SHALL be sequential (replica1, replica2, replica3, replica1, ...)

### Requirement: Backward Compatibility
The SDK SHALL maintain full backward compatibility with existing single-node configurations.

#### Scenario: Existing baseURL config works
Given an existing application using baseURL configuration
When the SDK is upgraded to support hosts configuration
Then the existing configuration MUST continue to work unchanged
And no code changes SHALL be required for single-node deployments

## Operation Classification

### Write Operations (Always Master)
- `insertTexts`
- `insertVectors`
- `updateVector`
- `deleteVector`
- `createCollection`
- `deleteCollection`
- `batchInsert`
- `batchUpdate`
- `batchDelete`

### Read Operations (Based on Preference)
- `searchVectors`
- `hybridSearch`
- `intelligentSearch`
- `semanticSearch`
- `getVector`
- `listVectors`
- `listCollections`
- `getCollectionInfo`
- `healthCheck`
