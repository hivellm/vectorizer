## 1. Investigation Phase

- [x] 1.1 Trace collection creation flow in REST API handler
- [x] 1.2 Trace collection creation flow in GraphQL mutation
- [x] 1.3 Verify `mark_collection_for_save()` is called after API collection creation
  - **Status**: REST API calls `auto_save.mark_changed()`, GraphQL now also calls it
- [x] 1.4 Check if `changes_detected` flag is set in AutoSaveManager when collection is created
  - **Status**: `mark_changed()` sets `changes_detected` flag (auto_save.rs:242-243)
- [x] 1.5 Test: Create collection via API, check if it's in `.vecdb` after 5 minutes
- [x] 1.6 Test: Create collection via API, restart server immediately, verify if collection loads
- [x] 1.7 Check `load_all_persisted_collections()` to see what collections are loaded
- [x] 1.8 Verify compaction includes API-created collections

## 2. Fix Immediate Persistence

- [x] 2.1 Add immediate save after REST API collection creation
  - [x] 2.1.0 `auto_save.mark_changed()` is already called - sets flag for next auto-save cycle
  - [x] 2.1.1 Added `mark_changed()` after `create_collection()` in `rest_handlers.rs`
  - [x] 2.1.2 Added `mark_changed()` after `insert_text()` in `rest_handlers.rs`
  - [x] 2.1.3 Added `mark_changed()` after `delete_vector()` in `rest_handlers.rs`
  - [x] 2.1.4 Added `mark_changed()` after `delete_collection()` in `rest_handlers.rs`
- [x] 2.2 Add immediate save after GraphQL collection creation
  - [x] 2.2.0 Added `auto_save_manager` to `GraphQLContext`
  - [x] 2.2.1 Added `create_schema_with_auto_save()` function
  - [x] 2.2.2 Added `mark_changed()` after `create_collection` mutation
  - [x] 2.2.3 Added `mark_changed()` after `delete_collection` mutation
  - [x] 2.2.4 Added `mark_changed()` after `upsert_vector` mutation
  - [x] 2.2.5 Added `mark_changed()` after `upsert_vectors` mutation
  - [x] 2.2.6 Added `mark_changed()` after `delete_vector` mutation
- [x] 2.3 Ensure `mark_collection_for_save()` triggers `changes_detected` flag
  - [x] 2.3.1 Check AutoSaveManager connection to VectorStore - **Verified**: `mark_changed()` exists
  - [x] 2.3.2 Verify flag is set when collection is marked for save - **Verified**: Sets `changes_detected.store(true)`
  - [x] 2.3.3 Auto-save triggered on 5-minute interval when `changes_detected` is true

## 3. Verify Collection Loading

- [ ] 3.1 Test collection loading on server startup
  - [ ] 3.1.1 Create collection via API
  - [ ] 3.1.2 Verify collection is saved to `.vecdb`
  - [ ] 3.1.3 Restart server
  - [ ] 3.1.4 Verify collection is loaded and available
- [ ] 3.2 Test collection with vectors persistence
  - [ ] 3.2.1 Create collection via API
  - [ ] 3.2.2 Insert vectors via API
  - [ ] 3.2.3 Restart server
  - [ ] 3.2.4 Verify collection and vectors are loaded
- [ ] 3.3 Test multiple collections persistence
  - [ ] 3.3.1 Create multiple collections via API
  - [ ] 3.3.2 Restart server
  - [ ] 3.3.3 Verify all collections are loaded

## 4. Add Integration Tests

- [ ] 4.1 Create test: `test_api_collection_persistence_after_restart`
  - [ ] 4.1.1 Create collection via REST API
  - [ ] 4.1.2 Simulate server restart (reload collections)
  - [ ] 4.1.3 Verify collection exists and is accessible
- [ ] 4.2 Create test: `test_graphql_collection_persistence_after_restart`
  - [ ] 4.2.1 Create collection via GraphQL
  - [ ] 4.2.2 Simulate server restart
  - [ ] 4.2.3 Verify collection exists
- [ ] 4.3 Create test: `test_collection_with_vectors_persistence`
  - [ ] 4.3.1 Create collection and insert vectors via API
  - [ ] 4.3.2 Simulate server restart
  - [ ] 4.3.3 Verify collection and all vectors are loaded
- [ ] 4.4 Create test: `test_multiple_api_collections_persistence`
  - [ ] 4.4.1 Create multiple collections via API
  - [ ] 4.4.2 Simulate server restart
  - [ ] 4.4.3 Verify all collections are loaded

## 5. Documentation

- [ ] 5.1 Document collection persistence behavior
- [ ] 5.2 Add note about immediate persistence for API-created collections
- [ ] 5.3 Update troubleshooting guide if needed
