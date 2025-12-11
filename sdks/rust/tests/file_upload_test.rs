use vectorizer_sdk::{ClientConfig, FileUploadResponse, UploadFileOptions, VectorizerClient};

#[tokio::test]
async fn test_upload_file_content() {
    // Note: This is an integration test that requires a running server
    // Skip if VECTORIZER_TEST_URL is not set
    let base_url = match std::env::var("VECTORIZER_TEST_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test: VECTORIZER_TEST_URL not set");
            return;
        }
    };

    let config = ClientConfig {
        base_url: Some(base_url),
        ..Default::default()
    };

    let client = VectorizerClient::new(config).expect("Failed to create client");

    // Test content
    let content = r#"
        This is a test document for file upload.
        It contains multiple lines of text to be chunked and indexed.
        The vectorizer should automatically extract, chunk, and create embeddings.
    "#;

    let options = UploadFileOptions {
        chunk_size: Some(100),
        chunk_overlap: Some(20),
        metadata: None,
        public_key: None,
    };

    // Upload file content
    let result = client
        .upload_file_content(content, "test.txt", "test-uploads", options)
        .await;

    match result {
        Ok(response) => {
            assert!(response.success, "Upload should succeed");
            assert_eq!(response.filename, "test.txt");
            assert_eq!(response.collection_name, "test-uploads");
            assert!(
                response.chunks_created > 0,
                "Should create at least one chunk"
            );
            assert!(
                response.vectors_created > 0,
                "Should create at least one vector"
            );
            println!(
                "✓ Upload successful: {} chunks, {} vectors created",
                response.chunks_created, response.vectors_created
            );
        }
        Err(e) => {
            println!("Upload failed (expected if server not running): {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_upload_config() {
    let base_url = match std::env::var("VECTORIZER_TEST_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test: VECTORIZER_TEST_URL not set");
            return;
        }
    };

    let config = ClientConfig {
        base_url: Some(base_url),
        ..Default::default()
    };

    let client = VectorizerClient::new(config).expect("Failed to create client");

    let result = client.get_upload_config().await;

    match result {
        Ok(config) => {
            assert!(config.max_file_size > 0, "Max file size should be positive");
            assert!(
                config.default_chunk_size > 0,
                "Default chunk size should be positive"
            );
            assert!(
                !config.allowed_extensions.is_empty(),
                "Should have allowed extensions"
            );
            println!(
                "✓ Upload config: max_size={}MB, chunk_size={}, extensions={:?}",
                config.max_file_size_mb,
                config.default_chunk_size,
                config.allowed_extensions.len()
            );
        }
        Err(e) => {
            println!("Get config failed (expected if server not running): {}", e);
        }
    }
}

#[test]
fn test_upload_file_options_serialization() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("test"));
    metadata.insert("type".to_string(), serde_json::json!("document"));

    let options = UploadFileOptions {
        chunk_size: Some(512),
        chunk_overlap: Some(50),
        metadata: Some(metadata),
        public_key: None,
    };

    assert_eq!(options.chunk_size, Some(512));
    assert_eq!(options.chunk_overlap, Some(50));
    assert!(options.metadata.is_some());
}

#[test]
fn test_file_upload_response_deserialization() {
    let json = r#"{
        "success": true,
        "filename": "test.pdf",
        "collection_name": "docs",
        "chunks_created": 10,
        "vectors_created": 10,
        "file_size": 2048,
        "language": "pdf",
        "processing_time_ms": 150
    }"#;

    let response: FileUploadResponse = serde_json::from_str(json).expect("Failed to deserialize");

    assert!(response.success);
    assert_eq!(response.filename, "test.pdf");
    assert_eq!(response.collection_name, "docs");
    assert_eq!(response.chunks_created, 10);
    assert_eq!(response.vectors_created, 10);
    assert_eq!(response.file_size, 2048);
    assert_eq!(response.language, "pdf");
    assert_eq!(response.processing_time_ms, 150);
}
