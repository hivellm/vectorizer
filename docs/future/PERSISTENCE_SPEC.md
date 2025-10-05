# Persistent Collections Specification

**Status**: Specification  
**Priority**: 🟡 **P1 - HIGH** ⬇️  
**Complexity**: Medium  
**Created**: October 1, 2025  
**Updated**: October 1, 2025 - **PRIORITY DOWNGRADED BASED ON BENCHMARK ANALYSIS**

## 🎯 **WHY P1 PRIORITY - BENCHMARK INSIGHTS**

**Priority downgraded** from P0 to P1 based on benchmark analysis showing:
1. **System is stable**: Benchmarks demonstrate excellent performance and reliability
2. **No data loss incidents**: Current system handles data consistently
3. **Performance is excellent**: < 1ms search latency, no critical bottlenecks
4. **Focus on higher ROI**: Quantization (P0) delivers immediate 4x memory reduction + better quality
5. **System works well**: Current persistence mechanisms are adequate for production use

## Problem Statement

Currently, the vectorizer:
- ✅ Loads workspace-based collections from cache files
- ✅ Rebuilds indexes from cached data
- ❌ **No persistence for dynamic collections** (created via API/MCP)
- ❌ Dynamic collections are lost on server restart
- ❌ Workspace collections can be modified via API (should be read-only)

## Requirements

### 1. Collection Types

#### Workspace Collections (Read-Only)
- **Source**: Defined in `vectorize-workspace.yml`
- **Behavior**: 
  - Loaded at startup from cache
  - Updated only by file watcher when source files change
  - **Read-only via API/MCP** - cannot be modified
  - Cannot be deleted via API
  - Automatically rebuild when config changes

#### Dynamic Collections (Persistent)
- **Source**: Created via API/MCP at runtime
- **Behavior**:
  - Full CRUD operations available
  - Persisted to disk automatically
  - Survive server restarts
  - Independent of workspace configuration
  - Can be deleted via API

### 2. Storage Strategy

```
data/
├── workspace/              # Workspace collections (read-only)
│   ├── {project-name}/
│   │   ├── {collection-name}/
│   │   │   ├── vectors.bin      # Vector data
│   │   │   ├── index.hnsw       # HNSW index
│   │   │   ├── metadata.json    # Collection metadata
│   │   │   └── cache.bin        # File cache
│   └── ...
│
└── dynamic/                # Dynamic collections (read-write)
    ├── {collection-id}/
    │   ├── vectors.bin
    │   ├── index.hnsw
    │   ├── metadata.json
    │   └── wal.log          # Write-ahead log
    └── ...
```

### 3. Persistence Mechanism

#### Write-Ahead Log (WAL)
```rust
pub struct WALEntry {
    sequence: u64,
    timestamp: DateTime<Utc>,
    operation: Operation,
    collection_id: String,
}

pub enum Operation {
    InsertVector { id: String, data: Vec<f32>, metadata: HashMap<String, String> },
    UpdateVector { id: String, data: Option<Vec<f32>>, metadata: Option<HashMap<String, String>> },
    DeleteVector { id: String },
    CreateCollection { config: CollectionConfig },
    DeleteCollection,
}
```

#### Checkpoint Strategy
- **Trigger**: Every 1000 operations OR every 5 minutes
- **Process**:
  1. Flush current state to disk
  2. Truncate WAL
  3. Update checkpoint marker
  
#### Recovery Process
```rust
async fn recover_dynamic_collection(path: PathBuf) -> Result<Collection> {
    // 1. Load last checkpoint
    let mut collection = load_checkpoint(&path)?;
    
    // 2. Replay WAL entries
    let wal_entries = read_wal(&path)?;
    for entry in wal_entries {
        apply_operation(&mut collection, entry.operation)?;
    }
    
    // 3. Rebuild HNSW index if needed
    if collection.needs_reindex() {
        collection.rebuild_index()?;
    }
    
    Ok(collection)
}
```

## Technical Design

