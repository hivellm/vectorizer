## 1. Investigation

- [ ] 1.1 Read the two sites in `src/server/rest_handlers.rs:1340` and `:1394`; identify the collection context available at call time
- [ ] 1.2 Check `src/db/collection.rs` and `src/models/mod.rs` for an existing persistent `uuid` / `id` field on `Collection`; record in `design.md`

## 2. Implementation

- [ ] 2.1 Add a `uuid: Uuid` field on `Collection` if absent; initialize on create, persist with the collection metadata
- [ ] 2.2 Replace both `Uuid::new_v4()` calls in the handler with a lookup: `collection.uuid`
- [ ] 2.3 Remove the stale T-A-S-K marker comment that said "use actual collection UUID"

## 3. Migration

- [ ] 3.1 For on-disk collections that predate the `uuid` field: on load, if absent, generate and persist once (idempotent backfill)
- [ ] 3.2 Add a simple migration step to `src/persistence/` ensuring this runs on startup

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `docs/api/collections.md` describing the stable-UUID contract
- [ ] 4.2 Add a regression test: create collection, read it twice, assert identical UUID; reload from disk, assert UUID stable
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
