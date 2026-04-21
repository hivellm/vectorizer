//! File-operations surface: content/listing/summary/chunks/outline,
//! related-file discovery, type-filtered search, file upload.
//!
//! 10 methods covering everything the file pipeline exposes today.
//! `upload_file` and `upload_file_content` build a one-off
//! [`HttpTransport`] for the multipart POST because the generic
//! `Transport` trait doesn't model multipart yet — every other
//! method goes through the dispatched transport.

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::*;

impl VectorizerClient {
    /// Retrieve the complete content of an indexed file.
    pub async fn get_file_content(
        &self,
        collection: &str,
        file_path: &str,
        max_size_kb: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );
        if let Some(max) = max_size_kb {
            payload.insert(
                "max_size_kb".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/file/content",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse file content response: {e}"))
        })
    }

    /// List indexed files in a collection, optionally filtered by
    /// extension and minimum chunk count.
    pub async fn list_files_in_collection(
        &self,
        collection: &str,
        filter_by_type: Option<Vec<String>>,
        min_chunks: Option<usize>,
        max_results: Option<usize>,
        sort_by: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        if let Some(types) = filter_by_type {
            payload.insert(
                "filter_by_type".to_string(),
                serde_json::to_value(types).unwrap(),
            );
        }
        if let Some(min) = min_chunks {
            payload.insert(
                "min_chunks".to_string(),
                serde_json::Value::Number(min.into()),
            );
        }
        if let Some(max) = max_results {
            payload.insert(
                "max_results".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        if let Some(sort) = sort_by {
            payload.insert(
                "sort_by".to_string(),
                serde_json::Value::String(sort.to_string()),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/file/list",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list files response: {e}"))
        })
    }

    /// Get an extractive or structural summary of one indexed file.
    pub async fn get_file_summary(
        &self,
        collection: &str,
        file_path: &str,
        summary_type: Option<&str>,
        max_sentences: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );
        if let Some(stype) = summary_type {
            payload.insert(
                "summary_type".to_string(),
                serde_json::Value::String(stype.to_string()),
            );
        }
        if let Some(max) = max_sentences {
            payload.insert(
                "max_sentences".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/file/summary",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse file summary response: {e}"))
        })
    }

    /// Retrieve chunks in original file order for progressive
    /// reading. Pair with `start_chunk` + `limit` to page.
    pub async fn get_file_chunks_ordered(
        &self,
        collection: &str,
        file_path: &str,
        start_chunk: Option<usize>,
        limit: Option<usize>,
        include_context: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );
        if let Some(start) = start_chunk {
            payload.insert(
                "start_chunk".to_string(),
                serde_json::Value::Number(start.into()),
            );
        }
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(ctx) = include_context {
            payload.insert("include_context".to_string(), serde_json::Value::Bool(ctx));
        }
        let response = self
            .make_request(
                "POST",
                "/file/chunks",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse chunks response: {e}")))
    }

    /// Generate a hierarchical project structure overview.
    pub async fn get_project_outline(
        &self,
        collection: &str,
        max_depth: Option<usize>,
        include_summaries: Option<bool>,
        highlight_key_files: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        if let Some(depth) = max_depth {
            payload.insert(
                "max_depth".to_string(),
                serde_json::Value::Number(depth.into()),
            );
        }
        if let Some(summ) = include_summaries {
            payload.insert(
                "include_summaries".to_string(),
                serde_json::Value::Bool(summ),
            );
        }
        if let Some(highlight) = highlight_key_files {
            payload.insert(
                "highlight_key_files".to_string(),
                serde_json::Value::Bool(highlight),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/file/outline",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse outline response: {e}")))
    }

    /// Find semantically-related files by vector similarity.
    pub async fn get_related_files(
        &self,
        collection: &str,
        file_path: &str,
        limit: Option<usize>,
        similarity_threshold: Option<f32>,
        include_reason: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(thresh) = similarity_threshold {
            payload.insert(
                "similarity_threshold".to_string(),
                serde_json::json!(thresh),
            );
        }
        if let Some(reason) = include_reason {
            payload.insert(
                "include_reason".to_string(),
                serde_json::Value::Bool(reason),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/file/related",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse related files response: {e}"))
        })
    }

    /// Semantic search filtered by file type. Empty `file_types` is
    /// rejected — pass at least one extension.
    pub async fn search_by_file_type(
        &self,
        collection: &str,
        query: &str,
        file_types: Vec<String>,
        limit: Option<usize>,
        return_full_files: Option<bool>,
    ) -> Result<serde_json::Value> {
        if file_types.is_empty() {
            return Err(VectorizerError::validation("file_types cannot be empty"));
        }
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        payload.insert(
            "file_types".to_string(),
            serde_json::to_value(file_types).unwrap(),
        );
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(full) = return_full_files {
            payload.insert(
                "return_full_files".to_string(),
                serde_json::Value::Bool(full),
            );
        }
        let response = self
            .make_request(
                "POST",
                "/file/search_by_type",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse search by type response: {e}"))
        })
    }

    /// Upload a file for automatic text extraction, chunking, and
    /// indexing.
    ///
    /// # Arguments
    /// * `file_bytes` - File content as bytes
    /// * `filename` - Name of the file (used for extension detection)
    /// * `collection_name` - Target collection name
    /// * `options` - Upload options (chunk size, overlap, metadata)
    ///
    /// # Example
    /// ```no_run
    /// use vectorizer_sdk::{VectorizerClient, ClientConfig, UploadFileOptions};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ClientConfig::default();
    ///     let client = VectorizerClient::new(config)?;
    ///
    ///     let file_bytes = std::fs::read("document.pdf")?;
    ///     let options = UploadFileOptions::default();
    ///
    ///     let response = client.upload_file(
    ///         file_bytes,
    ///         "document.pdf",
    ///         "my-docs",
    ///         options
    ///     ).await?;
    ///
    ///     println!("Uploaded: {} chunks created", response.chunks_created);
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file(
        &self,
        file_bytes: Vec<u8>,
        filename: &str,
        collection_name: &str,
        options: UploadFileOptions,
    ) -> Result<FileUploadResponse> {
        let mut form_fields = std::collections::HashMap::new();
        form_fields.insert("collection_name".to_string(), collection_name.to_string());
        if let Some(chunk_size) = options.chunk_size {
            form_fields.insert("chunk_size".to_string(), chunk_size.to_string());
        }
        if let Some(chunk_overlap) = options.chunk_overlap {
            form_fields.insert("chunk_overlap".to_string(), chunk_overlap.to_string());
        }
        if let Some(metadata) = options.metadata {
            let metadata_json = serde_json::to_string(&metadata).map_err(|e| {
                VectorizerError::validation(format!("Failed to serialize metadata: {e}"))
            })?;
            form_fields.insert("metadata".to_string(), metadata_json);
        }
        if let Some(public_key) = options.public_key {
            form_fields.insert("public_key".to_string(), public_key);
        }

        // The generic `Transport` trait doesn't model multipart yet,
        // so we build a one-off `HttpTransport` here. When the
        // trait grows a multipart method (or the RPC backend lands
        // its own file-upload primitive), this branch collapses
        // back into `self.make_request`.
        let http_transport = crate::http_transport::HttpTransport::new(
            self.base_url(),
            self.config.api_key.as_deref(),
            self.config.timeout_secs.unwrap_or(30),
        )?;
        let response = http_transport
            .post_multipart("/files/upload", file_bytes, filename, form_fields)
            .await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse upload response: {e}")))
    }

    /// Upload file content directly as a string. Convenience wrapper
    /// around [`Self::upload_file`].
    ///
    /// # Example
    /// ```no_run
    /// use vectorizer_sdk::{VectorizerClient, ClientConfig, UploadFileOptions};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = VectorizerClient::new(ClientConfig::default())?;
    ///     let code = r#"fn main() { println!("Hello!"); }"#;
    ///     let response = client.upload_file_content(
    ///         code, "main.rs", "rust-code", UploadFileOptions::default()
    ///     ).await?;
    ///     println!("Uploaded: {} vectors created", response.vectors_created);
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file_content(
        &self,
        content: &str,
        filename: &str,
        collection_name: &str,
        options: UploadFileOptions,
    ) -> Result<FileUploadResponse> {
        let file_bytes = content.as_bytes().to_vec();
        self.upload_file(file_bytes, filename, collection_name, options)
            .await
    }

    /// Get file upload configuration from the server (max file size,
    /// allowed extensions, default chunk settings).
    ///
    /// # Example
    /// ```no_run
    /// use vectorizer_sdk::{VectorizerClient, ClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = VectorizerClient::new(ClientConfig::default())?;
    ///     let upload_config = client.get_upload_config().await?;
    ///     println!("Max file size: {}MB", upload_config.max_file_size_mb);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_upload_config(&self) -> Result<FileUploadConfig> {
        let response = self.make_request("GET", "/files/config", None).await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse upload config: {e}")))
    }
}
