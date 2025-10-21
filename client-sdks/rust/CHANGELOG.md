# Changelog

All notable changes to the Hive Vectorizer Rust SDK will be documented in this file.

## [1.0.0] - 2025-10-21

### Changed
- **Version Sync**: Updated to v1.0.0 to match Vectorizer server release
- **Server Compatibility**: Compatible with Vectorizer v1.0.0 (19 individual MCP tools)
- **No Breaking Changes**: REST API remains unchanged, full backward compatibility
- **Production Ready**: Stable release aligned with server v1.0.0

### Note
This release maintains full compatibility with Vectorizer REST API. The MCP refactoring in server v1.0.0 does not affect SDK functionality as SDKs use REST/UMICP protocols.

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

