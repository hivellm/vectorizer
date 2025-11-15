# Changelog

All notable changes to the Hive Vectorizer Python SDK will be documented in this file.

## [1.3.0] - 2025-11-15

### Added

- **Hybrid Search Support**: Complete implementation of hybrid search combining dense and sparse vectors

  - `SparseVector`: Model for sparse vector representation with indices and values
  - `HybridSearchRequest`: Request model with alpha, algorithm (rrf/weighted/alpha), and k parameters
  - `HybridSearchResponse` and `HybridSearchResult`: Response models for hybrid search results
  - `hybrid_search()`: Method in VectorizerClient for performing hybrid searches
  - Full validation and error handling

- **Qdrant Compatibility**: Full Qdrant REST API compatibility methods
  - `qdrant_list_collections()`: List all collections (Qdrant format)
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

### Note

This release adds hybrid search and Qdrant compatibility features. All existing functionality remains unchanged and backward compatible.

## [1.2.0] - 2025-10-25

### Added

- **Replication Models**: New data models for replication monitoring
  - `ReplicaStatus`: Status enum for replica nodes (Connected, Syncing, Lagging, Disconnected)
  - `ReplicaInfo`: Detailed information about replica nodes (host, port, status, heartbeat, operations synced)
  - `ReplicationStats`: Enhanced statistics with new v1.2.0 fields:
    - `role`: Node role (Master or Replica)
    - `bytes_sent`: Total bytes sent to replicas
    - `bytes_received`: Total bytes received from master
    - `last_sync`: Timestamp of last synchronization
    - `operations_pending`: Number of operations waiting to be replicated
    - `snapshot_size`: Size of snapshot data
    - `connected_replicas`: Number of connected replica nodes (Master only)
  - `ReplicationStatusResponse`: Response structure for `/replication/status` endpoint
  - `ReplicaListResponse`: Response structure for `/replication/replicas` endpoint

### Changed

- **Backwards Compatible**: All new replication fields are optional to maintain compatibility with older servers
- **Legacy Fields Maintained**: Existing replication fields (`master_offset`, `replica_offset`, `lag_operations`, `total_replicated`) continue to work

### Technical

- Added comprehensive validation for new replication models
- Enhanced type hints with proper Optional types for new fields
- Maintained strict dataclass validation for all new models

## [1.0.0] - 2025-10-21

### Changed

- **Version Sync**: Updated to v1.0.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.2.3 (20 individual MCP tools including search_hybrid)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.0.0

### Note

This release maintains full compatibility with Vectorizer REST API. The MCP refactoring in server v1.0.0 does not affect SDK functionality as SDKs use REST/UMICP protocols.

## [0.4.0] - 2025-10-12

### 🎉 Published to PyPI

- **Package**: [hive-vectorizer](https://pypi.org/project/hive-vectorizer/0.4.0/)
- **Installation**: `pip install hive-vectorizer`
- Migrated from `setup.py` to modern `pyproject.toml` configuration
- Added `.gitignore` to prevent committing build artifacts and credentials

### Added

- **UMICP Protocol Support**: Added support for the UMICP (Universal Messaging and Inter-process Communication Protocol)
  - New `UMICPClient` using official `umicp-python` package
  - Transport abstraction layer supporting multiple protocols (HTTP/HTTPS and UMICP)
  - Connection string support for easy protocol switching (e.g., `umicp://localhost:15003`)
  - `TransportFactory` for creating protocol-specific clients
  - `parse_connection_string` utility for parsing connection URIs
  - `HTTPClient` module extracted for better separation of concerns
- Build and publish scripts (`build.sh`, `publish.sh`, `build.ps1`, `publish.ps1`)
- Comprehensive test suite for UMICP transport

### Changed

- Refactored `VectorizerClient` to use transport abstraction instead of direct aiohttp calls
- Updated `VectorizerClient` constructor to support multiple protocols:
  - Added `connection_string` parameter for URI-based configuration
  - Added `protocol` parameter to specify transport protocol
  - Added `umicp` parameter for UMICP-specific options
- Updated `connect()` and `close()` methods to handle multiple transport types
- Migrated package configuration from `setup.py` to `pyproject.toml` (PEP 517/518)

### New API

- `client.get_protocol()`: Get the current transport protocol being used
- Multiple transport options:
  - HTTP/HTTPS (default)
  - UMICP (via `umicp-python` package)

### Dependencies

- Added `umicp-python>=0.1.3` for UMICP protocol support
- Added `aiohttp>=3.8.0` as primary dependency

### Documentation

- Created `examples/umicp_usage.py` demonstrating UMICP usage
- Updated README.md with UMICP configuration examples
- Added protocol comparison table

### Technical

- Created transport abstraction for protocol independence
- Separated HTTP logic into `utils/http_client.py`
- Created `utils/umicp_client.py` wrapper around `umicp-python`
- Created `utils/transport.py` for transport factory and parsing
- Added comprehensive error handling for both protocols
- Maintained backward compatibility with existing HTTP-only configurations
- Translated all test comments and strings to English

## [0.3.4] - Previous Version

- (Previous changes...)
