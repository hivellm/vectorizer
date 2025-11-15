# Changelog

All notable changes to the Hive Vectorizer Rust SDK will be documented in this file.

## [1.3.0] - 2025-11-15

### Added

- **Hybrid Search Support**: Complete Rust implementation with full type safety
  - `SparseVector`: Struct for sparse vector representation with validation
  - `HybridSearchRequest`: Request struct with serde serialization
  - `HybridSearchResponse` and `HybridSearchResult`: Response structs
  - `HybridScoringAlgorithm`: Enum for scoring algorithms (RRF, Weighted, Alpha)
  - `hybrid_search()`: Method in VectorizerClient with full error handling
  - Module `models::hybrid_search` for all hybrid search types

- **Qdrant Compatibility**: Full Qdrant REST API compatibility methods
  - `qdrant_list_collections()`: List all collections
  - `qdrant_get_collection()`: Get collection information
  - `qdrant_create_collection()`: Create collection with Qdrant config
  - `qdrant_upsert_points()`: Upsert points to collection
  - `qdrant_search_points()`: Search points in collection
  - `qdrant_delete_points()`: Delete points from collection
  - `qdrant_retrieve_points()`: Retrieve points by IDs
  - `qdrant_count_points()`: Count points in collection

### Changed

- **Version Sync**: Updated to v1.3.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **Type Safety**: Full Rust type safety with serde serialization for all new methods

### Note

This release adds hybrid search and Qdrant compatibility features. All existing functionality remains unchanged and backward compatible.

## [1.2.0] - 2025-10-25

### Added

- **Replication Models**: New data structures for replication monitoring
  - `ReplicaStatus`: Enum for replica node status (Connected, Syncing, Lagging, Disconnected)
  - `ReplicaInfo`: Struct for replica node details with all fields
  - `ReplicationStats`: Enhanced statistics struct with new v1.2.0 fields:
    - `role`: Node role (Master or Replica)
    - `bytes_sent`: Total bytes sent to replicas
    - `bytes_received`: Total bytes received from master
    - `last_sync`: Timestamp of last synchronization
    - `operations_pending`: Number of operations waiting to be replicated
    - `snapshot_size`: Size of snapshot data
    - `connected_replicas`: Number of connected replica nodes (Master only)
  - `ReplicationStatusResponse`: Response struct for `/replication/status` endpoint
  - `ReplicaListResponse`: Response struct for `/replication/replicas` endpoint

### Changed

- **Backwards Compatible**: All new replication fields are `Option<T>` to maintain compatibility with older servers
- **Legacy Fields Maintained**: Existing replication fields continue to work and are non-optional for stability

### Technical

- Used `#[serde(skip_serializing_if = "Option::is_none")]` for new optional fields
- Added comprehensive documentation for all new types
- Used `DateTime<Utc>` from chrono for timestamp fields
- Maintained strict typing and Rust best practices

## [1.0.0] - 2025-10-21

### Changed

- **Version Sync**: Updated to v1.2.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.3.0 (hybrid search and Qdrant compatibility)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.3.0

### Note

This release maintains full compatibility with Vectorizer REST API. Version 1.3.0 adds hybrid search support (search_hybrid tool) and Qdrant compatibility, but does not affect SDK functionality as SDKs use REST/UMICP protocols.

## [0.4.0] - 2025-10-12

### Added

- **UMICP Protocol Support**: Added support for the UMICP (Universal Messaging and Inter-process Communication Protocol)
  - New `UmicpTransport` using `umicp-core` crate (optional feature)
  - Transport abstraction layer supporting multiple protocols (HTTP/HTTPS and UMICP)
  - Connection string support for easy protocol switching (e.g., `umicp://localhost:15003`)
  - `parse_connection_string` utility for parsing connection URIs
  - `Protocol` enum for protocol selection

### Changed

- Refactored `VectorizerClient` to use transport abstraction instead of direct reqwest
- Updated `VectorizerClient` with new configuration options:
  - Added `ClientConfig` struct for flexible initialization
  - Added `protocol` field to specify transport protocol
  - Added `connection_string` field for URI-based configuration
  - Added `umicp` field for UMICP-specific options
- All HTTP requests now go through transport layer for protocol flexibility

### New API

- `VectorizerClient::new(config: ClientConfig)`: Create client with full configuration
- `VectorizerClient::from_connection_string(conn_str, api_key)`: Create from connection string
- `client.protocol()`: Get the current transport protocol being used
- Multiple transport options:
  - HTTP/HTTPS (default)
  - UMICP (optional feature, requires `--features umicp`)

### Dependencies

- Added `async-trait@0.1` for transport trait
- Added `umicp-core@0.1` as optional dependency (feature-gated)
- Updated `reqwest` to `0.11.24` for compatibility

### Features

- `umicp`: Enable UMICP protocol support (opt-in via cargo feature)

### Documentation

- Created `examples/umicp_usage.rs` demonstrating UMICP usage
- Created comprehensive UMICP tests

### Technical

- Implemented `Transport` trait for protocol abstraction
- Created separate transport implementations:
  - `HttpTransport` for HTTP/HTTPS
  - `UmicpTransport` for UMICP protocol (feature-gated)
- Added comprehensive error handling for both protocols
- Maintained backward compatibility with existing HTTP-only configurations

### Requirements

- **Minimum Rust Version**: 1.75.0 for HTTP transport only
- **For UMICP Feature**: Rust 1.82+ (due to transitive dependencies from reqwest 0.12)

## [0.3.4] - Previous Version

- (Previous changes...)
