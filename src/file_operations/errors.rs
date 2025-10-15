use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileOperationError {
    #[error("File not found in collection: {file_path} in collection {collection}")]
    FileNotFound {
        file_path: String,
        collection: String,
    },

    #[error("File too large: {size_kb}KB exceeds limit of {max_size_kb}KB")]
    FileTooLarge {
        size_kb: usize,
        max_size_kb: usize,
    },

    #[error("Invalid file path: {path} - {reason}")]
    InvalidPath {
        path: String,
        reason: String,
    },

    #[error("Invalid parameter '{param}': {reason}")]
    InvalidParameter {
        param: String,
        reason: String,
    },

    #[error("Collection not found: {collection}")]
    CollectionNotFound {
        collection: String,
    },

    #[error("No chunks found for file: {file_path}")]
    NoChunksFound {
        file_path: String,
    },

    #[error("Cache error: {message}")]
    CacheError {
        message: String,
    },

    #[error("Summarization error: {message}")]
    SummarizationError {
        message: String,
    },

    #[error("Vector store error: {0}")]
    VectorStoreError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type FileOperationResult<T> = Result<T, FileOperationError>;

