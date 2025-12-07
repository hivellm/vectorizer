# Proposal: fix-collection-persistence-on-restart

## Why

Collections created dynamically via API (REST or GraphQL) are not persisting after server restart. This is a critical data loss issue that affects user trust and system reliability.

**Current Behavior**:
- Collections created via `POST /collections` or GraphQL `createCollection` mutation are added to in-memory `VectorStore`
- `mark_collection_for_save()` is called to mark collection for auto-save
- Auto-save manager runs every 5 minutes and compacts to `vectorizer.vecdb`
- However, collections created via API may not be triggering the `changes_detected` flag
- On server restart, only collections from `vectorizer.vecdb` are loaded
- API-created collections that weren't saved before restart are lost

**Problem Scenarios**:
- User creates collection via API
- Server restarts before 5-minute auto-save interval
- Collection is lost and not available after restart
- User must recreate collection manually
- Data loss for any vectors inserted into the collection

**Root Cause Analysis Needed**:
1. Does `mark_collection_for_save()` properly trigger `changes_detected` in AutoSaveManager?
2. Are API-created collections being included in compaction to `.vecdb`?
3. Is there a race condition where collections are created but auto-save hasn't run yet?
4. Are collections being saved immediately on creation, or only on next auto-save cycle?

## What Changes

### 1. Investigate Current Persistence Flow

- Trace collection creation via API through the codebase
- Verify `mark_collection_for_save()` is called for API-created collections
- Check if `changes_detected` flag is set in AutoSaveManager
- Verify collections are included in compaction to `.vecdb`
- Test persistence by creating collection, restarting server, checking if it loads

### 2. Fix Immediate Persistence

**Option A: Force immediate save on creation**
- Call `save_collection_to_file()` immediately after API collection creation
- Ensure collection is written to disk before returning success response
- This guarantees persistence even if server crashes before auto-save

**Option B: Trigger immediate compaction**
- Set `changes_detected` flag when collection is created
- Trigger immediate compaction (don't wait for 5-minute interval)
- This ensures collection is in `.vecdb` quickly

**Option C: Hybrid approach**
- Save collection to raw files immediately on creation
- Trigger compaction on next auto-save cycle (or immediately if needed)
- This provides both immediate persistence and efficient compaction

### 3. Verify Collection Loading

- Ensure `load_all_persisted_collections()` loads all collections from `.vecdb`
- Verify API-created collections are included in loaded collections
- Test that collections created via API are available after restart

### 4. Add Integration Tests

- Test: Create collection via API, restart server, verify collection exists
- Test: Create collection with vectors, restart, verify collection and vectors exist
- Test: Multiple collections created via API, all persist after restart

## Impact

- **Affected code**:
  - Modified `src/server/rest_handlers.rs` - Force save after collection creation
  - Modified `src/api/graphql/schema.rs` - Force save after collection creation
  - Modified `src/db/vector_store.rs` - Ensure `mark_collection_for_save()` triggers persistence
  - Modified `src/db/auto_save.rs` - Potentially trigger immediate save for new collections
  - New tests in `tests/api/rest/` - Test collection persistence
- **Breaking change**: NO - Only fixes persistence, doesn't change API
- **User benefit**: Collections created via API persist across server restarts, preventing data loss
