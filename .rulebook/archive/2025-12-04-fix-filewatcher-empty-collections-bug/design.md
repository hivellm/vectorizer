# File Watcher Bug Fix - Technical Design

## Problem Analysis

### Root Cause
The `determine_collection_name()` function in `src/file_watcher/operations.rs` (lines 358-377) has a fallback mechanism that generates collection names from file path components when no known pattern matches. This causes:

1. **Unintended Collection Creation**: Files like `F:\Node\hivellm\vectorizer\src\main.rs` generate collections named `vectorizer-src`
2. **Empty Collections**: These auto-generated collections often remain empty because the file loader creates its own collection
3. **Duplicate Logic**: File watcher and file loader use different collection naming strategies

### Example of Buggy Behavior
```
File Changed: F:\Node\hivellm\vectorizer\rulebook\tasks\hub-integration\tasks.md
↓
determine_collection_name() fallback logic
↓
Generates: "tasks-hub-integration" (from last 2 path components)
↓
Creates empty collection (file loader may use different name)
```

## Solution Architecture

### Phase 1: Fix Collection Name Determination

#### New Priority Order
```rust
pub fn determine_collection_name(&self, path: &std::path::Path) -> String {
    // Priority 1: Check FileIndex (file already indexed)
    if let Some(collection) = self.file_index.get_collection_for_file(path) {
        return collection;
    }
    
    // Priority 2: Check workspace.yml collection_mapping
    if let Some(collection) = self.config.get_collection_for_path(path) {
        return collection;
    }
    
    // Priority 3: Check known patterns (existing logic)
    if let Some(collection) = self.check_known_patterns(path) {
        return collection;
    }
    
    // Priority 4: Use configured default (NO AUTOMATIC GENERATION)
    self.config.default_collection.clone()
}
```

#### FileIndex Extension
Add method to track file-to-collection mappings:
```rust
impl FileIndex {
    pub fn get_collection_for_file(&self, file_path: &PathBuf) -> Option<String> {
        self.file_to_collections
            .get(file_path)
            .and_then(|mappings| mappings.first())
            .map(|m| m.collection_name.clone())
    }
}
```

#### WorkspaceConfig Extension
Add collection mapping support:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FileWatcherConfig {
    // ... existing fields ...
    
    #[serde(default = "default_collection_name")]
    pub default_collection: String,
    
    #[serde(default)]
    pub collection_mapping: HashMap<String, String>,
    
    #[serde(default)]
    pub startup_cleanup_empty: bool,
}

fn default_collection_name() -> String {
    "workspace-default".to_string()
}
```

### Phase 2: Add Cleanup Functionality

#### VectorStore Methods
```rust
impl VectorStore {
    /// Get statistics for a collection
    pub fn get_collection_stats(&self, name: &str) -> Result<CollectionStats> {
        let collection = self.get_collection(name)?;
        
        Ok(CollectionStats {
            name: name.to_string(),
            vector_count: collection.get_vector_count(),
            file_count: collection.get_unique_file_count(),
            size_bytes: collection.estimate_size_bytes(),
            created_at: collection.created_at,
            last_modified: collection.last_modified,
            is_empty: collection.get_vector_count() == 0,
        })
    }
    
    /// Check if collection is empty
    pub fn is_collection_empty(&self, name: &str) -> Result<bool> {
        let collection = self.get_collection(name)?;
        Ok(collection.get_vector_count() == 0)
    }
    
    /// List all empty collections
    pub fn list_empty_collections(&self) -> Vec<String> {
        self.list_collections()
            .into_iter()
            .filter(|name| {
                self.is_collection_empty(name).unwrap_or(false)
            })
            .collect()
    }
    
    /// Cleanup empty collections
    pub fn cleanup_empty_collections(
        &self,
        dry_run: bool
    ) -> Result<CleanupStats> {
        let empty_collections = self.list_empty_collections();
        let mut deleted_count = 0;
        let mut bytes_freed = 0;
        let mut deleted_collections = Vec::new();
        let mut errors = Vec::new();
        
        for collection_name in &empty_collections {
            // Get size before deletion
            if let Ok(stats) = self.get_collection_stats(collection_name) {
                bytes_freed += stats.size_bytes;
                
                if !dry_run {
                    match self.delete_collection(collection_name) {
                        Ok(_) => {
                            deleted_count += 1;
                            deleted_collections.push(collection_name.clone());
                        }
                        Err(e) => {
                            errors.push((collection_name.clone(), e.to_string()));
                        }
                    }
                } else {
                    deleted_count += 1;
                    deleted_collections.push(collection_name.clone());
                }
            }
        }
        
        Ok(CleanupStats {
            deleted_count,
            bytes_freed,
            deleted_collections,
            errors,
            dry_run,
        })
    }
}
```

#### New Data Structures
```rust
#[derive(Debug, Clone, Serialize)]
pub struct CollectionStats {
    pub name: String,
    pub vector_count: usize,
    pub file_count: usize,
    pub size_bytes: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
    pub is_empty: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupStats {
    pub deleted_count: usize,
    pub bytes_freed: usize,
    pub deleted_collections: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub dry_run: bool,
}
```

### Phase 3: REST API Integration

#### Handler Implementation
```rust
// DELETE /api/v1/collections/cleanup
pub async fn cleanup_empty_collections_handler(
    State(state): State<AppState>,
    Query(params): Query<CleanupParams>,
) -> Result<Json<CleanupResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = std::time::Instant::now();
    
