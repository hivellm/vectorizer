---
title: Document Conversion API
module: api
id: document-conversion
order: 11
description: API for converting documents to markdown using the transmutation engine
tags: [api, documents, conversion, pdf, docx, markdown, transmutation]
---

# Document Conversion API

REST API endpoints for converting various document formats to Markdown using the transmutation engine.

## Overview

The Document Conversion API enables automatic conversion of documents to Markdown format optimized for LLM processing and vector embedding. It supports:

- PDF documents
- Microsoft Office formats (DOCX, XLSX, PPTX)
- Web formats (HTML, XML)
- Images with OCR (optional)

## Requirements

Document conversion requires the `transmutation` feature to be enabled:

```bash
# Build with transmutation support
cargo build --release --features transmutation

# Or with full features
cargo build --release --features full
```

## Endpoints

### Convert Document

Convert a document file to Markdown format.

```http
POST /documents/convert
```

#### Request

**Content-Type:** `multipart/form-data`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | file | Yes | Document file to convert |
| `split_pages` | boolean | No | Split by pages for paginated formats (default: true) |
| `optimize_for_llm` | boolean | No | Optimize output for LLM processing (default: true) |

#### Response

```json
{
  "content": "# Document Title\n\n--- Page 1 ---\n\nContent...",
  "pages": [
    {
      "page_number": 1,
      "start_char": 0,
      "end_char": 1500
    },
    {
      "page_number": 2,
      "start_char": 1501,
      "end_char": 3200
    }
  ],
  "metadata": {
    "source_format": "pdf",
    "converted_via": "transmutation",
    "page_count": "2",
    "title": "Document Title",
    "author": "Author Name",
    "language": "en",
    "input_size_bytes": "125000",
    "output_size_bytes": "3200",
    "conversion_duration_ms": "450",
    "tables_extracted": "3"
  }
}
```

#### Example

```bash
curl -X POST http://localhost:15002/documents/convert \
  -F "file=@document.pdf" \
  -F "split_pages=true"
```

### Check Format Support

Check if a file format is supported for conversion.

```http
GET /documents/formats
```

#### Response

```json
{
  "supported_formats": [
    {
      "extension": "pdf",
      "mime_type": "application/pdf",
      "description": "PDF Documents"
    },
    {
      "extension": "docx",
      "mime_type": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      "description": "Microsoft Word"
    },
    {
      "extension": "xlsx",
      "mime_type": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      "description": "Microsoft Excel"
    },
    {
      "extension": "pptx",
      "mime_type": "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      "description": "Microsoft PowerPoint"
    },
    {
      "extension": "html",
      "mime_type": "text/html",
      "description": "HTML Documents"
    },
    {
      "extension": "xml",
      "mime_type": "application/xml",
      "description": "XML Documents"
    }
  ],
  "ocr_supported": true
}
```

## Supported Formats

| Format | Extension | Description | Features |
|--------|-----------|-------------|----------|
| PDF | `.pdf` | Portable Document Format | Page splitting, tables, images |
| Word | `.docx` | Microsoft Word | Headings, tables, lists |
| Excel | `.xlsx` | Microsoft Excel | Tables, sheets |
| PowerPoint | `.pptx` | Microsoft PowerPoint | Slides as pages |
| HTML | `.html`, `.htm` | Web pages | Structure preservation |
| XML | `.xml` | XML documents | Element extraction |
| Images | `.jpg`, `.png`, `.tiff` | Image files | OCR text extraction |

## Conversion Features

### Page Splitting

For paginated formats (PDF, DOCX, PPTX), content is split by page with markers:

```markdown
--- Page 1 ---

First page content...

--- Page 2 ---

Second page content...
```

### Metadata Extraction

The converter extracts document metadata when available:

- **Title**: Document title
- **Author**: Document author
- **Language**: Document language
- **Page Count**: Number of pages
- **Tables Extracted**: Number of tables found

### LLM Optimization

When `optimize_for_llm=true`, the output is optimized for language model processing:

- Clean formatting without excessive whitespace
- Consistent heading structure
- Table formatting in markdown
- Preserved semantic structure

## Integration with File Upload

Document conversion is automatically applied during file upload when appropriate:

```bash
# Upload and convert PDF
curl -X POST http://localhost:15002/files/upload \
  -F "file=@document.pdf" \
  -F "collection=documents" \
  -F "convert=true"
```

## SDK Examples

### Python

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Convert a document
result = await client.convert_document(
    file_path="document.pdf",
    split_pages=True,
    optimize_for_llm=True
)

print(f"Converted {result['metadata']['page_count']} pages")
print(f"Content length: {len(result['content'])} characters")

# Access page information
for page in result['pages']:
    print(f"Page {page['page_number']}: chars {page['start_char']}-{page['end_char']}")
```

### TypeScript

```typescript
import { VectorizerClient } from 'vectorizer-sdk';

const client = new VectorizerClient('http://localhost:15002');

// Convert a document
const result = await client.convertDocument({
  file: await fs.readFile('document.pdf'),
  fileName: 'document.pdf',
  splitPages: true,
  optimizeForLlm: true
});

console.log(`Converted ${result.metadata.page_count} pages`);
```

## Error Responses

### 400 Bad Request

```json
{
  "error": "Unsupported file format",
  "code": "UNSUPPORTED_FORMAT",
  "supported": ["pdf", "docx", "xlsx", "pptx", "html"]
}
```

### 413 Payload Too Large

```json
{
  "error": "File too large",
  "code": "FILE_TOO_LARGE",
  "max_size_bytes": 104857600
}
```

### 500 Internal Server Error

```json
{
  "error": "Conversion failed",
  "code": "CONVERSION_ERROR",
  "details": "Failed to parse PDF structure"
}
```

### 501 Not Implemented

```json
{
  "error": "Transmutation feature is not enabled",
  "code": "FEATURE_DISABLED"
}
```

## Performance Considerations

1. **File Size**: Large documents take longer to convert. Consider chunking very large files.
2. **Page Count**: Processing time scales with page count for paginated formats.
3. **Tables**: Documents with many tables require additional processing.
4. **Images**: Image-heavy documents may be slower, especially with OCR.

## Best Practices

1. **Check Format Support**: Verify format is supported before uploading
2. **Use Split Pages**: Enable for paginated documents to preserve structure
3. **Enable LLM Optimization**: Always enable for vector embedding use cases
4. **Handle Errors**: Implement proper error handling for conversion failures
5. **Cache Results**: Cache converted documents to avoid re-processing

## Configuration

Configure document conversion in `config.yml`:

```yaml
documents:
  conversion:
    enabled: true
    max_file_size_mb: 100
    timeout_seconds: 300
    default_split_pages: true
    default_optimize_for_llm: true
```

## See Also

- [File Operations](./FILE_OPERATIONS.md) - File upload and management
- [Discovery API](./DISCOVERY.md) - Content discovery
- [Configuration](../configuration/CONFIGURATION.md) - Server configuration
