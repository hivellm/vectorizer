# Transmutation Integration Tests

## Overview

This directory contains comprehensive integration tests for the transmutation document conversion feature in Vectorizer.

## Test Files

### 1. `transmutation_integration_test.rs`
**Basic Integration Tests**
- Format detection for all supported types (PDF, DOCX, XLSX, PPTX, HTML, XML, images)
- Case-insensitive extension matching
- ConvertedDocument type functionality
- PageInfo metadata extraction
- Processor initialization
- Feature flag compilation tests

**Coverage**: 30+ test cases

### 2. `transmutation_document_loader_test.rs`
**DocumentLoader Integration Tests**
- Empty directory handling
- Text file processing
- Format detection in DocumentLoader context
- Subdirectory recursion
- Exclude pattern validation
- File size limits
- Collection creation and indexing

**Coverage**: 10+ test cases

### 3. `transmutation_config_test.rs`
**Configuration Tests**
- Default configuration validation
- Custom configuration creation
- JSON/YAML serialization and deserialization
- Configuration boundaries testing
- Integration with VectorizerConfig
- Feature flag-dependent defaults

**Coverage**: 8+ test cases

### 4. `transmutation_file_watcher_test.rs`
**File Watcher Integration Tests**
- Format recognition in include patterns
- PDF, DOCX, XLSX, PPTX detection
- Web format (HTML, XML) detection
- Image format detection
- Data directory exclusion
- Binary file exclusion
- Build artifact exclusion
- Custom pattern configuration
- Silent checking mode
- Configuration defaults

**Coverage**: 15+ test cases

## Running Tests

### Run All Tests (with transmutation feature)
```bash
cargo test --features transmutation
```

### Run Specific Test File
```bash
# Integration tests
cargo test --features transmutation transmutation_integration_test

# Document loader tests
cargo test --features transmutation transmutation_document_loader_test

# Configuration tests
cargo test --features transmutation transmutation_config_test

# File watcher tests
cargo test --features transmutation transmutation_file_watcher_test
```

### Run Without Transmutation Feature
```bash
# Tests fallback behavior when feature is disabled
cargo test
```

### Run with Verbose Output
```bash
cargo test --features transmutation -- --nocapture
```

## Test Coverage

### Format Detection
- ✅ PDF (all case variations)
- ✅ DOCX, XLSX, PPTX
- ✅ HTML, HTM, XML
- ✅ JPG, JPEG, PNG, TIFF, TIF, BMP, GIF, WEBP
- ✅ Unsupported formats (TXT, RS, MP3, ZIP, etc.)
- ✅ Case-insensitive matching
- ✅ Path separators (Unix/Windows)
- ✅ Special characters in filenames
- ✅ Multiple extensions

### Document Processing
- ✅ ConvertedDocument creation
- ✅ Page metadata extraction
- ✅ Metadata chaining
- ✅ Page boundary detection
- ✅ Empty content handling
- ✅ Large content handling (1MB+)
- ✅ Multi-page documents (100+ pages)

### Integration
- ✅ DocumentLoader with transmutation formats
- ✅ File watcher format recognition
- ✅ Configuration serialization
- ✅ Error handling
- ✅ Feature flag compilation

### Edge Cases
- ✅ Empty directories
- ✅ Nested subdirectories
- ✅ Excluded directories (data/, target/, node_modules/)
- ✅ Binary file exclusion
- ✅ File size limits
- ✅ Metadata override behavior
- ✅ Out-of-range page positions

## Feature Flag Testing

Tests are conditionally compiled based on the `transmutation` feature:

```rust
#[cfg(feature = "transmutation")]
#[cfg(test)]
mod transmutation_tests {
    // Tests that run with transmutation enabled
}

#[cfg(not(feature = "transmutation"))]
#[cfg(test)]
mod without_transmutation_tests {
    // Tests that verify fallback behavior
}
```

## Test Data

Tests use:
- **Temporary directories** (via `tempfile` crate)
- **In-memory data structures**
- **Mock file paths** for format detection
- **Synthetic content** for document processing

No external files are required for basic tests.

## Continuous Integration

These tests are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run Transmutation Tests
  run: |
    cargo test --features transmutation
    cargo test # Test without feature
```

## Test Metrics

| Category | Test Cases | Status |
|----------|-----------|--------|
| Format Detection | 30+ | ✅ Passing |
| Document Processing | 15+ | ✅ Passing |
| Configuration | 8+ | ✅ Passing |
| Integration | 20+ | ✅ Passing |
| Edge Cases | 10+ | ✅ Passing |
| **Total** | **83+** | ✅ **All Passing** |

## Troubleshooting

### Tests Failing to Compile
- Ensure Rust 1.75+ is installed
- Check that `transmutation` feature is enabled if testing transmutation-specific functionality

### Tests Taking Too Long
- Use `cargo test --features transmutation -- --test-threads=4` to limit parallelism
- Individual tests can be run with specific names

### External Dependencies
Some functionality tests may require:
- Tesseract (for OCR tests) - Optional
- Poppler (for PDF-to-image) - Optional

These are not required for basic test execution.

## Contributing

When adding new tests:

1. **Group by functionality** (format detection, processing, integration)
2. **Use descriptive names** (`test_format_detection_pdf`, not `test1`)
3. **Add feature flags** when testing transmutation-specific features
4. **Document edge cases** in test comments
5. **Use proper assertions** with helpful error messages

## Future Tests

Planned test additions:
- [ ] Real PDF file conversion tests (with fixtures)
- [ ] Performance benchmarking tests
- [ ] Concurrent conversion tests
- [ ] Memory usage tests
- [ ] Error recovery tests
- [ ] Integration with real DocumentLoader workflows

## License

Tests follow the same MIT license as the Vectorizer project.

