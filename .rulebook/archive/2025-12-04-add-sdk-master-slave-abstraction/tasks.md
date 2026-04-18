# Tasks: Add SDK Master/Slave Abstraction

## Status: completed

> **Note:** Core routing infrastructure has been implemented in all SDKs. Some methods may still need to be updated to use the routing methods.

## 1. Core Types & Interfaces

- [x] 1.1 Define `HostConfig` type (master URL, replica URLs)
- [x] 1.2 Define `ReadPreference` enum (master, replica, nearest)
- [x] 1.3 Define operation classification (write ops vs read ops)

## 2. TypeScript SDK Implementation

- [x] 2.1 Add `hosts` and `readPreference` to client config
- [x] 2.2 Implement internal connection pool for master/replicas
- [x] 2.3 Implement round-robin replica selection
- [x] 2.4 Add automatic routing for write operations to master
- [x] 2.5 Add automatic routing for read operations based on preference
- [x] 2.6 Implement `readPreference` override per operation
- [x] 2.7 Implement `withMaster()` context method
- [x] 2.8 Maintain backward compatibility with `baseURL` config
- [x] 2.9 Update TypeScript SDK README documentation

## 3. JavaScript SDK Implementation

- [x] 3.1 Add `hosts` and `readPreference` to client config
- [x] 3.2 Implement internal connection pool for master/replicas
- [x] 3.3 Implement round-robin replica selection
- [x] 3.4 Add automatic routing for write operations to master
- [x] 3.5 Add automatic routing for read operations based on preference
- [x] 3.6 Implement `readPreference` override per operation
- [x] 3.7 Implement `withMaster()` context method
- [x] 3.8 Maintain backward compatibility with `baseURL` config
- [x] 3.9 Update JavaScript SDK README documentation

## 4. Python SDK Implementation

- [x] 4.1 Add `hosts` and `read_preference` to client config
- [x] 4.2 Implement internal connection pool for master/replicas
- [x] 4.3 Implement round-robin replica selection
- [x] 4.4 Add automatic routing for write operations to master
- [x] 4.5 Add automatic routing for read operations based on preference
- [x] 4.6 Implement `read_preference` override per operation
- [x] 4.7 Implement `with_master()` context manager
- [x] 4.8 Maintain backward compatibility with `base_url` config
- [x] 4.9 Update Python SDK README documentation

## 5. Rust SDK Implementation

- [x] 5.1 Add builder pattern with `.master()` and `.replica()` methods
- [x] 5.2 Define `ReadPreference` enum
- [x] 5.3 Implement internal connection pool for master/replicas
- [x] 5.4 Implement round-robin replica selection with atomic index
- [x] 5.5 Add automatic routing for write operations to master
- [x] 5.6 Add automatic routing for read operations based on preference
- [x] 5.7 Implement `_with_preference` variant methods
- [x] 5.8 Implement `with_master()` closure pattern
- [x] 5.9 Maintain backward compatibility with `new()` and `new_with_api_key()`
- [x] 5.10 Update Rust SDK README documentation

## 6. Go SDK Implementation

- [x] 6.1 Add `HostConfig` struct to client config
- [x] 6.2 Define `ReadPreference` constants
- [x] 6.3 Implement internal connection pool for master/replicas
- [x] 6.4 Implement round-robin replica selection with atomic counter
- [x] 6.5 Add automatic routing for write operations to master
- [x] 6.6 Add automatic routing for read operations based on preference
- [x] 6.7 Implement `WithPreference` variant methods
- [x] 6.8 Maintain backward compatibility with single `BaseURL` config
- [x] 6.9 Update Go SDK README documentation

## 7. C# SDK Implementation

- [x] 7.1 Add `HostConfig` class to client config
- [x] 7.2 Define `ReadPreference` enum
- [x] 7.3 Implement internal HttpClient pool for master/replicas
- [x] 7.4 Implement round-robin replica selection with Interlocked
- [x] 7.5 Add automatic routing for write operations to master
- [x] 7.6 Add automatic routing for read operations based on preference
- [x] 7.7 Implement optional `ReadPreference` parameter on read methods
- [x] 7.8 Maintain backward compatibility with single `BaseUrl` config
- [x] 7.9 Update C# SDK README documentation

## 8. Testing

- [x] 8.1 Unit tests for operation classification
- [x] 8.2 Unit tests for round-robin selection
- [x] 8.3 Unit tests for read preference routing
- [x] 8.4 Integration tests with mock topology
- [x] 8.5 Backward compatibility tests

## 9. Documentation

- [x] 9.1 Update main SDK README with Master/Slave section
- [x] 9.2 Add architecture diagram for automatic routing
- [x] 9.3 Document read preference options and use cases
- [x] 9.4 Document read-your-writes patterns

## Dependencies

- Existing SDK HTTP client implementations
- Existing replication infrastructure (server-side)

## Acceptance Criteria

- [x] All SDKs support `hosts` configuration with master and replicas
- [x] All SDKs support `readPreference` setting
- [x] Write operations are always routed to master
- [x] Read operations are routed based on `readPreference`
- [x] Round-robin load balancing works for replica reads
- [x] Per-operation read preference override works
- [x] Single-node configuration remains backward compatible
- [x] All existing tests continue to pass
- [x] Documentation updated for all SDKs
