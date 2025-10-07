# File-Level MCP Tools - Technical Implementation Specification

**Version**: 1.0  
**Date**: October 7, 2025  
**Status**: Proposed  
**Priority**: High  
**Estimated Effort**: 2-3 weeks

## 🎯 Executive Summary

Current MCP implementation works exclusively with chunks, forcing LLMs to fall back to traditional `read_file` operations when full file context is needed. This specification proposes 7 new file-level tools that bridge the gap between chunk-based search and complete file access, making the MCP a complete solution for code understanding and documentation tasks.

**Key Problem**: LLMs prefer reading entire files over fragmented chunks for context understanding and summarization, bypassing the faster MCP system.

**Solution**: Implement file-level abstraction layer on top of existing chunk infrastructure.

---

## 📋 Proposed Tools Overview

### Priority 1 (Critical - Week 1)
1. **`get_file_content`** - Retrieve complete indexed files
2. **`get_file_summary`** - Get file-level summaries
3. **`list_files_in_collection`** - Discover available files

### Priority 2 (High Value - Week 2)
4. **`get_file_chunks_ordered`** - Progressive file reading
5. **`get_project_outline`** - Project structure overview

### Priority 3 (Enhancement - Week 3)
6. **`get_related_files`** - Semantic file navigation
7. **`search_by_file_type`** - Type-aware search

---

## 🔧 Technical Implementation

### Architecture Changes

```rust
// New module: src/mcp/file_operations.rs

pub struct FileOperations {
    store: Arc<VectorStore>,
    file_cache: Arc<RwLock<LruCache<String, CachedFile>>>,
    summarizer: Arc<Summarizer>,
}

pub struct CachedFile {
    path: String,
    content: String,
    chunks: Vec<ChunkReference>,
    summary: Option<String>,
    metadata: FileMetadata,
    cached_at: DateTime<Utc>,
}

pub struct FileMetadata {
    file_type: String,
    size_bytes: usize,
    chunk_count: usize,
    last_indexed: DateTime<Utc>,
    language: Option<String>,
}
```

---

## 📖 Tool Specifications

### 1. `get_file_content`

**Purpose**: Retrieve complete file content from indexed collections.

**Parameters**:
```typescript
interface GetFileContentParams {
  collection: string;           // Collection name
  file_path: string;            // Relative file path
  max_size_kb?: number;         // Safety limit (default: 500kb)
  include_metadata?: boolean;   // Include file metadata (default: true)
}
```

**Response**:
```typescript
interface GetFileContentResponse {
  file_path: string;
  content: string;              // Complete file content
  metadata: {
    file_type: string;
    size_kb: number;
    chunk_count: number;
    last_indexed: string;       // ISO 8601
    language?: string;
  };
  chunks_available: number;
  collection: string;
  from_cache: boolean;
}
```

**Implementation**:
```rust
pub async fn get_file_content(
    &self,
    collection: &str,
    file_path: &str,
    max_size_kb: usize,
) -> Result<FileContent, FileOperationError> {
    // 1. Check cache first
    if let Some(cached) = self.file_cache.read().await.get(&format!("{}:{}", collection, file_path)) {
        if cached.is_fresh(Duration::minutes(10)) {
            return Ok(cached.to_content());
        }
    }
    
    // 2. Query all chunks for this file
    let chunks = self.store
        .search_with_filter(collection, "", |meta| {
            meta.get("file_path") == Some(&json!(file_path))
        })
        .await?;
    
    // 3. Sort by chunk_index
    let mut sorted_chunks: Vec<_> = chunks
        .into_iter()
        .filter_map(|chunk| {
            let index = chunk.metadata.get("chunk_index")?.as_u64()?;
            Some((index, chunk))
        })
        .collect();
    sorted_chunks.sort_by_key(|(index, _)| *index);
    
    // 4. Reconstruct file content
    let content = sorted_chunks
        .iter()
        .map(|(_, chunk)| chunk.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    
    // 5. Check size limit
    let size_kb = content.len() / 1024;
    if size_kb > max_size_kb {
        return Err(FileOperationError::FileTooLarge {
            size_kb,
            max_size_kb,
        });
    }
    
    // 6. Extract metadata
    let metadata = self.extract_file_metadata(&sorted_chunks[0].1)?;
    
    // 7. Cache result
    let cached_file = CachedFile {
        path: file_path.to_string(),
        content: content.clone(),
        chunks: sorted_chunks.iter().map(|(_, c)| c.id.clone()).collect(),
        summary: None,
        metadata,
        cached_at: Utc::now(),
    };
    
    self.file_cache.write().await.put(
        format!("{}:{}", collection, file_path),
        cached_file,
    );
    
    Ok(FileContent {
        file_path: file_path.to_string(),
        content,
        metadata,
        chunks_available: sorted_chunks.len(),
        collection: collection.to_string(),
        from_cache: false,
    })
}
```

