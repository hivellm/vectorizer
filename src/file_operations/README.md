# File Operations Module

Implementation of file-level MCP tools for the Vectorizer.

## Overview

This module provides file-centric abstractions over the chunk-based vector storage system, enabling LLMs to work with complete files, summaries, and project structures instead of fragmented chunks.

## Structure

```
src/file_operations/
├── mod.rs          - Module exports and public API
├── types.rs        - Type definitions and structures
├── errors.rs       - Error types and handling
├── cache.rs        - LRU caching system
├── operations.rs   - Core implementation
└── tests.rs        - Integration tests
```

## Implemented Features (Priority 1)

### ✅ `get_file_content`
Retrieve complete indexed files with metadata.

```rust
let content = file_ops.get_file_content(
    "vectorizer-source",
    "src/main.rs",
    500 // max KB
).await?;

println!("File: {}", content.file_path);
println!("Size: {}KB", content.metadata.size_kb);
println!("Content: {}", content.content);
```

**Features:**
- Path validation (prevents directory traversal)
- Size limits (default 1MB, max 5MB)
- LRU caching with 10-minute TTL
- Automatic file type detection
- Language detection for code files

### ✅ `list_files_in_collection`
List all files in a collection with filtering and sorting.

```rust
let filter = FileListFilter {
    filter_by_type: Some(vec!["rs".to_string()]),
    min_chunks: Some(3),
    max_results: Some(10),
    sort_by: SortBy::Size,
};

let list = file_ops.list_files_in_collection(
    "vectorizer-source",
    filter
).await?;

for file in list.files {
    println!("{}: {} chunks, {}KB", 
        file.path, file.chunk_count, file.size_estimate_kb);
}
```

**Features:**
- Filter by file type
- Filter by minimum chunks
- Sort by name, size, chunks, or date
- Pagination support
- 5-minute cache TTL

### ✅ `get_file_summary`
Generate extractive or structural summaries of files.

```rust
let summary = file_ops.get_file_summary(
    "vectorizer-docs",
    "README.md",
    SummaryType::Both,
    5 // max sentences
).await?;

if let Some(text) = summary.extractive_summary {
    println!("Summary: {}", text);
}

if let Some(structure) = summary.structural_summary {
    println!("Outline:\n{}", structure.outline);
    println!("Key points: {:?}", structure.key_points);
}
```

**Features:**
- **Extractive**: Key sentence extraction
- **Structural**: Outline and key sections
- 30-minute cache TTL
- Automatic file type detection

## Cache System

The module uses an LRU cache with three layers:

| Cache Type | Capacity | TTL | Purpose |
|------------|----------|-----|---------|
| File Content | 100 files | 10 min | Complete files |
| Summaries | 500 items | 30 min | File summaries |
| File Lists | 50 collections | 5 min | Directory listings |

**Memory Budget:** ~120MB

### Cache Operations

```rust
// Get cache statistics
let stats = file_ops.cache_stats().await;
println!("Files cached: {}", stats.file_content_entries);
println!("Summaries cached: {}", stats.summary_entries);

// Clear cache for specific collection
file_ops.clear_cache("vectorizer-source").await;
```

## Security

### Path Validation
- ❌ No directory traversal (`../`)
- ❌ No absolute paths (`/`, `\`)
- ❌ No empty paths
- ✅ Only relative paths within collections

### Size Limits
- Default limit: 1MB (1000 KB)
- Absolute maximum: 5MB (5000 KB)
- Configurable per request
- Prevents OOM attacks

## Error Handling

```rust
use file_operations::FileOperationError;

match file_ops.get_file_content(...).await {
    Ok(content) => { /* use content */ },
    Err(FileOperationError::FileNotFound { file_path, collection }) => {
        eprintln!("File {} not found in {}", file_path, collection);
    },
    Err(FileOperationError::FileTooLarge { size_kb, max_size_kb }) => {
        eprintln!("File too large: {}KB > {}KB", size_kb, max_size_kb);
    },
    Err(FileOperationError::InvalidPath { path, reason }) => {
        eprintln!("Invalid path '{}': {}", path, reason);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

### Unit Tests
```bash
cargo test --package vectorizer --lib file_operations
```

### Integration Tests
```bash
cargo test --package vectorizer --lib file_operations::tests::integration_tests
```

### Test Coverage
- ✅ Path validation
- ✅ Size limit validation
- ✅ File type detection
- ✅ Language detection
- ✅ Filter application
- ✅ Cache behavior
- ✅ Error handling
- ✅ Full workflow

## Current Status

### ✅ Completed (Priority 1)
- [x] Module structure
- [x] Error types
- [x] Type definitions
- [x] LRU cache system
- [x] `get_file_content` (with mock)
- [x] `list_files_in_collection` (with mock)
- [x] `get_file_summary` (with mock)
- [x] Unit tests
- [x] Integration tests

### 🚧 In Progress
- [ ] Vector store integration
- [ ] MCP endpoint integration

### 📋 TODO (Priority 2 & 3)
- [ ] `get_file_chunks_ordered`
- [ ] `get_project_outline`
- [ ] `get_related_files`
- [ ] `search_by_file_type`
- [ ] Real extractive summarization
- [ ] Structural summary for code files

## Integration Guide

### Step 1: Add to Vector Store

```rust
// In src/db/vector_store.rs or similar
use crate::file_operations::FileOperations;

pub struct VectorStore {
    // ... existing fields
    pub file_ops: FileOperations,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            // ... existing initialization
            file_ops: FileOperations::new(),
        }
    }
}
```

### Step 2: Add MCP Endpoints

Create MCP tool definitions (to be added to MCP server):

```json
{
  "tools": [
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
            "description": "Maximum file size in KB",
            "default": 500
          }
        },
        "required": ["collection", "file_path"]
      }
    }
  ]
}
```

## Performance Targets

| Operation | Target | Cache Hit | Cache Miss |
|-----------|--------|-----------|------------|
| `get_file_content` | <50ms | 5ms | 100ms |
| `get_file_summary` | <100ms | 10ms | 500ms |
| `list_files` | <200ms | 20ms | 1s |

## Monitoring

Add metrics to track usage:

```rust
use prometheus::{register_histogram_vec, register_counter_vec};

lazy_static! {
    static ref FILE_OPERATION_DURATION: HistogramVec = 
        register_histogram_vec!(
            "file_operation_duration_seconds",
            "Duration of file operations",
            &["operation", "collection"]
        ).unwrap();
    
    static ref FILE_CACHE_HITS: CounterVec = 
        register_counter_vec!(
            "file_cache_hits_total",
            "Number of cache hits",
            &["cache_type"]
        ).unwrap();
}
```

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
lru = "0.12"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["sync", "rt-multi-thread"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
```

## License

Same as parent project.

## Contributors

See main project CONTRIBUTORS.md
