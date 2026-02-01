//! File upload REST API handlers
//!
//! Provides endpoints for direct file upload and indexing.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Extension;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, error, info, warn};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_bad_request_error, create_not_found_error};
use super::file_validation::{FileValidationError, FileValidator, ValidatedFile};
use crate::config::FileUploadConfig;
use crate::file_loader::chunker::Chunker;
use crate::file_loader::config::{DocumentChunk, LoaderConfig};
use crate::hub::middleware::RequestTenantContext;
use crate::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector,
};
#[cfg(feature = "transmutation")]
use crate::transmutation_integration::TransmutationProcessor;
use uuid::Uuid;

/// Request for file upload with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadRequest {
    /// Target collection name (required)
    pub collection_name: String,
    /// Chunk size in characters (optional, uses config default)
    pub chunk_size: Option<usize>,
    /// Chunk overlap in characters (optional, uses config default)
    pub chunk_overlap: Option<usize>,
    /// Additional metadata to attach to chunks
    pub metadata: Option<HashMap<String, Value>>,
}

/// Response for successful file upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadResponse {
    /// Whether the upload was successful
    pub success: bool,
    /// Original filename
    pub filename: String,
    /// Target collection
    pub collection_name: String,
    /// Number of chunks created
    pub chunks_created: usize,
    /// Number of vectors created
    pub vectors_created: usize,
    /// File size in bytes
    pub file_size: usize,
    /// Detected language/file type
    pub language: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Load file upload config from config.yml