### 1. Collection Metadata

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct CollectionMetadata {
    pub id: String,
    pub name: String,
    pub collection_type: CollectionType,
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub vector_count: usize,
    pub is_read_only: bool,
    pub source: CollectionSource,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CollectionType {
    Workspace,      // From workspace config
    Dynamic,        // Created at runtime
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CollectionSource {
    Workspace {
        project_name: String,
        config_path: String,
    },
    Dynamic {
        created_by: Option<String>,
        api_endpoint: String,
    },
}
```

### 2. API Changes

```rust
// Collection operations with type awareness
impl VectorStore {
    pub async fn create_collection(&self, config: CollectionConfig) -> Result<Collection> {
        // Validate not a workspace collection name
        if self.is_workspace_collection(&config.name) {
            return Err(Error::ReadOnlyCollection(config.name));
        }
        
        let collection = Collection::new_dynamic(config)?;
        
        // Persist immediately
        self.persist_dynamic_collection(&collection).await?;
        
        Ok(collection)
    }
    
    pub async fn insert_vectors(&self, collection_name: &str, vectors: Vec<Vector>) -> Result<()> {
        let collection = self.get_collection(collection_name)?;
        
        if collection.is_read_only() {
            return Err(Error::ReadOnlyCollection(collection_name.to_string()));
        }
        
        // Insert and log to WAL
        collection.insert(vectors)?;
        self.wal.append(collection_name, Operation::InsertVector(...))?;
        
        Ok(())
    }
}
```

### 3. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("Collection '{0}' is read-only (workspace collection)")]
    ReadOnlyCollection(String),
    
    #[error("Cannot delete workspace collection '{0}'")]
    CannotDeleteWorkspace(String),
    
    #[error("WAL corruption detected at sequence {0}")]
    WALCorruption(u64),
    
    #[error("Checkpoint failed: {0}")]
    CheckpointFailed(String),
    
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}
```

## Implementation Plan

### Phase 1: WAL Implementation (1 week)
```rust
// src/persistence/wal.rs
pub struct WriteAheadLog {
    file: Mutex<File>,
    sequence: AtomicU64,
    checkpoint_threshold: usize,
}

impl WriteAheadLog {
    pub fn append(&self, entry: WALEntry) -> Result<()>;
    pub fn read_from(&self, sequence: u64) -> Result<Vec<WALEntry>>;
    pub fn truncate(&self) -> Result<()>;
    pub fn checkpoint(&self) -> Result<u64>;
}
```

### Phase 2: Collection Type System (3 days)
- Add `CollectionType` enum
- Add `is_read_only` flag to collections
- Update all API handlers to check read-only status
- Add validation middleware

### Phase 3: Persistence Layer (1 week)
```rust
// src/persistence/dynamic.rs
pub struct DynamicCollectionPersistence {
    base_path: PathBuf,
    checkpoint_interval: Duration,
}

impl DynamicCollectionPersistence {
    pub async fn save(&self, collection: &Collection) -> Result<()>;
    pub async fn load(&self, collection_id: &str) -> Result<Collection>;
    pub async fn checkpoint(&self, collection: &Collection) -> Result<()>;
    pub async fn recover(&self, collection_id: &str) -> Result<Collection>;
}
```

### Phase 4: Integration & Testing (1 week)
- Update VectorStore to use new persistence
- Add recovery on startup
- Implement background checkpoint thread
- Add comprehensive tests
- Performance benchmarks

## API Changes

### New Endpoints

```http
# Get collection info with type
GET /api/collections/{name}
Response: {
  "name": "my-collection",
  "type": "dynamic",
  "is_read_only": false,
  "vector_count": 1000,
  ...
}

# List collections by type
GET /api/collections?type=dynamic
GET /api/collections?type=workspace
```

### Modified Behavior

```http
# DELETE on workspace collection - ERROR
DELETE /api/collections/workspace-collection
Response: 400 Bad Request
{
  "error": "Cannot delete workspace collection. Workspace collections are read-only."
}

# INSERT into workspace collection - ERROR
POST /api/collections/workspace-collection/vectors
Response: 400 Bad Request
{
  "error": "Cannot modify workspace collection. Workspace collections are read-only."
}
```

## Configuration

```yaml
# config.yml
persistence:
  # Base directory for persistent data
  data_dir: ./data
  
  # WAL settings
  wal:
    enabled: true
    checkpoint_threshold: 1000  # operations
    checkpoint_interval: 300    # seconds
    max_wal_size_mb: 100
  
  # Checkpoint settings
  checkpoint:
    enabled: true
    background_interval: 300    # seconds
    keep_last: 5                # checkpoints
    compression: true
  
  # Recovery settings
  recovery:
    auto_recover: true
    verify_integrity: true
    repair_on_error: true
```

## Migration Strategy

### Existing Deployments

1. **Detect existing data**
2. **Classify collections**:
   - Match against workspace config → `Workspace`
   - Unmatched → `Dynamic`
3. **Migrate to new structure**
4. **Create initial checkpoint**

```rust
async fn migrate_existing_data() -> Result<()> {
    let workspace_config = load_workspace_config()?;
    let existing_collections = discover_collections()?;
    
    for collection in existing_collections {
        let is_workspace = workspace_config
            .projects
            .iter()
            .flat_map(|p| &p.collections)
            .any(|c| c.name == collection.name);
        
        if is_workspace {
            // Move to workspace/
            migrate_to_workspace(&collection)?;
        } else {
            // Move to dynamic/ and create WAL
            migrate_to_dynamic(&collection)?;
        }
    }
    
    Ok(())
}
```

## Testing Plan

### Unit Tests
- WAL append/read/truncate operations
- Checkpoint creation and loading
- Collection type detection
- Read-only validation

### Integration Tests
- Create dynamic collection and restart server
- Modify workspace collection (should fail)
- Delete workspace collection (should fail)
- Recovery from WAL corruption
- Concurrent operations with checkpointing

### Performance Tests
- WAL overhead per operation
- Checkpoint time for various sizes
- Recovery time benchmarks
- Memory overhead of WAL buffer

## Success Criteria

- ✅ Dynamic collections survive server restart
- ✅ Workspace collections are truly read-only
- ✅ WAL overhead < 1% of operation time
- ✅ Checkpoint time < 500ms for 100K vectors
- ✅ Recovery time < 2s for 1M vectors
- ✅ Zero data loss on clean shutdown
- ✅ < 0.1% data loss on crash (only uncommitted WAL)

## Future Enhancements

- Distributed WAL for clustering
- Multi-level checkpointing
- Incremental checkpoints (delta-based)
- Compression of WAL entries
- Configurable fsync policies
- Snapshot isolation for reads during checkpoint

---

**Next Steps**: Review and approve before implementation