**MCP Endpoint**:
```json
{
  "name": "get_file_content",
  "description": "Retrieve complete file content from a collection",
  "inputSchema": {
    "type": "object",
    "properties": {
      "collection": {
        "type": "string",
        "description": "Collection name"
      },
      "file_path": {
        "type": "string",
        "description": "Relative file path within collection"
      },
      "max_size_kb": {
        "type": "number",
        "description": "Maximum file size in KB (default: 500)",
        "default": 500
      }
    },
    "required": ["collection", "file_path"]
  }
}
```

**Use Cases**:
- Read complete configuration files
- Analyze entire source files for refactoring
- Generate documentation from full context
- Code review of specific files

---

### 2. `get_file_summary`

**Purpose**: Get extractive or structural summary of indexed files.

**Parameters**:
```typescript
interface GetFileSummaryParams {
  collection: string;
  file_path: string;
  summary_type?: 'extractive' | 'structural' | 'both';  // default: 'both'
  max_sentences?: number;                                // default: 5
  include_outline?: boolean;                             // default: true
}
```

**Response**:
```typescript
interface GetFileSummaryResponse {
  file_path: string;
  extractive_summary?: string;      // Key sentences extracted
  structural_summary?: {
    outline: string;                // Markdown outline
    key_sections: string[];
    key_points: string[];
  };
  metadata: {
    chunk_count: number;
    file_type: string;
    summary_method: string;
  };
  generated_at: string;
}
```

**Implementation**:
```rust
pub async fn get_file_summary(
    &self,
    collection: &str,
    file_path: &str,
    summary_type: SummaryType,
    max_sentences: usize,
) -> Result<FileSummary, FileOperationError> {
    // 1. Get all chunks for file
    let chunks = self.get_file_chunks(collection, file_path).await?;
    
    // 2. Check if summary already exists in cache
    let cache_key = format!("{}:{}:summary", collection, file_path);
    if let Some(cached) = self.file_cache.read().await.get(&cache_key) {
        if let Some(summary) = &cached.summary {
            return Ok(summary.clone());
        }
    }
    
    // 3. Generate extractive summary
    let extractive = if matches!(summary_type, SummaryType::Extractive | SummaryType::Both) {
        let combined_text = chunks.iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        Some(self.summarizer.extractive_summary(
            &combined_text,
            max_sentences,
        )?)
    } else {
        None
    };
    
    // 4. Generate structural summary
    let structural = if matches!(summary_type, SummaryType::Structural | SummaryType::Both) {
        Some(self.generate_structural_summary(&chunks)?)
    } else {
        None
    };
    
    Ok(FileSummary {
        file_path: file_path.to_string(),
        extractive_summary: extractive,
        structural_summary: structural,
        metadata: FileSummaryMetadata {
            chunk_count: chunks.len(),
            file_type: self.detect_file_type(file_path),
            summary_method: format!("{:?}", summary_type),
        },
        generated_at: Utc::now(),
    })
}

fn generate_structural_summary(&self, chunks: &[Chunk]) -> Result<StructuralSummary, Error> {
    let mut outline = String::new();
    let mut key_sections = Vec::new();
    let mut key_points = Vec::new();
    
    // Parse markdown headers, code sections, etc.
    for chunk in chunks {
        // Extract headers (##, ###, etc.)
        for line in chunk.content.lines() {
            if line.starts_with('#') {
                outline.push_str(line);
                outline.push('\n');
                
                let section = line.trim_start_matches('#').trim();
                if !section.is_empty() {
                    key_sections.push(section.to_string());
                }
            }
        }
        
        // Extract key points (sentences with important keywords)
        let important_keywords = ["important", "note:", "warning:", "critical", "must", "required"];
        for line in chunk.content.lines() {
            if important_keywords.iter().any(|kw| line.to_lowercase().contains(kw)) {
                let clean = line.trim();
                if clean.len() > 20 && clean.len() < 200 {
                    key_points.push(clean.to_string());
                }
            }
        }
    }
    
    Ok(StructuralSummary {
        outline,
        key_sections,
        key_points: key_points.into_iter().take(10).collect(),
    })
}
```

**Use Cases**:
- Quick understanding of large documentation files
- Generate README summaries
- Extract key points from technical specs
- Overview before diving into details

---

### 3. `list_files_in_collection`

**Purpose**: List all indexed files in a collection with metadata.

**Parameters**:
```typescript
interface ListFilesParams {
  collection: string;
  filter_by_type?: string[];        // e.g., ["rs", "md", "toml"]
  min_chunks?: number;              // Filter small files
  max_results?: number;             // Pagination (default: 100)
  sort_by?: 'name' | 'size' | 'chunks' | 'recent';  // default: 'name'
}
```

