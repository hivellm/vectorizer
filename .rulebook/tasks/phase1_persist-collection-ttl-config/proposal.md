# Proposal: phase1_persist-collection-ttl-config

## Why

`phase1_collection-ttl-is-never-applied` made a configured collection TTL
actually stamp `__expires_at` on every vector that arrives, but left the rule
itself where it already lived: the `VectorStore` metadata `DashMap`, under the
key `ttl:{collection}`. That map is created empty in all three `VectorStore`
constructors and nothing ever writes it to disk (`replication_role` and the
replication addresses share the same fate).

So the TTL is process-scoped. After a restart the vectors already stamped keep
their expiry — that part is durable, it lives in the payload — but new inserts
stop expiring until someone re-issues `POST /collections/{name}/ttl`. Nothing
warns about it, and `GET /collections/{name}/ttl` will honestly report `null`,
so the failure is quiet: a collection that was expiring yesterday accumulates
immortal vectors today.

A durability guarantee that holds only until the next deploy is the kind of
half-truth the TTL work has been removing.

## What Changes

Give the collection TTL a durable home and load it at boot. Options, cheapest
first:

- **A sidecar file in the data dir.** `VectorStore::get_data_dir()` already
  resolves the directory (env-overridable via `VECTORIZER_DATA_DIR`). Write
  `collection_ttls.json` on every set/clear, read it during bootstrap and
  replay it through `VectorStore::set_collection_ttl`. Note the constraint
  that made this out of scope for the original task: the in-process REST test
  harness would then write into the resolved data dir, so the save/load pair
  must take an explicit path (pure functions over `(path, map)`) and the
  handler passes the resolved one.
- **`CollectionIndex.metadata`.** `crates/vectorizer/src/storage/index.rs`
  already carries a `HashMap<String, String>` per collection in the `.vecidx`
  index, with `#[serde(default)]`, and it is persisted. It appears to be
  written empty and never read — confirm that before relying on it, then
  populate it during compaction and consume it on load.
- **`CollectionConfig`.** The semantically correct home, and it round-trips
  through `.vecdb` already. Rejected for the original task because
  `CollectionConfig` is built with struct literals in ~358 places, of which
  only ~200 spread `..Default::default()`; adding a field breaks the rest.
  Viable if those literals are normalised first.

Whichever lands, `GET /collections/{name}/ttl` must report the persisted value
after a restart, and the process-scoped caveat has to come out of the docs it
was written into: `docs/users/api/API_REFERENCE.md` ("Collection TTL
semantics"), `docs/specs/VECTORIZER_RPC.md`, the `set_collection_ttl` doc
comment in all five SDKs, and the CHANGELOG's "Known limitation" bullet.

## Impact

- Affected specs: db/ttl
- Affected code: `crates/vectorizer/src/db/vector_store/metadata.rs`,
  `crates/vectorizer-server/src/server/core/bootstrap.rs`,
  `crates/vectorizer-server/src/server/rest_handlers/collections.rs`,
  `crates/vectorizer-server/src/protocol/rpc/dispatch.rs`, plus whichever
  persistence surface is chosen
- Breaking change: NO — it makes an existing setting survive a restart
- User benefit: a collection configured to expire its vectors keeps doing so
  after a deploy, without an operator remembering to re-apply it
