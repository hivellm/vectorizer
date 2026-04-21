# File Watcher Collection Management Specification

## ADDED Requirements

### Requirement: Smart Collection Name Determination
The file watcher SHALL determine collection names intelligently without creating empty collections.

##### Scenario: File Change in Already Indexed Collection
Given a file is already indexed in a collection
When the file is modified
Then the system MUST update the existing collection
And the system MUST NOT create a new collection

##### Scenario: New File in Monitored Directory
Given a new file is created in a monitored directory
And the directory is part of an existing project
When the file watcher processes the file
Then the system MUST use the project's existing collection
And the system MUST NOT generate a collection name from path components

##### Scenario: File Without Known Collection
Given a file is not part of any existing collection
And the file does not match any known pattern
When the file watcher processes the file
Then the system MUST use the configured default_collection
And the system MUST NOT create a new collection based on path components

### Requirement: Collection Cleanup Functionality
The system SHALL provide tools to identify and remove empty collections.

##### Scenario: List Empty Collections
Given the database contains multiple collections
And some collections have zero vectors
When list_empty_collections is called
Then the system MUST return only collections with zero vectors
And the system MUST include collection metadata (name, created_at)

##### Scenario: Cleanup Empty Collections
Given the database contains empty collections
When cleanup_empty_collections is called
Then the system MUST delete all collections with zero vectors
And the system MUST return statistics (count deleted, bytes freed)
And the system MUST preserve all non-empty collections

##### Scenario: Cleanup Dry Run
Given the database contains empty collections
When cleanup_empty_collections is called with dry_run=true
Then the system MUST NOT delete any collections
And the system MUST return what would be deleted
And the system MUST include estimated bytes to be freed

### Requirement: Collection Statistics
The system SHALL provide detailed statistics for each collection.

##### Scenario: Get Collection Stats
Given a collection exists in the database
When get_collection_stats is called
Then the system MUST return vector count
And the system MUST return file count
And the system MUST return total size in bytes
And the system MUST return created_at timestamp
And the system MUST return last_modified timestamp

## MODIFIED Requirements

### Requirement: Collection Name Generation
**BEFORE**: The system generated collection names from file path components as fallback.

**AFTER**: The system SHALL use the following priority order:
1. Check FileIndex for existing collection mapping
2. Check workspace.yml collection_mapping configuration
3. Use patterns for known directories (docs/, src/, etc.)
4. Use configured default_collection from FileWatcherConfig
5. NEVER generate names from path components

##### Delta: Removed Path-Based Name Generation
```rust
// REMOVED: Lines 358-377 in src/file_watcher/operations.rs
else {
    // Fallback: try to extract project/workspace name
    if let Some(parent) = path.parent() {
        let components: Vec<_> = parent.components().collect();
        if components.len() >= 2 {
            // Use last two components as collection name
            let last_two: Vec<_> = components.iter().rev().take(2).collect();
            format!(
                "{}-{}",
                last_two[1].as_os_str().to_string_lossy(),
                last_two[0].as_os_str().to_string_lossy()
            )
        } else if let Some(last) = components.last() {
            last.as_os_str().to_string_lossy().to_string()
        } else {
            "default".to_string()
        }
    } else {
        "default".to_string()
    }
}
```

##### Delta: Added Lookup-Based Name Resolution
```rust
// ADDED: New logic in determine_collection_name()
pub fn determine_collection_name(&self, path: &std::path::Path) -> String {
    // 1. Check if file is already indexed
    if let Some(collection) = self.file_index.get_collection_for_file(path) {
        return collection;
    }
    
    // 2. Check workspace.yml collection_mapping
    if let Some(collection) = self.config.get_collection_for_path(path) {
        return collection;
    }
    
    // 3. Check known patterns (existing logic for docs/, src/, etc.)
    if let Some(collection) = self.check_known_patterns(path) {
        return collection;
    }
    
    // 4. Use configured default
    self.config.default_collection.clone()
}
```

## REST API Requirements

### Endpoint: DELETE /api/v1/collections/cleanup
**Purpose**: Remove all empty collections from the database.

**Request**:
```json
{
  "dry_run": false  // Optional, default: false
}
```

**Response Success (200)**:
```json
{
  "success": true,
  "deleted_count": 15,
  "bytes_freed": 1048576,
  "deleted_collections": [
    "vectorizer-src",
    "rulebook-tasks",
    "docs-specs"
  ],
  "duration_ms": 123
}
```

