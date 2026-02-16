# File Upload with Transmutation - API Documentation

## Overview

The Vectorizer `/files/upload` endpoint supports automatic document conversion using transmutation for PDF, DOCX, XLSX, PPTX, HTML, XML, and image formats.

## Endpoint

**POST** `/files/upload`

## Request Format

**Content-Type:** `multipart/form-data`

## Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `file` | File | The file to upload (required) |
| `collection_name` | String | Target collection name (required) |

## Optional Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `chunk_size` | Integer | Chunk size in characters | 2048 |
| `chunk_overlap` | Integer | Chunk overlap in characters | 256 |
| `metadata` | JSON String | Additional metadata as JSON string | `null` |
| `use_transmutation` | Boolean String | Enable transmutation conversion (`"true"` or `"false"`) | `false` |
| `public_key` | String | Public key for payload encryption | `null` |

## Transmutation Support

### Supported Formats

When `use_transmutation=true`, the following formats are automatically converted to Markdown:

- **Documents**: PDF, DOCX, XLSX, PPTX
- **Web**: HTML, HTM, XML
- **Images**: JPG, JPEG, PNG, TIFF, TIF, BMP, GIF, WEBP (requires OCR)

### How It Works

1. If `use_transmutation=true` and the file format is supported:
   - File is temporarily saved
   - Converted to Markdown using transmutation
   - Markdown content is used for chunking instead of raw file content
   - Metadata includes conversion info (page count, source format, etc.)

2. If `use_transmutation=false` or format not supported:
   - File is processed as plain text (for text files)
   - Binary files are rejected unless `reject_binary=false` in config

## Example: Go Backend Implementation

```go
// internal/vectorizer/client.go

func (c *Client) UploadFile(ctx context.Context, collectionName string, file io.Reader, filename string, useTransmutation bool) error {
    // Create multipart form
    var buf bytes.Buffer
    writer := multipart.NewWriter(&buf)
    
    // Add file
    fileWriter, err := writer.CreateFormFile("file", filename)
    if err != nil {
        return fmt.Errorf("failed to create form file: %w", err)
    }
    if _, err := io.Copy(fileWriter, file); err != nil {
        return fmt.Errorf("failed to copy file: %w", err)
    }
    
    // Add collection_name
    if err := writer.WriteField("collection_name", collectionName); err != nil {
        return fmt.Errorf("failed to write collection_name: %w", err)
    }
    
    // ✅ IMPORTANT: Add use_transmutation field
    if err := writer.WriteField("use_transmutation", strconv.FormatBool(useTransmutation)); err != nil {
        return fmt.Errorf("failed to write use_transmutation: %w", err)
    }
    
    // Optional: Add metadata
    if metadata != nil {
        metadataJSON, _ := json.Marshal(metadata)
        writer.WriteField("metadata", string(metadataJSON))
    }
    
    writer.Close()
    
    // Create request
    req, err := http.NewRequestWithContext(ctx, "POST", c.baseURL+"/files/upload", &buf)
    if err != nil {
        return fmt.Errorf("failed to create request: %w", err)
    }
    
    req.Header.Set("Content-Type", writer.FormDataContentType())
    
    // Add authentication headers
    if c.apiKey != "" {
        req.Header.Set("X-API-Key", c.apiKey)
    } else if c.jwtToken != "" {
        req.Header.Set("Authorization", "Bearer "+c.jwtToken)
    }
    
    // Add HiveHub headers if applicable
    if c.tenantID != "" {
        req.Header.Set("X-HiveHub-User-ID", c.tenantID)
    }
    if c.serviceName != "" {
        req.Header.Set("X-HiveHub-Service", c.serviceName)
    }
    
    // Execute request
    resp, err := c.httpClient.Do(req)
    if err != nil {
        return fmt.Errorf("failed to execute request: %w", err)
    }
    defer resp.Body.Close()
    
    if resp.StatusCode != http.StatusOK {
        body, _ := io.ReadAll(resp.Body)
        return fmt.Errorf("upload failed: %s", string(body))
    }
    
    return nil
}
```

## Example: Handler Implementation

