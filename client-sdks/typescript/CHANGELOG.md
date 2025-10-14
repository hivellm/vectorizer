# Changelog

All notable changes to the Hive Vectorizer TypeScript Client SDK will be documented in this file.

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

