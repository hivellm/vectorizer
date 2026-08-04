# In-process harness immediately caught a production deadlock (phase39)

**Category**: testing
**Tags**: testing, deadlock, harness, phase39, analysis:2026-07-11-improvement-analysis

## Description

The phase39 in-process REST harness (TestApp over the real build_router via tower oneshot, crates/vectorizer-server/tests/common) unlocked ~150 tests that never ran in CI. The FIRST run of handler coverage caught a real production deadlock: bulk_update_metadata held the DashMap collection Ref across store.update, which takes get_collection_mut (RefMut) on the same shard — every production call hung forever (fixed: drop the Ref before mutating, vectors.rs). Two prior 'known bugs' turned out long-fixed but invisible: the replication snapshot-sync bug (tests compiled via #[path] includes that a mod.rs-only scan misread as orphaned) and the gRPC 'update fails in CI' bug (4 ignores removed). Lessons: (1) handler with zero coverage = assume broken until proven; (2) when scanning for orphaned test files, check #[path] includes, not just mod declarations; (3) 'known bug' ignore reasons rot — re-run them before believing them; (4) store.update is the only VectorStore mutation taking an unconditional RefMut — never call it while holding a get_collection Ref.

## When to Use

When adding REST handler tests, scanning for dead test files, triaging old #[ignore] reasons, or calling store.update from any code holding a collection Ref.
