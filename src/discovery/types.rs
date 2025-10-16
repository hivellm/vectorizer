//! Core types for the discovery system

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Reference to a collection with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRef {
    pub name: String,
    pub dimension: usize,
    pub vector_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// A scored chunk from search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredChunk {
    pub collection: String,
    pub doc_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: ChunkMetadata,
}

/// Metadata for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub file_path: String,
    pub chunk_index: usize,
    pub file_extension: String,
    pub line_range: Option<(usize, usize)>,
}

/// A compressed evidence bullet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bullet {
    pub text: String,
    pub source_id: String,
    pub collection: String,
    pub file_path: String,
    pub score: f32,
    pub category: BulletCategory,
}

/// Category for organizing bullets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BulletCategory {
    Definition,
    Feature,
    Architecture,
    Performance,
    Integration,
    UseCase,
    Other,
}

/// Type of section in answer plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SectionType {
    Definition,
    Features,
    Architecture,
    Performance,
    Integrations,
    UseCases,
}

/// A section in the answer plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub section_type: SectionType,
    pub bullets: Vec<Bullet>,
    pub priority: usize,
}

/// Complete answer plan with organized sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerPlan {
    pub sections: Vec<Section>,
    pub total_bullets: usize,
    pub sources: Vec<String>,
}

/// Discovery response with all results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub answer_prompt: String,
    pub plan: AnswerPlan,
    pub bullets: Vec<Bullet>,
    pub chunks: Vec<ScoredChunk>,
    pub metrics: DiscoveryMetrics,
}

/// Metrics for discovery operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryMetrics {
    pub total_time_ms: u64,
    pub collections_searched: usize,
    pub queries_generated: usize,
    pub chunks_found: usize,
    pub chunks_after_dedup: usize,
    pub bullets_extracted: usize,
    pub final_prompt_tokens: usize,
}

/// Discovery error types
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type DiscoveryResult<T> = Result<T, DiscoveryError>;
