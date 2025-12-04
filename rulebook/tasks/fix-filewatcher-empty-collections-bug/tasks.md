## 1. Analysis and Planning
- [ ] 1.1 Analyze existing empty collections in current database
- [ ] 1.2 Document all code paths that call determine_collection_name()
- [ ] 1.3 Create comprehensive test cases for collection name determination

## 2. Fix determine_collection_name() Logic
- [ ] 2.1 Add file-to-collection lookup in FileIndex
- [ ] 2.2 Modify determine_collection_name() to check existing collections first
- [ ] 2.3 Add default_collection to FileWatcherConfig
- [ ] 2.4 Remove aggressive path-based collection name generation (lines 358-377)
- [ ] 2.5 Add optional collection_mapping in workspace.yml config

## 3. Add Collection Statistics
- [ ] 3.1 Add get_collection_stats() to Collection struct
- [ ] 3.2 Add is_collection_empty() helper to VectorStore
- [ ] 3.3 Add list_empty_collections() to VectorStore
- [ ] 3.4 Add get_collection_file_count() helper

## 4. Implement Cleanup Functionality
- [ ] 4.1 Create cleanup_empty_collections() in VectorStore
- [ ] 4.2 Add batch delete optimization for multiple collections
- [ ] 4.3 Add dry-run mode to preview what would be deleted
- [ ] 4.4 Add cleanup statistics return (count deleted, bytes freed, etc.)

## 5. REST API Integration
- [ ] 5.1 Add DELETE /api/v1/collections/cleanup endpoint
- [ ] 5.2 Add GET /api/v1/collections/stats endpoint
- [ ] 5.3 Add query parameter ?dry_run=true for preview
- [ ] 5.4 Add response with cleanup statistics

## 6. MCP Tool Integration
- [ ] 6.1 Add cleanup_empty_collections MCP tool
- [ ] 6.2 Add list_empty_collections MCP tool
- [ ] 6.3 Add get_collection_stats MCP tool
- [ ] 6.4 Update MCP tool documentation

## 7. Server Integration
- [ ] 7.1 Add startup_cleanup_empty option to config
- [ ] 7.2 Run cleanup on startup if configured
- [ ] 7.3 Add cleanup metrics to monitoring
- [ ] 7.4 Log cleanup operations with details

## 8. Testing - Unit Tests
- [ ] 8.1 Test determine_collection_name() with existing collections
- [ ] 8.2 Test determine_collection_name() with new files
- [ ] 8.3 Test cleanup_empty_collections() functionality
- [ ] 8.4 Test is_collection_empty() edge cases
- [ ] 8.5 Test batch delete operations
- [ ] 8.6 Test dry-run mode

## 9. Testing - Integration Tests
- [ ] 9.1 Test file watcher creates correct collections
- [ ] 9.2 Test file watcher updates existing collections
- [ ] 9.3 Test cleanup via REST API
- [ ] 9.4 Test cleanup via MCP tool
- [ ] 9.5 Test startup cleanup
- [ ] 9.6 Test collection stats accuracy

## 10. Testing - Real World Scenarios
- [ ] 10.1 Test with current database (with empty collections)
- [ ] 10.2 Verify cleanup removes only empty collections
- [ ] 10.3 Verify file changes update correct collections
- [ ] 10.4 Monitor search performance improvement
- [ ] 10.5 Verify no data loss in populated collections

## 11. Documentation
- [ ] 11.1 Update docs/specs/FILE_WATCHER.md
- [ ] 11.2 Update docs/specs/API_REFERENCE.md
- [ ] 11.3 Update docs/specs/MCP_TOOLS.md
- [ ] 11.4 Add cleanup guide to docs/users/
- [ ] 11.5 Update CHANGELOG.md

## 12. Verification and Cleanup
- [ ] 12.1 Run full test suite
- [ ] 12.2 Verify linter passes
- [ ] 12.3 Check test coverage (95%+)
- [ ] 12.4 Manual testing with real workspace
- [ ] 12.5 Performance benchmarking before/after
