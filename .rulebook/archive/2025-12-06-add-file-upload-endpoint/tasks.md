## 1. Configuration

- [x] 1.1 Add FileUploadConfig struct to src/config/vectorizer.rs
- [x] 1.2 Add file_upload section to config.yml with max_file_size (10MB default)
- [x] 1.3 Add allowed_extensions list (text and code file extensions)
- [x] 1.4 Add reject_binary flag (default: true)
- [x] 1.5 Load and validate config on server startup

## 2. File Type Validation

- [x] 2.1 Create src/server/file_validation.rs
- [x] 2.2 Implement MIME type detection
- [x] 2.3 Implement file extension validation against allowed list
- [x] 2.4 Implement binary file detection and rejection
- [x] 2.5 Add validation error messages

## 3. REST API Endpoint

- [x] 3.1 Create src/server/file_upload_handlers.rs
- [x] 3.2 Implement POST /files/upload handler
- [x] 3.3 Parse multipart/form-data request
- [x] 3.4 Extract file and metadata (collection_name, chunk_size, etc.)
- [x] 3.5 Validate file size against config
- [x] 3.6 Validate file type (extension + MIME type)
- [x] 3.7 Process file through FileLoader pipeline
- [x] 3.8 Return indexing results (vectors_created, chunks_created, etc.)
- [x] 3.9 Add error handling for validation failures
- [x] 3.10 Add route to src/server/mod.rs

## 4. GraphQL Mutation

- [x] 4.1 Add UploadFileInput type to src/api/graphql/types.rs
- [x] 4.2 Add GqlFileUploadResult type to src/api/graphql/types.rs
- [x] 4.3 Add uploadFile mutation to src/api/graphql/schema.rs
- [x] 4.4 Implement mutation handler (reuse REST logic)
- [x] 4.5 Support base64 file encoding
- [ ] 4.6 Support multipart file upload (GraphQL multipart spec - optional)
- [x] 4.7 Add GraphQL error handling

## 5. Integration with File Processing

- [x] 5.1 Create temporary file from upload
- [x] 5.2 Use FileLoader to process file
- [x] 5.3 Use Chunker for text chunking
- [x] 5.4 Use Indexer for vector creation
- [x] 5.5 Clean up temporary files after processing
- [x] 5.6 Handle processing errors gracefully

## 6. SDK Updates

- [x] 6.1 Update Python SDK (sdks/python/)
  - [x] 6.1.1 Add upload_file() method
  - [x] 6.1.2 Add upload_file_content() method
  - [x] 6.1.3 Add get_upload_config() method
  - [x] 6.1.4 Add FileUploadRequest, FileUploadResponse, FileUploadConfig models
  - [x] 6.1.5 Update __init__.py with new exports
- [x] 6.2 Update TypeScript SDK (sdks/typescript/)
  - [x] 6.2.1 Add uploadFile() method
  - [x] 6.2.2 Add uploadFileContent() method
  - [x] 6.2.3 Add getUploadConfig() method
  - [x] 6.2.4 Add FileUploadResponse, FileUploadConfig, UploadFileOptions interfaces
  - [x] 6.2.5 Add postFormData() to ITransport interface
  - [x] 6.2.6 Implement postFormData() in HttpClient
  - [x] 6.2.7 Implement postFormData() in UMICPClient
  - [x] 6.2.8 Update index.ts with new exports
- [x] 6.3 Update JavaScript SDK (sdks/javascript/)
  - [x] 6.3.1 Add uploadFile() method
  - [x] 6.3.2 Add uploadFileContent() method
  - [x] 6.3.3 Add getUploadConfig() method
  - [x] 6.3.4 Add file-upload.js model with types
  - [x] 6.3.5 Add postFormData() to http-client.js
  - [x] 6.3.6 Add postFormData() to umicp-client.js
  - [x] 6.3.7 Update models/index.js exports

## 7. OpenAPI Documentation

- [x] 7.1 Update docs/api/openapi.yaml
- [x] 7.2 Add /files/upload endpoint schema
- [x] 7.3 Document multipart/form-data request format
- [x] 7.4 Document response schema (FileUploadResponse)
- [x] 7.5 Add FileUploadConfig schema
- [x] 7.6 Add /files/config endpoint schema
- [x] 7.7 Add File Upload tag to tags list

## 8. Testing

- [x] 8.1 Add unit tests for file validation (22 tests)
- [x] 8.2 Add integration tests for REST endpoint (6 tests)
- [x] 8.3 Add integration tests for GraphQL mutation
- [x] 8.4 Test file size limit enforcement
- [x] 8.5 Test file type rejection (binary files)
- [x] 8.6 Test file type acceptance (text and code files)
- [x] 8.7 Test chunking and indexing pipeline
- [x] 8.8 Test error handling (invalid files, missing collection, etc.)
- [x] 8.9 Add unit tests for FileUploadConfig (11 tests)
- [x] 8.10 Add unit tests for file_upload_handlers (10 tests)

## 9. Documentation

- [x] 9.1 Update docs/users/api/API_REFERENCE.md with upload endpoint
- [x] 9.2 Update docs/users/api/GRAPHQL.md with uploadFile mutation
- [x] 9.3 Add file upload examples to documentation (SDK examples in API_REFERENCE.md)
- [x] 9.4 Document configuration options (config.yml section in API_REFERENCE.md)
- [x] 9.5 Add troubleshooting section for common issues

---

## Implementation Summary

### Completed (43 tests total):

**Configuration:**
- `FileUploadConfig` struct in `src/config/vectorizer.rs`
- `file_upload` section in `config.yml`

**Core Files Created:**
- `src/server/file_validation.rs` - File validation with extension, size, and binary detection
- `src/server/file_upload_handlers.rs` - REST handlers for file upload

**REST API:**
- `POST /files/upload` - Multipart file upload with chunking and embedding
- `GET /files/config` - Get upload configuration

**GraphQL:**
- `mutation uploadFile(input: UploadFileInput!)` - Upload file via base64
- `query fileUploadConfig` - Get upload configuration

**Tests:**
- `tests/api/rest/file_upload.rs` - Integration tests (6 tests)
- Unit tests in `file_validation.rs` (22 tests)
- Unit tests in `file_upload_handlers.rs` (10 tests)
- Unit tests in `vectorizer.rs` for FileUploadConfig (11 tests)

**SDKs:**
- Python SDK: `upload_file()`, `upload_file_content()`, `get_upload_config()` methods
- TypeScript SDK: `uploadFile()`, `uploadFileContent()`, `getUploadConfig()` methods
- JavaScript SDK: `uploadFile()`, `uploadFileContent()`, `getUploadConfig()` methods
- All SDKs have models for FileUploadRequest, FileUploadResponse, FileUploadConfig

**Documentation:**
- OpenAPI spec updated with `/files/upload` and `/files/config` endpoints
- API Reference updated with File Upload Endpoints section
