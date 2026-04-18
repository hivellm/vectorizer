# File Upload API Specification (Vectorizer)

## ADDED Requirements

### Requirement: File Upload Configuration
The system SHALL provide configuration for file upload limits and allowed file types.

#### Scenario: Default configuration
Given the system starts with default configuration
When file upload is requested
Then the system SHALL enforce 10MB maximum file size
And the system SHALL only accept text and code file extensions
And the system SHALL reject binary files

#### Scenario: Custom configuration
Given file_upload.max_file_size is set to 20MB in config.yml
When file upload is requested
Then the system SHALL enforce 20MB maximum file size

### Requirement: File Type Validation
The system SHALL validate uploaded files and reject invalid file types.

#### Scenario: Valid text file upload
Given a user uploads a .txt file
When the file is validated
Then the system SHALL accept the file
And the system SHALL process it for indexing

#### Scenario: Valid code file upload
Given a user uploads a .rs, .py, .js, or other code file
When the file is validated
Then the system SHALL accept the file
And the system SHALL process it for indexing

#### Scenario: Binary file rejection
Given a user uploads a .pdf, .docx, or other binary file
When the file is validated
Then the system SHALL reject the file
And the system SHALL return 400 Bad Request with error message

#### Scenario: File size limit exceeded
Given a user uploads a file larger than max_file_size
When the file is validated
Then the system SHALL reject the file
And the system SHALL return 413 Payload Too Large with error message

### Requirement: REST File Upload Endpoint
The system SHALL provide a REST endpoint for file upload and indexing.

#### Scenario: Successful file upload
Given a user uploads a valid file via POST /files/upload
When the file is processed
Then the system SHALL chunk the file content
And the system SHALL create embeddings for each chunk
And the system SHALL store vectors in the specified collection
And the system SHALL return indexing statistics (vectors_created, chunks_created)

#### Scenario: File upload with metadata
Given a user uploads a file with collection_name, chunk_size, and chunk_overlap
When the file is processed
Then the system SHALL use the provided collection_name
And the system SHALL use the provided chunk_size and chunk_overlap for chunking

#### Scenario: Missing collection
Given a user uploads a file without specifying collection_name
When the file is processed
Then the system SHALL return 400 Bad Request with error message

#### Scenario: Invalid multipart request
Given a user sends an invalid multipart/form-data request
When the request is processed
Then the system SHALL return 400 Bad Request with error message

### Requirement: GraphQL File Upload Mutation
The system SHALL provide a GraphQL mutation for file upload and indexing.

#### Scenario: Upload file via GraphQL
Given a user calls uploadFile mutation with file data
When the mutation is processed
Then the system SHALL validate the file
And the system SHALL process it through the same pipeline as REST endpoint
And the system SHALL return UploadFileResponse with indexing statistics

#### Scenario: Base64 file encoding
Given a user provides file as base64 string in GraphQL mutation
When the mutation is processed
Then the system SHALL decode the base64 data
And the system SHALL process it as a file

### Requirement: File Processing Pipeline
The system SHALL process uploaded files through the existing chunking and indexing pipeline.

#### Scenario: Text file chunking
Given a text file is uploaded
When the file is processed
Then the system SHALL use Chunker to split content into chunks
And the system SHALL use configured chunk_size and chunk_overlap
And the system SHALL preserve file metadata in chunk payloads

#### Scenario: Code file chunking
Given a code file is uploaded
When the file is processed
Then the system SHALL use Chunker to split content into chunks
And the system SHALL preserve code structure in chunk payloads
And the system SHALL include file path and language in metadata

#### Scenario: Vector creation
Given chunks are created from uploaded file
When vectors are created
Then the system SHALL use EmbeddingManager to create embeddings
And the system SHALL store vectors with file metadata in payload
And the system SHALL use the specified collection

### Requirement: Error Handling
The system SHALL handle errors gracefully and return appropriate error responses.

#### Scenario: File validation error
Given a file fails validation (wrong type, too large, etc.)
When the upload is processed
Then the system SHALL return appropriate HTTP status code
And the system SHALL return error message describing the issue

#### Scenario: Processing error
Given file processing fails (chunking error, embedding error, etc.)
When the upload is processed
Then the system SHALL return 500 Internal Server Error
And the system SHALL return error message
And the system SHALL clean up temporary files

#### Scenario: Collection not found
Given a user uploads to a non-existent collection
When the upload is processed
Then the system SHALL return 404 Not Found
And the system SHALL return error message