**Response**:
```typescript
interface ListFilesResponse {
  collection: string;
  files: FileInfo[];
  total_files: number;
  total_chunks: number;
}

interface FileInfo {
  path: string;
  file_type: string;
  chunk_count: number;
  size_estimate_kb: number;
  last_indexed: string;
  has_summary: boolean;
}
```

**Implementation**:
```rust
pub async fn list_files_in_collection(
    &self,
    collection: &str,
    filter: FileListFilter,
) -> Result<FileList, FileOperationError> {
    // 1. Get all vectors in collection
    let all_chunks = self.store.list_collection_vectors(collection).await?;
    
    // 2. Group by file_path
    let mut file_map: HashMap<String, Vec<Chunk>> = HashMap::new();
    
    for chunk in all_chunks {
        if let Some(file_path) = chunk.metadata.get("file_path").and_then(|v| v.as_str()) {
            file_map.entry(file_path.to_string())
                .or_insert_with(Vec::new)
                .push(chunk);
        }
    }
    
    // 3. Create FileInfo for each file
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
            
            let size_estimate_kb = chunks.iter()
                .map(|c| c.content.len())
                .sum::<usize>() / 1024;
            
            let last_indexed = chunks.iter()
                .filter_map(|c| c.metadata.get("indexed_at"))
                .max()
                .cloned()
                .unwrap_or_default();
            
            Some(FileInfo {
                path,
                file_type,
                chunk_count: chunks.len(),
                size_estimate_kb,
                last_indexed: last_indexed.to_string(),
                has_summary: false, // TODO: check summary cache
            })
        })
        .collect();
    
    // 4. Sort
    match filter.sort_by {
        SortBy::Name => files.sort_by(|a, b| a.path.cmp(&b.path)),
        SortBy::Size => files.sort_by(|a, b| b.size_estimate_kb.cmp(&a.size_estimate_kb)),
        SortBy::Chunks => files.sort_by(|a, b| b.chunk_count.cmp(&a.chunk_count)),
        SortBy::Recent => files.sort_by(|a, b| b.last_indexed.cmp(&a.last_indexed)),
    }
    
    // 5. Paginate
    let total_files = files.len();
    let total_chunks = files.iter().map(|f| f.chunk_count).sum();
    
    if let Some(max) = filter.max_results {
        files.truncate(max);
    }
    
    Ok(FileList {
        collection: collection.to_string(),
        files,
        total_files,
        total_chunks,
    })
}
```

**Use Cases**:
- Discover project structure
- Find configuration files
- Identify large/important files
- Navigation and exploration

---

### 4. `get_file_chunks_ordered`

**Purpose**: Retrieve chunks in original file order for progressive reading.

**Parameters**:
```typescript
interface GetFileChunksOrderedParams {
  collection: string;
  file_path: string;
  start_chunk?: number;     // default: 0
  limit?: number;           // default: 10
  include_context?: boolean; // Include prev/next chunk hints
}
```

**Response**:
```typescript
interface GetFileChunksOrderedResponse {
  file_path: string;
  total_chunks: number;
  chunks: OrderedChunk[];
  has_more: boolean;
  next_start?: number;
}

interface OrderedChunk {
  index: number;
  content: string;
  line_range?: [number, number];
  context_hint?: {
    prev_chunk_preview?: string;
    next_chunk_preview?: string;
  };
}
```

**Implementation**:
```rust
pub async fn get_file_chunks_ordered(
    &self,
    collection: &str,
    file_path: &str,
    start_chunk: usize,
    limit: usize,
) -> Result<OrderedChunks, FileOperationError> {
    // 1. Get all chunks for file
    let mut all_chunks = self.get_file_chunks(collection, file_path).await?;
    
    // 2. Sort by chunk_index
    all_chunks.sort_by_key(|chunk| {
        chunk.metadata
            .get("chunk_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    });
    
    let total_chunks = all_chunks.len();
    
    // 3. Slice requested range
    let end_chunk = std::cmp::min(start_chunk + limit, total_chunks);
    let chunks = all_chunks[start_chunk..end_chunk]
        .iter()
        .enumerate()
        .map(|(offset, chunk)| {
            let index = start_chunk + offset;
            
            OrderedChunk {
                index,
                content: chunk.content.clone(),
                line_range: None, // TODO: extract from metadata
                context_hint: None, // TODO: add previews
            }
        })
        .collect();
    
    Ok(OrderedChunks {
        file_path: file_path.to_string(),
        total_chunks,
        chunks,
        has_more: end_chunk < total_chunks,
        next_start: if end_chunk < total_chunks {
            Some(end_chunk)
        } else {
            None
        },
    })
}
```

**Use Cases**:
- Progressive file reading without loading entire file
- Streaming large files
- Context-aware chunk reading
- Memory-efficient file processing

---

### 5. `get_project_outline`

**Purpose**: Generate hierarchical project structure overview.

**Parameters**:
```typescript
interface GetProjectOutlineParams {
  collection: string;
  max_depth?: number;           // default: 5
  include_summaries?: boolean;  // default: false
  highlight_key_files?: boolean; // README, etc. (default: true)
}
```

