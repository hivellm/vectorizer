## 1. Investigation Phase

- [ ] 1.1 Trace collection creation flow in REST API handler
- [ ] 1.2 Trace collection creation flow in GraphQL mutation
- [ ] 1.3 Verify `mark_collection_for_save()` is called after API collection creation
- [ ] 1.4 Check if `changes_detected` flag is set in AutoSaveManager when collection is created
- [ ] 1.5 Test: Create collection via API, check if it's in `.vecdb` after 5 minutes
- [ ] 1.6 Test: Create collection via API, restart server immediately, verify if collection loads
- [ ] 1.7 Check `load_all_persisted_collections()` to see what collections are loaded
- [ ] 1.8 Verify compaction includes API-created collections

## 2. Fix Immediate Persistence

- [ ] 2.1 Add immediate save after REST API collection creation
  - [ ] 2.1.1 Call `save_collection_to_file()` after `create_collection()` in `rest_handlers.rs`
  - [ ] 2.1.2 Handle save errors gracefully (log warning, don't fail request)
  - [ ] 2.1.3 Test immediate save works correctly
- [ ] 2.2 Add immediate save after GraphQL collection creation
  - [ ] 2.2.1 Call `save_collection_to_file()` after `create_collection()` in GraphQL schema
  - [ ] 2.2.2 Handle save errors gracefully
  - [ ] 2.2.3 Test immediate save works correctly
- [ ] 2.3 Ensure `mark_collection_for_save()` triggers `changes_detected` flag
  - [ ] 2.3.1 Check AutoSaveManager connection to VectorStore
  - [ ] 2.3.2 Verify flag is set when collection is marked for save
  - [ ] 2.3.3 Test flag triggers immediate or next-cycle compaction

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
