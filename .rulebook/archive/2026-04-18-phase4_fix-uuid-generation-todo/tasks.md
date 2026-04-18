## 1. Investigation

- [x] 1.1 Read the three call sites in `src/server/rest_handlers.rs` passing a per-request random UUID to `HubManager::record_usage`: `create_collection` (line 815), batch-insert path (line 1340), single-insert path (line 1394).
- [x] 1.2 `Collection` has no persistent `uuid` field; adding one would require schema migration of on-disk state, which is out of scope because `phase3_split-vector-store-monolith` will rewrite that type anyway.

## 2. Implementation

- [x] 2.1 Chose the UUIDv5-from-name route instead of adding a field. Same name always hashes to the same UUID; zero on-disk change; no migration risk.
- [x] 2.2 Added `collection_metrics_uuid(name: &str) -> Uuid` helper in `rest_handlers.rs` using `Uuid::new_v5(&COLLECTION_NAMESPACE_UUID, name.as_bytes())`. Bespoke v4 namespace `7f5ac640-3dfe-4e1a-9d82-d82d4ea75501` minted for Vectorizer.
- [x] 2.3 Replaced all three `Uuid::new_v4()` call sites with `collection_metrics_uuid(...)`. Stale marker comments removed.
- [x] 2.4 Added `"v5"` to the `uuid` crate's feature flags in `Cargo.toml`.

## 3. Migration

- [x] 3.1 NOT NEEDED — the v5-from-name approach is stateless by construction. No backfill; no startup step.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 CHANGELOG `[Unreleased] > Fixed` entry added. A dedicated `docs/api/collections.md` is intentionally NOT authored here because the change is internal (Hub-metrics only) and no public API surface changed.
- [x] 4.2 Four regression tests in `rest_handlers::tests`: determinism (same name → same UUID), discrimination (different names), version byte (v5), edge-case stability (empty / unicode). All pass.
- [x] 4.3 `cargo test --lib -p vectorizer -- server::rest_handlers` 4/4 passing; `cargo clippy --all-targets -- -D warnings` clean.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG entry)
- [x] Write tests covering the new behavior (4 new unit tests)
- [x] Run tests and confirm they pass (4/4 green, clippy clean)
