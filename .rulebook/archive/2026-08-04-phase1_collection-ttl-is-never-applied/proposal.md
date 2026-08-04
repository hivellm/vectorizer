# Proposal: phase1_collection-ttl-is-never-applied

## Why

`POST /collections/{name}/ttl` (`set_collection_ttl`) validates `ttl_secs`,
writes it to store metadata under the key `ttl:{collection}`, answers
`{"status":"ok"}` — and nothing ever reads that key. `grep -rn '"ttl:'` finds
the write and no read; the only readers of store metadata are the generic
Qdrant metadata-key browsing endpoints.

So a caller can configure a collection-wide TTL, get a success response, and
have it apply to nothing. This is distinct from the per-vector
`__expires_at` path, which the TTL reaper now sweeps
(`phase1_ttl-reaper-never-spawned`): a collection TTL means "vectors in this
collection expire N seconds after they arrive", and nothing implements that.

Found while auditing the TTL feature for the reaper task.

## What Changes

Pick one and make the surface honest either way:

- **Implement it.** The natural shape is to stamp `__expires_at = now + ttl`
  on insert for collections that carry a `ttl:{name}` entry, so the existing
  reaper and the per-vector path do the rest. Requires the insert paths (REST,
  RPC, MCP, gRPC, batch) to consult the TTL, which is the same
  many-call-sites problem the reaper had — a single choke point in
  `VectorStore::insert` is preferable to hooking each handler.
- **Or remove it.** Drop the route and the metadata key, and let callers set
  per-vector expiries, which do work.

Either way the endpoint must stop reporting success for a no-op.

## Impact

- Affected specs: db/ttl
- Affected code: `crates/vectorizer-server/src/server/rest_handlers/collections.rs`,
  `crates/vectorizer/src/db/vector_store/`
- Breaking change: only if the route is removed — and it currently does
  nothing, so no working behaviour depends on it.
- User benefit: a configured collection TTL either takes effect or is not
  offered.
