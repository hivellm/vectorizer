//! Essential utilities for the Vectorizer SDK

use crate::error::{VectorizerError, Result};

/// Input validation
pub mod validation {
    use super::*;

    /// Validate that a string is not empty
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<()> {
        if value.trim().is_empty() {
            return Err(VectorizerError::validation(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate that a number is positive
    pub fn validate_positive_number(value: f32, field_name: &str) -> Result<()> {
        if value.is_nan() {
            return Err(VectorizerError::validation(format!(
                "{} must be a valid number, got NaN",
                field_name
            )));
        }
        if value.is_infinite() {
            return Err(VectorizerError::validation(format!(
                "{} must be a valid number, got infinity",
                field_name
            )));
        }
        if value <= 0.0 {
            return Err(VectorizerError::validation(format!(
                "{} must be positive, got {}",
                field_name, value
            )));
        }
        Ok(())
    }

    /// Validate that a number is non-negative
    pub fn validate_non_negative_number(value: f32, field_name: &str) -> Result<()> {
        if value < 0.0 {
            return Err(VectorizerError::validation(format!(
                "{} must be non-negative, got {}",
                field_name, value
            )));
        }
        Ok(())
    }

    /// Validate collection name
    pub fn validate_collection_name(name: &str) -> Result<()> {
        validate_non_empty_string(name, "collection name")?;
        
        // Check for specific invalid characters first (for specific error messages)
        if name.contains(' ') {
            return Err(VectorizerError::validation(
                "Collection name cannot contain spaces"
            ));
        }
        if name.contains('/') {
            return Err(VectorizerError::validation(
                "Collection name cannot contain slashes"
            ));
        }
        if name.contains('\\') {
            return Err(VectorizerError::validation(
                "Collection name cannot contain backslashes"
            ));
        }
        if name.contains('@') {
            return Err(VectorizerError::validation(
                "Collection name cannot contain @ symbols"
            ));
        }
        
        // Only allow alphanumeric characters, hyphens, and underscores
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(VectorizerError::validation(
                "Collection name can only contain alphanumeric characters, hyphens, and underscores"
            ));
        }
        
        Ok(())
    }

    /// Validate vector ID
    pub fn validate_vector_id(id: &str) -> Result<()> {
        validate_non_empty_string(id, "vector ID")
    }

    /// Validate similarity metric
    pub fn validate_similarity_metric(metric: &str) -> Result<()> {
        match metric {
            "cosine" | "euclidean" | "dot_product" => Ok(()),
            _ => Err(VectorizerError::validation(format!(
                "Invalid similarity metric: {}. Must be: cosine, euclidean, dot_product",
                metric
            ))),
        }
    }
}

/// Serialization utilities
pub mod serialization {
    use super::*;

    /// Serialize value to JSON
    pub fn to_json<T: serde::Serialize>(value: &T) -> Result<String> {
        serde_json::to_string(value).map_err(VectorizerError::from)
    }

    /// Deserialize JSON to value
    pub fn from_json<T: for<'de> serde::Deserialize<'de>>(json: &str) -> Result<T> {
        serde_json::from_str(json).map_err(VectorizerError::from)
    }
}