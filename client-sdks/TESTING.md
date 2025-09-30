# Testing Guide for Hive Vectorizer Client SDKs

This document provides comprehensive information about testing the TypeScript, JavaScript, Python, and Rust client SDKs for Hive Vectorizer.

## Test Structure

### TypeScript SDK Tests

```
client-sdks/typescript/tests/
├── setup.ts                    # Global test setup
├── client.test.ts              # Main client tests
├── models/                     # Model validation tests
│   ├── vector.test.ts
│   ├── collection.test.ts
│   └── search-result.test.ts
├── exceptions/                 # Exception class tests
│   └── vectorizer-error.test.ts
├── utils/                      # Utility function tests
│   ├── http-client.test.ts
│   ├── websocket-client.test.ts
│   └── validation.test.ts
├── integration/                # Integration tests
│   └── client-integration.test.ts
└── performance/                # Performance tests
    └── client-performance.test.ts
```

### JavaScript SDK Tests

```
client-sdks/javascript/tests/
├── setup.js                    # Global test setup
├── client.test.js              # Main client tests
├── models/                     # Model validation tests
│   ├── vector.test.js
│   ├── collection.test.js
│   └── search-result.test.js
├── exceptions/                 # Exception class tests
│   └── vectorizer-error.test.js
├── utils/                      # Utility function tests
│   ├── http-client.test.js
│   └── validation.test.js
├── integration/                # Integration tests
│   └── client-integration.test.js
└── performance/                # Performance tests
    └── client-performance.test.js
```

### Python SDK Tests

```
client-sdks/python/
├── test_exceptions.py          # Exception class tests (44 tests)
├── test_models.py              # Data model validation tests (29 tests)
├── test_validation.py          # Validation utility tests (20 tests)
├── test_http_client.py         # HTTP client tests (16 tests)
├── test_client_integration.py  # Integration tests (20 tests)
├── test_sdk_comprehensive.py   # Comprehensive SDK tests (55 tests)
├── run_tests.py               # Test runner with reporting
└── TESTES_RESUMO.md           # Test documentation
```

### Rust SDK Tests

```
client-sdks/rust/tests/
├── models_tests.rs            # Model validation tests (20 tests)
├── error_tests.rs             # Exception class tests (25 tests)
├── validation_tests.rs        # Validation utility tests (13 tests)
├── http_client_tests.rs       # HTTP client tests (17 tests)
├── client_integration_tests.rs # Integration tests (13 tests)
└── integration_tests.rs       # Original integration tests (legacy)
```

```
client-sdks/rust/
├── run_tests.rs              # Test runner script
├── examples/                 # Example implementations
│   ├── basic_example.rs
│   ├── comprehensive_test.rs
│   └── test_working.rs
└── Cargo.toml               # Dependencies and test configuration
```

## Test Categories

### 1. Unit Tests

#### Model Validation Tests
- **Vector Model**: Tests for vector data validation, including ID, data array, and metadata validation
- **Collection Model**: Tests for collection creation and validation, including dimension and similarity metric validation
- **Search Result Model**: Tests for search result validation and response structure validation

#### Exception Tests
- **VectorizerError**: Base exception class with error codes and details
- **Specific Exceptions**: AuthenticationError, ValidationError, NetworkError, etc.
- **Error Inheritance**: Proper inheritance chain and error type checking

#### Utility Tests
- **HTTP Client**: Request/response handling, error mapping, timeout handling
- **Validation Utilities**: Input validation, type checking, range validation

### 2. Integration Tests

#### Complete Workflow Tests
- **Vector Lifecycle**: Create collection → Insert vectors → Search → Delete
- **Error Recovery**: Handle failures and recover gracefully
- **Configuration Updates**: Dynamic configuration changes

#### Error Scenario Tests
- **Network Errors**: Connection failures, timeouts, server errors
- **Validation Errors**: Invalid input data, missing required fields
- **Authentication Errors**: Invalid API keys, expired tokens
- **Partial Failures**: Batch operations with mixed success/failure

### 3. Performance Tests

#### Batch Operations
- **Large Vector Insertion**: 1000+ vectors in single batch
- **Concurrent Operations**: Multiple simultaneous requests
- **Search Performance**: Large result sets, complex queries

#### Memory Usage
- **Large Data Handling**: High-dimensional vectors, large metadata
- **Memory Efficiency**: No memory leaks, reasonable memory usage

#### Network Performance
- **High-Frequency Requests**: 500+ concurrent requests
- **WebSocket Throughput**: 1000+ messages per second
- **Error Handling Performance**: Efficient error processing

## Running Tests

### TypeScript SDK

```bash
cd client-sdks/typescript

# Install dependencies
npm install

# Run all tests
npm test

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm test -- --testPathPattern=client.test.ts

# Run tests in watch mode
npm run test:watch

# Run performance tests only
npm test -- --testPathPattern=performance
```

### JavaScript SDK

```bash
cd client-sdks/javascript

# Install dependencies
npm install

# Run all tests
npm test

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm test -- --testPathPattern=client.test.js

# Run tests in watch mode
npm run test:watch

# Run performance tests only
npm test -- --testPathPattern=performance
```

### Python SDK

```bash
cd client-sdks/python

# Install dependencies
pip install -r requirements.txt

# Run all tests
python run_tests.py

# Run specific test file
python -m pytest test_exceptions.py -v

# Run tests with coverage (if pytest-cov installed)
python -m pytest --cov=src --cov-report=html

# Run performance tests
python -c "from test_models import *; run_performance_tests()"
```

