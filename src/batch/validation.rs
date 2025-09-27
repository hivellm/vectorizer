//! Batch Operations Validation
//! 
//! This module provides validation utilities for batch operations,
//! including input validation, data integrity checks, and constraint validation.

use std::collections::HashSet;
use crate::models::Vector;
use super::{BatchConfig, BatchError, BatchErrorType, VectorUpdate, SearchQuery};

/// Validator for batch operations
pub struct BatchValidator {
    config: BatchConfig,
}

impl BatchValidator {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// Validate batch insert operation
    pub fn validate_batch_insert(
        &self,
        collection: &str,
        vectors: &[Vector],
    ) -> Result<(), BatchError> {
        // Validate collection name
        self.validate_collection_name(collection)?;

        // Validate batch size
        self.validate_batch_size(vectors.len())?;

        // Validate vectors
        for (index, vector) in vectors.iter().enumerate() {
            self.validate_vector_data(vector, index)?;
        }

        // Check for duplicate IDs
        self.validate_unique_ids(vectors)?;

        // Validate memory usage
        self.validate_memory_usage(vectors)?;

        Ok(())
    }

    /// Validate batch update operation
    pub fn validate_batch_update(
        &self,
        collection: &str,
        updates: &[VectorUpdate],
    ) -> Result<(), BatchError> {
        // Validate collection name
        self.validate_collection_name(collection)?;

        // Validate batch size
        self.validate_batch_size(updates.len())?;

        // Validate updates
        for (index, update) in updates.iter().enumerate() {
            self.validate_vector_update(update, index)?;
        }

        // Check for duplicate IDs
        self.validate_unique_update_ids(updates)?;

        Ok(())
    }

    /// Validate batch delete operation
    pub fn validate_batch_delete(
        &self,
        collection: &str,
        vector_ids: &[String],
    ) -> Result<(), BatchError> {
        // Validate collection name
        self.validate_collection_name(collection)?;

        // Validate batch size
        self.validate_batch_size(vector_ids.len())?;

        // Validate vector IDs
        for (index, id) in vector_ids.iter().enumerate() {
            self.validate_vector_id(id, index)?;
        }

        // Check for duplicate IDs
        self.validate_unique_delete_ids(vector_ids)?;

        Ok(())
    }

    /// Validate batch search operation
    pub fn validate_batch_search(
        &self,
        collection: &str,
        queries: &[SearchQuery],
    ) -> Result<(), BatchError> {
        // Validate collection name
        self.validate_collection_name(collection)?;

        // Validate batch size
        self.validate_batch_size(queries.len())?;

        // Validate queries
        for (index, query) in queries.iter().enumerate() {
            self.validate_search_query(query, index)?;
        }

        Ok(())
    }

    // Private validation methods