**Response**:
```typescript
interface ProjectOutlineResponse {
  collection: string;
  structure: DirectoryNode;
  key_files: KeyFileInfo[];
  statistics: {
    total_files: number;
    total_directories: number;
    file_types: Record<string, number>;
  };
}

interface DirectoryNode {
  name: string;
  type: 'directory' | 'file';
  children?: DirectoryNode[];
  file_info?: {
    chunks: number;
    size_kb: number;
    summary?: string;
  };
}
```

**Implementation**:
```rust
pub async fn get_project_outline(
    &self,
    collection: &str,
    options: OutlineOptions,
) -> Result<ProjectOutline, FileOperationError> {
    // 1. List all files
    let file_list = self.list_files_in_collection(collection, FileListFilter::default()).await?;
    
    // 2. Build tree structure
    let mut root = DirectoryNode::new("/");
    
    for file in &file_list.files {
        let parts: Vec<&str> = file.path.split('/').collect();
        let mut current = &mut root;
        
        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            
            if is_last {
                // File node
                current.children.push(DirectoryNode {
                    name: part.to_string(),
                    node_type: NodeType::File,
                    children: Vec::new(),
                    file_info: Some(FileNodeInfo {
                        chunks: file.chunk_count,
                        size_kb: file.size_estimate_kb,
                        summary: None,
                    }),
                });
            } else {
                // Directory node
                if let Some(child) = current.children.iter_mut()
                    .find(|c| c.name == *part && c.node_type == NodeType::Directory) {
                    current = child;
                } else {
                    current.children.push(DirectoryNode {
                        name: part.to_string(),
                        node_type: NodeType::Directory,
                        children: Vec::new(),
                        file_info: None,
                    });
                    current = current.children.last_mut().unwrap();
                }
            }
        }
    }
    
    // 3. Identify key files
    let key_files = self.identify_key_files(&file_list.files);
    
    // 4. Calculate statistics
    let mut file_types: HashMap<String, usize> = HashMap::new();
    for file in &file_list.files {
        *file_types.entry(file.file_type.clone()).or_insert(0) += 1;
    }
    
    Ok(ProjectOutline {
        collection: collection.to_string(),
        structure: root,
        key_files,
        statistics: ProjectStatistics {
            total_files: file_list.total_files,
            total_directories: self.count_directories(&root),
            file_types,
        },
    })
}

fn identify_key_files(&self, files: &[FileInfo]) -> Vec<KeyFileInfo> {
    let key_names = ["README.md", "CHANGELOG.md", "package.json", "Cargo.toml", "main.rs", "index.ts"];
    
    files.iter()
        .filter(|f| key_names.iter().any(|key| f.path.ends_with(key)))
        .map(|f| KeyFileInfo {
            path: f.path.clone(),
            reason: "Important project file".to_string(),
            chunk_count: f.chunk_count,
        })
        .collect()
}
```

**Use Cases**:
- Project exploration and navigation
- Understanding codebase structure
- Documentation generation
- Onboarding new developers

---

### 6. `get_related_files`

**Purpose**: Find semantically related files using vector similarity.

**Parameters**:
```typescript
interface GetRelatedFilesParams {
  collection: string;
  file_path: string;
  limit?: number;               // default: 5
  similarity_threshold?: number; // default: 0.6
  include_reason?: boolean;     // default: true
}
```

**Response**:
```typescript
interface GetRelatedFilesResponse {
  source_file: string;
  related_files: RelatedFile[];
}

interface RelatedFile {
  path: string;
  similarity_score: number;
  reason?: string;
  shared_concepts?: string[];
}
```

