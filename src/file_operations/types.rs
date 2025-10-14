use serde::{Deserialize, Serialize};
use std::time::Instant;
use chrono::{DateTime, Utc};

/// Complete file content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub file_path: String,
    pub content: String,
    pub metadata: FileMetadata,
    pub chunks_available: usize,
    pub collection: String,
    pub from_cache: bool,
}

/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_type: String,
    pub size_kb: f64,
    pub chunk_count: usize,
    pub last_indexed: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Cached file with metadata
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub path: String,
    pub content: String,
    pub chunks: Vec<String>, // Chunk IDs
    pub summary: Option<String>,
    pub metadata: FileMetadata,
    pub cached_at: Instant,
}

impl CachedFile {
    pub fn is_fresh(&self, max_age: std::time::Duration) -> bool {
        self.cached_at.elapsed() < max_age
    }

    pub fn to_content(&self, collection: String) -> FileContent {
        FileContent {
            file_path: self.path.clone(),
            content: self.content.clone(),
            metadata: self.metadata.clone(),
            chunks_available: self.chunks.len(),
            collection,
            from_cache: true,
        }
    }
}

/// File summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extractive_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structural_summary: Option<StructuralSummary>,
    pub metadata: FileSummaryMetadata,
    pub generated_at: DateTime<Utc>,
}

/// Structural summary with outline and key points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSummary {
    pub outline: String,
    pub key_sections: Vec<String>,
    pub key_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummaryMetadata {
    pub chunk_count: usize,
    pub file_type: String,
    pub summary_method: String,
}

/// Summary type options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummaryType {
    Extractive,
    Structural,
    Both,
}

/// File list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileList {
    pub collection: String,
    pub files: Vec<FileInfo>,
    pub total_files: usize,
    pub total_chunks: usize,
}

/// Individual file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub file_type: String,
    pub chunk_count: usize,
    pub size_estimate_kb: f64,
    pub last_indexed: String,
    pub has_summary: bool,
}

/// Filter for listing files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileListFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_by_type: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_chunks: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<usize>,
    #[serde(default = "default_sort_by")]
    pub sort_by: SortBy,
}

fn default_sort_by() -> SortBy {
    SortBy::Name
}

/// Sorting options for file lists
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    #[default]
    Name,
    Size,
    Chunks,
    Recent,
}

/// Ordered chunks response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedChunks {
    pub file_path: String,
    pub total_chunks: usize,
    pub chunks: Vec<OrderedChunk>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_start: Option<usize>,
}

/// Individual chunk with ordering information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedChunk {
    pub index: usize,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<(usize, usize)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_hint: Option<ContextHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextHint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_chunk_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_chunk_preview: Option<String>,
}

/// Project outline response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOutline {
    pub collection: String,
    pub structure: DirectoryNode2,
    pub key_files: Vec<String>,
    pub statistics: ProjectStatistics,
}

/// Directory/file tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<DirectoryNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_info: Option<FileNodeInfo>,
}

impl DirectoryNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            node_type: NodeType::Directory,
            children: Vec::new(),
            file_info: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Directory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNodeInfo {
    pub chunks: usize,
    pub size_kb: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFileInfo {
    pub path: String,
    pub reason: String,
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub total_files: usize,
    pub total_directories: usize,
    pub file_types: std::collections::HashMap<String, usize>,
}

/// Related files response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedFiles {
    pub source_file: String,
    pub related_files: Vec<RelatedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedFile {
    pub path: String,
    pub similarity_score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_concepts: Option<Vec<String>>,
}

/// File type search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeSearchResults {
    pub query: String,
    pub file_types: Vec<String>,
    pub results: Vec<FileSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub file_path: String,
    pub file_type: String,
    pub relevance_score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_chunks: Option<Vec<ChunkMatch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMatch {
    pub chunk_index: usize,
    pub content: String,
    pub score: f32,
}

/// Chunk reference from vector store
#[derive(Debug, Clone)]
pub struct ChunkReference {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
}

/// Options for project outline generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineOptions {
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default)]
    pub include_summaries: bool,
    #[serde(default = "default_true")]
    pub highlight_key_files: bool,
}

fn default_max_depth() -> usize {
    5
}

fn default_true() -> bool {
    true
}

impl Default for OutlineOptions {
    fn default() -> Self {
        Self {
            max_depth: 5,
            include_summaries: false,
            highlight_key_files: true,
        }
    }
}

// New types for the implemented features

/// File chunks ordered response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunksOrdered {
    pub file_path: String,
    pub total_chunks: usize,
    pub chunks: Vec<FileChunk>,
    pub has_more: bool,
}

/// Individual file chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub chunk_index: usize,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<(usize, usize)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_chunk_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_chunk_hint: Option<String>,
}

/// Updated DirectoryNode with more fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode2 {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<DirectoryNode2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_info: Option<FileInfo>,
    pub is_key_file: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// File type search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeSearchResult {
    pub query: String,
    pub file_types: Vec<String>,
    pub results: Vec<FileTypeSearchMatch>,
    pub total_matches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeSearchMatch {
    pub file_path: String,
    pub file_type: String,
    pub score: f32,
    pub matching_chunk: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_content: Option<String>,
}