```go
// internal/vectorizer/handler.go

func (h *Handler) UploadFile(c *fiber.Ctx) error {
    collectionName := c.Params("name")
    
    // Get file from multipart form
    file, err := c.FormFile("file")
    if err != nil {
        return c.Status(400).JSON(fiber.Map{
            "error": "missing file",
        })
    }
    
    // Get use_transmutation flag (default: auto-detect based on file extension)
    useTransmutation := c.FormValue("use_transmutation") == "true"
    
    // Auto-enable transmutation for supported formats if not explicitly set
    if c.FormValue("use_transmutation") == "" {
        ext := strings.ToLower(filepath.Ext(file.Filename))
        supportedFormats := []string{".pdf", ".docx", ".xlsx", ".pptx", ".html", ".htm", ".xml", 
            ".jpg", ".jpeg", ".png", ".tiff", ".tif", ".bmp", ".gif", ".webp"}
        for _, format := range supportedFormats {
            if ext == format {
                useTransmutation = true
                break
            }
        }
    }
    
    // Open file
    src, err := file.Open()
    if err != nil {
        return c.Status(500).JSON(fiber.Map{
            "error": "failed to open file",
        })
    }
    defer src.Close()
    
    // Add tenant prefix to collection name
    tenantID := c.Get("X-HiveHub-User-ID")
    if tenantID != "" {
        collectionName = fmt.Sprintf("user_%s_%s", tenantID, collectionName)
    }
    
    // Forward to Vectorizer
    err = h.vectorizerClient.UploadFile(
        c.Context(),
        collectionName,
        src,
        file.Filename,
        useTransmutation, // ✅ Pass transmutation flag
    )
    if err != nil {
        return c.Status(500).JSON(fiber.Map{
            "error": err.Error(),
        })
    }
    
    return c.JSON(fiber.Map{
        "success": true,
        "message": "File uploaded successfully",
    })
}
```

## Response Format

```json
{
  "success": true,
  "filename": "document.pdf",
  "collection_name": "my-collection",
  "chunks_created": 15,
  "vectors_created": 15,
  "file_size": 24274156,
  "language": "markdown",
  "processing_time_ms": 1234
}
```

## Metadata Added by Transmutation

When transmutation is used, the following metadata is automatically added to each chunk:

```json
{
  "converted_via": "transmutation",
  "source_format": "pdf",
  "page_number": 1,
  "total_pages": 15
}
```

## Configuration

### File Size Limits

All limits must be set to 200MB in `config.yml`:

```yaml
api:
  rest:
    max_request_size_mb: 200

file_upload:
  max_file_size: 209715200  # 200MB in bytes

file_watcher:
  max_file_size_bytes: 209715200  # 200MB

transmutation:
  max_file_size_mb: 200
```

### Transmutation Feature

Transmutation is **enabled by default** in version 2.4.0+. No build flags needed.

If using an older version, compile with:
```bash
cargo build --release --features transmutation
```

## Best Practices

1. **Auto-detect transmutation**: Enable transmutation automatically for supported formats
2. **Error handling**: Handle cases where transmutation fails gracefully (falls back to text processing)
3. **File size validation**: Validate file size before sending to Vectorizer
4. **Progress tracking**: Use chunked uploads for very large files (>50MB)
5. **Metadata enrichment**: Add custom metadata to help with search and filtering

## Troubleshooting

### Transmutation not working

1. **Check build**: Ensure server was compiled with transmutation feature (default in v2.4.0+)
2. **Check format**: Verify file format is in supported list
3. **Check flag**: Ensure `use_transmutation` field is sent as `"true"` (string, not boolean)
4. **Check logs**: Look for `🔄 Using transmutation to convert:` in server logs

### File size errors

1. **Check all limits**: Ensure `api.rest.max_request_size_mb`, `file_upload.max_file_size`, and `transmutation.max_file_size_mb` are all set to 200MB
2. **Restart server**: Configuration changes require server restart
3. **Check effective limit**: Server uses the minimum of all limits

## Complete Flow Diagram

```
Frontend
  ↓
POST /api/v1/knowledge/collections/{name}/upload
  ↓
Backend Handler (Go)
  ├─ Extract file from multipart
  ├─ Detect file format
  ├─ Set use_transmutation flag
  └─ Add tenant prefix to collection name
  ↓
POST /files/upload (Vectorizer)
  ├─ Receives multipart with:
  │  ├─ file: <binary>
  │  ├─ collection_name: "user_{tenantID}_{name}"
  │  ├─ use_transmutation: "true" or "false"
  │  └─ metadata: <optional JSON>
  │
  ├─ If use_transmutation=true AND format supported:
  │  ├─ Save to temp file
  │  ├─ Convert to Markdown
  │  ├─ Extract metadata (pages, format, etc.)
  │  └─ Use Markdown for chunking
  │
  └─ If use_transmutation=false OR format not supported:
     └─ Process as plain text
  ↓
Chunking & Embedding
  ↓
Vector Storage
```