**Implementation**:
```rust
pub async fn get_related_files(
    &self,
    collection: &str,
    file_path: &str,
    limit: usize,
    threshold: f32,
) -> Result<RelatedFiles, FileOperationError> {
    // 1. Get source file chunks
    let source_chunks = self.get_file_chunks(collection, file_path).await?;
    
    // 2. Create aggregated embedding (average of all chunks)
    let source_embedding = self.aggregate_chunk_embeddings(&source_chunks)?;
    
    // 3. Search for similar vectors
    let similar = self.store
        .search_by_vector(collection, &source_embedding, limit * 5) // Get more candidates
        .await?;
    
    // 4. Group by file and calculate average similarity
    let mut file_scores: HashMap<String, Vec<f32>> = HashMap::new();
    
    for result in similar {
        if let Some(path) = result.metadata.get("file_path").and_then(|v| v.as_str()) {
            if path != file_path { // Exclude source file
                file_scores.entry(path.to_string())
                    .or_insert_with(Vec::new)
                    .push(result.score);
            }
        }
    }
    
    // 5. Calculate average similarity per file
    let mut related: Vec<_> = file_scores
        .into_iter()
        .map(|(path, scores)| {
            let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
            (path, avg_score)
        })
        .filter(|(_, score)| *score >= threshold)
        .collect();
    
    related.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    related.truncate(limit);
    
    // 6. Generate reasons (analyze why files are related)
    let related_files = related
        .into_iter()
        .map(|(path, score)| RelatedFile {
            path: path.clone(),
            similarity_score: score,
            reason: Some(self.analyze_relationship(file_path, &path)),
            shared_concepts: None, // TODO: extract shared keywords
        })
        .collect();
    
    Ok(RelatedFiles {
        source_file: file_path.to_string(),
        related_files,
    })
}

fn aggregate_chunk_embeddings(&self, chunks: &[Chunk]) -> Result<Vec<f32>, Error> {
    if chunks.is_empty() {
        return Err(Error::EmptyChunkList);
    }
    
    let dimension = chunks[0].embedding.len();
    let mut aggregated = vec![0.0f32; dimension];
    
    for chunk in chunks {
        for (i, val) in chunk.embedding.iter().enumerate() {
            aggregated[i] += val;
        }
    }
    
    // Average
    for val in &mut aggregated {
        *val /= chunks.len() as f32;
    }
    
    // Normalize
    let magnitude: f32 = aggregated.iter().map(|v| v * v).sum::<f32>().sqrt();
    for val in &mut aggregated {
        *val /= magnitude;
    }
    
    Ok(aggregated)
}
```

**Use Cases**:
- Navigate related code modules
- Find similar documentation
- Discover dependencies
- Code exploration

---

### 7. `search_by_file_type`

**Purpose**: Semantic search filtered by file type.

**Parameters**:
```typescript
interface SearchByFileTypeParams {
  collection: string;
  query: string;
  file_types: string[];         // e.g., ["yaml", "toml", "json"]
  limit?: number;               // default: 10
  return_full_files?: boolean;  // default: false
}
```

**Response**:
```typescript
interface SearchByFileTypeResponse {
  query: string;
  file_types: string[];
  results: FileSearchResult[];
}

interface FileSearchResult {
  file_path: string;
  file_type: string;
  relevance_score: number;
  matching_chunks?: ChunkMatch[];
  full_content?: string;        // if return_full_files = true
}
```

**Implementation**:
```rust
pub async fn search_by_file_type(
    &self,
    collection: &str,
    query: &str,
    file_types: &[String],
    limit: usize,
    return_full: bool,
) -> Result<FileTypeSearchResults, FileOperationError> {
    // 1. Perform semantic search
    let search_results = self.store
        .search(collection, query, limit * 3) // Get more candidates
        .await?;
    
    // 2. Filter by file type and group by file
    let mut file_results: HashMap<String, Vec<SearchResult>> = HashMap::new();
    
    for result in search_results {
        if let Some(path) = result.metadata.get("file_path").and_then(|v| v.as_str()) {
            let file_ext = Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            
            if file_types.contains(&file_ext.to_string()) {
                file_results.entry(path.to_string())
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }
    }
    
    // 3. Calculate file-level relevance
    let mut results: Vec<_> = file_results
        .into_iter()
        .map(|(path, chunks)| {
            let avg_score = chunks.iter().map(|c| c.score).sum::<f32>() / chunks.len() as f32;
            let file_type = Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            
            let matching_chunks = chunks
                .into_iter()
                .map(|c| ChunkMatch {
                    chunk_index: c.metadata.get("chunk_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    content: c.content,
                    score: c.score,
                })
                .collect();
            
            (path, file_type, avg_score, matching_chunks)
        })
        .collect();
    
    results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    results.truncate(limit);
    
    // 4. Optionally load full files
    let file_results: Vec<_> = results
        .into_iter()
        .map(|(path, file_type, score, chunks)| {
            let full_content = if return_full {
                self.get_file_content(collection, &path, 1000)
                    .ok()
                    .map(|f| f.content)
            } else {
                None
            };
            
            FileSearchResult {
                file_path: path,
                file_type,
                relevance_score: score,
                matching_chunks: Some(chunks),
                full_content,
            }
        })
        .collect();
    
    Ok(FileTypeSearchResults {
        query: query.to_string(),
        file_types: file_types.to_vec(),
        results: file_results,
    })
}
```

**Use Cases**:
- Find configuration examples
- Search specific file types (configs, docs, tests)
- Type-specific code search
- Documentation-only queries

---

## 🔄 Enhancements to Existing Tools

### `discover` Enhancement

**Add to response**:
```rust
pub struct DiscoveryResult {
    pub answer_prompt: String,
    pub bullets: usize,
    pub chunks: usize,
    pub metrics: DiscoveryMetrics,
    
    // NEW: File-level information
    pub source_files: Vec<SourceFileInfo>,
}

pub struct SourceFileInfo {
    pub path: String,
    pub relevant_chunks: Vec<usize>,
    pub relevance_score: f32,
    pub has_full_file_available: bool,  // Can fetch with get_file_content
    pub file_summary: Option<String>,
}
```

