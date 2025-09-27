//! Cache validation system

use super::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Cache validator
pub struct CacheValidator {
    /// Validation configuration
    config: ValidationConfig,
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable file existence checks
    pub check_file_existence: bool,

    /// Enable file size validation
    pub check_file_sizes: bool,

    /// Enable content hash validation
    pub check_content_hashes: bool,

    /// Enable metadata consistency checks
    pub check_metadata_consistency: bool,

    /// Maximum file size to validate (in bytes)
    pub max_file_size: u64,

    /// Timeout for validation operations (in seconds)
    pub timeout_seconds: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_file_existence: true,
            check_file_sizes: true,
            check_content_hashes: false, // Expensive operation
            check_metadata_consistency: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            timeout_seconds: 300,             // 5 minutes
        }
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall validation status
    pub status: ValidationStatus,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,

    /// Validation statistics
    pub stats: ValidationStats,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error type
    pub error_type: ValidationErrorType,

    /// Error message
    pub message: String,

    /// Affected file path (if applicable)
    pub file_path: Option<String>,

    /// Collection name (if applicable)
    pub collection_name: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning type
    pub warning_type: ValidationWarningType,

    /// Warning message
    pub message: String,

    /// Affected file path (if applicable)
    pub file_path: Option<String>,

    /// Collection name (if applicable)
    pub collection_name: Option<String>,
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationErrorType {
    /// File does not exist
    FileNotFound,
    /// File size mismatch
    FileSizeMismatch,
    /// Content hash mismatch
    ContentHashMismatch,
    /// Metadata corruption
    MetadataCorruption,
    /// Collection name mismatch
    CollectionNameMismatch,
    /// Cache version mismatch
    CacheVersionMismatch,
    /// File permission error
    FilePermissionError,
    /// Invalid file format
    InvalidFileFormat,
}

/// Validation warning types
#[derive(Debug, Clone)]
pub enum ValidationWarningType {
    /// File is very large
    LargeFile,
    /// File is very old
    OldFile,
    /// Unusual file extension
    UnusualExtension,
    /// Duplicate file hash
    DuplicateHash,
    /// Empty collection
    EmptyCollection,
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStats {
    /// Total files checked
    pub total_files: usize,

    /// Total collections checked
    pub total_collections: usize,

    /// Total cache size checked (in bytes)
    pub total_size_bytes: u64,

    /// Validation duration
    pub duration_ms: u64,

    /// Files with errors
    pub files_with_errors: usize,

    /// Files with warnings
    pub files_with_warnings: usize,
}

impl Default for ValidationStats {
    fn default() -> Self {
        Self {
            total_files: 0,
            total_collections: 0,
            total_size_bytes: 0,
            duration_ms: 0,
            files_with_errors: 0,
            files_with_warnings: 0,
        }
    }
}

impl CacheValidator {
    /// Create new cache validator
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate cache metadata
    pub async fn validate_metadata(&self, metadata: &CacheMetadata) -> ValidationResult {
        let start_time = std::time::Instant::now();
        let mut result = ValidationResult {
            status: ValidationStatus::Valid,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats::default(),
        };

        // Validate metadata structure
        self.validate_metadata_structure(metadata, &mut result)
            .await;

        // Validate collections
        for (collection_name, collection_info) in &metadata.collections {
            self.validate_collection(collection_name, collection_info, &mut result)
                .await;
        }

        // Calculate statistics
        result.stats.total_collections = metadata.collections.len();
        result.stats.total_files = metadata
            .collections
            .values()
            .map(|info| info.file_hashes.len())
            .sum();
        result.stats.total_size_bytes = metadata.calculate_total_size();
        result.stats.duration_ms = start_time.elapsed().as_millis() as u64;

        // Determine overall status
        if !result.errors.is_empty() {
            result.status = ValidationStatus::Invalid;
        } else if !result.warnings.is_empty() {
            result.status = ValidationStatus::ValidWithWarnings;
        }

        result
    }

    /// Validate metadata structure
    async fn validate_metadata_structure(
        &self,
        metadata: &CacheMetadata,
        result: &mut ValidationResult,
    ) {
        // Check version
        if metadata.version.is_empty() {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::CacheVersionMismatch,
                message: "Cache version is empty".to_string(),
                file_path: None,
                collection_name: None,
            });
        }