/// Tries multiple paths to find config.yml
fn load_file_upload_config() -> FileUploadConfig {
    let possible_paths = vec![
        "./config.yml",
        "config.yml",
        "../config.yml",
    ];

    for path in &possible_paths {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_yaml::from_str::<crate::config::VectorizerConfig>(&content) {
                    Ok(config) => {
                        let max_size_mb = config.file_upload.max_file_size / (1024 * 1024);
                        info!("✅ Loaded file upload config from {}: max_file_size={}MB, allowed_extensions={}", 
                            path, max_size_mb, config.file_upload.allowed_extensions.len());
                        return config.file_upload;
                    }
                    Err(e) => {
                        warn!("Failed to parse config.yml from {}: {}", path, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                debug!("Failed to read config.yml from {}: {}", path, e);
                continue;
            }
        }
    }

    warn!("⚠️ Failed to load file upload config from any path, using defaults (10MB limit)");
    FileUploadConfig::default()
}

/// Handle file upload via multipart/form-data
///
/// POST /files/upload
///
/// **IMPORTANT:** Axum has a default 2MB limit for multipart requests.
/// For larger files, consider using the `/api/v1/insert` endpoint with
/// manual chunking, or configure a reverse proxy (nginx) to handle
/// larger uploads.
///
/// Multipart fields:
/// - file: The file to upload (required)
/// - collection_name: Target collection (required)
/// - chunk_size: Chunk size in characters (optional)
/// - chunk_overlap: Chunk overlap in characters (optional)
/// - metadata: JSON string with additional metadata (optional)
pub async fn upload_file(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, ErrorResponse> {
    let start_time = std::time::Instant::now();

    // Load config
    let upload_config = load_file_upload_config();
    let validator = FileValidator::new(upload_config.clone());

    // Parse multipart fields
    let mut file_data: Option<(String, Vec<u8>)> = None;
    let mut collection_name: Option<String> = None;
    let mut chunk_size: Option<usize> = None;
    let mut chunk_overlap: Option<usize> = None;
    let mut extra_metadata: Option<HashMap<String, Value>> = None;
    let mut public_key: Option<String> = None;
    let mut use_transmutation: bool = false;

    // Get max file size from config (use max_request_size_mb from state if available)
    let max_file_size_bytes = upload_config.max_file_size;
    let max_request_size_bytes = state.max_request_size_mb * 1024 * 1024;
    let effective_max_size = max_file_size_bytes.min(max_request_size_bytes);

    debug!(
        "File upload limits: config={}MB, request={}MB, effective={}MB",
        max_file_size_bytes / (1024 * 1024),
        state.max_request_size_mb,
        effective_max_size / (1024 * 1024)
    );

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            error!("Failed to parse multipart field: {}", e);
            // Provide more helpful error messages
            let error_msg = if e.to_string().contains("limit") || e.to_string().contains("too large") {
                format!(
                    "Request too large. Maximum file size is {}MB. Error: {}",
                    effective_max_size / (1024 * 1024),
                    e
                )
            } else {
                format!("Failed to parse multipart: {}", e)
            };
            create_bad_request_error(&error_msg)
        })?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .ok_or_else(|| create_bad_request_error("Missing filename"))?;

                debug!("Reading file field: {}", filename);

                // Read file data with size checking
                let data = field.bytes().await.map_err(|e| {
                    error!("Failed to read file bytes: {}", e);
                    let error_msg = if e.to_string().contains("limit") || e.to_string().contains("too large") {
                        format!(
                            "File too large. Maximum size is {}MB. Error: {}",
                            effective_max_size / (1024 * 1024),
                            e
                        )
                    } else {
                        format!("Failed to read file: {}", e)
                    };
                    create_bad_request_error(&error_msg)
                })?;

                // Check size before storing
                if data.len() > effective_max_size {
                    return Err(ErrorResponse::new(
                        "file_too_large".to_string(),
                        format!(
                            "File size {} bytes exceeds maximum of {} bytes ({}MB)",
                            data.len(),
                            effective_max_size,
                            effective_max_size / (1024 * 1024)
                        ),
                        StatusCode::PAYLOAD_TOO_LARGE,
                    ));
                }

                debug!("File read successfully: {} bytes", data.len());
                file_data = Some((filename, data.to_vec()));
            }
            "collection_name" => {
                let text = field.text().await.map_err(|e| {
                    create_bad_request_error(&format!("Failed to read collection_name: {}", e))
                })?;
                collection_name = Some(text);
            }
            "chunk_size" => {
                let text = field.text().await.map_err(|e| {
                    create_bad_request_error(&format!("Failed to read chunk_size: {}", e))
                })?;
                chunk_size = text.parse().ok();
            }
            "chunk_overlap" => {
                let text = field.text().await.map_err(|e| {
                    create_bad_request_error(&format!("Failed to read chunk_overlap: {}", e))
                })?;
                chunk_overlap = text.parse().ok();
            }
            "metadata" => {
                let text = field.text().await.map_err(|e| {
                    create_bad_request_error(&format!("Failed to read metadata: {}", e))
                })?;
                if let Ok(parsed) = serde_json::from_str::<HashMap<String, Value>>(&text) {
                    extra_metadata = Some(parsed);
                }
            }
            "public_key" => {
                let text = field.text().await.map_err(|e| {
                    create_bad_request_error(&format!("Failed to read public_key: {}", e))
                })?;
                public_key = Some(text);
            }
            "use_transmutation" => {
                let text = field.text().await.map_err(|e| {
                    create_bad_request_error(&format!("Failed to read use_transmutation: {}", e))
                })?;
                use_transmutation = text.parse().unwrap_or(false);
            }
            _ => {
                debug!("Ignoring unknown field: {}", field_name);
            }
        }
    }

    // Validate required fields
    let (filename, file_bytes) =
        file_data.ok_or_else(|| create_bad_request_error("Missing file in multipart request"))?;

    let collection_name = collection_name
        .ok_or_else(|| create_bad_request_error("Missing collection_name parameter"))?;

    // Apply tenant prefix if in hub mode
    let collection_name = if let Some(Extension(ref ctx)) = tenant_ctx {
        format!("user_{}_{}", ctx.0.tenant_id, collection_name)
    } else {
        collection_name
    };

    // Validate file
    let validated_file = validator
        .validate(&filename, &file_bytes)
        .map_err(|e| match e {
            FileValidationError::ExtensionNotAllowed(ext) => ErrorResponse::new(
                "extension_not_allowed".to_string(),
                format!("File extension '{}' is not allowed", ext),
                StatusCode::BAD_REQUEST,
            ),
            FileValidationError::FileTooLarge(size, max) => ErrorResponse::new(
                "file_too_large".to_string(),
                format!("File size {} bytes exceeds maximum of {} bytes", size, max),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            FileValidationError::BinaryFileRejected => ErrorResponse::new(
                "binary_file_rejected".to_string(),
                "Binary files are not allowed".to_string(),
                StatusCode::BAD_REQUEST,
            ),
            FileValidationError::MissingExtension => ErrorResponse::new(
                "missing_extension".to_string(),
                "File is missing extension".to_string(),
                StatusCode::BAD_REQUEST,
            ),
            FileValidationError::InvalidFileName => ErrorResponse::new(
                "invalid_filename".to_string(),
                "Invalid file name".to_string(),
                StatusCode::BAD_REQUEST,
            ),
        })?;

    info!(
        "Processing file upload: {} ({} bytes, language: {}, encrypted: {})",
        validated_file.filename,
        validated_file.size,
        validated_file.language(),
        public_key.is_some()
    );

    // Check if collection exists, create if not
    if !state.store.has_collection_in_memory(&collection_name) {
        let config = CollectionConfig {
            dimension: 512, // BM25 default dimension
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::SQ { bits: 8 },
            compression: Default::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
            sharding: None,
            graph: None,
            encryption: None,
        };

        state
            .store
            .create_collection_with_quantization(&collection_name, config)
            .map_err(|e| {
                error!("Failed to create collection: {}", e);
                ErrorResponse::new(
                    "collection_creation_failed".to_string(),
                    format!("Failed to create collection: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        info!("Created new collection: {}", collection_name);
    }

    // Process file content - use transmutation if enabled and supported
    let file_path = PathBuf::from(&validated_file.filename);
    let mut content_to_chunk = validated_file.content.clone();
    let mut transmutation_metadata: HashMap<String, Value> = HashMap::new();

    #[cfg(feature = "transmutation")]
    {
        if use_transmutation && TransmutationProcessor::is_supported_format(&file_path) {
            info!("🔄 Using transmutation to convert: {}", validated_file.filename);
            
            // Create temporary file for transmutation
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!("vectorizer_upload_{}.{}", 
                Uuid::new_v4(), 
                file_path.extension().and_then(|e| e.to_str()).unwrap_or("tmp")));
            
            // Write file bytes to temp file
            if let Err(e) = std::fs::write(&temp_file, &file_bytes) {
                warn!("Failed to write temp file for transmutation: {}", e);
            } else {
                // Convert using transmutation
                match TransmutationProcessor::new() {
                    Ok(processor) => {
                        match processor.convert_to_markdown(&temp_file).await {
                            Ok(converted_doc) => {
                                content_to_chunk = converted_doc.content;
                                
                                // Add transmutation metadata
                                transmutation_metadata.insert(
                                    "converted_via".to_string(),
                                    json!("transmutation")
                                );
                                
                                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                                    transmutation_metadata.insert(
                                        "source_format".to_string(),
                                        json!(ext.to_lowercase())
                                    );
                                }
                                
                                // Add page count if available
                                if let Some(page_count) = converted_doc.metadata.get("page_count") {
                                    transmutation_metadata.insert(
                                        "page_count".to_string(),
                                        json!(page_count)
                                    );
                                }
                                
                                // Add other metadata from conversion
                                for (key, value) in converted_doc.metadata {
                                    if !transmutation_metadata.contains_key(&key) {
                                        transmutation_metadata.insert(key, json!(value));
                                    }
                                }
                                
                                info!("✅ Transmutation conversion successful: {} characters", content_to_chunk.len());
                            }
                            Err(e) => {
                                warn!("Transmutation conversion failed: {}, using original content", e);
                                // Fall back to original content
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to initialize transmutation processor: {}, using original content", e);
                        // Fall back to original content
                    }
                }
                
                // Clean up temp file
                if let Err(e) = std::fs::remove_file(&temp_file) {
                    debug!("Failed to remove temp file: {}", e);
                }
            }
        }
    }

    // Create chunks using the file loader chunker
    let loader_config = LoaderConfig {
        max_chunk_size: chunk_size.unwrap_or(upload_config.default_chunk_size),
        chunk_overlap: chunk_overlap.unwrap_or(upload_config.default_chunk_overlap),
        include_patterns: vec![],
        exclude_patterns: vec![],
        embedding_dimension: 512,
        embedding_type: "bm25".to_string(),
        collection_name: collection_name.clone(),
        max_file_size: upload_config.max_file_size,
    };

    let chunker = Chunker::new(loader_config);

    let chunks = chunker
        .chunk_text(&content_to_chunk, &file_path)
        .map_err(|e| {
            error!("Failed to chunk file: {}", e);
            ErrorResponse::new(
                "chunking_failed".to_string(),
                format!("Failed to chunk file: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let chunks_created = chunks.len();

    // Extract values we need before any moves
    let filename = validated_file.filename.clone();
    let file_size = validated_file.size;
    let file_extension = validated_file.extension.clone();
    let language = validated_file.language().to_string();

    if chunks_created == 0 {
        return Ok(Json(FileUploadResponse {
            success: true,
            filename,
            collection_name,
            chunks_created: 0,
            vectors_created: 0,
            file_size,
            language,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        }));
    }

    // Create embeddings and store vectors
    let mut vectors_created = 0;

    for chunk in &chunks {
        // Create embedding using the embedding manager
        let embedding = match state.embedding_manager.embed(&chunk.content) {
            Ok(emb) => emb,
            Err(e) => {
                warn!("Failed to embed chunk: {}", e);
                continue;
            }
        };

        // Skip zero vectors
        if embedding.iter().all(|&x| x == 0.0) {
            continue;
        }

        // Build payload with metadata
        let mut payload_data = json!({
            "content": chunk.content,
            "file_path": chunk.file_path,
            "chunk_index": chunk.chunk_index,
            "language": &language,
            "source": "file_upload",
            "original_filename": &filename,
            "file_extension": &file_extension,
        });

        // Merge chunk metadata
        if let Some(obj) = payload_data.as_object_mut() {
            for (k, v) in &chunk.metadata {
                obj.insert(k.clone(), v.clone());
            }

            // Merge transmutation metadata if available
            for (k, v) in &transmutation_metadata {
                obj.insert(k.clone(), v.clone());
            }

            // Merge extra metadata if provided
            if let Some(ref extra) = extra_metadata {
                for (k, v) in extra {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        // Normalize and optionally encrypt payload
        let mut payload_value = payload_data;
        if let Some(obj) = payload_value.as_object_mut() {
            // Normalize values
            for (_k, v) in obj.iter_mut() {
                if let Some(s) = v.as_str() {
                    *v = json!(s.to_lowercase());
                }
            }
        }

        // Encrypt payload if public_key is provided
        let payload = if let Some(ref key) = public_key {
            let encrypted =
                match crate::security::payload_encryption::encrypt_payload(&payload_value, key) {
                    Ok(enc) => enc,
                    Err(e) => {
                        warn!("Failed to encrypt payload: {}", e);
                        continue;
                    }
                };
            Payload::from_encrypted(encrypted)
        } else {
            Payload {
                data: payload_value,
            }
        };

        let vector = Vector {
            id: Uuid::new_v4().to_string(),
            data: embedding,
            sparse: None,
            payload: Some(payload),
        };

        // Insert vector
        if let Err(e) = state.store.insert(&collection_name, vec![vector]) {
            warn!("Failed to insert vector: {}", e);
            continue;
        }

        vectors_created += 1;
    }

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    info!(
        "File upload completed: {} - {} chunks, {} vectors, {}ms",
        filename, chunks_created, vectors_created, processing_time_ms
    );

    Ok(Json(FileUploadResponse {
        success: true,
        filename,
        collection_name,
        chunks_created,
        vectors_created,
        file_size,
        language,
        processing_time_ms,
    }))
}

/// Get file upload configuration
///
/// GET /files/config
pub async fn get_upload_config(State(_state): State<VectorizerServer>) -> Json<Value> {
    let config = load_file_upload_config();

    Json(json!({
        "max_file_size": config.max_file_size,
        "max_file_size_mb": config.max_file_size / (1024 * 1024),
        "allowed_extensions": config.allowed_extensions,
        "reject_binary": config.reject_binary,
        "default_chunk_size": config.default_chunk_size,
        "default_chunk_overlap": config.default_chunk_overlap,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_upload_response_serialization() {
        let response = FileUploadResponse {
            success: true,
            filename: "test.rs".to_string(),
            collection_name: "test-collection".to_string(),
            chunks_created: 5,
            vectors_created: 5,
            file_size: 1024,
            language: "rust".to_string(),
            processing_time_ms: 100,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("test.rs"));
        assert!(json.contains("rust"));
    }

    #[test]
    fn test_file_upload_response_deserialization() {
        let json = r#"{
            "success": true,
            "filename": "example.py",
            "collection_name": "python-docs",
            "chunks_created": 10,
            "vectors_created": 10,
            "file_size": 2048,
            "language": "python",
            "processing_time_ms": 50
        }"#;

        let response: FileUploadResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.filename, "example.py");
        assert_eq!(response.collection_name, "python-docs");
        assert_eq!(response.chunks_created, 10);
        assert_eq!(response.vectors_created, 10);
        assert_eq!(response.file_size, 2048);
        assert_eq!(response.language, "python");
        assert_eq!(response.processing_time_ms, 50);
    }

    #[test]
    fn test_file_upload_response_zero_chunks() {
        let response = FileUploadResponse {
            success: true,
            filename: "empty.txt".to_string(),
            collection_name: "test".to_string(),
            chunks_created: 0,
            vectors_created: 0,
            file_size: 0,
            language: "plaintext".to_string(),
            processing_time_ms: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: FileUploadResponse = serde_json::from_str(&json).unwrap();

        assert!(parsed.success);
        assert_eq!(parsed.chunks_created, 0);
        assert_eq!(parsed.vectors_created, 0);
    }

    #[test]
    fn test_load_file_upload_config_default() {
        let config = FileUploadConfig::default();

        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert!(config.reject_binary);
        assert!(!config.allowed_extensions.is_empty());
        assert_eq!(config.default_chunk_size, 2048);
        assert_eq!(config.default_chunk_overlap, 256);
    }

    #[test]
    fn test_file_upload_config_default_extensions() {
        let config = FileUploadConfig::default();

        // Check common text file extensions
        assert!(config.allowed_extensions.contains(&"txt".to_string()));
        assert!(config.allowed_extensions.contains(&"md".to_string()));

        // Check common code file extensions
        assert!(config.allowed_extensions.contains(&"rs".to_string()));
        assert!(config.allowed_extensions.contains(&"py".to_string()));
        assert!(config.allowed_extensions.contains(&"js".to_string()));
        assert!(config.allowed_extensions.contains(&"ts".to_string()));
        assert!(config.allowed_extensions.contains(&"go".to_string()));
        assert!(config.allowed_extensions.contains(&"java".to_string()));

        // Check config file extensions
        assert!(config.allowed_extensions.contains(&"json".to_string()));
        assert!(config.allowed_extensions.contains(&"yaml".to_string()));
        assert!(config.allowed_extensions.contains(&"toml".to_string()));

        // Check web file extensions
        assert!(config.allowed_extensions.contains(&"html".to_string()));
        assert!(config.allowed_extensions.contains(&"css".to_string()));
    }

    #[test]
    fn test_file_upload_config_no_binary_extensions() {
        let config = FileUploadConfig::default();

        // Binary extensions should NOT be in the allowed list
        assert!(!config.allowed_extensions.contains(&"exe".to_string()));
        assert!(!config.allowed_extensions.contains(&"dll".to_string()));
        assert!(!config.allowed_extensions.contains(&"bin".to_string()));
        assert!(!config.allowed_extensions.contains(&"so".to_string()));
        assert!(!config.allowed_extensions.contains(&"png".to_string()));
        assert!(!config.allowed_extensions.contains(&"jpg".to_string()));
        assert!(!config.allowed_extensions.contains(&"pdf".to_string()));
        assert!(!config.allowed_extensions.contains(&"zip".to_string()));
    }

    #[test]
    fn test_file_upload_request_serialization() {
        let request = FileUploadRequest {
            collection_name: "my-collection".to_string(),
            chunk_size: Some(1024),
            chunk_overlap: Some(128),
            metadata: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("my-collection"));
        assert!(json.contains("1024"));
        assert!(json.contains("128"));
    }

    #[test]
    fn test_file_upload_request_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("project".to_string(), serde_json::json!("test-project"));
        metadata.insert("version".to_string(), serde_json::json!("1.0.0"));
        metadata.insert("tags".to_string(), serde_json::json!(["doc", "api"]));

        let request = FileUploadRequest {
            collection_name: "docs".to_string(),
            chunk_size: None,
            chunk_overlap: None,
            metadata: Some(metadata),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test-project"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_file_upload_request_deserialization() {
        let json = r#"{
            "collection_name": "code-collection",
            "chunk_size": 512,
            "chunk_overlap": 64
        }"#;

        let request: FileUploadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.collection_name, "code-collection");
        assert_eq!(request.chunk_size, Some(512));
        assert_eq!(request.chunk_overlap, Some(64));
        assert!(request.metadata.is_none());
    }

    #[test]
    fn test_file_upload_request_minimal() {
        let json = r#"{"collection_name": "minimal"}"#;

        let request: FileUploadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.collection_name, "minimal");
        assert!(request.chunk_size.is_none());
        assert!(request.chunk_overlap.is_none());
        assert!(request.metadata.is_none());
    }
}
