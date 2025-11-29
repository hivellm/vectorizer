//! Migration validation tools
//!
//! Validates data integrity, compatibility, and performance after migration.

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{Result, VectorizerError};
use crate::migration::qdrant::data_migration::ExportedCollection;

/// Migration validator
pub struct MigrationValidator;

impl MigrationValidator {
    /// Validate exported collection data
    pub fn validate_export(exported: &ExportedCollection) -> Result<ValidationReport> {
        info!("🔍 Validating exported collection '{}'", exported.name);

        let mut report = ValidationReport {
            collection_name: exported.name.clone(),
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            statistics: ValidationStatistics {
                total_points: exported.points.len(),
                points_with_payload: 0,
                points_without_payload: 0,
                average_vector_dimension: 0.0,
                min_vector_dimension: usize::MAX,
                max_vector_dimension: 0,
            },
        };

        // Validate points
        let mut total_dimension = 0;
        let mut dimension_count = 0;

        for (idx, point) in exported.points.iter().enumerate() {
            // Validate vector dimension
            let dimension = match &point.vector {
                crate::migration::qdrant::data_migration::QdrantVector::Dense(data) => data.len(),
                crate::migration::qdrant::data_migration::QdrantVector::Sparse(_) => {
                    report
                        .errors
                        .push(format!("Point {}: Sparse vectors not supported", point.id));
                    report.is_valid = false;
                    continue;
                }
            };

            if dimension == 0 {
                report
                    .errors
                    .push(format!("Point {}: Empty vector", point.id));
                report.is_valid = false;
            }

            total_dimension += dimension;
            dimension_count += 1;

            if dimension < report.statistics.min_vector_dimension {
                report.statistics.min_vector_dimension = dimension;
            }
            if dimension > report.statistics.max_vector_dimension {
                report.statistics.max_vector_dimension = dimension;
            }

            // Validate payload
            if point.payload.is_some() {
                report.statistics.points_with_payload += 1;
            } else {
                report.statistics.points_without_payload += 1;
            }

            // Check for duplicate IDs
            for other_idx in (idx + 1)..exported.points.len() {
                if exported.points[other_idx].id == point.id {
                    report
                        .warnings
                        .push(format!("Duplicate point ID found: {}", point.id));
                }
            }
        }

        // Calculate average dimension
        if dimension_count > 0 {
            report.statistics.average_vector_dimension =
                total_dimension as f64 / dimension_count as f64;
        }

        // Validate dimension consistency
        if report.statistics.min_vector_dimension != report.statistics.max_vector_dimension {
            report.warnings.push(format!(
                "Inconsistent vector dimensions: min={}, max={}",
                report.statistics.min_vector_dimension, report.statistics.max_vector_dimension
            ));
        }

        // Validate collection config
        if let Some(dimension) = Self::extract_collection_dimension(&exported.config) {
            if dimension != report.statistics.max_vector_dimension as u32 {
                report.warnings.push(format!(
                    "Collection config dimension ({}) doesn't match actual vectors ({})",
                    dimension, report.statistics.max_vector_dimension
                ));
            }
        }

        info!(
            "✅ Validation complete: {} errors, {} warnings",
            report.errors.len(),
            report.warnings.len()
        );
        Ok(report)
    }

    /// Validate data integrity after import
    pub fn validate_integrity(
        exported: &ExportedCollection,
        imported_count: usize,
    ) -> Result<IntegrityReport> {
        info!("🔍 Validating data integrity");

        let expected_count = exported.points.len();
        let missing_count = expected_count.saturating_sub(imported_count);

        let report = IntegrityReport {
            expected_count,
            imported_count,
            missing_count,
            integrity_percentage: if expected_count > 0 {
                (imported_count as f64 / expected_count as f64) * 100.0
            } else {
                0.0
            },
            is_complete: missing_count == 0,
        };

        if !report.is_complete {
            warn!("⚠️ Data integrity issue: {} points missing", missing_count);
        } else {
            info!("✅ Data integrity validated: all points imported");
        }

        Ok(report)
    }

    /// Validate compatibility
    pub fn validate_compatibility(exported: &ExportedCollection) -> CompatibilityReport {
        info!("🔍 Validating compatibility");

        let mut report = CompatibilityReport {
            is_compatible: true,
            incompatible_features: Vec::new(),
            warnings: Vec::new(),
        };

        // Check for named vectors
        if Self::has_named_vectors(&exported.config) {
            report
                .incompatible_features
                .push("Named vectors".to_string());
            report.is_compatible = false;
        }

        // Check for sparse vectors
        for point in &exported.points {
            if matches!(
                point.vector,
                crate::migration::qdrant::data_migration::QdrantVector::Sparse(_)
            ) {
                report
                    .incompatible_features
                    .push("Sparse vectors".to_string());
                report.is_compatible = false;
                break;
            }
        }

        // Check quantization type
        if let Some(quant_config) = Self::extract_quantization_config(&exported.config) {
            match quant_config {
                crate::migration::qdrant::data_migration::QdrantQuantizationTypeResponse::Product => {
                    report.warnings.push("Product quantization will be converted to SQ8".to_string());
                }
                crate::migration::qdrant::data_migration::QdrantQuantizationTypeResponse::Binary => {
                    report.warnings.push("Binary quantization will be converted to SQ8".to_string());
                }
                _ => {}
            }
        }

        info!(
            "✅ Compatibility check complete: compatible={}",
            report.is_compatible
        );
        report
    }

    /// Extract collection dimension from config
    fn extract_collection_dimension(
        config: &crate::migration::qdrant::data_migration::QdrantCollectionConfig,
    ) -> Option<u32> {
        match &config.params.vectors {
            crate::migration::qdrant::data_migration::QdrantVectorsConfigResponse::Vector {
                size,
                distance: _,
            } => Some(*size),
            _ => None,
        }
    }

    /// Check if config has named vectors
    fn has_named_vectors(
        config: &crate::migration::qdrant::data_migration::QdrantCollectionConfig,
    ) -> bool {
        matches!(
            config.params.vectors,
            crate::migration::qdrant::data_migration::QdrantVectorsConfigResponse::NamedVectors(_)
        )
    }

    /// Extract quantization config
    fn extract_quantization_config(
        config: &crate::migration::qdrant::data_migration::QdrantCollectionConfig,
    ) -> Option<crate::migration::qdrant::data_migration::QdrantQuantizationTypeResponse> {
        config
            .params
            .quantization_config
            .as_ref()
            .map(|q| q.quantization)
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Collection name
    pub collection_name: String,
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation statistics
    pub statistics: ValidationStatistics,
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStatistics {
    /// Total number of points
    pub total_points: usize,
    /// Points with payload
    pub points_with_payload: usize,
    /// Points without payload
    pub points_without_payload: usize,
    /// Average vector dimension
    pub average_vector_dimension: f64,
    /// Minimum vector dimension
    pub min_vector_dimension: usize,
    /// Maximum vector dimension
    pub max_vector_dimension: usize,
}

/// Integrity report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Expected point count
    pub expected_count: usize,
    /// Imported point count
    pub imported_count: usize,
    /// Missing point count
    pub missing_count: usize,
    /// Integrity percentage
    pub integrity_percentage: f64,
    /// Whether import is complete
    pub is_complete: bool,
}

/// Compatibility report
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    /// Whether collection is compatible
    pub is_compatible: bool,
    /// Incompatible features found
    pub incompatible_features: Vec<String>,
    /// Compatibility warnings
    pub warnings: Vec<String>,
}
