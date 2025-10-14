# Transmutation Integration

## Overview

Vectorizer integrates with [Transmutation](https://crates.io/crates/transmutation), a high-performance document conversion engine, to automatically convert various document formats to Markdown before chunking and embedding. This enables seamless indexing of PDFs, Office documents, HTML, XML, and images with OCR support.

## Features

- **Automatic Conversion**: Documents are automatically converted during indexing
- **Format Support**: PDF, DOCX, XLSX, PPTX, HTML, XML, JPG, PNG, TIFF, BMP, GIF, WEBP
- **Page Metadata**: Paginated documents (PDF, DOCX, PPTX) preserve page numbers in chunk metadata
- **Fallback Handling**: Graceful fallback if conversion fails
- **Optional Feature**: Compile-time opt-in via `transmutation` feature flag

## Supported Formats

### Document Formats

| Format | Status | Page Metadata | Notes |
|--------|--------|---------------|-------|
| **PDF** | ✅ Production | Yes | Fast mode: 98x faster than Docling |
| **DOCX** | ✅ Production | Yes | Pure Rust, LibreOffice for images |
| **XLSX** | ✅ Production | No | Converted to Markdown tables |
| **PPTX** | ✅ Production | Yes | Slides numbered as pages |
| **HTML** | ✅ Production | No | Clean Markdown output |
| **XML** | ✅ Production | No | Structured Markdown |

### Image Formats (OCR)

| Format | Status | OCR Engine | Notes |
|--------|--------|------------|-------|
| **JPG/JPEG** | ✅ Production | Tesseract | Requires tesseract installed |
| **PNG** | ✅ Production | Tesseract | Requires tesseract installed |
| **TIFF** | ✅ Production | Tesseract | Requires tesseract installed |
| **BMP** | ✅ Production | Tesseract | Requires tesseract installed |
| **GIF** | ✅ Production | Tesseract | Requires tesseract installed |
| **WEBP** | ✅ Production | Tesseract | Requires tesseract installed |

## Installation

### Building with Transmutation Support

```bash
# Build with transmutation feature enabled
cargo build --release --features transmutation

# Or add to full feature set
cargo build --release --features full
```

### External Dependencies

Transmutation has minimal external dependencies:

- **Core formats** (PDF, DOCX, XLSX, PPTX, HTML, XML): No external dependencies
- **PDF to Image**: Requires `poppler-utils` (optional)
- **Image OCR**: Requires `tesseract-ocr` (optional)

#### Installing External Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install poppler-utils tesseract-ocr
```

**macOS:**
```bash
brew install poppler tesseract
```

**Windows:**
```powershell
# Using Chocolatey
choco install poppler tesseract

# Or download installers from:
# - Poppler: https://blog.alivate.com.au/poppler-windows/
# - Tesseract: https://github.com/UB-Mannheim/tesseract/wiki
```

## Configuration

### Global Configuration

Add to your `config.yml`:

```yaml
transmutation:
  # Enable/disable transmutation conversion
  enabled: true
  
  # Maximum file size in MB for conversion (default: 50)
  max_file_size_mb: 50
  
  # Conversion timeout in seconds (default: 300)
  conversion_timeout_secs: 300
  
  # Preserve images during conversion (default: false)
  preserve_images: false
```

### Workspace Configuration

Transmutation automatically processes supported formats when files are indexed via workspace configurations:

```yaml
projects:
  - name: "documents"
    path: "/path/to/documents"
    collections:
      - name: "research-papers"
        include_patterns:
          - "**/*.pdf"
          - "**/*.docx"
        exclude_patterns:
          - "**/drafts/**"
        chunk_size: 2048
        chunk_overlap: 256
        embedding_provider: "bm25"
        dimension: 512
```

## How It Works

### Conversion Pipeline

1. **File Detection**: When a file is encountered during indexing, Vectorizer checks if it's a transmutation-supported format
2. **Format Validation**: File extension and mime type are validated
3. **Conversion**: Transmutation converts the document to Markdown
4. **Page Extraction**: For paginated documents, page boundaries are extracted
5. **Chunking**: Converted Markdown is split into chunks with page metadata
6. **Embedding**: Chunks are embedded and stored in the collection

### Page Metadata

For paginated documents (PDF, DOCX, PPTX), each chunk includes:

```json
{
  "file_path": "document.pdf",
  "chunk_index": 0,
  "file_extension": "pdf",
  "converted_via": "transmutation",
  "source_format": "pdf",
  "page_number": 1,
  "total_pages": 15
}
```

### Fallback Behavior

- If transmutation feature is **disabled**: Binary formats are skipped
- If conversion **fails**: File is skipped with a warning (no error thrown)
- If format is **unsupported**: Normal text extraction is used

## File Watcher Support

The file watcher automatically recognizes transmutation-supported formats when the feature is enabled. Supported formats are automatically added to the default include patterns:

```rust
// Default patterns (when transmutation feature is enabled)
include_patterns: [
    "*.md", "*.txt", "*.rs", // ... standard formats
    "*.pdf", "*.docx", "*.xlsx", "*.pptx", // Documents
    "*.html", "*.htm", "*.xml", // Web
    "*.jpg", "*.jpeg", "*.png" // Images
]
```

## Usage Examples

### Basic Indexing

```rust
use vectorizer::{VectorStore, document_loader::{DocumentLoader, LoaderConfig}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = VectorStore::new();
    
    let config = LoaderConfig {
        collection_name: "documents".to_string(),
        embedding_type: "bm25".to_string(),
        ..Default::default()
    };
    
    let mut loader = DocumentLoader::new(config);
    
    // This will automatically convert PDFs, DOCX, etc. when transmutation is enabled
    loader.load_project_async("./documents", &store).await?;
    
    Ok(())
}
```

### Querying with Page Metadata

```bash
# Search and filter by page number
curl -X POST http://localhost:15002/collections/documents/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning",
    "limit": 10,
    "filter": {
      "source_format": "pdf",
      "page_number": {"$gte": 1, "$lte": 5}
    }
  }'