### `search_vectors` Enhancement

**Add parameters**:
```rust
pub struct SearchParams {
    pub query: String,
    pub limit: usize,
    
    // NEW: Grouping options
    pub group_by_file: bool,           // Group results by file
    pub include_file_summary: bool,     // Include file summaries
    pub max_chunks_per_file: Option<usize>, // Limit chunks per file
}
```

### `intelligent_search` Enhancement

**Add file-level metadata to results**:
```rust
pub struct IntelligentSearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: ChunkMetadata,
    
    // NEW: File context
    pub file_context: FileContext,
}

pub struct FileContext {
    pub file_path: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub file_summary: Option<String>,
    pub can_fetch_full: bool,
}
```

---

## 📊 Performance Considerations

### Caching Strategy

```rust
pub struct FileLevelCache {
    // LRU cache for complete files (max 100 files)
    file_content_cache: LruCache<String, CachedFile>,
    
    // LRU cache for summaries (max 500 summaries)
    summary_cache: LruCache<String, FileSummary>,
    
    // File list cache per collection (TTL: 5 minutes)
    file_list_cache: HashMap<String, (FileList, Instant)>,
    
    // Project outline cache (TTL: 10 minutes)
    outline_cache: HashMap<String, (ProjectOutline, Instant)>,
}
```

**Cache Invalidation**:
- File content: 10 minutes or on file update
- Summaries: 30 minutes
- File lists: 5 minutes
- Outlines: 10 minutes

### Performance Targets

| Operation | Target Latency | Cache Hit | Cache Miss |
|-----------|---------------|-----------|------------|
| `get_file_content` | < 50ms | 5ms | 100ms |
| `get_file_summary` | < 100ms | 10ms | 500ms |
| `list_files` | < 200ms | 20ms | 1s |
| `get_chunks_ordered` | < 50ms | 5ms | 100ms |
| `get_outline` | < 500ms | 50ms | 2s |
| `get_related_files` | < 300ms | - | 300ms |
| `search_by_type` | < 200ms | - | 200ms |

### Memory Budget

