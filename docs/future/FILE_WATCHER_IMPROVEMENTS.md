# File Watcher Improvements Specification

**Status**: Specification  
**Priority**: 🟡 **P1 - HIGH** ⬇️  
**Complexity**: Medium  
**Created**: October 1, 2025  
**Updated**: October 1, 2025 - **PRIORITY DOWNGRADED BASED ON BENCHMARK ANALYSIS**

## 🎯 **WHY P1 PRIORITY - BENCHMARK INSIGHTS**

**Priority downgraded** from P0 to P1 based on benchmark analysis showing:
1. **System works well**: Current file watcher handles modifications effectively
2. **Performance is excellent**: Benchmarks show < 1ms search latency consistently
3. **No critical issues**: File synchronization is reliable and stable
4. **Focus on higher ROI**: Quantization (P0) delivers immediate 4x memory reduction
5. **Optimizations can wait**: System functions properly, improvements are nice-to-have

## Current Limitations

The file watcher currently:
- ✅ Monitors file modifications
- ✅ Detects changes via hash comparison
- ✅ Triggers re-indexing on changes
- ❌ **Does NOT detect new files** added to workspace
- ❌ **Does NOT detect deleted files**
- ❌ No cleanup of deleted file vectors

## Requirements

### 1. New File Detection

**Scenario**: User adds new file matching collection pattern
```
// Before
workspace/src/
  - main.rs
  - lib.rs

// User adds
workspace/src/
  - main.rs
  - lib.rs
  - utils.rs  ← NEW FILE
```

**Expected Behavior**:
1. Watcher detects new file
2. Checks if matches any collection pattern
3. Indexes file into appropriate collections
4. Updates collection metadata
5. Logs operation

### 2. Deleted File Detection

**Scenario**: User deletes file from workspace
```
// Before
workspace/src/
  - main.rs
  - lib.rs
  - old_code.rs

// User deletes old_code.rs
workspace/src/
  - main.rs
  - lib.rs
```

**Expected Behavior**:
1. Watcher detects file deletion
2. Identifies affected collections
3. Removes all vectors associated with deleted file
4. Updates collection metadata
5. Logs operation

### 3. Directory Operations

**New Directory**:
- Scan recursively for matching files
- Index all files following patterns

**Deleted Directory**:
- Remove all vectors from files in directory
- Handle nested directories

## Technical Design

### Enhanced File Watcher

```rust
#[derive(Debug, Clone)]
pub enum FileSystemEvent {
    Created { path: PathBuf },
    Modified { path: PathBuf },
    Deleted { path: PathBuf },
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct EnhancedFileWatcher {
    debouncer: Debouncer,
    workspace_config: Arc<RwLock<WorkspaceConfig>>,
    grpc_client: Arc<Mutex<VectorizerGrpcClient>>,
    file_index: Arc<RwLock<FileIndex>>,
}

// Track all indexed files and their collections
pub struct FileIndex {
    // file_path -> Vec<(collection_name, vector_ids)>
    file_to_collections: HashMap<PathBuf, Vec<CollectionVectorMapping>>,
    // collection_name -> Vec<file_path>
    collection_to_files: HashMap<String, HashSet<PathBuf>>,
}

#[derive(Clone)]
pub struct CollectionVectorMapping {
    pub collection_name: String,
    pub vector_ids: Vec<String>,
    pub last_hash: String,
}
```

### Event Handlers

#### 1. Handle Created Event

```rust
async fn handle_file_created(&self, path: PathBuf) -> Result<()> {
    // 1. Check if file matches any collection patterns
    let matching_collections = self.find_matching_collections(&path)?;
    
    if matching_collections.is_empty() {
        return Ok(()); // Not relevant to any collection
    }
    
    // 2. Read and process file
    let content = tokio::fs::read_to_string(&path).await?;
    let chunks = chunk_text(&content, self.config.chunk_size, self.config.chunk_overlap);
    
    // 3. Index into each matching collection
    for collection_name in matching_collections {
        let vector_ids = self.index_chunks(
            &collection_name,
            &path,
            chunks.clone()
        ).await?;
        
        // 4. Update file index
        self.file_index.write().await.add_mapping(
            path.clone(),
            collection_name.clone(),
            vector_ids,
            calculate_hash(&content)?
        );
    }
    
    info!("Indexed new file: {} into {} collections", 
        path.display(), matching_collections.len());
    
    Ok(())
}
```

#### 2. Handle Deleted Event

```rust
async fn handle_file_deleted(&self, path: PathBuf) -> Result<()> {
    // 1. Get all collections that had this file
    let mappings = self.file_index.read().await
        .get_mappings(&path)
        .ok_or(Error::FileNotIndexed)?;
    
    // 2. Remove vectors from each collection
    for mapping in mappings {
        self.delete_vectors_from_collection(
            &mapping.collection_name,
            &mapping.vector_ids
        ).await?;
        
        info!("Removed {} vectors from {} (file deleted)",
            mapping.vector_ids.len(), mapping.collection_name);
    }
    
    // 3. Remove from file index
    self.file_index.write().await.remove_file(&path);
    
    Ok(())
}
```

#### 3. Handle Modified Event

