use super::cache::FileLevelCache;
use super::errors::{FileOperationError, FileOperationResult};
use super::types::*;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use crate::VectorStore;

// Constants
pub const MAX_FILE_SIZE_KB: usize = 1000; // 1MB default
pub const ABSOLUTE_MAX_SIZE_KB: usize = 5000; // 5MB hard limit
pub const FILE_CACHE_TTL: Duration = Duration::from_secs(600); // 10 minutes
pub const SUMMARY_CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
pub const FILE_LIST_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// File operations implementation
pub struct FileOperations {
    cache: FileLevelCache,
    store: Option<Arc<VectorStore>>,
}

impl FileOperations {
    pub fn new() -> Self {
        Self {
            cache: FileLevelCache::new(),
            store: None,
        }
    }

    pub fn with_store(store: Arc<VectorStore>) -> Self {
        Self {
            cache: FileLevelCache::new(),
            store: Some(store),
        }
    }

    // ============================================
    // Priority 1: get_file_content
    // ============================================

    /// Retrieve complete file content from indexed collections
    pub async fn get_file_content(
        &self,
        collection: &str,
        file_path: &str,
        max_size_kb: usize,
    ) -> FileOperationResult<FileContent> {
        info!(
            collection = %collection,
            file_path = %file_path,
            max_size_kb = max_size_kb,
            "Fetching file content"
        );

        // Validate parameters
        Self::validate_file_path(file_path)?;
        Self::validate_size_limit(max_size_kb)?;

        // Check cache first
        let cache_key = format!("{}:{}", collection, file_path);
        if let Some(cached) = self.cache.get_file_content(&cache_key).await {
            if cached.is_fresh(FILE_CACHE_TTL) {
                info!(
                    collection = %collection,
                    file_path = %file_path,
                    "File content retrieved from cache"
                );
                return Ok(cached.to_content(collection.to_string()));
            }
        }

        // Get VectorStore
        let store = self.store.as_ref()
            .ok_or_else(|| FileOperationError::VectorStoreError("VectorStore not initialized".to_string()))?;

        // Get collection
        let coll = store.get_collection(collection)
            .map_err(|e| FileOperationError::CollectionNotFound {
                collection: collection.to_string(),
            })?;

        // Get all vectors from collection
        let all_vectors = coll.get_all_vectors();
        
        // Filter vectors by file_path in metadata
        let mut file_chunks: Vec<_> = all_vectors.into_iter()
            .filter_map(|v| {
                if let Some(payload) = &v.payload {
                    if let Some(metadata) = payload.data.get("metadata") {
                        if let Some(fp) = metadata.get("file_path").and_then(|v| v.as_str()) {
                            if fp == file_path {
                                let chunk_index = metadata.get("chunk_index")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as usize;
                                let content = payload.data.get("content")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                return Some((chunk_index, content, metadata.clone()));
                            }
                        }
                    }
                }
                None
            })
            .collect();

        if file_chunks.is_empty() {
            return Err(FileOperationError::FileNotFound {
                file_path: file_path.to_string(),
                collection: collection.to_string(),
            });
        }

        // Sort by chunk_index
        file_chunks.sort_by_key(|(index, _, _)| *index);

        // Reconstruct file content
        let content = file_chunks.iter()
            .map(|(_, content, _)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Check size limit
        let size_kb = content.len() as f64 / 1024.0;
        if size_kb > max_size_kb as f64 {
            return Err(FileOperationError::FileTooLarge {
                size_kb: size_kb as usize,
                max_size_kb,
            });
        }

        // Extract metadata from first chunk
        let metadata = FileMetadata {
            file_type: Self::detect_file_type(file_path),
            size_kb,
            chunk_count: file_chunks.len(),
            last_indexed: Utc::now(),
            language: Self::detect_language(file_path),
        };

        // Cache result
        let cached_file = CachedFile {
            path: file_path.to_string(),
            content: content.clone(),
            chunks: vec![],
            summary: None,
            metadata: metadata.clone(),
            cached_at: std::time::Instant::now(),
        };
        
        self.cache.put_file_content(cache_key, cached_file).await;

        Ok(FileContent {
            file_path: file_path.to_string(),
            content,
            metadata,
            chunks_available: file_chunks.len(),
            collection: collection.to_string(),
            from_cache: false,
        })
    }

    // ============================================
    // Priority 1: list_files_in_collection
    // ============================================

    /// List all indexed files in a collection with metadata
    pub async fn list_files_in_collection(
        &self,
        collection: &str,
        filter: FileListFilter,
    ) -> FileOperationResult<FileList> {
        info!(
            collection = %collection,
            "Listing files in collection"
        );

        // Get VectorStore
        let store = self.store.as_ref()
            .ok_or_else(|| FileOperationError::VectorStoreError("VectorStore not initialized".to_string()))?;

        // Get collection
        let coll = store.get_collection(collection)
            .map_err(|e| FileOperationError::CollectionNotFound {
                collection: collection.to_string(),
            })?;

        // Get all vectors
        let all_vectors = coll.get_all_vectors();
        
        // Group by file_path
        let mut file_map: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        
        for vector in all_vectors {
            if let Some(payload) = &vector.payload {
                if let Some(metadata) = payload.data.get("metadata") {
                    if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                        file_map.entry(file_path.to_string())
                            .or_insert_with(Vec::new)
                            .push(metadata.clone());
                    }
                }
            }
        }

        // Create FileInfo for each file
        let mut files: Vec<FileInfo> = file_map
            .into_iter()
            .filter_map(|(path, chunks)| {
                let file_type = Path::new(&path)
                    .extension()?
                    .to_str()?
                    .to_string();
                
                // Apply filters
                if let Some(types) = &filter.filter_by_type {
                    if !types.contains(&file_type) {
                        return None;
                    }
                }
                
                if let Some(min_chunks) = filter.min_chunks {
                    if chunks.len() < min_chunks {
                        return None;
                    }
                }
                
                // Estimate size from chunks
                let size_estimate_kb = chunks.iter()
                    .filter_map(|c| c.get("chunk_size")?.as_u64())
                    .sum::<u64>() as f64 / 1024.0;
                
                let last_indexed = chunks.iter()
                    .filter_map(|c| c.get("indexed_at")?.as_str())
                    .max()
                    .unwrap_or("unknown")
                    .to_string();
                
                Some(FileInfo {
                    path,
                    file_type,
                    chunk_count: chunks.len(),
                    size_estimate_kb,
                    last_indexed,
                    has_summary: false,
                })
            })
            .collect();

        // Apply filters
        files = Self::apply_file_filters(files, &filter);

        let total_chunks = files.iter().map(|f| f.chunk_count).sum();

        Ok(FileList {
            collection: collection.to_string(),
            total_files: files.len(),
            files,
            total_chunks,
        })
    }

