# Holding a DashMap Ref from get_collection across a store-mutating call deadlocks

**Category**: concurrency
**Tags**: dashmap, deadlock, testing, vector-store

## Description

`VectorStore::get_collection` returns a DashMap `Ref`, which holds a READ LOCK on its shard. A `Ref` binding has a `Drop` impl, so it lives to the end of its BLOCK — not to its last use. NLL does not shorten it.

Any store call that mutates the same collection then wants the shard's WRITE lock and blocks forever against the reader you are still holding. `restore_native_snapshot` is a live example: it begins with `delete_collection`.

This hung a test for 3.8 hours with no panic and no output, and it is the same re-entrancy as the phase39 production deadlock (never hold a `Ref` while taking another).

BAD — `restored` is alive until the closing brace:

    let restored = store.get_collection(name).unwrap();
    assert!(restored.config().is_raw_vector());
    store.restore_native_snapshot(name, &id).unwrap();  // hangs

GOOD — clone out of a temporary, so the Ref dies at the end of the statement:

    fn config_of(store: &VectorStore, name: &str) -> CollectionConfig {
        store.get_collection(name).unwrap().config().clone()
    }

Diagnosis technique that found it: the hang produced no output at all, so bisect with `eprintln!` markers between statements and run under `timeout N cargo test -- --nocapture > file` (piping to `tail` loses the buffer when the timeout kills it). Run a known-good sibling test first as a control — `collection_ttl_persistence` passing in 0.02s proved the environment was fine and the fault was in the new test.
