---
title: File Operations API
module: api
id: file-operations-api
order: 6
description: File-level operations for indexed content
tags: [api, files, content, indexing]
---

# File Operations API

The File Operations API provides file-level abstractions over chunk-based vector storage, enabling you to work with complete files, summaries, and project structures.

## Overview

File operations enable:

- Retrieving complete file content from indexed collections
- Listing files with filtering and sorting
- Getting file summaries
- Accessing file chunks in order
- Project structure exploration
- Finding related files
- Type-aware file search

## File Content

### Get File Content

Retrieve complete file content from an indexed collection.

**Endpoint:** `POST /file/content`

**Request Body:**

```json
{
  "collection": "codebase",
  "file_path": "src/main.rs",
  "max_size_kb": 500
}
```

**Parameters:**

| Parameter     | Type   | Required | Description                                       |
| ------------- | ------ | -------- | ------------------------------------------------- |
| `collection`  | string | Yes      | Collection name                                   |
| `file_path`   | string | Yes      | File path within collection                       |
| `max_size_kb` | number | No       | Maximum file size in KB (default: 500, max: 5000) |

**Response:**

```json
{
  "file_path": "src/main.rs",
  "content": "use std::io;\n\nfn main() {\n    ...\n}",
  "metadata": {
    "size_kb": 45,
    "chunk_count": 3,
    "language": "rust",
    "file_type": "rs",
    "indexed_at": "2024-01-15T10:30:00Z"
  }
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/file/content \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "codebase",
    "file_path": "src/main.rs"
  }'
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

file_content = await client.get_file_content(
    collection="codebase",
    file_path="src/main.rs",
    max_size_kb=500
)

print(f"File: {file_content['file_path']}")
print(f"Size: {file_content['metadata']['size_kb']}KB")
print(f"Content:\n{file_content['content']}")
```

## List Files

### List Files in Collection

List all indexed files in a collection with filtering and sorting options.

**Endpoint:** `POST /file/list`

**Request Body:**

```json
{
  "collection": "codebase",
  "filter_by_type": ["rs", "md"],
  "min_chunks": 3,
  "max_results": 100,
  "sort_by": "chunks"
}
```

**Parameters:**

| Parameter        | Type          | Required | Description                                                   |
| ---------------- | ------------- | -------- | ------------------------------------------------------------- |
| `collection`     | string        | Yes      | Collection name                                               |
| `filter_by_type` | array[string] | No       | Filter by file extensions                                     |
| `min_chunks`     | number        | No       | Minimum chunk count                                           |
| `max_results`    | number        | No       | Maximum results (default: 100)                                |
| `sort_by`        | string        | No       | Sort by: `name`, `size`, `chunks`, `recent` (default: `name`) |

**Response:**

```json
{
  "collection": "codebase",
  "files": [
    {
      "path": "src/main.rs",
      "chunk_count": 5,
      "size_estimate_kb": 45,
      "file_type": "rs",
      "language": "rust",
      "indexed_at": "2024-01-15T10:30:00Z"
    },
    {
      "path": "README.md",
      "chunk_count": 3,
      "size_estimate_kb": 12,
      "file_type": "md",
      "language": "markdown",
      "indexed_at": "2024-01-15T10:25:00Z"
    }
  ],
  "total": 2
}
```

**Example:**

```python
files = await client.list_files_in_collection(
    collection="codebase",
    filter_by_type=["rs", "ts"],
    min_chunks=3,
    sort_by="size"
)

for file in files["files"]:
    print(f"{file['path']}: {file['chunk_count']} chunks")
```

## File Summaries

### Get File Summary

Get a summary of a file's content.

**Endpoint:** `POST /file/summary`

**Request Body:**

```json
{
  "collection": "codebase",
  "file_path": "src/main.rs",
  "summary_type": "extractive",
  "max_length": 500
}
```

**Parameters:**

| Parameter      | Type   | Required | Description                                          |
| -------------- | ------ | -------- | ---------------------------------------------------- |
| `collection`   | string | Yes      | Collection name                                      |
| `file_path`    | string | Yes      | File path                                            |
| `summary_type` | string | No       | `extractive` or `structural` (default: `extractive`) |
| `max_length`   | number | No       | Maximum summary length (default: 500)                |

**Response:**

```json
{
  "file_path": "src/main.rs",
  "summary": "Main entry point for the application. Handles initialization and routing.",
  "summary_type": "extractive",
  "key_points": ["Initializes server", "Sets up routes", "Handles errors"],
  "length": 245
}
```

**Example:**

```python
summary = await client.get_file_summary(
    collection="codebase",
    file_path="src/main.rs",
    summary_type="extractive",
    max_length=500
)

print(f"Summary: {summary['summary']}")
print(f"Key points: {summary['key_points']}")
```

## File Chunks

### Get File Chunks Ordered

Retrieve file chunks in their original order.

**Endpoint:** `POST /file/chunks`

**Request Body:**

```json
{
  "collection": "codebase",
  "file_path": "src/main.rs",
  "limit": 10,
  "offset": 0
}
```

**Parameters:**

| Parameter    | Type   | Required | Description                        |
| ------------ | ------ | -------- | ---------------------------------- |
| `collection` | string | Yes      | Collection name                    |
| `file_path`  | string | Yes      | File path                          |
| `limit`      | number | No       | Maximum chunks (default: 10)       |
| `offset`     | number | No       | Offset for pagination (default: 0) |

**Response:**

```json
{
  "file_path": "src/main.rs",
  "chunks": [
    {
      "id": "chunk_001",
      "chunk_index": 0,
      "content": "use std::io;\n\nfn main() {",
      "start_line": 1,
      "end_line": 3
    },
    {
      "id": "chunk_002",
      "chunk_index": 1,
      "content": "    println!(\"Hello\");",
      "start_line": 4,
      "end_line": 4
    }
  ],
  "total_chunks": 5
}
```

