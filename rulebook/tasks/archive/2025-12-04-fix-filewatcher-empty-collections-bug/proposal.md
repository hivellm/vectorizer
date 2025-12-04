# Proposal: fix-filewatcher-empty-collections-bug

## Why

The file watcher has a critical bug in the `determine_collection_name()` function. When a file doesn't match any known pattern (lines 358-377 in `src/file_watcher/operations.rs`), it generates a new collection name based on the file path components instead of using an existing collection or a proper default.

This causes:
1. **Empty collections proliferation**: Every file change creates a new empty collection with names like `vectorizer-src`, `rulebook-tasks`, etc.
2. **Data fragmentation**: Files that should update existing collections create new ones instead
3. **Storage waste**: Hundreds of empty collections consuming memory and disk space
4. **Search degradation**: Search has to scan through empty collections unnecessarily

The fallback logic at lines 358-377 is too aggressive and should instead:
- Check if the file belongs to an already indexed collection
- Use a configurable default collection name
- Only create new collections when explicitly configured to do so

## What Changes

### 1. Fix `determine_collection_name()` Logic
- Remove the aggressive fallback that creates collection names from path components
- Add lookup to check existing collections for files already indexed
- Use a proper default collection name from configuration
- Add optional collection mapping in workspace.yml

### 2. Add Cleanup Utility
- Create `cleanup_empty_collections()` function in VectorStore
- Add REST API endpoint: `DELETE /api/v1/collections/cleanup`
- Add MCP tool: `cleanup_empty_collections`
- Add option to run cleanup on server startup

### 3. Improve Collection Management
- Add `get_collection_stats()` to return vector count, file count, etc.
- Add `is_collection_empty()` helper
- Add batch delete operation for cleanup efficiency

## Impact

- **Affected specs**: 
  - `docs/specs/FILE_WATCHER.md` - File watcher behavior
  - `docs/specs/API_REFERENCE.md` - New cleanup endpoint
  - `docs/specs/MCP_TOOLS.md` - New MCP tool

- **Affected code**: 
  - `src/file_watcher/operations.rs` - Fix determine_collection_name()
  - `src/db/vector_store.rs` - Add cleanup_empty_collections()
  - `src/server/rest_handlers.rs` - Add cleanup endpoint
  - `src/mcp/tools.rs` - Add cleanup tool

- **Breaking change**: NO
  - Existing collections remain unchanged
  - New behavior only affects new file changes
  - Cleanup is opt-in via API call

- **User benefit**: 
  - No more empty collections cluttering the database
  - File changes update correct collections
  - Better search performance (fewer empty collections to scan)
  - Easy cleanup of existing empty collections
  - Better disk and memory usage
