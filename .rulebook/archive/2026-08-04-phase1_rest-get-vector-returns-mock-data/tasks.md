## 1. Implementation

- [x] 1.1 Implement `get_vector` against `VectorStore::get_vector`, returning the stored data and payload
- [x] 1.2 Give `POST /vector` a body-based handler so its declared registry route works (`{collection, vector_id}`, `id` accepted too)
- [x] 1.3 Return 404 for an absent vector instead of a fabricated body

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [x] 2.1 Update or create documentation covering the implementation — module header lists the new route; CHANGELOG entry
- [x] 2.2 Write tests covering the new behavior — `tests/rest_get_vector.rs`: real data on both routes, payload round-trip, 404 for an unknown id, body validation
- [x] 2.3 Run tests and confirm they pass — 4/4 in `rest_get_vector`, full server suite green