- File content cache: 100MB (100 files × 1MB avg)
- Summary cache: 10MB (500 summaries × 20KB avg)
- File list cache: 5MB
- Outline cache: 5MB
- **Total**: ~120MB additional memory

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_file_content_success() {
        let ops = setup_test_file_operations().await;
        
        let result = ops.get_file_content(
            "test-collection",
            "src/main.rs",
            500
        ).await.unwrap();
        
        assert_eq!(result.file_path, "src/main.rs");
        assert!(result.content.len() > 0);
        assert!(result.chunks_available > 0);
    }
    
    #[tokio::test]
    async fn test_get_file_content_too_large() {
        let ops = setup_test_file_operations().await;
        
        let result = ops.get_file_content(
            "test-collection",
            "large_file.json",
            10 // Very small limit
        ).await;
        
        assert!(matches!(result, Err(FileOperationError::FileTooLarge { .. })));
    }
    
    #[tokio::test]
    async fn test_file_summary_extractive() {
        let ops = setup_test_file_operations().await;
        
        let summary = ops.get_file_summary(
            "test-collection",
            "README.md",
            SummaryType::Extractive,
            3
        ).await.unwrap();
        
        assert!(summary.extractive_summary.is_some());
        let text = summary.extractive_summary.unwrap();
        assert!(text.split('.').count() <= 3);
    }
    
    #[tokio::test]
    async fn test_list_files_with_filter() {
        let ops = setup_test_file_operations().await;
        
        let list = ops.list_files_in_collection(
            "test-collection",
            FileListFilter {
                filter_by_type: Some(vec!["rs".to_string()]),
                min_chunks: Some(3),
                ..Default::default()
            }
        ).await.unwrap();
        
        assert!(list.files.iter().all(|f| f.file_type == "rs"));
        assert!(list.files.iter().all(|f| f.chunk_count >= 3));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_file_level_workflow() {
    // 1. List files
    let list = file_ops.list_files_in_collection("vectorizer-source", Default::default())
        .await
        .unwrap();
    
    assert!(list.total_files > 0);
    
    // 2. Get summary of first file
    let first_file = &list.files[0].path;
    let summary = file_ops.get_file_summary(
        "vectorizer-source",
        first_file,
        SummaryType::Both,
        5
    ).await.unwrap();
    
    assert!(summary.extractive_summary.is_some());
    
    // 3. Get full content if needed
    let content = file_ops.get_file_content(
        "vectorizer-source",
        first_file,
        1000
    ).await.unwrap();
    
    assert!(content.content.len() > 0);
    
    // 4. Find related files
    let related = file_ops.get_related_files(
        "vectorizer-source",
        first_file,
        5,
        0.6
    ).await.unwrap();
    
    assert!(related.related_files.len() > 0);
}
```

---

## 📈 Monitoring & Metrics

### Prometheus Metrics

```rust
lazy_static! {
    static ref FILE_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "file_operation_duration_seconds",
        "Duration of file operations",
        &["operation", "collection"]
    ).unwrap();
    
    static ref FILE_CACHE_HITS: CounterVec = register_counter_vec!(
        "file_cache_hits_total",
        "Number of cache hits",
        &["cache_type"]
    ).unwrap();
    
    static ref FILE_CACHE_MISSES: CounterVec = register_counter_vec!(
        "file_cache_misses_total",
        "Number of cache misses",
        &["cache_type"]
    ).unwrap();
    
    static ref FILE_CONTENT_SIZE: HistogramVec = register_histogram_vec!(
        "file_content_size_bytes",
        "Size of retrieved files",
        &["collection"]
    ).unwrap();
}
```

### Logging

```rust
impl FileOperations {
    async fn get_file_content_with_metrics(&self, ...) -> Result<FileContent> {
        let start = Instant::now();
        
        info!(
            collection = %collection,
            file_path = %file_path,
            "Fetching file content"
        );
        
        let result = self.get_file_content(collection, file_path, max_size_kb).await;
        
        let duration = start.elapsed();
        FILE_OPERATION_DURATION
            .with_label_values(&["get_file_content", collection])
            .observe(duration.as_secs_f64());
        
        match &result {
            Ok(content) => {
                info!(
                    collection = %collection,
                    file_path = %file_path,
                    size_kb = content.metadata.size_kb,
                    from_cache = content.from_cache,
                    duration_ms = duration.as_millis(),
                    "File content retrieved successfully"
                );
                
                if content.from_cache {
                    FILE_CACHE_HITS.with_label_values(&["file_content"]).inc();
                } else {
                    FILE_CACHE_MISSES.with_label_values(&["file_content"]).inc();
                }
                
                FILE_CONTENT_SIZE
                    .with_label_values(&[collection])
                    .observe(content.metadata.size_kb as f64 * 1024.0);
            }
            Err(e) => {
                error!(
                    collection = %collection,
                    file_path = %file_path,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "Failed to retrieve file content"
                );
            }
        }
        
        result
    }
}
```

---

## 🚀 Implementation Roadmap

### Week 1: Core Infrastructure (Priority 1)

**Days 1-2**: Foundation
- [ ] Create `src/mcp/file_operations.rs` module
- [ ] Implement `FileOperations` struct with caching
- [ ] Add error types (`FileOperationError`)
- [ ] Setup basic tests

**Days 3-4**: Core Tools
- [ ] Implement `get_file_content`
- [ ] Implement `list_files_in_collection`
- [ ] Add MCP endpoints
- [ ] Integration tests

**Day 5**: Summarization
- [ ] Implement `get_file_summary`
- [ ] Integrate with existing summarizer
- [ ] Add structural summary generation
- [ ] Tests

### Week 2: Advanced Features (Priority 2)

**Days 1-2**: Progressive Reading
- [ ] Implement `get_file_chunks_ordered`
- [ ] Add context hints
- [ ] Pagination support
- [ ] Tests

**Days 3-5**: Project Overview
- [ ] Implement `get_project_outline`
- [ ] Build directory tree structure
- [ ] Identify key files
- [ ] Statistics calculation
- [ ] Tests

### Week 3: Enhancements (Priority 3)

**Days 1-2**: Related Files
- [ ] Implement `get_related_files`
- [ ] Embedding aggregation
- [ ] Relationship analysis
- [ ] Tests

**Days 3-4**: Type-Specific Search
- [ ] Implement `search_by_file_type`
- [ ] File grouping logic
- [ ] Full file fetching option
- [ ] Tests

**Day 5**: Integration & Documentation
- [ ] Update existing tools (discover, search_vectors)
- [ ] Performance optimization
- [ ] Documentation
- [ ] Examples

---

## 🔒 Security Considerations

### File Size Limits

```rust
pub const MAX_FILE_SIZE_KB: usize = 1000; // 1MB default
pub const ABSOLUTE_MAX_SIZE_KB: usize = 5000; // 5MB hard limit

impl FileOperations {
    pub async fn get_file_content(&self, ...) -> Result<FileContent> {
        // Enforce limits
        if max_size_kb > ABSOLUTE_MAX_SIZE_KB {
            return Err(FileOperationError::InvalidParameter {
                param: "max_size_kb",
                reason: format!("Exceeds absolute limit of {}", ABSOLUTE_MAX_SIZE_KB),
            });
        }
        
        // ... implementation
    }
}
```

### Path Validation

```rust
fn validate_file_path(path: &str) -> Result<(), FileOperationError> {
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
    
    Ok(())
}
```

### Rate Limiting

```rust
pub struct FileOperations {
    rate_limiter: Arc<RateLimiter>,
    // ...
}