**Example:**

```python
chunks = await client.get_file_chunks_ordered(
    collection="codebase",
    file_path="src/main.rs",
    limit=10,
    offset=0
)

for chunk in chunks["chunks"]:
    print(f"Chunk {chunk['chunk_index']}: {chunk['content'][:50]}...")
```

## Project Structure

### Get Project Outline

Get an overview of project structure from indexed files.

**Endpoint:** `POST /file/outline`

**Request Body:**

```json
{
  "collection": "codebase",
  "max_depth": 3,
  "include_files": true
}
```

**Parameters:**

| Parameter       | Type    | Required | Description                              |
| --------------- | ------- | -------- | ---------------------------------------- |
| `collection`    | string  | Yes      | Collection name                          |
| `max_depth`     | number  | No       | Maximum directory depth (default: 3)     |
| `include_files` | boolean | No       | Include files in outline (default: true) |

**Response:**

```json
{
  "collection": "codebase",
  "outline": {
    "src": {
      "type": "directory",
      "files": ["main.rs", "lib.rs"],
      "subdirectories": {
        "api": {
          "type": "directory",
          "files": ["handlers.rs"]
        }
      }
    }
  },
  "total_files": 3,
  "total_directories": 2
}
```

**Example:**

```python
outline = await client.get_project_outline(
    collection="codebase",
    max_depth=3
)

print(f"Project structure: {outline['outline']}")
```

## Related Files

### Get Related Files

Find files semantically related to a given file.

**Endpoint:** `POST /file/related`

**Request Body:**

```json
{
  "collection": "codebase",
  "file_path": "src/main.rs",
  "max_results": 10,
  "similarity_threshold": 0.7
}
```

**Parameters:**

| Parameter              | Type   | Required | Description                         |
| ---------------------- | ------ | -------- | ----------------------------------- |
| `collection`           | string | Yes      | Collection name                     |
| `file_path`            | string | Yes      | Reference file path                 |
| `max_results`          | number | No       | Maximum related files (default: 10) |
| `similarity_threshold` | number | No       | Minimum similarity (default: 0.7)   |

**Response:**

```json
{
  "file_path": "src/main.rs",
  "related_files": [
    {
      "path": "src/lib.rs",
      "similarity": 0.85,
      "reason": "Shared imports and types"
    },
    {
      "path": "src/api/handlers.rs",
      "similarity": 0.72,
      "reason": "Related API functionality"
    }
  ]
}
```

**Example:**

```python
related = await client.get_related_files(
    collection="codebase",
    file_path="src/main.rs",
    max_results=10
)

for file in related["related_files"]:
    print(f"{file['path']}: {file['similarity']:.2f} similarity")
```

## Type-Aware Search

### Search by File Type

Search for files of specific types.

**Endpoint:** `POST /file/search_by_type`

**Request Body:**

```json
{
  "collection": "codebase",
  "file_types": ["rs", "toml"],
  "query": "configuration",
  "limit": 20
}
```

**Parameters:**

| Parameter    | Type          | Required | Description                   |
| ------------ | ------------- | -------- | ----------------------------- |
| `collection` | string        | Yes      | Collection name               |
| `file_types` | array[string] | Yes      | File extensions to search     |
| `query`      | string        | Yes      | Search query                  |
| `limit`      | number        | No       | Maximum results (default: 20) |

**Response:**

```json
{
  "results": [
    {
      "file_path": "Cargo.toml",
      "score": 0.92,
      "matches": [
        {
          "chunk_id": "chunk_123",
          "content": "...",
          "score": 0.92
        }
      ]
    }
  ],
  "total": 1
}
```

**Example:**

```python
results = await client.search_by_file_type(
    collection="codebase",
    file_types=["rs", "toml"],
    query="configuration",
    limit=20
)

for result in results["results"]:
    print(f"{result['file_path']}: {result['score']:.2f}")
```

## Use Cases

### Code Review

Review complete files during code analysis:

```python
# Get file content
file = await client.get_file_content(
    collection="codebase",
    file_path="src/api/handlers.rs"
)

# Get related files
related = await client.get_related_files(
    collection="codebase",
    file_path="src/api/handlers.rs"
)

# Review file and related context
print(f"Reviewing: {file['file_path']}")
print(f"Related files: {len(related['related_files'])}")
```

### Documentation Generation

Generate documentation from code:

```python
# List all Rust files
rust_files = await client.list_files_in_collection(
    collection="codebase",
    filter_by_type=["rs"],
    sort_by="name"
)

# Get summaries for each file
for file in rust_files["files"]:
    summary = await client.get_file_summary(
        collection="codebase",
        file_path=file["path"],
        summary_type="extractive"
    )
    print(f"## {file['path']}\n{summary['summary']}\n")
```

### Project Exploration

Explore project structure:

```python
# Get project outline
outline = await client.get_project_outline(
    collection="codebase",
    max_depth=3
)

# List files by type
config_files = await client.list_files_in_collection(
    collection="codebase",
    filter_by_type=["toml", "yaml", "json"],
    sort_by="name"
)
```

## Best Practices

1. **Use file operations for complete context**: When you need full file content, not just chunks
2. **Filter by file type**: Use `filter_by_type` to focus on specific file types
3. **Set appropriate size limits**: Use `max_size_kb` to prevent loading very large files
4. **Cache file content**: File content is cached for 10 minutes, reuse when possible
5. **Use summaries for overview**: Get file summaries before loading full content
6. **Explore related files**: Use `get_related_files` to understand file relationships

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection management
- [Search Guide](../search/SEARCH.md) - Search operations
- [Discovery API](./DISCOVERY.md) - Content discovery
