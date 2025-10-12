# Changelog

All notable changes to the Hive Vectorizer Python SDK will be documented in this file.

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