```

## Performance

### Conversion Speed

Transmutation is designed for high performance:

- **PDF**: ~71 pages/second
- **DOCX**: ~98x faster than Docling
- **XLSX**: ~148 pages/second
- **PPTX**: ~1639 pages/second
- **HTML**: ~2110 pages/second

### Resource Usage

- **Memory**: ~20MB base + minimal per conversion (streaming)
- **CPU**: Single-threaded per file, parallelized across files
- **Disk**: Temporary files cleaned up automatically

## Troubleshooting

### Conversion Failures

If documents fail to convert:

1. **Check file integrity**: Ensure the file is not corrupted
2. **Verify dependencies**: Install required external tools (tesseract, poppler)
3. **Check file size**: Reduce `max_file_size_mb` if needed
4. **Increase timeout**: Adjust `conversion_timeout_secs` for large files
5. **Review logs**: Check vectorizer logs for detailed error messages

### Common Issues

**Issue**: PDF conversion fails with "Transmutation error"
- **Solution**: Install poppler-utils or reduce file size

**Issue**: Image OCR returns empty text
- **Solution**: Install tesseract-ocr and verify image quality

**Issue**: DOCX images not extracted
- **Solution**: Install LibreOffice (optional dependency)

**Issue**: Conversion timeout
- **Solution**: Increase `conversion_timeout_secs` in configuration

## Architecture

### Integration Points

1. **DocumentLoader** (`document_loader.rs`):
   - Main integration point for document conversion
   - Async collection with transmutation support
   - Fallback handling

2. **FileWatcher** (`file_watcher/config.rs`):
   - Auto-recognition of transmutation formats
   - Dynamic include patterns

3. **Configuration** (`config/vectorizer.rs`):
   - Global transmutation settings
   - Feature-gated configuration

4. **Error Handling** (`error.rs`):
   - `VectorizerError::TransmutationError` for conversion failures

### Module Structure

```
vectorizer/src/
├── transmutation_integration/
│   ├── mod.rs           # Main processor
│   ├── types.rs         # ConvertedDocument, PageInfo
│   └── tests.rs         # Integration tests
├── document_loader.rs   # Integration logic
├── file_watcher/
│   ├── config.rs        # Format recognition
│   └── operations.rs    # File processing
└── config/vectorizer.rs # Global configuration
```

## Testing

### Running Tests

```bash
# Run all tests with transmutation feature
cargo test --features transmutation

# Run transmutation-specific tests
cargo test --features transmutation transmutation_integration

# Test without feature (should fall back gracefully)
cargo test
```

### Test Coverage

- ✅ Format detection for all supported types
- ✅ Page metadata extraction
- ✅ Feature flag compilation (enabled/disabled)
- ✅ Error handling and fallback behavior
- ✅ Integration with DocumentLoader

## Roadmap

### Future Enhancements

- [ ] Audio transcription support (MP3, WAV, M4A)
- [ ] Video transcription support (MP4, AVI, MKV)
- [ ] Archive extraction (ZIP, TAR, GZ)
- [ ] Parallel document conversion
- [ ] Conversion result caching
- [ ] Custom conversion pipelines
- [ ] Streaming conversion for large files

## References

- [Transmutation Documentation](https://docs.rs/transmutation)
- [Transmutation GitHub](https://github.com/hivellm/transmutation)
- [Transmutation Crates.io](https://crates.io/crates/transmutation)
- [Vectorizer Documentation](../README.md)

## License

This integration follows the same MIT license as Vectorizer and Transmutation.

