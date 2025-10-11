# File-Level Operations - MCP Tools

**Version**: 1.0  
**Status**: ✅ Implemented (Priority 1 Tools)  
**Last Updated**: 2025-10-07

---

## Overview

File-level MCP tools bridge the gap between chunk-based search and complete file access, providing LLMs with efficient full-file context retrieval without falling back to traditional `read_file` operations.

### Available Tools

**✅ Implemented (Priority 1)**:
1. `get_file_content` - Retrieve complete indexed files
2. `list_files_in_collection` - Discover available files
3. `get_file_summary` - Get file-level summaries

**🔄 Registered (Priority 2-3)**:
4. `get_file_chunks_ordered` - Progressive file reading
5. `get_project_outline` - Project structure overview
6. `get_related_files` - Semantic file navigation
7. `search_by_file_type` - Type-aware search

---

## Tool Specifications

### 1. get_file_content

**Purpose**: Retrieve complete file content from indexed collections

**Parameters**:
```json
{
  "collection": "string",      // Required
  "file_path": "string",       // Required
  "max_size_kb": 500          // Optional, default: 500
}
```

**Implementation**:
- Queries all chunks by `file_path` metadata
- Sorts by `chunk_index`
- Reconstructs complete file
- LRU cache (10 minutes TTL)
- Automatic language detection

**Performance**: ~100-300ms (cache miss), ~5ms (cache hit)

### 2. list_files_in_collection

**Purpose**: List all indexed files with metadata

**Parameters**:
```json
{
  "collection": "string",      // Required
  "filter_by_type": ["rs", "md"], // Optional
  "min_chunks": 3,             // Optional
  "max_results": 100,          // Optional, default: 100
  "sort_by": "chunks"          // Optional: name, size, chunks, recent
}
```

**Response**:
```json
{
  "collection": "documents",
  "files": [
    {
      "path": "src/main.rs",
      "file_type": "rs",
      "chunk_count": 15,
      "size_estimate_kb": 42,
      "last_indexed": "2025-10-07T..."
    }
  ],
  "total_files": 150,
  "total_chunks": 2340
}
```

**Performance**: ~500ms-2s (cache miss), ~20ms (cache hit)

### 3. get_file_summary

**Purpose**: Get extractive or structural summary of indexed files

**Parameters**:
```json
{
  "collection": "string",           // Required
  "file_path": "string",            // Required
  "summary_type": "both",           // extractive | structural | both
  "max_sentences": 5                // Optional, default: 5
}
```

**Summary Types**:

**Extractive**:
- Extracts first N significant sentences (>20 chars)
- Skips headers and formatting
- Preserves original text

**Structural**:
- Extracts markdown headers (`#`, `##`, etc.)
- Identifies key sections
- Finds keywords: important, note, warning, critical, TODO, FIXME
- Limits to 10 sections and 10 key points

**Performance**: ~200-500ms (cache miss), ~10ms (cache hit)

### 4. get_file_chunks_ordered

**Purpose**: Retrieve chunks in original file order for progressive reading

**Parameters**:
```json
{
  "collection": "string",      // Required
  "file_path": "string",       // Required
  "start_chunk": 0,            // Optional, default: 0
  "limit": 10,                 // Optional, default: 10
  "include_context": false     // Optional, default: false
}
```

### 5. get_project_outline

**Purpose**: Generate hierarchical project structure overview

**Parameters**:
```json
{
  "collection": "string",            // Required
  "max_depth": 5,                    // Optional, default: 5
  "include_summaries": false,        // Optional, default: false
  "highlight_key_files": true        // Optional, default: true
}
```

### 6. get_related_files

**Purpose**: Find semantically related files using vector similarity

**Parameters**:
```json
{
  "collection": "string",            // Required
  "file_path": "string",             // Required
  "limit": 5,                        // Optional, default: 5
  "similarity_threshold": 0.6,       // Optional, default: 0.6
  "include_reason": true             // Optional, default: true
}
```

### 7. search_by_file_type

**Purpose**: Semantic search filtered by file type

**Parameters**:
```json
{
  "collection": "string",            // Required
  "query": "string",                 // Required
  "file_types": ["yaml", "toml"],    // Required
  "limit": 10,                       // Optional, default: 10
  "return_full_files": false         // Optional, default: false
}
```

---

## Architecture

### Data Flow

```
1. MCP Tool Call
   ↓
2. FileOperations Handler
   ↓
3. VectorStore Query (filter by file_path)
   ↓
4. Sort by chunk_index
   ↓
5. Reconstruct / Summarize / List
   ↓
6. LRU Cache (optional)
   ↓
7. Return JSON Response
```

### Metadata Structure

```json
{
  "vector_id": "uuid",
  "payload": {
    "content": "chunk content",
    "metadata": {
      "file_path": "src/main.rs",     // Used for filtering
      "chunk_index": 0,                // Used for sorting
      "chunk_size": 2048,
      "file_extension": "rs",
      "indexed_at": "2025-10-07T..."
    }
  }
}
```

---

## Use Cases

**Configuration Files**:
- Read complete YAML/TOML files without fragmentation
- Get summaries of complex configurations

**Source Code**:
- Analyze entire source files for refactoring
- Navigate related files semantically
- Generate documentation from full context

**Documentation**:
- Extract key points from READMEs
- Build project outlines automatically
- Find related documentation

**Project Exploration**:
- Discover project structure
- Identify important files
- Navigate codebase semantically

---

## Performance Metrics

| Operation | Latency (Miss) | Latency (Hit) |
|-----------|---------------|---------------|
| get_file_content | 100-300ms | ~5ms |
| list_files_in_collection | 500ms-2s | ~20ms |
| get_file_summary | 200-500ms | ~10ms |

**Note**: Latency varies with:
- Collection size
- File chunk count
- File size

---

## Testing

```bash
# List files
curl -X POST http://localhost:8080/mcp/message \
  -H "Content-Type: application/json" \
  -d '{
    "tool": "list_files_in_collection",
    "arguments": {
      "collection": "vectorizer-source"
    }
  }'

# Get file content
curl -X POST http://localhost:8080/mcp/message \
  -H "Content-Type: application/json" \
  -d '{
    "tool": "get_file_content",
    "arguments": {
      "collection": "vectorizer-source",
      "file_path": "src/main.rs"
    }
  }'

# Get summary
curl -X POST http://localhost:8080/mcp/message \
  -H "Content-Type: application/json" \
  -d '{
    "tool": "get_file_summary",
    "arguments": {
      "collection": "vectorizer-docs",
      "file_path": "README.md",
      "summary_type": "both"
    }
  }'
```

---

**Status**: ✅ Priority 1 tools production-ready  
**Maintained by**: HiveLLM Team

