# Summarization Test Suite Documentation

## Overview

This document describes the comprehensive test suite for the Vectorizer summarization system. The test suite ensures that all summarization functionality works correctly across all interfaces (REST API, MCP) and handles edge cases properly.

## Test Structure

### 1. Unit Tests (`src/summarization/tests.rs`)

Tests the core `SummarizationManager` functionality:

- **Manager Creation**: Tests initialization with custom and default configurations
- **Text Summarization**: Tests all summarization methods (extractive, keyword, sentence, abstractive)
- **Context Summarization**: Tests context-based summarization
- **Metadata Handling**: Tests metadata preservation and retrieval
- **Error Handling**: Tests error conditions (empty input, invalid methods)
- **Summary Retrieval**: Tests getting summaries by ID
- **Summary Listing**: Tests listing summaries with filters and pagination
- **Method Parsing**: Tests string-to-enum conversion for methods
- **Compression Ratios**: Tests various compression ratio values
- **Length Constraints**: Tests max_length parameter enforcement
- **Multi-language Support**: Tests different language codes
- **Summary Persistence**: Tests that summaries are stored and retrievable

### 2. REST Tests (`src/server/rest_handlers/tests.rs`)

Tests the GRPC server implementation:

- **GRPC Summarize Text**: Tests text summarization via GRPC
- **GRPC Summarize Context**: Tests context summarization via GRPC
- **GRPC Get Summary**: Tests summary retrieval via GRPC
- **GRPC List Summaries**: Tests summary listing via GRPC
- **Error Handling**: Tests GRPC error responses
- **Method Validation**: Tests invalid method handling
- **Metadata Support**: Tests metadata in GRPC requests/responses
- **Pagination**: Tests GRPC pagination parameters
- **Filtering**: Tests GRPC filtering by method and language
- **Multiple Languages**: Tests different language codes via GRPC

### 3. REST API Tests (`src/api/summarization_tests.rs`)

Tests the REST API implementation:

- **REST Summarize Text**: Tests text summarization via REST API
- **REST Summarize Context**: Tests context summarization via REST API
- **REST Get Summary**: Tests summary retrieval via REST API
- **REST List Summaries**: Tests summary listing via REST API
- **HTTP Status Codes**: Tests proper HTTP status code responses
- **JSON Parsing**: Tests JSON request/response handling
- **Content-Type Validation**: Tests proper Content-Type headers
- **Error Responses**: Tests REST API error responses
- **Query Parameters**: Tests URL query parameters for filtering
- **Request Validation**: Tests request body validation

### 4. MCP Tests (`src/mcp_service_tests.rs`)

Tests the MCP (Model Context Protocol) implementation:

- **MCP Summarize Text**: Tests text summarization via MCP
- **MCP Summarize Context**: Tests context summarization via MCP
- **MCP Get Summary**: Tests summary retrieval via MCP
- **MCP List Summaries**: Tests summary listing via MCP
- **Tool Arguments**: Tests MCP tool argument parsing
- **Error Handling**: Tests MCP error responses
- **Default Parameters**: Tests default parameter handling
- **Invalid Tools**: Tests invalid tool name handling
- **Missing Arguments**: Tests missing argument handling
- **JSON Response Format**: Tests MCP response format

### 5. Integration Tests (`tests/summarization_integration_tests.rs`)

Tests the complete integration flow:

- **End-to-End Flow**: Tests Manager → GRPC → REST → MCP flow
- **Error Handling Integration**: Tests error handling across all interfaces
- **Performance Integration**: Tests concurrent request handling
- **Method Integration**: Tests all summarization methods
- **Persistence Integration**: Tests summary storage and retrieval
- **Language Support Integration**: Tests multi-language support
- **Metadata Integration**: Tests metadata handling across interfaces

## Test Coverage

The test suite covers:

### Functional Coverage
- ✅ All summarization methods (extractive, keyword, sentence, abstractive)
- ✅ All interfaces (GRPC, REST API, MCP)
- ✅ All CRUD operations (create, read, list, filter)
- ✅ Error handling and validation
- ✅ Metadata handling
- ✅ Multi-language support
- ✅ Pagination and filtering

### Edge Cases
- ✅ Empty input handling
- ✅ Very short text handling
- ✅ Invalid method names
- ✅ Missing required parameters
- ✅ Invalid JSON parsing
- ✅ Non-existent summary IDs
- ✅ Boundary conditions (max_length, compression_ratio)

### Performance
- ✅ Concurrent request handling
- ✅ Large text processing
- ✅ Multiple language processing
- ✅ Memory usage validation

## Running Tests

### Run All Summarization Tests
```bash
./scripts/test-summarization.sh
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test --lib summarization

# GRPC tests only
cargo test --lib grpc::summarization_tests

# REST API tests only
cargo test --lib api::summarization_tests

# MCP tests only
cargo test --lib mcp_service_tests

# Integration tests only
cargo test --test summarization_integration_tests
```

### Run with Coverage
```bash
# Install cargo-tarpaulin first
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage
```

## Test Data

### Sample Texts Used in Tests
- **Short Text**: "Hi"
- **Medium Text**: "This is a test document about technology and innovation."
- **Long Text**: "This is a long text that needs to be summarized using the [interface] interface. It contains multiple sentences and should be compressed to a shorter version while maintaining the key information."
- **Technical Text**: "Machine learning is a subset of artificial intelligence that focuses on algorithms and statistical models. Deep learning uses neural networks with multiple layers."

### Test Parameters
- **Methods**: extractive, keyword, sentence, abstractive
- **Languages**: en, pt, es, fr
- **Max Lengths**: 10, 20, 30, 50
- **Compression Ratios**: 0.1, 0.2, 0.3, 0.5, 0.7, 0.9
- **Metadata**: source, category, version, timestamp

## Assertions

### Common Assertions
- Summary is not empty
- Summary length is less than original text length
- Method matches requested method
- Language matches requested language
- Status is "success" for successful operations
- Compression ratio is between 0.0 and 1.0
- Metadata is preserved correctly
- Error responses contain appropriate error messages

### Interface-Specific Assertions
- **GRPC**: Proper tonic::Status responses
- **REST**: Proper HTTP status codes and JSON responses
- **MCP**: Proper CallToolResult format with is_error flag

## Continuous Integration

The test suite is designed to run in CI/CD pipelines:

1. **Build Phase**: Compile the project
2. **Unit Tests**: Run core functionality tests
3. **Interface Tests**: Run GRPC, REST, and MCP tests
4. **Integration Tests**: Run end-to-end tests
5. **Regression Tests**: Run full test suite
6. **Coverage Report**: Generate coverage metrics

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure all dependencies are installed
2. **Test Timeouts**: Increase timeout values for slow tests
3. **Memory Issues**: Reduce concurrent test count
4. **Port Conflicts**: Use different ports for integration tests

### Debug Mode
```bash
# Run tests with debug output
RUST_LOG=debug cargo test --lib summarization

# Run specific test with debug
RUST_LOG=debug cargo test test_summarize_text_extractive --lib summarization
```

## Future Enhancements

### Planned Test Additions
- **Load Testing**: High-volume concurrent requests
- **Stress Testing**: Memory and CPU stress tests
- **Fuzz Testing**: Random input validation
- **Benchmark Tests**: Performance benchmarking
- **Security Tests**: Input sanitization and validation

### Test Infrastructure Improvements
- **Parallel Test Execution**: Faster test runs
- **Test Data Management**: Centralized test data
- **Mock Services**: Isolated component testing
- **Test Reporting**: Enhanced test result reporting

## Conclusion

The summarization test suite provides comprehensive coverage of all functionality, ensuring reliability and correctness across all interfaces. The tests serve as both validation and documentation of the expected behavior of the summarization system.
