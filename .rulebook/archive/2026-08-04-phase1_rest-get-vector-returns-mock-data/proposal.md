# Proposal: phase1_rest-get-vector-returns-mock-data

## Why

`rest_handlers::vectors::get_vector` — `GET /collections/{name}/vectors/{id}` —
never reads the vector. It checks that the collection exists and then returns a
fabricated body:

```rust
// Returns mock data — real retrieval by ID is tracked in a separate task
Ok(Json(json!({
    "id": vector_id,
    "vector": vec![0.1; 512],
    "metadata": { "collection": collection_name }
})))
```

A caller therefore gets `200 OK` with a 512-dimensional vector of `0.1` for any
id, including ids that do not exist and collections whose dimension is not 512.
That is worse than a 404: the response is indistinguishable from real data.

The same handler is also mounted on `POST /vector`
(`routing.rs`), where its `Path<(String, String)>` extractor cannot be
satisfied — that route has no path parameters at all, so the request fails
extraction rather than reaching the body.

The equivalent RPC command (`vectors.get`) is real and returns the stored
vector, which is how this was noticed: a verification script had to avoid the
REST endpoint because it can never report a vector as absent.

## What Changes

- Implement `get_vector` against `VectorStore::get_vector`, returning the
  stored data and payload, and `404` when the vector is absent — matching what
  `vectors.get` does over RPC.
- Fix or remove the `POST /vector` route, whose extractor can never match.
- Cover both: a present vector returns its real data, an absent id returns 404,
  and the dimension reflects the collection rather than a hardcoded 512.

## Impact

- Affected specs: api/rest
- Affected code: `crates/vectorizer-server/src/server/rest_handlers/vectors.rs`,
  `crates/vectorizer-server/src/server/core/routing.rs`
- Breaking change: NO in any useful sense — the current response is fabricated,
  so nothing correct can depend on it.
- User benefit: fetching a vector by id over REST returns the vector.
