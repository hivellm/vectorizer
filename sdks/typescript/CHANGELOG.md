# Changelog

All notable changes to the Hive Vectorizer TypeScript Client SDK will be documented in this file.

## [1.2.0] - 2025-10-25

### Added

- **Replication Models**: New data models and types for replication monitoring
  - `ReplicaStatus`: Enum for replica node status (Connected, Syncing, Lagging, Disconnected)
  - `ReplicaInfo`: Interface for replica node details (host, port, status, heartbeat, operations synced)
  - `ReplicationStats`: Enhanced interface with new v1.2.0 fields:
    - `role`: Node role (Master or Replica)
    - `bytes_sent`: Total bytes sent to replicas
    - `bytes_received`: Total bytes received from master
    - `last_sync`: Timestamp of last synchronization
    - `operations_pending`: Number of operations waiting to be replicated
    - `snapshot_size`: Size of snapshot data
    - `connected_replicas`: Number of connected replica nodes (Master only)
  - `ReplicationStatusResponse`: Interface for `/replication/status` endpoint response
  - `ReplicaListResponse`: Interface for `/replication/replicas` endpoint response
  - Validation functions: `isReplicaStatus()`, `validateReplicaInfo()`, `validateReplicationStats()`

### Changed

- **Backwards Compatible**: All new replication fields are optional to maintain compatibility with older servers
- **Legacy Fields Maintained**: Existing replication fields (`master_offset`, `replica_offset`, `lag_operations`, `total_replicated`) continue to work

### Technical

- Added comprehensive type guards and validation functions for replication models
- Enhanced TypeScript interfaces with proper optional types for new fields
- Maintained strict type safety for all new models

## [1.0.0] - 2025-10-21

### Changed

- **Version Sync**: Updated to v1.0.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.2.3 (20 individual MCP tools including search_hybrid)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.0.0

### Note

This release maintains full compatibility with Vectorizer REST API. The MCP refactoring in server v1.0.0 does not affect SDK functionality as SDKs use REST/UMICP protocols.

## [0.4.2] - 2025-10-15

### Added

- **GUI-Specific API Endpoints** for server management and monitoring:
  - `getStatus()` - Get server status, version, uptime, and statistics
  - `getLogs(params?)` - Retrieve recent server logs with optional filtering
  - `forceSaveCollection(name)` - Force immediate save of a specific collection
  - `addWorkspace(params)` - Add a new workspace configuration
  - `removeWorkspace(params)` - Remove an existing workspace
  - `listWorkspaces()` - List all configured workspaces
  - `getConfig()` - Get current server configuration
  - `updateConfig(config)` - Update server configuration
  - `restartServer()` - Admin endpoint to restart the server
  - `listBackups()` - List available backup files
  - `createBackup(params?)` - Create a new backup
  - `restoreBackup(params)` - Restore from a backup file
  - `getBackupDirectory()` - Get the backup directory path

## [0.4.1] - 2025-10-15

### Changed

- Updated `@hivellm/umicp` dependency to `^0.1.5`
- Installation no longer requires C++ build tools
- Faster and more reliable installation process

### Fixed

- Fixed installation failures on systems without build tools
- Removed build errors during package installation

## [0.4.0] - 2025-10-12

### Added

- **UMICP Protocol Support**: Added support for the UMICP (Universal Messaging and Inter-process Communication Protocol)
  - New `UMICPClient` for high-performance communication
  - Transport abstraction layer supporting multiple protocols (HTTP/HTTPS and UMICP)
  - Connection string support for easy protocol switching (e.g., `umicp://localhost:15003`)
  - `TransportFactory` for creating protocol-specific clients
  - `parseConnectionString` utility for parsing connection URIs

### Changed

- Refactored `VectorizerClient` to use transport abstraction instead of direct HTTP client
- Updated `VectorizerClientConfig` to support multiple protocols:
  - Added `protocol` field to specify transport protocol
  - Added `connectionString` field for URI-based configuration
  - Added `umicp` field for UMICP-specific options
- Updated `setApiKey()` method to reinitialize transport with new API key

### New API

- `client.getProtocol()`: Get the current transport protocol being used
- Multiple transport options:
  - HTTP/HTTPS (default)
  - UMICP (via `@hivellm/umicp` package)

### Dependencies

- Added `@hivellm/umicp@^0.1.3` as a dependency

### Documentation

- Updated README with UMICP configuration examples
- Added protocol comparison table
- Added examples for using UMICP transport
- Created `examples/umicp-usage.ts` demonstrating UMICP usage

### Technical

- Implemented `ITransport` interface for protocol abstraction
- Created separate transport implementations:
  - `HttpClient` for HTTP/HTTPS
  - `UMICPClient` for UMICP protocol
- Added comprehensive error handling for both protocols
- Maintained backward compatibility with existing HTTP-only configurations

## [0.3.4] - Previous Version

- (Previous changes...)