        // Check timestamps
        if metadata.created_at > metadata.last_updated {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::MetadataCorruption,
                message: "Created timestamp is after last updated timestamp".to_string(),
                file_path: None,
                collection_name: None,
            });
        }

        // Check if metadata is too old
        let age_days = metadata.age_seconds() / (24 * 60 * 60);
        if age_days > 365 {
            result.warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::OldFile,
                message: format!("Cache metadata is {} days old", age_days),
                file_path: None,
                collection_name: None,
            });
        }
    }

    /// Validate collection
    async fn validate_collection(
        &self,
        collection_name: &str,
        collection_info: &CollectionCacheInfo,
        result: &mut ValidationResult,
    ) {
        // Check collection name consistency
        if collection_info.name != collection_name {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::CollectionNameMismatch,
                message: format!(
                    "Collection name mismatch: {} != {}",
                    collection_name, collection_info.name
                ),
                file_path: None,
                collection_name: Some(collection_name.to_string()),
            });
        }

        // Check if collection is empty
        if collection_info.file_hashes.is_empty() {
            result.warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::EmptyCollection,
                message: "Collection has no files".to_string(),
                file_path: None,
                collection_name: Some(collection_name.to_string()),
            });
        }

        // Validate files in collection
        for (file_path, file_info) in &collection_info.file_hashes {
            self.validate_file(file_path, file_info, collection_name, result)
                .await;
        }

        // Check for duplicate hashes
        self.check_duplicate_hashes(collection_info, collection_name, result);
    }

    /// Validate file
    async fn validate_file(
        &self,
        file_path: &std::path::PathBuf,
        file_info: &FileHashInfo,
        collection_name: &str,
        result: &mut ValidationResult,
    ) {
        let file_path_str = file_path.to_string_lossy().to_string();

        // Check file existence
        if self.config.check_file_existence {
            if !file_path.exists() {
                result.errors.push(ValidationError {
                    error_type: ValidationErrorType::FileNotFound,
                    message: "File does not exist".to_string(),
                    file_path: Some(file_path_str.clone()),
                    collection_name: Some(collection_name.to_string()),
                });
                return; // Skip other checks if file doesn't exist
            }
        }

        // Check file size
        if self.config.check_file_sizes && file_path.exists() {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                if metadata.len() != file_info.size {
                    result.errors.push(ValidationError {
                        error_type: ValidationErrorType::FileSizeMismatch,
                        message: format!(
                            "File size mismatch: expected {}, found {}",
                            file_info.size,
                            metadata.len()
                        ),
                        file_path: Some(file_path_str.clone()),
                        collection_name: Some(collection_name.to_string()),
                    });
                }
            }
        }

        // Check content hash (expensive operation)
        if self.config.check_content_hashes && file_path.exists() {
            if let Ok(calculated_hash) = self.calculate_file_hash(file_path).await {
                if calculated_hash != file_info.content_hash {
                    result.errors.push(ValidationError {
                        error_type: ValidationErrorType::ContentHashMismatch,
                        message: format!(
                            "Content hash mismatch: expected {}, found {}",
                            file_info.content_hash, calculated_hash
                        ),
                        file_path: Some(file_path_str.clone()),
                        collection_name: Some(collection_name.to_string()),
                    });
                }
            }
        }

        // Check file size warnings
        if file_info.size > self.config.max_file_size {
            result.warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::LargeFile,
                message: format!("File is very large: {} bytes", file_info.size),
                file_path: Some(file_path_str.clone()),
                collection_name: Some(collection_name.to_string()),
            });
        }

        // Check file age
        let file_age_days = file_info.processing_age_seconds() / (24 * 60 * 60);
        if file_age_days > 30 {
            result.warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::OldFile,
                message: format!("File is {} days old", file_age_days),
                file_path: Some(file_path_str.clone()),
                collection_name: Some(collection_name.to_string()),
            });
        }

        // Check file extension
        if let Some(extension) = file_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            let unusual_extensions = ["exe", "dll", "so", "dylib", "bin"];
            if unusual_extensions.contains(&ext_str.as_str()) {
                result.warnings.push(ValidationWarning {
                    warning_type: ValidationWarningType::UnusualExtension,
                    message: format!("Unusual file extension: {}", ext_str),
                    file_path: Some(file_path_str.clone()),
                    collection_name: Some(collection_name.to_string()),
                });
            }
        }
    }

    /// Check for duplicate hashes
    fn check_duplicate_hashes(
        &self,
        collection_info: &CollectionCacheInfo,
        collection_name: &str,
        result: &mut ValidationResult,
    ) {
        let mut hash_counts: HashMap<String, Vec<String>> = HashMap::new();

        for (file_path, file_info) in &collection_info.file_hashes {
            let file_path_str = file_path.to_string_lossy().to_string();
            hash_counts
                .entry(file_info.content_hash.clone())
                .or_insert_with(Vec::new)
                .push(file_path_str);
        }

        for (hash, files) in hash_counts {
            if files.len() > 1 {
                result.warnings.push(ValidationWarning {
                    warning_type: ValidationWarningType::DuplicateHash,
                    message: format!("Duplicate content hash found in {} files", files.len()),
                    file_path: Some(files.join(", ")),
                    collection_name: Some(collection_name.to_string()),
                });
            }
        }
    }

    /// Calculate file hash
    async fn calculate_file_hash(&self, file_path: &std::path::PathBuf) -> CacheResult<String> {
        let content = tokio::fs::read(file_path).await?;
        let mut hasher = Sha256::default();
        hasher.update(&content);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Validate cache integrity
    pub async fn validate_cache_integrity(&self, cache_path: &Path) -> ValidationResult {
        let start_time = std::time::Instant::now();
        let mut result = ValidationResult {
            status: ValidationStatus::Valid,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats::default(),
        };

        // Check if cache directory exists
        if !cache_path.exists() {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::FileNotFound,
                message: "Cache directory does not exist".to_string(),
                file_path: Some(cache_path.to_string_lossy().to_string()),
                collection_name: None,
            });
            result.status = ValidationStatus::Invalid;
            return result;
        }

        // Check if cache directory is readable
        if !cache_path.is_dir() {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::InvalidFileFormat,
                message: "Cache path is not a directory".to_string(),
                file_path: Some(cache_path.to_string_lossy().to_string()),
                collection_name: None,
            });
            result.status = ValidationStatus::Invalid;
            return result;
        }

        // Check metadata file
        let metadata_path = cache_path.join("metadata.json");
        if !metadata_path.exists() {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::FileNotFound,
                message: "Metadata file does not exist".to_string(),
                file_path: Some(metadata_path.to_string_lossy().to_string()),
                collection_name: None,
            });
            result.status = ValidationStatus::Invalid;
            return result;
        }

        // Try to load and validate metadata
        if let Ok(content) = tokio::fs::read_to_string(&metadata_path).await {
            if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&content) {
                result = self.validate_metadata(&metadata).await;
            } else {
                result.errors.push(ValidationError {
                    error_type: ValidationErrorType::MetadataCorruption,
                    message: "Failed to parse metadata file".to_string(),
                    file_path: Some(metadata_path.to_string_lossy().to_string()),
                    collection_name: None,
                });
                result.status = ValidationStatus::Invalid;
            }
        } else {
            result.errors.push(ValidationError {
                error_type: ValidationErrorType::FilePermissionError,
                message: "Failed to read metadata file".to_string(),
                file_path: Some(metadata_path.to_string_lossy().to_string()),
                collection_name: None,
            });
            result.status = ValidationStatus::Invalid;
        }

        result.stats.duration_ms = start_time.elapsed().as_millis() as u64;
        result
    }
}

impl ValidationResult {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        matches!(
            self.status,
            ValidationStatus::Valid | ValidationStatus::ValidWithWarnings
        )
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get summary message
    pub fn summary(&self) -> String {
        match self.status {
            ValidationStatus::Valid => "Cache validation passed".to_string(),
            ValidationStatus::ValidWithWarnings => {
                format!(
                    "Cache validation passed with {} warnings",
                    self.warning_count()
                )
            }
            ValidationStatus::Invalid => {
                format!("Cache validation failed with {} errors", self.error_count())
            }
            ValidationStatus::Skipped => "Cache validation skipped".to_string(),
            ValidationStatus::Unknown => "Cache validation status unknown".to_string(),
        }
    }
}
