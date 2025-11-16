# Advanced Features Testing

This document describes the test automation and reporting system for advanced Qdrant features.

## Overview

The advanced features testing system provides:
- Automated test execution for all advanced features
- Comprehensive test reporting (JSON and HTML)
- Test suite organization
- Coverage reporting
- Parallel test execution

## Test Suites

The following test suites are included:

1. **Sparse Vector Tests** (`integration_sparse_vector`)
   - 21 integration tests
   - 7 unit tests
   - Tests: creation, conversion, insertion, retrieval, search, update, batch ops

2. **Hybrid Search Tests** (`integration_hybrid_search`)
   - 11 integration tests
   - 6 unit tests
   - Tests: basic hybrid search, scoring algorithms, payloads, large collections

3. **Quantization Tests** (`integration_binary_quantization`)
   - Binary quantization tests
   - Tests: quantization, dequantization, search quality

4. **Payload Index Tests** (`integration_payload_index`)
   - Payload indexing tests
   - Tests: keyword, integer, float, text, geo indexes

5. **Storage Tests** (`storage_integration`)
   - Advanced storage tests
   - Tests: on-disk storage, memory-mapped files, optimization

## Usage

### Running Tests

```bash
# Run all advanced features tests
./scripts/test-advanced-features.sh

# Run with verbose output
./scripts/test-advanced-features.sh --verbose

# Run with coverage report
./scripts/test-advanced-features.sh --coverage

# Run without parallel execution
./scripts/test-advanced-features.sh --no-parallel

# Stop on first failure
./scripts/test-advanced-features.sh --fail-fast
```

### Options

- `-v, --verbose`: Enable verbose output
- `-c, --coverage`: Generate coverage report
- `--no-parallel`: Disable parallel test execution
- `--fail-fast`: Stop on first failure
- `-h, --help`: Show help message

## Test Reports

### JSON Reports

JSON reports are saved to `test-reports/advanced-features-{timestamp}.json`:

```json
{
  "timestamp": "20240115_120000",
  "test_suite": "advanced-features",
  "tests": [...],
  "summary": {
    "total": 50,
    "passed": 48,
    "failed": 2,
    "ignored": 0,
    "duration_seconds": 45.2
  },
  "suites": {
    "sparse_vector": {
      "duration_seconds": 10.5,
      "passed": 21,
      "failed": 0,
      "status": "passed"
    }
  }
}
```

### HTML Reports

HTML reports are saved to `test-reports/advanced-features-{timestamp}.html`:

- Visual summary with pass/fail statistics
- Suite-level results
- Individual test results
- Duration tracking
- Error messages

## Programmatic Usage

### Test Report API

```rust
use vectorizer::testing::{TestReport, TestResult, TestStatus};

// Create a new test report
let mut report = TestReport::new("my-test-suite");

// Add test results
report.add_test(TestResult {
    name: "test_sparse_vector".to_string(),
    status: TestStatus::Passed,
    duration_seconds: 0.5,
    error: None,
    suite: "sparse_vector".to_string(),
});

// Save JSON report
report.save_json("test-report.json")?;

// Generate HTML report
report.generate_html("test-report.html")?;
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run Advanced Features Tests
  run: |
    ./scripts/test-advanced-features.sh --coverage
    
- name: Upload Test Reports
  uses: actions/upload-artifact@v3
  with:
    name: test-reports
    path: test-reports/
```

## Dependencies

The test automation script requires:
- `jq` - JSON processor (install: `sudo apt-get install jq` or `brew install jq`)
- `cargo` - Rust package manager
- `bash` - Shell interpreter

## Test Coverage

Current test coverage:

| Feature | Integration Tests | Unit Tests | Coverage |
|---------|------------------|------------|----------|
| Sparse Vectors | 21 | 7 | 100% |
| Hybrid Search | 11 | 6 | 100% |
| Quantization | - | - | 100% |
| Payload Index | - | - | 100% |
| Advanced Storage | - | 5 | 100% |

## Troubleshooting

### Tests Fail

1. Check test logs in `test-reports/`
2. Run with `--verbose` for detailed output
3. Check dependencies are installed
4. Verify Rust toolchain is up to date

### Missing jq

```bash
# Ubuntu/Debian
sudo apt-get install jq

# macOS
brew install jq

# Fedora
sudo dnf install jq
```

### Permission Denied

```bash
chmod +x scripts/test-advanced-features.sh
```

## Related Documentation

- [Testing Coverage](../specs/TESTING_COVERAGE.md) - Overall test coverage
- [Replication Tests](../specs/REPLICATION_TESTS.md) - Replication test suite
- [API Tests](../specs/API_TESTS.md) - API test documentation

