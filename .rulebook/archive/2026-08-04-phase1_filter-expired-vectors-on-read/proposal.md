# Proposal: phase1_filter-expired-vectors-on-read

## Why

`Payload::is_expired` has exactly one caller for vectors: the TTL reaper's
sweep. No read path consults it — `VectorStore::get_vector`, `search`,
`get_all_vectors` and the paginated list all return a vector whose
`__expires_at` is in the past.

With the reaper now running (`phase1_ttl-reaper-never-spawned`) the exposure is
bounded to one sweep interval — up to 60 s by default — but it is still a
window in which a caller who set an expiry gets the vector back, and in which
search results include documents the caller has already retired. Systems with
this feature normally pair active expiry with lazy expiry on read for exactly
this reason.

The window widens whenever a sweep is slow or the store is large, since the
sweep walks every vector of every collection.

## What Changes

- Filter on read: `get_vector` returns not-found for an expired vector, and
  the search / list paths drop expired hits.
- Decide whether a filtered read also deletes (lazy expiry) or leaves removal
  to the reaper. Deleting on read costs a write on the read path; leaving it
  keeps reads cheap and lets the sweep reclaim memory.
- Keep the cost off the hot path: the check is a payload field lookup per hit,
  so it belongs after the ANN search returns candidates, not inside the
  distance loop.
- Cover it: an expired vector must be absent from `get_vector`, from search
  results, and from the paginated list, without waiting for a sweep.

## Impact

- Affected specs: db/ttl
- Affected code: `crates/vectorizer/src/db/vector_store/`,
  `crates/vectorizer/src/db/collection/`
- Breaking change: NO — an expired vector is already meant to be gone.
- User benefit: an expiry takes effect immediately on reads instead of at the
  next sweep.