    fn validate_collection_name(&self, collection: &str) -> Result<(), BatchError> {
        if collection.is_empty() {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidCollection,
                "Collection name cannot be empty".to_string(),
                None,
            ));
        }

        if collection.len() > 255 {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidCollection,
                "Collection name too long (max 255 characters)".to_string(),
                None,
            ));
        }

        // Check for invalid characters
        if !collection.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidCollection,
                "Collection name contains invalid characters".to_string(),
                None,
            ));
        }

        Ok(())
    }

    fn validate_batch_size(&self, size: usize) -> Result<(), BatchError> {
        if size == 0 {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidBatchSize,
                "Batch size cannot be zero".to_string(),
                None,
            ));
        }

        if size > self.config.max_batch_size {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidBatchSize,
                format!(
                    "Batch size {} exceeds maximum allowed size {}",
                    size, self.config.max_batch_size
                ),
                None,
            ));
        }

        Ok(())
    }

    fn validate_vector_data(&self, vector: &Vector, index: usize) -> Result<(), BatchError> {
        // Validate vector ID
        self.validate_vector_id(&vector.id, index)?;

        // Validate vector data
        if vector.data.is_empty() {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidVectorData,
                format!("Vector data cannot be empty for vector at index {}", index),
                Some(vector.id.clone()),
            ));
        }

        if vector.data.len() > 10000 {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidVectorData,
                format!(
                    "Vector dimension {} exceeds maximum allowed dimension 10000 for vector at index {}",
                    vector.data.len(), index
                ),
                Some(vector.id.clone()),
            ));
        }

        // Check for NaN or infinite values
        for (i, &value) in vector.data.iter().enumerate() {
            if !value.is_finite() {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidVectorData,
                    format!(
                        "Vector contains invalid value at dimension {} for vector at index {}",
                        i, index
                    ),
                    Some(vector.id.clone()),
                ));
            }
        }

        Ok(())
    }

    fn validate_vector_update(&self, update: &VectorUpdate, index: usize) -> Result<(), BatchError> {
        // Validate vector ID
        self.validate_vector_id(&update.id, index)?;

        // Validate vector data if provided
        if let Some(data) = &update.data {
            if data.is_empty() {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidVectorData,
                    format!("Vector data cannot be empty for update at index {}", index),
                    Some(update.id.clone()),
                ));
            }

            if data.len() > 10000 {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidVectorData,
                    format!(
                        "Vector dimension {} exceeds maximum allowed dimension 10000 for update at index {}",
                        data.len(), index
                    ),
                    Some(update.id.clone()),
                ));
            }

            // Check for NaN or infinite values
            for (i, &value) in data.iter().enumerate() {
                if !value.is_finite() {
                    return Err(BatchError::new(
                        "validation".to_string(),
                        BatchErrorType::InvalidVectorData,
                        format!(
                            "Vector contains invalid value at dimension {} for update at index {}",
                            i, index
                        ),
                        Some(update.id.clone()),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_vector_id(&self, id: &str, index: usize) -> Result<(), BatchError> {
        if id.is_empty() {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidVectorId,
                format!("Vector ID cannot be empty at index {}", index),
                None,
            ));
        }

        if id.len() > 255 {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidVectorId,
                format!("Vector ID too long (max 255 characters) at index {}", index),
                None,
            ));
        }

        Ok(())
    }

    fn validate_search_query(&self, query: &SearchQuery, index: usize) -> Result<(), BatchError> {
        // Must have either query_vector or query_text
        if query.query_vector.is_none() && query.query_text.is_none() {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidQuery,
                format!("Search query must have either vector or text at index {}", index),
                None,
            ));
        }

        // Validate query vector if provided
        if let Some(vector) = &query.query_vector {
            if vector.is_empty() {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidQuery,
                    format!("Query vector cannot be empty at index {}", index),
                    None,
                ));
            }

            if vector.len() > 10000 {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidQuery,
                    format!(
                        "Query vector dimension {} exceeds maximum allowed dimension 10000 at index {}",
                        vector.len(), index
                    ),
                    None,
                ));
            }

            // Check for NaN or infinite values
            for (i, &value) in vector.iter().enumerate() {
                if !value.is_finite() {
                    return Err(BatchError::new(
                        "validation".to_string(),
                        BatchErrorType::InvalidQuery,
                        format!(
                            "Query vector contains invalid value at dimension {} at index {}",
                            i, index
                        ),
                        None,
                    ));
                }
            }
        }

        // Validate query text if provided
        if let Some(text) = &query.query_text {
            if text.is_empty() {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidQuery,
                    format!("Query text cannot be empty at index {}", index),
                    None,
                ));
            }

            if text.len() > 10000 {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidQuery,
                    format!(
                        "Query text length {} exceeds maximum allowed length 10000 at index {}",
                        text.len(), index
                    ),
                    None,
                ));
            }
        }

        // Validate limit
        if query.limit <= 0 {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidQuery,
                format!("Query limit must be positive at index {}", index),
                None,
            ));
        }

        if query.limit > 10000 {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::InvalidQuery,
                format!(
                    "Query limit {} exceeds maximum allowed limit 10000 at index {}",
                    query.limit, index
                ),
                None,
            ));
        }

        // Validate threshold
        if let Some(threshold) = query.threshold {
            if !threshold.is_finite() || threshold < 0.0 || threshold > 1.0 {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidQuery,
                    format!(
                        "Query threshold must be between 0.0 and 1.0 at index {}",
                        index
                    ),
                    None,
                ));
            }
        }

        Ok(())
    }

    fn validate_unique_ids(&self, vectors: &[Vector]) -> Result<(), BatchError> {
        let mut seen_ids = HashSet::new();
        
        for (index, vector) in vectors.iter().enumerate() {
            if !seen_ids.insert(&vector.id) {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidVectorId,
                    format!("Duplicate vector ID '{}' found at index {}", vector.id, index),
                    Some(vector.id.clone()),
                ));
            }
        }

        Ok(())
    }

    fn validate_unique_update_ids(&self, updates: &[VectorUpdate]) -> Result<(), BatchError> {
        let mut seen_ids = HashSet::new();
        
        for (index, update) in updates.iter().enumerate() {
            if !seen_ids.insert(&update.id) {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidVectorId,
                    format!("Duplicate update ID '{}' found at index {}", update.id, index),
                    Some(update.id.clone()),
                ));
            }
        }

        Ok(())
    }

    fn validate_unique_delete_ids(&self, vector_ids: &[String]) -> Result<(), BatchError> {
        let mut seen_ids = HashSet::new();
        
        for (index, id) in vector_ids.iter().enumerate() {
            if !seen_ids.insert(id) {
                return Err(BatchError::new(
                    "validation".to_string(),
                    BatchErrorType::InvalidVectorId,
                    format!("Duplicate delete ID '{}' found at index {}", id, index),
                    Some(id.clone()),
                ));
            }
        }

        Ok(())
    }

    fn validate_memory_usage(&self, vectors: &[Vector]) -> Result<(), BatchError> {
        if vectors.is_empty() {
            return Ok(());
        }

        let vector_dimension = vectors[0].data.len();
        let estimated_memory = self.config.estimate_memory_usage(vectors.len(), vector_dimension);

        if estimated_memory > self.config.max_memory_usage_mb {
            return Err(BatchError::new(
                "validation".to_string(),
                BatchErrorType::MemoryLimitExceeded,
                format!(
                    "Estimated memory usage {}MB exceeds limit {}MB",
                    estimated_memory, self.config.max_memory_usage_mb
                ),
                None,
            ));
        }

        Ok(())
    }
}