```rust
async fn handle_file_modified(&self, path: PathBuf) -> Result<()> {
    // 1. Calculate new hash
    let content = tokio::fs::read_to_string(&path).await?;
    let new_hash = calculate_hash(&content)?;
    
    // 2. Get existing mappings
    let mappings = self.file_index.read().await.get_mappings(&path);
    
    if let Some(mappings) = mappings {
        // 3. Check if actually changed
        if mappings.first().map(|m| &m.last_hash) == Some(&new_hash) {
            return Ok(()); // No actual change
        }
        
        // 4. Delete old vectors
        for mapping in &mappings {
            self.delete_vectors_from_collection(
                &mapping.collection_name,
                &mapping.vector_ids
            ).await?;
        }
    }
    
    // 5. Re-index (same as created)
    self.handle_file_created(path).await?;
    
    Ok(())
}
```

#### 4. Handle Renamed Event

```rust
async fn handle_file_renamed(&self, from: PathBuf, to: PathBuf) -> Result<()> {
    // Check if both old and new match patterns
    let old_matches = self.find_matching_collections(&from)?;
    let new_matches = self.find_matching_collections(&to)?;
    
    match (old_matches.is_empty(), new_matches.is_empty()) {
        (false, false) => {
            // Both match - update metadata, keep vectors
            self.update_file_metadata(&from, &to).await?;
        },
        (false, true) => {
            // Old matched, new doesn't - delete vectors
            self.handle_file_deleted(from).await?;
        },
        (true, false) => {
            // New matches, old didn't - index new
            self.handle_file_created(to).await?;
        },
        (true, true) => {
            // Neither matches - ignore
            Ok(())
        },
    }
}
```

### Pattern Matching

```rust
pub struct PatternMatcher {
    include_patterns: Vec<glob::Pattern>,
    exclude_patterns: Vec<glob::Pattern>,
}

impl PatternMatcher {
    pub fn matches(&self, path: &Path, base_path: &Path) -> bool {
        let relative_path = path.strip_prefix(base_path).ok()?;
        let path_str = relative_path.to_string_lossy();
        
        // Check exclude first
        if self.exclude_patterns.iter().any(|p| p.matches(&path_str)) {
            return false;
        }
        
        // Check include
        self.include_patterns.iter().any(|p| p.matches(&path_str))
    }
}
```

## Configuration Changes

```yaml
# vectorize-workspace.yml
global:
  file_watcher:
    enabled: true
    debounce_delay_ms: 1000
    
    # NEW: Event types to monitor
    events:
      created: true      # NEW
      modified: true     # existing
      deleted: true      # NEW
      renamed: true      # NEW
    
    # NEW: Scanning
    initial_scan: true   # Scan for new files on startup
    scan_interval: 300   # Re-scan every 5 minutes
    
    # NEW: Performance
    batch_size: 50       # Process N events before indexing
    max_queue_size: 1000
```

## Performance Considerations

### Initial Scan Optimization

```rust
async fn initial_workspace_scan(&self) -> Result<()> {
    // 1. Get all expected files from workspace config
    let expected_files = self.get_expected_files_from_config()?;
    
    // 2. Scan actual filesystem
    let actual_files = self.scan_filesystem()?;
    
    // 3. Find differences
    let new_files: Vec<_> = actual_files
        .difference(&expected_files)
        .collect();
    
    let deleted_files: Vec<_> = expected_files
        .difference(&actual_files)
        .collect();
    
    // 4. Process in batches
    for chunk in new_files.chunks(50) {
        let handles: Vec<_> = chunk.iter()
            .map(|path| self.handle_file_created(path.clone()))
            .collect();
        
        futures::future::join_all(handles).await;
    }
    
    Ok(())
}
```

### Event Batching

```rust
pub struct EventBatcher {
    pending: Vec<FileSystemEvent>,
    last_flush: Instant,
    batch_size: usize,
    flush_interval: Duration,
}

impl EventBatcher {
    pub async fn add_event(&mut self, event: FileSystemEvent) {
        self.pending.push(event);
        
        if self.should_flush() {
            self.flush().await;
        }
    }
    
    fn should_flush(&self) -> bool {
        self.pending.len() >= self.batch_size ||
        self.last_flush.elapsed() >= self.flush_interval
    }
}
```

## Testing Plan

### Unit Tests
```rust
#[tokio::test]
async fn test_detect_new_file() {
    let watcher = setup_test_watcher();
    let new_file = create_test_file("new.rs");
    
    // Trigger event
    watcher.handle_event(FileSystemEvent::Created(new_file)).await?;
    
    // Verify indexed
    assert!(watcher.file_index.contains(&new_file));
}

#[tokio::test]
async fn test_detect_deleted_file() {
    let watcher = setup_test_watcher();
    let file = create_and_index_test_file("to_delete.rs");
    
    // Delete file
    std::fs::remove_file(&file)?;
    watcher.handle_event(FileSystemEvent::Deleted(file.clone())).await?;
    
    // Verify vectors removed
    assert!(!watcher.file_index.contains(&file));
}
```

### Integration Tests
- Full workflow: create → modify → delete
- Concurrent file operations
- Bulk file operations (git pull)
- Directory operations

## Backwards Compatibility

- Existing file watcher behavior unchanged for modified files
- New events are opt-in via configuration
- Graceful fallback if event type not supported
- Migration tool for existing deployments

## Success Criteria

- ✅ New files detected within debounce delay
- ✅ Deleted files cleaned up within debounce delay
- ✅ No duplicate indexing
- ✅ No orphaned vectors
- ✅ < 100ms overhead per event
- ✅ Handles 1000+ file changes in bulk (git operations)

---

**Estimated Effort**: 2-3 weeks  
**Dependencies**: None  
**Risk**: Low (additive feature)