    match state.vector_store.cleanup_empty_collections(params.dry_run) {
        Ok(stats) => {
            let response = CleanupResponse {
                success: true,
                deleted_count: stats.deleted_count,
                bytes_freed: stats.bytes_freed,
                deleted_collections: stats.deleted_collections,
                errors: stats.errors,
                duration_ms: start.elapsed().as_millis() as u64,
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Cleanup failed: {}", e),
                }),
            ))
        }
    }
}

// GET /api/v1/collections/stats
pub async fn get_collections_stats_handler(
    State(state): State<AppState>,
    Query(params): Query<StatsParams>,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let collections = if let Some(name) = params.collection {
        vec![name]
    } else {
        state.vector_store.list_collections()
    };
    
    let mut stats = Vec::new();
    for collection_name in collections {
        if let Ok(collection_stats) = state.vector_store.get_collection_stats(&collection_name) {
            if params.empty_only && !collection_stats.is_empty {
                continue;
            }
            stats.push(collection_stats);
        }
    }
    
    Ok(Json(StatsResponse {
        success: true,
        collections: stats,
    }))
}
```

### Phase 4: MCP Tool Integration

#### Tool Definitions
```rust
// In src/mcp/tools.rs

pub fn cleanup_empty_collections_tool() -> Tool {
    Tool {
        name: "cleanup_empty_collections".to_string(),
        description: "Remove all empty collections from the database".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview what would be deleted without actually deleting",
                    "default": false
                }
            }
        }),
    }
}

pub fn list_empty_collections_tool() -> Tool {
    Tool {
        name: "list_empty_collections".to_string(),
        description: "List all collections with zero vectors".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    }
}

pub fn get_collection_stats_tool() -> Tool {
    Tool {
        name: "get_collection_stats".to_string(),
        description: "Get detailed statistics for collections".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "collection_name": {
                    "type": "string",
                    "description": "Specific collection name (optional)"
                }
            }
        }),
    }
}
```

## Performance Considerations

### Efficient Empty Collection Check
```rust
impl Collection {
    pub fn get_vector_count(&self) -> usize {
        // Fast: O(1) operation, just read the length
        self.vectors.read().len()
    }
    
    pub fn estimate_size_bytes(&self) -> usize {
        // Estimate based on vector count and dimension
        let vectors = self.vectors.read();
        let vector_size = self.dimension * std::mem::size_of::<f32>();
        vectors.len() * vector_size
    }
}
```

### Batch Delete Optimization
```rust
impl VectorStore {
    pub fn delete_collections_batch(&self, names: &[String]) -> Result<Vec<String>> {
        let mut deleted = Vec::new();
        
        // Use write lock once for all deletions
        let mut collections = self.collections.write();
        
        for name in names {
            if collections.remove(name).is_some() {
                deleted.push(name.clone());
            }
        }
        
        Ok(deleted)
    }
}
```

## Migration Strategy

### Step 1: Add New Functionality (Non-Breaking)
- Add `default_collection` field to config with sensible default
- Add cleanup methods to VectorStore
- Add REST endpoints and MCP tools
- Deploy and test in production

### Step 2: Fix File Watcher Logic (Breaking for New Files)
- Modify `determine_collection_name()` priority order
- Remove path-based generation fallback
- Monitor logs for any issues

### Step 3: Clean Up Existing Empty Collections
- Run cleanup tool on production database
- Verify no data loss
- Monitor search performance improvement

### Step 4: Documentation and Training
- Update user documentation
- Add cleanup guide
- Update API reference

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_determine_collection_name_with_existing_file() {
    // Test Priority 1: FileIndex lookup
}

#[test]
fn test_determine_collection_name_with_workspace_mapping() {
    // Test Priority 2: workspace.yml mapping
}

#[test]
fn test_cleanup_empty_collections_dry_run() {
    // Test dry run doesn't delete
}

#[test]
fn test_cleanup_preserves_non_empty_collections() {
    // Test data safety
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_file_watcher_updates_existing_collection() {
    // Create collection, index file, modify file, verify no new collection
}

#[tokio::test]
async fn test_cleanup_via_rest_api() {
    // Test REST endpoint
}
```

## Monitoring and Metrics

### New Metrics to Track
- `vectorizer_empty_collections_total` - Number of empty collections
- `vectorizer_cleanup_operations_total` - Number of cleanup operations
- `vectorizer_cleanup_deleted_total` - Number of collections deleted
- `vectorizer_cleanup_bytes_freed_total` - Bytes freed by cleanup

### Logging
```rust
tracing::info!(
    collection_name = %collection_name,
    vector_count = 0,
    "Empty collection detected"
);

tracing::info!(
    deleted_count = stats.deleted_count,
    bytes_freed = stats.bytes_freed,
    duration_ms = duration.as_millis(),
    "Cleanup completed successfully"
);
```

## Rollback Plan

If issues occur:
1. Restore from database backup (before cleanup)
2. Revert file watcher changes
3. Keep cleanup functionality but don't auto-run
4. Investigate and fix issues
5. Gradually re-enable features