### Rust SDK

```bash
cd client-sdks/rust

# Install dependencies
cargo build

# Run all tests
cargo test

# Run specific test file
cargo test --test models_tests
cargo test --test error_tests
cargo test --test validation_tests
cargo test --test http_client_tests
cargo test --test client_integration_tests

# Run tests with output
cargo test -- --nocapture

# Run tests with coverage (if tarpaulin installed)
cargo tarpaulin --out Html

# Run test runner script
cargo run --bin run_tests
```

## Test Configuration

### Jest Configuration

Both SDKs use Jest with the following configuration:

```javascript
module.exports = {
  testEnvironment: 'node',
  roots: ['<rootDir>/src', '<rootDir>/tests'],
  testMatch: ['**/?(*.)+(spec|test).(js|ts)'],
  collectCoverageFrom: ['src/**/*.(js|ts)', '!src/**/index.(js|ts)'],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html', 'json'],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  },
  setupFilesAfterEnv: ['<rootDir>/tests/setup.(js|ts)'],
  testTimeout: 10000,
  verbose: true
};
```

### Coverage Thresholds

- **Branches**: 80% - Ensure all code paths are tested
- **Functions**: 80% - All functions have test coverage
- **Lines**: 80% - All lines of code are executed during tests
- **Statements**: 80% - All statements are covered

## Mocking Strategy

### HTTP Client Mocking
- Mock `fetch` API for HTTP requests
- Simulate various response scenarios (success, error, timeout)
- Test error handling and response parsing


### AbortController Mocking
- Mock AbortController for timeout handling
- Test request cancellation and timeout scenarios

## Test Data

### Sample Vectors
```javascript
const sampleVector = {
  id: 'test-vector-1',
  data: [0.1, 0.2, 0.3, 0.4],
  metadata: { source: 'test.pdf', category: 'document' }
};
```

### Sample Collections
```javascript
const sampleCollection = {
  name: 'test-collection',
  dimension: 384,
  similarity_metric: 'cosine',
  description: 'Test collection for unit tests'
};
```

### Sample Search Results
```javascript
const sampleSearchResult = {
  results: [
    {
      id: 'result-1',
      score: 0.95,
      data: [0.1, 0.2, 0.3, 0.4],
      metadata: { source: 'doc1.pdf' }
    }
  ],
  total: 1
};
```

## Performance Benchmarks

### Expected Performance Metrics

#### Batch Operations
- **Vector Insertion**: 1000 vectors in < 1 second
- **Concurrent Requests**: 10 batches of 100 vectors in < 2 seconds
- **Search Operations**: 100 searches in < 3 seconds

#### Network Performance
- **High-Frequency Requests**: 500 requests in < 5 seconds
- **Error Handling**: 100 errors in < 2 seconds

#### Memory Usage
- **Large Vectors**: 100 vectors of 4096 dimensions
- **Large Results**: 1000 search results with metadata
- **Memory Efficiency**: No memory leaks, reasonable usage

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: Test SDKs
on: [push, pull_request]
jobs:
  test-typescript:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: cd client-sdks/typescript && npm ci
      - run: cd client-sdks/typescript && npm test
      - run: cd client-sdks/typescript && npm run test:coverage

  test-javascript:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: cd client-sdks/javascript && npm ci
      - run: cd client-sdks/javascript && npm test
      - run: cd client-sdks/javascript && npm run test:coverage

  test-python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: '3.9'
      - run: cd client-sdks/python && pip install -r requirements.txt
      - run: cd client-sdks/python && python run_tests.py

  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd client-sdks/rust && cargo test
      - run: cd client-sdks/rust && cargo run --bin run_tests
```

## Debugging Tests

### Common Issues

1. **Timeout Errors**: Increase test timeout for slow operations
2. **Mock Issues**: Ensure mocks are properly reset between tests
3. **Async Issues**: Use proper async/await patterns
4. **Coverage Issues**: Check for untested code paths

### Debug Commands

```bash
# Run tests with debug output
npm test -- --verbose

# Run specific test with debug
npm test -- --testNamePattern="should handle" --verbose

# Run tests with Node.js debugger
node --inspect-brk node_modules/.bin/jest --runInBand
```

## Best Practices

### Test Organization
- Group related tests in describe blocks
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)
- Keep tests focused and independent

### Mocking
- Mock external dependencies
- Use realistic test data
- Reset mocks between tests
- Test both success and failure scenarios

### Performance Testing
- Set realistic performance thresholds
- Test with various data sizes
- Monitor memory usage
- Test concurrent operations

### Error Testing
- Test all error scenarios
- Verify error messages and codes
- Test error recovery
- Test edge cases and boundary conditions

## Contributing

When adding new tests:

1. Follow existing test patterns
2. Maintain coverage thresholds
3. Add performance tests for new features
4. Update this documentation
5. Ensure tests pass in CI/CD

## Resources

- [Jest Documentation](https://jestjs.io/docs/getting-started)
- [TypeScript Testing](https://jestjs.io/docs/getting-started#using-typescript)
- [Mocking Guide](https://jestjs.io/docs/mock-functions)
- [Coverage Reports](https://jestjs.io/docs/cli#--coverage)