**Response Dry Run (200)**:
```json
{
  "success": true,
  "would_delete_count": 15,
  "estimated_bytes_freed": 1048576,
  "collections_to_delete": [
    "vectorizer-src",
    "rulebook-tasks",
    "docs-specs"
  ]
}
```

### Endpoint: GET /api/v1/collections/stats
**Purpose**: Get statistics for all or specific collections.

**Query Parameters**:
- `collection` (optional): Specific collection name
- `empty_only` (optional): Show only empty collections

**Response Success (200)**:
```json
{
  "success": true,
  "collections": [
    {
      "name": "vectorizer-source",
      "vector_count": 1523,
      "file_count": 87,
      "size_bytes": 5242880,
      "created_at": "2025-12-01T10:00:00Z",
      "last_modified": "2025-12-04T14:30:00Z",
      "is_empty": false
    }
  ]
}
```

## MCP Tools Requirements

### Tool: cleanup_empty_collections
**Purpose**: Remove empty collections via MCP.

**Parameters**:
```json
{
  "dry_run": {
    "type": "boolean",
    "description": "Preview what would be deleted without actually deleting",
    "default": false
  }
}
```

**Returns**:
```json
{
  "deleted_count": 15,
  "bytes_freed": 1048576,
  "deleted_collections": ["collection1", "collection2"]
}
```

### Tool: list_empty_collections
**Purpose**: List all collections with zero vectors.

**Parameters**: None

**Returns**:
```json
{
  "empty_collections": [
    {
      "name": "vectorizer-src",
      "size_bytes": 0,
      "created_at": "2025-12-04T12:00:00Z"
    }
  ],
  "count": 15
}
```

### Tool: get_collection_stats
**Purpose**: Get detailed statistics for collections.

**Parameters**:
```json
{
  "collection_name": {
    "type": "string",
    "description": "Specific collection name (optional)",
    "required": false
  }
}
```

**Returns**:
```json
{
  "collections": [
    {
      "name": "vectorizer-source",
      "vector_count": 1523,
      "file_count": 87,
      "size_bytes": 5242880,
      "is_empty": false
    }
  ]
}
```

## Configuration Requirements

### FileWatcherConfig Extension
The FileWatcherConfig SHALL include:

```rust
pub struct FileWatcherConfig {
    // ... existing fields ...
    
    /// Default collection name for files that don't match any pattern
    pub default_collection: String,
    
    /// Optional custom collection mapping
    pub collection_mapping: Option<HashMap<String, String>>,
    
    /// Run cleanup on startup
    pub startup_cleanup_empty: bool,
}
```

### workspace.yml Extension
The workspace.yml SHALL support collection mapping:

```yaml
file_watcher:
  default_collection: "workspace-default"
  startup_cleanup_empty: true
  collection_mapping:
    "/path/to/project/src": "project-source"
    "/path/to/project/docs": "project-docs"
    "/path/to/project/tests": "project-tests"
```

## Performance Requirements

### Requirement: Efficient Batch Deletion
The cleanup operation SHALL:
- Delete collections in batch (not one-by-one)
- Complete within 5 seconds for 100 empty collections
- Use minimal memory overhead (< 100MB for cleanup operation)

### Requirement: Fast Empty Collection Check
The is_collection_empty() check SHALL:
- Complete in < 1ms per collection
- Not load all vectors into memory
- Use efficient count query

## Error Handling

##### Scenario: Cleanup Fails Partially
Given cleanup is in progress
And some collections fail to delete
When an error occurs
Then the system MUST continue with remaining collections
And the system MUST return partial success with error details
And the system MUST NOT corrupt the database

##### Scenario: Collection Stats for Non-Existent Collection
Given a collection name that doesn't exist
When get_collection_stats is called
Then the system MUST return CollectionNotFound error
And the system MUST NOT create an empty collection

## Testing Requirements

### Unit Test Coverage
- determine_collection_name() MUST have 100% branch coverage
- cleanup_empty_collections() MUST test dry_run mode
- is_collection_empty() MUST test edge cases (0, 1, many vectors)

### Integration Test Coverage
- File watcher MUST NOT create empty collections during normal operation
- Cleanup MUST preserve all non-empty collections
- REST API MUST handle concurrent cleanup requests safely

### Performance Test Coverage
- Cleanup of 100 empty collections MUST complete in < 5 seconds
- File watcher collection lookup MUST complete in < 10ms