    // ============================================
    // Priority 1: get_file_summary
    // ============================================

    /// Get extractive or structural summary of indexed files
    pub async fn get_file_summary(
        &self,
        collection: &str,
        file_path: &str,
        summary_type: SummaryType,
        max_sentences: usize,
    ) -> FileOperationResult<FileSummary> {
        info!(
            collection = %collection,
            file_path = %file_path,
            summary_type = ?summary_type,
            "Generating file summary"
        );

        // Validate path
        Self::validate_file_path(file_path)?;

        // Check cache
        let cache_key = format!("{}:{}:{:?}", collection, file_path, summary_type);
        if let Some(cached) = self.cache.get_summary(&cache_key, SUMMARY_CACHE_TTL).await {
            info!(
                collection = %collection,
                file_path = %file_path,
                "Summary retrieved from cache"
            );
            return Ok(cached);
        }

        // Get file content first
        let file_content = self.get_file_content(collection, file_path, ABSOLUTE_MAX_SIZE_KB).await?;

        // Generate extractive summary
        let extractive = if matches!(summary_type, SummaryType::Extractive | SummaryType::Both) {
            Some(Self::generate_extractive_summary(&file_content.content, max_sentences))
        } else {
            None
        };

        // Generate structural summary
        let structural = if matches!(summary_type, SummaryType::Structural | SummaryType::Both) {
            Some(Self::generate_structural_summary(&file_content.content))
        } else {
            None
        };

        let summary = FileSummary {
            file_path: file_path.to_string(),
            extractive_summary: extractive,
            structural_summary: structural,
            metadata: FileSummaryMetadata {
                chunk_count: file_content.chunks_available,
                file_type: Self::detect_file_type(file_path),
                summary_method: format!("{:?}", summary_type),
            },
            generated_at: Utc::now(),
        };

        // Cache the result
        self.cache.put_summary(cache_key, summary.clone()).await;

        Ok(summary)
    }

