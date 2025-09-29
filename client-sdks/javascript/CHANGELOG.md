# Changelog

All notable changes to the Hive Vectorizer JavaScript SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-09-29

### Added
- **REST-Only Architecture**: Complete removal of WebSocket functionality for REST-only API communication
- **Complete Test Suite**: Full test coverage with 100% passing tests
- **Enhanced Error Handling**: Improved exception classes with consistent error codes and messages
- **Robust Data Validation**: Enhanced vector and data validation with `isFinite()` checks
- **HTTP Client Improvements**: Better error handling and response parsing

### Changed
- **WebSocket Removal**: Eliminated all WebSocket dependencies and functionality
- **Package Dependencies**: Removed `ws` dependency, updated dev dependencies
- **Client Architecture**: Streamlined to REST-only operations
- **Error Messages**: Standardized error messages across all exception types

### Fixed
- **Vector Validation**: Fixed `Infinity` and `NaN` handling using `isFinite()` instead of `!isNaN()`
- **HTTP Error Handling**: Improved error response parsing and exception mapping
- **Test Environment**: Fixed `RangeError: Maximum call stack size exceeded` in tests
- **Exception Constructors**: Corrected optional parameter handling
- **Validation Functions**: Enhanced number and array validation logic

### Technical Details
- **Test Coverage**: All tests passing (100% success rate)
- **Dependencies**: Removed WebSocket libraries, optimized package.json
- **Error Handling**: 12 custom exception classes with consistent behavior
- **Data Validation**: Robust validation for vectors, collections, and search parameters

## [0.9.0] - 2025-09-25

### Added
- Initial release of Hive Vectorizer JavaScript SDK
- Complete client implementation with REST API support
- Collection management (create, delete, list, info)
- Vector operations (insert, search, get, delete)
- Text embedding generation
- Comprehensive error handling with custom exceptions
- Data validation and type checking
- Command-line interface (CLI)
- Basic test suite
- Documentation and examples

### Features
- **VectorizerClient**: Main client class with full API coverage
- **Data Models**: Vector, Collection, SearchResult, CollectionInfo
- **Exception Handling**: Custom exceptions for all error scenarios
- **Validation**: Input validation for all data types
- **CLI Tool**: Command-line interface for all operations
- **Type Safety**: JavaScript with JSDoc type annotations

## [Unreleased]

### Planned Features
- Advanced search filters and aggregations
- Vector similarity calculations
- Performance optimizations
- Additional data formats support
- Enhanced error recovery mechanisms
- Continuous integration testing pipeline
- WebSocket support (future consideration)
