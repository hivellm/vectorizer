//! Qdrant API Compatibility Tests
//!
//! Tests for Qdrant API compatibility features:
//! - Quantization Configuration API (Scalar, Product, Binary)
//! - Cluster Models
//! - Point ID Handling

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Common Types for Qdrant API Testing
// ============================================================================

/// Qdrant-style API response wrapper
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QdrantResponse<T> {
    pub result: T,
    pub status: String,
    pub time: f64,
}

/// Point structure for Qdrant API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct QdrantPoint {
    pub id: Value,
    pub vector: Option<Vec<f32>>,
    pub payload: Option<HashMap<String, Value>>,
}

/// Search result structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ScoredPoint {
    pub id: Value,
    pub score: f32,
    pub payload: Option<HashMap<String, Value>>,
}

// ============================================================================
// Unit Tests for Qdrant Quantization API Models
// ============================================================================

#[cfg(test)]
// ============================================================================
// Sub-modules — split by capability (phase4_split-qdrant-api-integration-tests).
// The original 1,595-line file had 14 inline test modules. Related modules
// are now grouped by feature (e.g. quantization model + API tests live
// together), so capability-scoped failures surface in focused files.
// ============================================================================
mod cluster;
mod distance;
mod points;
mod quantization;
mod search;
mod sharding;
mod snapshots;
mod status;