/// Quick validation functions for common use cases

/// Validate a single vector
pub fn validate_vector(vector: &Vector) -> Result<(), BatchError> {
    let config = BatchConfig::default();
    let validator = BatchValidator::new(config);
    validator.validate_vector_data(vector, 0)
}

/// Validate a vector ID
pub fn validate_vector_id(id: &str) -> Result<(), BatchError> {
    let config = BatchConfig::default();
    let validator = BatchValidator::new(config);
    validator.validate_vector_id(id, 0)
}

/// Validate a collection name
pub fn validate_collection_name(collection: &str) -> Result<(), BatchError> {
    let config = BatchConfig::default();
    let validator = BatchValidator::new(config);
    validator.validate_collection_name(collection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Vector;

    fn create_test_vector(id: &str, dimension: usize) -> Vector {
        Vector {
            id: id.to_string(),
            data: vec![0.0; dimension],
            payload: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_validate_collection_name() {
        let validator = BatchValidator::new(BatchConfig::default());

        // Valid names
        assert!(validator.validate_collection_name("test_collection").is_ok());
        assert!(validator.validate_collection_name("test-collection").is_ok());
        assert!(validator.validate_collection_name("test123").is_ok());

        // Invalid names
        assert!(validator.validate_collection_name("").is_err());
        assert!(validator.validate_collection_name("test collection").is_err());
        assert!(validator.validate_collection_name("test@collection").is_err());
    }

    #[test]
    fn test_validate_batch_size() {
        let validator = BatchValidator::new(BatchConfig::default());

        // Valid sizes
        assert!(validator.validate_batch_size(1).is_ok());
        assert!(validator.validate_batch_size(100).is_ok());
        assert!(validator.validate_batch_size(1000).is_ok());

        // Invalid sizes
        assert!(validator.validate_batch_size(0).is_err());
        assert!(validator.validate_batch_size(1001).is_err());
    }

    #[test]
    fn test_validate_vector_data() {
        let validator = BatchValidator::new(BatchConfig::default());

        // Valid vector
        let valid_vector = create_test_vector("test_id", 384);
        assert!(validator.validate_vector_data(&valid_vector, 0).is_ok());

        // Invalid vector ID
        let invalid_id_vector = create_test_vector("", 384);
        assert!(validator.validate_vector_data(&invalid_id_vector, 0).is_err());

        // Empty vector data
        let empty_vector = Vector {
            id: "test_id".to_string(),
            data: vec![],
            payload: std::collections::HashMap::new(),
        };
        assert!(validator.validate_vector_data(&empty_vector, 0).is_err());

        // Vector with NaN
        let mut nan_vector = create_test_vector("test_id", 384);
        nan_vector.data[0] = f32::NAN;
        assert!(validator.validate_vector_data(&nan_vector, 0).is_err());

        // Vector with infinity
        let mut inf_vector = create_test_vector("test_id", 384);
        inf_vector.data[0] = f32::INFINITY;
        assert!(validator.validate_vector_data(&inf_vector, 0).is_err());
    }

    #[test]
    fn test_validate_unique_ids() {
        let validator = BatchValidator::new(BatchConfig::default());

        // Unique IDs
        let vectors = vec![
            create_test_vector("id1", 384),
            create_test_vector("id2", 384),
            create_test_vector("id3", 384),
        ];
        assert!(validator.validate_unique_ids(&vectors).is_ok());

        // Duplicate IDs
        let duplicate_vectors = vec![
            create_test_vector("id1", 384),
            create_test_vector("id2", 384),
            create_test_vector("id1", 384),
        ];
        assert!(validator.validate_unique_ids(&duplicate_vectors).is_err());
    }

    #[test]
    fn test_validate_search_query() {
        let validator = BatchValidator::new(BatchConfig::default());

        // Valid query with vector
        let valid_vector_query = SearchQuery {
            query_vector: Some(vec![0.1, 0.2, 0.3]),
            query_text: None,
            limit: 10,
            threshold: Some(0.8),
            filters: std::collections::HashMap::new(),
        };
        assert!(validator.validate_search_query(&valid_vector_query, 0).is_ok());

        // Valid query with text
        let valid_text_query = SearchQuery {
            query_vector: None,
            query_text: Some("test query".to_string()),
            limit: 10,
            threshold: Some(0.8),
            filters: std::collections::HashMap::new(),
        };
        assert!(validator.validate_search_query(&valid_text_query, 0).is_ok());

        // Invalid query (no vector or text)
        let invalid_query = SearchQuery {
            query_vector: None,
            query_text: None,
            limit: 10,
            threshold: Some(0.8),
            filters: std::collections::HashMap::new(),
        };
        assert!(validator.validate_search_query(&invalid_query, 0).is_err());

        // Invalid limit
        let invalid_limit_query = SearchQuery {
            query_vector: Some(vec![0.1, 0.2, 0.3]),
            query_text: None,
            limit: 0,
            threshold: Some(0.8),
            filters: std::collections::HashMap::new(),
        };
        assert!(validator.validate_search_query(&invalid_limit_query, 0).is_err());

        // Invalid threshold
        let invalid_threshold_query = SearchQuery {
            query_vector: Some(vec![0.1, 0.2, 0.3]),
            query_text: None,
            limit: 10,
            threshold: Some(1.5),
            filters: std::collections::HashMap::new(),
        };
        assert!(validator.validate_search_query(&invalid_threshold_query, 0).is_err());
    }

    #[test]
    fn test_quick_validation_functions() {
        // Test validate_vector
        let valid_vector = create_test_vector("test_id", 384);
        assert!(validate_vector(&valid_vector).is_ok());

        // Test validate_vector_id
        assert!(validate_vector_id("valid_id").is_ok());
        assert!(validate_vector_id("").is_err());

        // Test validate_collection_name
        assert!(validate_collection_name("valid_collection").is_ok());
        assert!(validate_collection_name("").is_err());
    }
}
