# Proposal: add-file-upload-endpoint

## Why

Currently, users must manually add files to a workspace directory and wait for the file watcher to index them, or use the workspace API to add directories. This creates friction for users who want to quickly index individual files:

1. **Workflow Friction**: Users must copy files to workspace, configure workspace, wait for file watcher
2. **No Direct Upload**: Cannot upload files directly via API for quick indexing
3. **Manual Process**: Requires filesystem access and manual file management
4. **SDK Limitation**: SDKs cannot provide file upload functionality

**Current Behavior**:
- Files must be added to workspace directories
- File watcher monitors directories for changes
- No direct file upload endpoint
- No programmatic way to upload and index a single file

**Problem Scenarios**:
- User wants to upload a code file and index it immediately
- SDK user wants to upload documents programmatically
- User wants to index files without filesystem access
- Need to index files from web applications or CI/CD pipelines

## What Changes

### 1. File Upload REST Endpoint

Add `POST /files/upload` endpoint that:
- Accepts multipart/form-data file upload
- Accepts metadata (collection_name, chunk_size, chunk_overlap, etc.)
- Validates file type (text and code files only)
- Validates file size (configurable max, default 10MB)
- Processes file through existing chunking and indexing pipeline
- Returns indexing results (vectors created, chunks created, etc.)

### 2. File Upload GraphQL Mutation

Add `uploadFile` mutation to GraphQL API:
- Accepts file as base64 or multipart
- Same validation and processing as REST endpoint
- Returns structured response with indexing statistics

### 3. Configuration

Add file upload configuration to `config.yml`:
- `file_upload.max_file_size`: Maximum file size in bytes (default: 10MB)
- `file_upload.allowed_extensions`: List of allowed file extensions (text and code files)
- `file_upload.reject_binary`: Boolean to reject binary files (default: true)

### 4. File Type Validation

Implement strict file type validation:
- **Allowed**: Text files (.txt, .md, .rst, etc.)
- **Allowed**: Code files (.rs, .py, .js, .ts, .go, .java, .cpp, .c, .h, etc.)
- **Rejected**: Binary files (.pdf, .docx, .xlsx, images, etc.)
- **Rejected**: Executables (.exe, .bin, etc.)
- Use MIME type detection and file extension validation

### 5. Integration with Existing Pipeline

Reuse existing file processing infrastructure:
- Use `FileLoader` for file processing
- Use `Chunker` for text chunking
- Use `Indexer` for vector creation and storage
- Use existing embedding providers
- Support all existing chunking strategies

### 6. SDK Updates

Update all SDKs (Python, TypeScript, JavaScript) to include:
- File upload methods
- File upload examples
- Error handling for file validation failures

### 7. OpenAPI Documentation

Update OpenAPI specification:
- Add `/files/upload` endpoint documentation
- Document request/response schemas
- Document file size and type constraints
- Add examples for different file types

## Impact

- **Affected code**:
  - New `src/server/file_upload_handlers.rs` - REST file upload handler
  - Modified `src/api/graphql/schema.rs` - Add uploadFile mutation
  - Modified `src/api/graphql/types.rs` - Add upload file types
  - Modified `src/models/config.rs` - Add file upload config
  - Modified `config.yml` - Add file upload settings
  - Modified `sdks/` - Update all SDKs
  - Modified `docs/api/openapi.yaml` - Update OpenAPI spec
- **Breaking change**: NO - New feature, backward compatible
- **User benefit**: Direct file upload and indexing without filesystem access