    // ============================================
    // Helper Methods
    // ============================================

    fn generate_extractive_summary(content: &str, max_sentences: usize) -> String {
        // Simple extractive summarization: take first N meaningful sentences
        let sentences: Vec<&str> = content
            .split(&['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 20) // Filter out short fragments
            .take(max_sentences)
            .collect();
        
        sentences.join(". ") + if !sentences.is_empty() { "." } else { "" }
    }

    fn generate_structural_summary(content: &str) -> StructuralSummary {
        let mut outline = String::new();
        let mut key_sections = Vec::new();
        let mut key_points = Vec::new();
        
        // Extract headers (markdown style)
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Markdown headers
            if trimmed.starts_with('#') {
                outline.push_str(line);
                outline.push('\n');
                
                let section = trimmed.trim_start_matches('#').trim();
                if !section.is_empty() && key_sections.len() < 10 {
                    key_sections.push(section.to_string());
                }
            }
            
            // Key points (lines with important keywords)
            let important_keywords = ["important", "note:", "warning:", "critical", "must", "required", "TODO", "FIXME"];
            if important_keywords.iter().any(|kw| trimmed.to_lowercase().contains(kw)) {
                if trimmed.len() > 20 && trimmed.len() < 200 && key_points.len() < 10 {
                    key_points.push(trimmed.to_string());
                }
            }
        }
        
        StructuralSummary {
            outline,
            key_sections,
            key_points,
        }
    }

    /// Validate file path for security
    fn validate_file_path(path: &str) -> FileOperationResult<()> {
        // Prevent directory traversal
        if path.contains("..") {
            return Err(FileOperationError::InvalidPath {
                path: path.to_string(),
                reason: "Path contains directory traversal".to_string(),
            });
        }

        // Ensure relative path
        if path.starts_with('/') || path.starts_with('\\') {
            return Err(FileOperationError::InvalidPath {
                path: path.to_string(),
                reason: "Absolute paths not allowed".to_string(),
            });
        }

        // Check for empty path
        if path.trim().is_empty() {
            return Err(FileOperationError::InvalidPath {
                path: path.to_string(),
                reason: "Path cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Validate size limit parameter
    fn validate_size_limit(max_size_kb: usize) -> FileOperationResult<()> {
        if max_size_kb > ABSOLUTE_MAX_SIZE_KB {
            return Err(FileOperationError::InvalidParameter {
                param: "max_size_kb".to_string(),
                reason: format!("Exceeds absolute limit of {}", ABSOLUTE_MAX_SIZE_KB),
            });
        }

        if max_size_kb == 0 {
            return Err(FileOperationError::InvalidParameter {
                param: "max_size_kb".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Detect file type from extension
    fn detect_file_type(file_path: &str) -> String {
        Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Detect programming language from file extension
    fn detect_language(file_path: &str) -> Option<String> {
        let ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())?;

        let language = match ext {
            "rs" => "rust",
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "tsx" => "typescript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            "cpp" | "cc" | "cxx" => "cpp",
            "c" | "h" => "c",
            "rb" => "ruby",
            "php" => "php",
            "swift" => "swift",
            "kt" | "kts" => "kotlin",
            "scala" | "sc" => "scala",
            "sh" | "bash" => "shell",
            "md" => "markdown",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" => "xml",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            "sql" => "sql",
            _ => return None,
        };

        Some(language.to_string())
    }

    /// Apply filters to file list
    fn apply_file_filters(mut files: Vec<FileInfo>, filter: &FileListFilter) -> Vec<FileInfo> {
        // Filter by type
        if let Some(types) = &filter.filter_by_type {
            files.retain(|f| types.contains(&f.file_type));
        }

        // Filter by minimum chunks
        if let Some(min_chunks) = filter.min_chunks {
            files.retain(|f| f.chunk_count >= min_chunks);
        }

        // Sort
        match filter.sort_by {
            SortBy::Name => files.sort_by(|a, b| a.path.cmp(&b.path)),
            SortBy::Size => files.sort_by(|a, b| {
                b.size_estimate_kb.partial_cmp(&a.size_estimate_kb).unwrap()
            }),
            SortBy::Chunks => files.sort_by(|a, b| b.chunk_count.cmp(&a.chunk_count)),
            SortBy::Recent => files.sort_by(|a, b| b.last_indexed.cmp(&a.last_indexed)),
        }

        // Limit results
        if let Some(max_results) = filter.max_results {
            files.truncate(max_results);
        }

        files
    }

    // ============================================
    // Priority 2: get_file_chunks_ordered
    // ============================================

    /// Retrieve chunks in original file order for progressive reading
    pub async fn get_file_chunks_ordered(
        &self,
        collection: &str,
        file_path: &str,
        start_chunk: usize,
        limit: usize,
        include_context: bool,
    ) -> FileOperationResult<FileChunksOrdered> {
        info!(
            collection = %collection,
            file_path = %file_path,
            start_chunk = start_chunk,
            limit = limit,
            "Fetching ordered file chunks"
        );

        // Validate path
        Self::validate_file_path(file_path)?;

        // Get VectorStore
        let store = self.store.as_ref()
            .ok_or_else(|| FileOperationError::VectorStoreError("VectorStore not initialized".to_string()))?;

        // Get collection
        let coll = store.get_collection(collection)
            .map_err(|e| FileOperationError::CollectionNotFound {
                collection: collection.to_string(),
            })?;

        // Get all vectors and filter by file_path
        let all_vectors = coll.get_all_vectors();
        let mut file_chunks: Vec<_> = all_vectors.into_iter()
            .filter_map(|v| {
                if let Some(payload) = &v.payload {
                    if let Some(metadata) = payload.data.get("metadata") {
                        if let Some(fp) = metadata.get("file_path").and_then(|v| v.as_str()) {
                            if fp == file_path {
                                let chunk_index = metadata.get("chunk_index")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as usize;
                                let content = payload.data.get("content")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let line_start = metadata.get("line_start")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as usize);
                                let line_end = metadata.get("line_end")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as usize);
                                
                                return Some((chunk_index, content, line_start, line_end));
                            }
                        }
                    }
                }
                None
            })
            .collect();

        if file_chunks.is_empty() {
            return Err(FileOperationError::FileNotFound {
                file_path: file_path.to_string(),
                collection: collection.to_string(),
            });
        }

        // Sort by chunk_index
        file_chunks.sort_by_key(|(index, _, _, _)| *index);

        let total_chunks = file_chunks.len();
        let end_chunk = std::cmp::min(start_chunk + limit, total_chunks);

        // Get requested range
        let chunks: Vec<FileChunk> = file_chunks
            .iter()
            .skip(start_chunk)
            .take(limit)
            .map(|(index, content, line_start, line_end)| {
                let (prev_hint, next_hint) = if include_context {
                    let prev = if *index > 0 {
                        file_chunks.get(index - 1)
                            .map(|(_, c, _, _)| c.chars().take(50).collect::<String>())
                    } else {
                        None
                    };
                    let next = file_chunks.get(index + 1)
                        .map(|(_, c, _, _)| c.chars().take(50).collect::<String>());
                    (prev, next)
                } else {
                    (None, None)
                };

                FileChunk {
                    chunk_index: *index,
                    content: content.clone(),
                    line_range: if line_start.is_some() && line_end.is_some() {
                        Some((line_start.unwrap(), line_end.unwrap()))
                    } else {
                        None
                    },
                    prev_chunk_hint: prev_hint,
                    next_chunk_hint: next_hint,
                }
            })
            .collect();

        Ok(FileChunksOrdered {
            file_path: file_path.to_string(),
            total_chunks,
            chunks,
            has_more: end_chunk < total_chunks,
        })
    }

    // ============================================
    // Priority 2: get_project_outline
    // ============================================

    /// Generate hierarchical project structure overview
    pub async fn get_project_outline(
        &self,
        collection: &str,
        max_depth: usize,
        include_summaries: bool,
        highlight_key_files: bool,
    ) -> FileOperationResult<ProjectOutline> {
        info!(
            collection = %collection,
            max_depth = max_depth,
            "Generating project outline"
        );

        // Get file list first
        let file_list = self.list_files_in_collection(
            collection,
            FileListFilter {
                filter_by_type: None,
                min_chunks: None,
                max_results: None,
                sort_by: SortBy::Name,
            },
        ).await?;

        // Build directory tree
        let mut root = DirectoryNode2 {
            name: "/".to_string(),
            node_type: NodeType::Directory,
            children: Vec::new(),
            file_info: None,
            is_key_file: false,
            summary: None,
        };

        // Key file patterns
        let key_patterns = vec!["README", "LICENSE", "Cargo.toml", "package.json", "pyproject.toml", "go.mod"];

        for file in &file_list.files {
            let path = file.path.replace("\\", "/");
            let parts: Vec<&str> = path.split('/').collect();
            
            // Skip if depth exceeds max_depth
            if parts.len() > max_depth {
                continue;
            }

            let mut current = &mut root;

            // Build tree
            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;

                if is_last {
                    // It's a file
                    let is_key = highlight_key_files && key_patterns.iter().any(|p| part.contains(p));
                    
                    let summary = if include_summaries && is_key {
                        // Get summary for key files
                        self.get_file_summary(collection, &file.path, SummaryType::Extractive, 3)
                            .await
                            .ok()
                            .and_then(|s| s.extractive_summary)
                    } else {
                        None
                    };

                    current.children.push(DirectoryNode2 {
                        name: part.to_string(),
                        node_type: NodeType::File,
                        children: Vec::new(),
                        file_info: Some(file.clone()),
                        is_key_file: is_key,
                        summary,
                    });
                } else {
                    // It's a directory
                    let dir_name = part.to_string();
                    
                    // Find or create directory
                    let dir_index = current.children.iter()
                        .position(|n| n.name == dir_name && matches!(n.node_type, NodeType::Directory));
                    
                    if let Some(idx) = dir_index {
                        current = &mut current.children[idx];
                    } else {
                        current.children.push(DirectoryNode2 {
                            name: dir_name.clone(),
                            node_type: NodeType::Directory,
                            children: Vec::new(),
                            file_info: None,
                            is_key_file: false,
                            summary: None,
                        });
                        let new_idx = current.children.len() - 1;
                        current = &mut current.children[new_idx];
                    }
                }
            }
        }

        // Collect key files
        let key_files: Vec<String> = file_list.files.iter()
            .filter(|f| key_patterns.iter().any(|p| f.path.contains(p)))
            .map(|f| f.path.clone())
            .collect();

        // Calculate statistics
        let mut file_types: HashMap<String, usize> = HashMap::new();
        for file in &file_list.files {
            *file_types.entry(file.file_type.clone()).or_insert(0) += 1;
        }

        let statistics = ProjectStatistics {
            total_files: file_list.total_files,
            total_directories: Self::count_directories(&root),
            file_types,
        };

        Ok(ProjectOutline {
            collection: collection.to_string(),
            structure: root,
            key_files,
            statistics,
        })
    }

    fn count_directories(node: &DirectoryNode2) -> usize {
        let mut count = if matches!(node.node_type, NodeType::Directory) { 1 } else { 0 };
        for child in &node.children {
            count += Self::count_directories(child);
        }
        count
    }

    // ============================================
    // Priority 3: search_by_file_type
    // ============================================

    /// Semantic search filtered by file type
    pub async fn search_by_file_type(
        &self,
        collection: &str,
        query: &str,
        file_types: Vec<String>,
        limit: usize,
        return_full_files: bool,
        embedding_manager: &Arc<crate::embedding::EmbeddingManager>,
    ) -> FileOperationResult<FileTypeSearchResult> {
        info!(
            collection = %collection,
            query = %query,
            file_types = ?file_types,
            limit = limit,
            "Searching by file type"
        );

        // Get VectorStore
        let store = self.store.as_ref()
            .ok_or_else(|| FileOperationError::VectorStoreError("VectorStore not initialized".to_string()))?;

        // Get collection
        let coll = store.get_collection(collection)
            .map_err(|e| FileOperationError::CollectionNotFound {
                collection: collection.to_string(),
            })?;

        // Generate embedding for query
        let query_embedding = embedding_manager.embed(query)
            .map_err(|e| FileOperationError::VectorStoreError(format!("Failed to embed query: {}", e)))?;

        // Search in collection
        let search_results = coll.search(&query_embedding, limit * 3) // Get more results to filter
            .map_err(|e| FileOperationError::VectorStoreError(format!("Search failed: {}", e)))?;

        // Filter by file type and build results
        let mut results = Vec::new();
        let mut seen_files = std::collections::HashSet::new();

        for result in search_results {
            if results.len() >= limit {
                break;
            }

            if let Some(payload) = &result.payload {
                if let Some(metadata) = payload.data.get("metadata") {
                    if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                        let file_type = Self::detect_file_type(file_path);
                        
                        // Check if file type matches
                        if file_types.contains(&file_type) {
                            let content = if return_full_files && !seen_files.contains(file_path) {
                                seen_files.insert(file_path.to_string());
                                self.get_file_content(collection, file_path, ABSOLUTE_MAX_SIZE_KB)
                                    .await
                                    .ok()
                                    .map(|f| f.content)
                            } else {
                                None
                            };

                            let chunk_content = payload.data.get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            results.push(FileTypeSearchMatch {
                                file_path: file_path.to_string(),
                                file_type: file_type.clone(),
                                score: result.score,
                                matching_chunk: chunk_content,
                                full_content: content,
                            });
                        }
                    }
                }
            }
        }

        let total_matches = results.len();

        Ok(FileTypeSearchResult {
            query: query.to_string(),
            file_types,
            results,
            total_matches,
        })
    }

    // ============================================
    // Priority 3: get_related_files
    // ============================================

    /// Find semantically related files using vector similarity
    pub async fn get_related_files(
        &self,
        collection: &str,
        file_path: &str,
        limit: usize,
        similarity_threshold: f32,
        include_reason: bool,
        embedding_manager: &Arc<crate::embedding::EmbeddingManager>,
    ) -> FileOperationResult<RelatedFiles> {
        info!(
            collection = %collection,
            file_path = %file_path,
            limit = limit,
            "Finding related files"
        );

        // Validate path
        Self::validate_file_path(file_path)?;

        // Get the file content to create a representative embedding
        let file_content = self.get_file_content(collection, file_path, ABSOLUTE_MAX_SIZE_KB).await?;

        // Create a query from first few chunks (or summary)
        let query_text = file_content.content.chars().take(1000).collect::<String>();

        // Get VectorStore
        let store = self.store.as_ref()
            .ok_or_else(|| FileOperationError::VectorStoreError("VectorStore not initialized".to_string()))?;

        // Get collection
        let coll = store.get_collection(collection)
            .map_err(|e| FileOperationError::CollectionNotFound {
                collection: collection.to_string(),
            })?;

        // Generate embedding
        let query_embedding = embedding_manager.embed(&query_text)
            .map_err(|e| FileOperationError::VectorStoreError(format!("Failed to embed query: {}", e)))?;

        // Search for similar chunks
        let search_results = coll.search(&query_embedding, limit * 5)
            .map_err(|e| FileOperationError::VectorStoreError(format!("Search failed: {}", e)))?;

        // Group by file and calculate average similarity
        let mut file_scores: HashMap<String, (f32, usize)> = HashMap::new();

        for result in search_results {
            if let Some(payload) = &result.payload {
                if let Some(metadata) = payload.data.get("metadata") {
                    if let Some(fp) = metadata.get("file_path").and_then(|v| v.as_str()) {
                        // Skip the source file itself
                        if fp == file_path {
                            continue;
                        }

                        let entry = file_scores.entry(fp.to_string()).or_insert((0.0, 0));
                        entry.0 += result.score;
                        entry.1 += 1;
                    }
                }
            }
        }

        // Calculate average scores and filter
        let mut related_files: Vec<_> = file_scores
            .into_iter()
            .map(|(path, (total_score, count))| {
                let avg_score = total_score / count as f32;
                (path, avg_score)
            })
            .filter(|(_, score)| *score >= similarity_threshold)
            .collect();

        // Sort by score descending
        related_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        related_files.truncate(limit);

        // Build results
        let results: Vec<RelatedFile> = related_files
            .into_iter()
            .map(|(path, score)| {
                let reason = if include_reason {
                    Some(Self::generate_relation_reason(&file_path, &path, score))
                } else {
                    None
                };

                RelatedFile {
                    path,
                    similarity_score: score,
                    reason,
                    shared_concepts: None,
                }
            })
            .collect();

        Ok(RelatedFiles {
            source_file: file_path.to_string(),
            related_files: results,
        })
    }

    fn generate_relation_reason(source: &str, related: &str, score: f32) -> String {
        let source_type = Self::detect_file_type(source);
        let related_type = Self::detect_file_type(related);

        if source_type == related_type {
            format!("Similar {} file with {:.1}% semantic similarity", source_type, score * 100.0)
        } else {
            format!("Related {} file (from {}) with {:.1}% semantic similarity", related_type, source_type, score * 100.0)
        }
    }

    /// Clear cache for a specific collection
    pub async fn clear_cache(&self, collection: &str) {
        info!(collection = %collection, "Clearing cache");
        self.cache.clear_collection(collection).await;
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats().await
    }
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_file_path() {
        // Valid paths
        assert!(FileOperations::validate_file_path("src/main.rs").is_ok());
        assert!(FileOperations::validate_file_path("docs/README.md").is_ok());

        // Invalid paths
        assert!(FileOperations::validate_file_path("../etc/passwd").is_err());
        assert!(FileOperations::validate_file_path("/absolute/path").is_err());
        assert!(FileOperations::validate_file_path("").is_err());
    }

    #[tokio::test]
    async fn test_validate_size_limit() {
        assert!(FileOperations::validate_size_limit(100).is_ok());
        assert!(FileOperations::validate_size_limit(1000).is_ok());
        assert!(FileOperations::validate_size_limit(0).is_err());
        assert!(FileOperations::validate_size_limit(10000).is_err());
    }

    #[tokio::test]
    async fn test_detect_file_type() {
        assert_eq!(FileOperations::detect_file_type("main.rs"), "rs");
        assert_eq!(FileOperations::detect_file_type("index.ts"), "ts");
        assert_eq!(FileOperations::detect_file_type("README.md"), "md");
        assert_eq!(FileOperations::detect_file_type("noextension"), "unknown");
    }

    #[tokio::test]
    async fn test_detect_language() {
        assert_eq!(FileOperations::detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(FileOperations::detect_language("app.py"), Some("python".to_string()));
        assert_eq!(FileOperations::detect_language("config.toml"), Some("toml".to_string()));
        assert_eq!(FileOperations::detect_language("unknown.xyz"), None);
    }

    #[tokio::test]
    async fn test_extractive_summary() {
        let content = "This is a test. This is another sentence with more content. Short. This is a third meaningful sentence.";
        let summary = FileOperations::generate_extractive_summary(content, 2);
        
        // Should have 2 sentences (filtered by length > 20)
        assert!(summary.contains("another sentence"));
        assert!(summary.contains("meaningful sentence"));
    }

    #[tokio::test]
    async fn test_structural_summary() {
        let content = r#"
# Main Title
Some content here.

## Subsection 1
Important: This is a critical point.

## Subsection 2
Note: Another important detail.
        "#;
        
        let summary = FileOperations::generate_structural_summary(content);
        
        assert!(summary.outline.contains("# Main Title"));
        assert!(summary.key_sections.contains(&"Main Title".to_string()));
        assert!(summary.key_points.len() > 0);
    }
}

