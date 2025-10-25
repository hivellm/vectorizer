use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_cached_file_is_fresh() {
        let metadata = FileMetadata {
            file_type: "rs".to_string(),
            size_kb: 10.5,
            chunk_count: 5,
            last_indexed: Utc::now(),
            language: Some("Rust".to_string()),
        };

        let cached = CachedFile {
            path: "/test/file.rs".to_string(),
            content: "test content".to_string(),
            chunks: vec!["chunk1".to_string(), "chunk2".to_string()],
            summary: Some("Test summary".to_string()),
            metadata,
            cached_at: Instant::now(),
        };

        assert!(cached.is_fresh(Duration::from_secs(60)));
        assert!(cached.is_fresh(Duration::from_millis(1)));
    }

    #[test]
    fn test_cached_file_to_content() {
        let metadata = FileMetadata {
            file_type: "rs".to_string(),
            size_kb: 10.5,
            chunk_count: 5,
            last_indexed: Utc::now(),
            language: Some("Rust".to_string()),
        };

        let cached = CachedFile {
            path: "/test/file.rs".to_string(),
            content: "test content".to_string(),
            chunks: vec!["chunk1".to_string(), "chunk2".to_string()],
            summary: Some("Test summary".to_string()),
            metadata,
            cached_at: Instant::now(),
        };

        let content = cached.to_content("test_collection".to_string());

        assert_eq!(content.file_path, "/test/file.rs");
        assert_eq!(content.content, "test content");
        assert_eq!(content.collection, "test_collection");
        assert_eq!(content.chunks_available, 2);
        assert!(content.from_cache);
    }

    #[test]
    fn test_directory_node_new() {
        let node = DirectoryNode::new("src");

        assert_eq!(node.name, "src");
        assert_eq!(node.node_type, NodeType::Directory);
        assert!(node.children.is_empty());
        assert!(node.file_info.is_none());
    }

    #[test]
    fn test_summary_type_equality() {
        assert_eq!(SummaryType::Extractive, SummaryType::Extractive);
        assert_ne!(SummaryType::Extractive, SummaryType::Structural);
        assert_ne!(SummaryType::Structural, SummaryType::Both);
    }

    #[test]
    fn test_sort_by_default() {
        let sort_by = SortBy::default();
        assert_eq!(sort_by, SortBy::Name);
    }

    #[test]
    fn test_sort_by_equality() {
        assert_eq!(SortBy::Name, SortBy::Name);
        assert_ne!(SortBy::Name, SortBy::Size);
        assert_ne!(SortBy::Chunks, SortBy::Recent);
    }

    #[test]
    fn test_node_type_equality() {
        assert_eq!(NodeType::Directory, NodeType::Directory);
        assert_ne!(NodeType::Directory, NodeType::File);
    }

    #[test]
    fn test_file_list_filter_default() {
        let filter = FileListFilter::default();

        assert!(filter.filter_by_type.is_none());
        assert!(filter.min_chunks.is_none());
        assert!(filter.max_results.is_none());
        assert_eq!(filter.sort_by, SortBy::Name);
    }

    #[test]
    fn test_outline_options_default() {
        let options = OutlineOptions::default();

        assert_eq!(options.max_depth, 5);
        assert!(!options.include_summaries);
        assert!(options.highlight_key_files);
    }

    #[test]
    fn test_file_metadata_creation() {
        let metadata = FileMetadata {
            file_type: "rs".to_string(),
            size_kb: 15.3,
            chunk_count: 10,
            last_indexed: Utc::now(),
            language: Some("Rust".to_string()),
        };

        assert_eq!(metadata.file_type, "rs");
        assert_eq!(metadata.size_kb, 15.3);
        assert_eq!(metadata.chunk_count, 10);
        assert_eq!(metadata.language, Some("Rust".to_string()));
    }

    #[test]
    fn test_file_content_creation() {
        let now = Utc::now();
        let metadata = FileMetadata {
            file_type: "ts".to_string(),
            size_kb: 20.0,
            chunk_count: 8,
            last_indexed: now,
            language: None,
        };

        let content = FileContent {
            file_path: "src/test.ts".to_string(),
            content: "const x = 1;".to_string(),
            metadata,
            chunks_available: 8,
            collection: "test_coll".to_string(),
            from_cache: false,
        };

        assert_eq!(content.file_path, "src/test.ts");
        assert_eq!(content.chunks_available, 8);
        assert!(!content.from_cache);
    }

    #[test]
    fn test_structural_summary() {
        let summary = StructuralSummary {
            outline: "Test outline".to_string(),
            key_sections: vec!["Section 1".to_string(), "Section 2".to_string()],
            key_points: vec!["Point 1".to_string(), "Point 2".to_string()],
        };

        assert_eq!(summary.outline, "Test outline");
        assert_eq!(summary.key_sections.len(), 2);
        assert_eq!(summary.key_points.len(), 2);
    }

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo {
            path: "src/main.rs".to_string(),
            file_type: "rs".to_string(),
            chunk_count: 15,
            size_estimate_kb: 25.5,
            last_indexed: "2024-01-01T00:00:00Z".to_string(),
            has_summary: true,
        };

        assert_eq!(info.path, "src/main.rs");
        assert_eq!(info.chunk_count, 15);
        assert!(info.has_summary);
    }

    #[test]
    fn test_ordered_chunk_creation() {
        let chunk = OrderedChunk {
            index: 0,
            content: "Test content".to_string(),
            line_range: Some((1, 10)),
            context_hint: None,
        };

        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.content, "Test content");
        assert_eq!(chunk.line_range, Some((1, 10)));
    }

    #[test]
    fn test_context_hint() {
        let hint = ContextHint {
            prev_chunk_preview: Some("Previous chunk".to_string()),
            next_chunk_preview: Some("Next chunk".to_string()),
        };

        assert!(hint.prev_chunk_preview.is_some());
        assert!(hint.next_chunk_preview.is_some());
    }

    #[test]
    fn test_related_file() {
        let related = RelatedFile {
            path: "src/utils.rs".to_string(),
            similarity_score: 0.85,
            reason: Some("Similar imports".to_string()),
            shared_concepts: Some(vec!["async".to_string(), "tokio".to_string()]),
        };

        assert_eq!(related.similarity_score, 0.85);
        assert_eq!(related.shared_concepts.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_chunk_match() {
        let chunk_match = ChunkMatch {
            chunk_index: 3,
            content: "Matching content".to_string(),
            score: 0.95,
        };

        assert_eq!(chunk_match.chunk_index, 3);
        assert_eq!(chunk_match.score, 0.95);
    }
}