impl FileOperations {
    pub async fn get_file_content(&self, ...) -> Result<FileContent> {
        // Check rate limit
        self.rate_limiter.check_limit("file_content", collection).await?;
        
        // ... implementation
    }
}
```

---

## 📚 Usage Examples

### Example 1: Explore Then Read

```python
from vectorizer_mcp import VectorizerMCP

mcp = VectorizerMCP()

# 1. List available files
files = mcp.list_files_in_collection(
    collection="vectorizer-source",
    filter_by_type=["rs"],
    min_chunks=5
)

print(f"Found {len(files['files'])} Rust files")

# 2. Get summary of interesting file
summary = mcp.get_file_summary(
    collection="vectorizer-source",
    file_path="src/search/intelligent.rs",
    summary_type="both"
)

print(f"Summary: {summary['extractive_summary']}")
print(f"Key sections: {summary['structural_summary']['key_sections']}")

# 3. If summary looks relevant, get full content
if "mmr" in summary['extractive_summary'].lower():
    content = mcp.get_file_content(
        collection="vectorizer-source",
        file_path="src/search/intelligent.rs"
    )
    print(f"Full file loaded: {len(content['content'])} bytes")
```

### Example 2: Progressive Large File Reading

```python
# Read large file progressively
file_path = "docs/long_specification.md"
start_chunk = 0
limit = 5

while True:
    response = mcp.get_file_chunks_ordered(
        collection="vectorizer-docs",
        file_path=file_path,
        start_chunk=start_chunk,
        limit=limit
    )
    
    for chunk in response['chunks']:
        print(f"Chunk {chunk['index']}: {chunk['content'][:100]}...")
        # Process chunk
    
    if not response['has_more']:
        break
    
    start_chunk = response['next_start']
```

### Example 3: Configuration File Search

```python
# Find all configuration files with specific content
results = mcp.search_by_file_type(
    collection="vectorizer-source",
    query="embedding model configuration",
    file_types=["yaml", "toml", "json"],
    return_full_files=True,
    limit=3
)

for result in results['results']:
    print(f"Found in {result['file_path']} (score: {result['relevance_score']})")
    if result['full_content']:
        print(result['full_content'])
```

### Example 4: Codebase Navigation

```python
# Start from main file
outline = mcp.get_project_outline(
    collection="vectorizer-source",
    include_summaries=True
)

print("Project structure:")
print(outline['structure'])
print(f"\nKey files: {[f['path'] for f in outline['key_files']]}")

# Find related files
main_file = "src/main.rs"
related = mcp.get_related_files(
    collection="vectorizer-source",
    file_path=main_file,
    limit=5
)

print(f"\nFiles related to {main_file}:")
for rel in related['related_files']:
    print(f"  - {rel['path']} (similarity: {rel['similarity_score']:.2f})")
    print(f"    Reason: {rel['reason']}")
```

---

## 🎯 Success Metrics

### Adoption Metrics
- **Target**: 60% of file access through MCP instead of `read_file`
- **Measure**: Track ratio of `get_file_content` calls vs `read_file` calls

### Performance Metrics
- **Cache hit rate**: Target > 70% for file content
- **Latency**: 95th percentile < 200ms for all operations
- **Memory usage**: Stay within 150MB budget

### Quality Metrics
- **Summary relevance**: User satisfaction > 80% (survey)
- **Related file accuracy**: > 70% of suggestions useful
- **Outline accuracy**: Correct structure generation > 95%

---

## 🔮 Future Enhancements

### Phase 2 (3-6 months)
- [ ] **Incremental file updates**: Update only changed chunks
- [ ] **File diff support**: Compare file versions
- [ ] **Code symbol extraction**: Function/class indexing
- [ ] **Cross-file reference tracking**: Import/dependency analysis

### Phase 3 (6-12 months)
- [ ] **AI-powered summaries**: Use LLM for abstractive summaries
- [ ] **Semantic code search**: Search by functionality, not text
- [ ] **Auto-documentation generation**: From code to docs
- [ ] **Intelligent refactoring suggestions**: Based on related files

---

## 📝 Conclusion

This specification provides a comprehensive solution to the current chunk-only limitation of the MCP system. By implementing these 7 file-level tools, we enable LLMs to:

1. **Discover** available files and project structure
2. **Summarize** files efficiently without loading full content
3. **Navigate** semantically through related files
4. **Access** complete files when needed
5. **Search** type-specifically for configs and docs

**Expected Impact**:
- 60-80% reduction in `read_file` usage
- 3-5x faster file operations (cached)
- Better LLM context utilization
- Improved developer experience

**Next Steps**:
1. Review and approve specification
2. Assign development team
3. Begin Week 1 implementation
4. Iterate based on early feedback

---

**Document Status**: Draft for Review  
**Author**: AI Assistant  
**Reviewers**: [To be assigned]  
**Approval Date**: [Pending]

