# Task 1.2: Code Paths Documentation

## All Code Paths Calling `determine_collection_name()`

### Path 1: File Watcher - File Change Events

**Location**: `src/file_watcher/operations.rs:33-79`

```rust
pub async fn process_file_change(&self, event: &FileChangeEventWithMetadata) -> Result<()>
```

**Flow**:
1. File watcher detects change (Created/Modified/Deleted/Renamed)
2. Calls `index_file_from_path()` or `remove_file_from_path()`
3. Both call `determine_collection_name()` to get target collection

**Trigger**: File system events (notify-rs)

### Path 2: File Indexing

**Location**: `src/file_watcher/operations.rs:203-279`

```rust
pub async fn index_file_from_path(&self, path: &std::path::Path) -> Result<()>
```

**Flow**:
1. Check if file should be processed (patterns)
2. Call `determine_collection_name(path)` to get collection name
3. Copy file to temp directory
4. Index via DocumentLoader

**Trigger**: File creation/modification events

### Path 3: File Removal

**Location**: `src/file_watcher/operations.rs:282-287`

```rust
async fn remove_file_from_path(&self, path: &std::path::Path) -> Result<()>
```

**Flow**:
1. Call `determine_collection_name(path)` to find collection
2. Remove file by path from that collection

**Trigger**: File deletion events

### Path 4: File Rename

**Location**: `src/file_watcher/operations.rs:62-76`

```rust
FileChangeEvent::Renamed(old_path, new_path)
```

**Flow**:
1. Remove from old path (calls `determine_collection_name(old_path)`)
2. Index to new path (calls `determine_collection_name(new_path)`)

**Trigger**: File rename events

## Collection Name Logic

**Location**: `src/file_watcher/operations.rs:303-368`

**Priority Order**:
1. **Known Project Patterns** (workspace.yml-based)
   - `/docs/architecture/` → `docs-architecture`
   - `/vectorizer/src/` → `vectorizer-source`
   - `/gov/bips/` → `gov-bips`
   - etc.

2. **Default Fallback** (NEW - prevents empty collections)
   - Uses `self.config.default_collection`
   - Defaults to `workspace-default`
   - **Previously**: Generated `parent-child` names from paths

## Configuration

**Config Field**: `src/file_watcher/config.rs:19`

```rust
pub default_collection: Option<String>
```

**Default Value**: `Some("workspace-default")`

## Summary

All paths converge on `determine_collection_name()` which now safely falls back to a configured default instead of creating new collections from path components.
