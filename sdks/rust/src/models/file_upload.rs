//! File upload models for the Vectorizer SDK

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Request to upload a file for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadRequest {
    /// Target collection name
    pub collection_name: String,
    /// Chunk size in characters (uses server default if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<u32>,
    /// Chunk overlap in characters (uses server default if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_overlap: Option<u32>,
    /// Additional metadata to attach to all chunks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Response from file upload operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadResponse {
    /// Whether the upload was successful
    pub success: bool,
    /// Original filename
    pub filename: String,
    /// Target collection
    pub collection_name: String,
    /// Number of chunks created from the file
    pub chunks_created: u32,
    /// Number of vectors created and stored
    pub vectors_created: u32,
    /// File size in bytes
    pub file_size: u64,
    /// Detected language/file type
    pub language: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Configuration for file uploads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadConfig {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Maximum file size in megabytes
    pub max_file_size_mb: u32,
    /// List of allowed file extensions
    pub allowed_extensions: Vec<String>,
    /// Whether binary files are rejected
    pub reject_binary: bool,
    /// Default chunk size in characters
    pub default_chunk_size: u32,
    /// Default chunk overlap in characters
    pub default_chunk_overlap: u32,
}

/// Options for uploading a file
#[derive(Debug, Clone, Default)]
pub struct UploadFileOptions {
    /// Chunk size in characters
    pub chunk_size: Option<u32>,
    /// Chunk overlap in characters
    pub chunk_overlap: Option<u32>,
    /// Additional metadata to attach to all chunks
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
