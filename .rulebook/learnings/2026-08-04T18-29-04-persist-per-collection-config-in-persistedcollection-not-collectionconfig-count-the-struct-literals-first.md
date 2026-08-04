# Persist per-collection config in PersistedCollection, not CollectionConfig — count the struct literals first
**Source**: manual
**Date**: 2026-08-04
**Related Task**: phase1_persist-collection-ttl-config
**Tags**: persistence, ttl, vecdb, serde
Adding a durable per-collection setting: the field-placement decision is settled by counting Rust struct literals, not by which struct reads best. `#[serde(default)]` fixes deserialization of old archives, but every existing literal without `..Default::default()` becomes a compile error.

- `CollectionConfig` — semantically the right home and already round-trips through `.vecdb`, but `grep -rn "CollectionConfig {" --include=*.rs | wc -l` = 358 literals, only ~200 near a `..Default::default()`. Rejected on cost.
- `PersistedCollection` — 8 literals, all fields already `#[serde(default)]`, and it is the record `.vecdb` actually serializes. That is where `ttl_secs` went.

Save/load sites to wire for anything in `PersistedCollection` (the live path is the first of each pair):
- save: `storage/compact.rs` `compact_from_memory` (has `&store`), `persistence/mod.rs` `VectorStore::save`, `persistence/snapshots.rs` native snapshot, `db/vector_store/autosave.rs` legacy (`&self` variant reachable; the two `*_static` variants hold only `&CollectionType` and cannot see store-level state), `file_loader/persistence.rs`.
- load: `db/vector_store/persistence/loading.rs` `load_all_persisted_collections` + `load_persisted_collection`, `persistence/mod.rs` `VectorStore::load`, `snapshots.rs` `restore_native_snapshot`.

Ordering trap on load: apply the setting after `create_collection*` but know why it is safe relative to the vectors. `load_collection_from_cache` writes into the collection directly rather than through `VectorStore::insert`, so a restored vector is not re-processed by insert-time rules — which is what keeps a restored `__expires_at` from being refreshed to the load time. Assert the exact timestamp in the test, not just its presence; "an expiry exists" passes either way.

Also check the name lifecycle, which is easy to miss when config is keyed by collection name: `delete_collection` must drop it (else the next collection created under that name inherits it), `rename_collection` must move it before registering the grace-window alias (afterwards the old key resolves to the new name and the move reads an empty key), and alias resolution must happen on lookup so writes addressed to an alias see the target's rule.