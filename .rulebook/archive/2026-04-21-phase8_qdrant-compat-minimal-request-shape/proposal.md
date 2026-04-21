# Proposal: phase8_qdrant-compat-minimal-request-shape

## Why

`PUT /qdrant/collections/{name}` currently deserializes
`QdrantCreateCollectionRequest` with a hard-required
`config: QdrantCollectionConfig` that carries **9 mandatory fields**:

```
vectors, shard_number, replication_factor, write_consistency_factor,
on_disk_payload, distance, hnsw_config, optimizer_config, wal_config
```

Qdrant's own REST spec only requires `vectors`; every other field is
optional with a sensible default. Probe 3.6 of
`phase8_release-v3-runtime-verification` confirmed the mismatch:

```
$ curl -X PUT /qdrant/collections/qdrant_probe \
    -d '{"vectors":{"size":4,"distance":"Cosine"}}'
422 Unprocessable Entity
Failed to deserialize the JSON body into the target type: missing field `config`

$ curl -X PUT /qdrant/collections/qdrant_probe \
    -d '{"config":{"vectors":{"size":4,"distance":"Cosine"}}}'
422 Unprocessable Entity
Failed to deserialize: config: missing field `distance`
```

Any real Qdrant client (qdrant-client-python, qdrant-client-js, etc.)
that talks to the Vectorizer qdrant-compat endpoint immediately fails
at collection creation. The feature is nominally available but cannot
be exercised by the tools the adapter is meant to support.

Source: `docs/releases/v3.0.0-verification.md` probe 3.6.

## What Changes

Relax `QdrantCreateCollectionRequest` + `QdrantCollectionConfig` to
match Qdrant's actual REST spec:

1. `vectors: QdrantVectorsConfig` stays required (it already carries
   `{size, distance}` internally, so the top-level duplicate
   `distance` is redundant — remove it).
2. Every other field becomes `Option<>` / `#[serde(default)]`:
   `shard_number`, `replication_factor`, `write_consistency_factor`,
   `on_disk_payload`, `hnsw_config`, `optimizer_config`,
   `wal_config`, `quantization_config`.
3. The `PUT /qdrant/collections/{name}` handler fills defaults
   server-side when the optional blocks are absent (e.g.
   `shard_number: 1`, `replication_factor: 1`,
   `on_disk_payload: false`).
4. Flatten the top-level wrapper: accept both
   `{config: {vectors: ...}}` AND `{vectors: ...}` — Qdrant's own
   clients send the latter.
5. Sweep the sibling endpoints (`PUT /points`,
   `POST /points/search`, `POST /points/scroll`, aliases) for the
   same "extra mandatory fields vs Qdrant's spec" pattern and relax
   each to match.

## Impact

- Affected specs: `docs/users/api/QDRANT.md` (if present) — update
  the request-shape block so it matches upstream Qdrant docs;
  `CHANGELOG.md > 3.0.0 > Fixed` entry explaining the shape
  relaxation.
- Affected code:
  - `crates/vectorizer/src/models/qdrant/collection.rs`
    (`QdrantCreateCollectionRequest`, `QdrantCollectionConfig`,
    `QdrantUpdateCollectionRequest`, possibly others).
  - `crates/vectorizer-server/src/server/qdrant/handlers.rs`
    (default-fill logic).
  - Possibly `crates/vectorizer/src/models/qdrant/points.rs`,
    `.../search.rs`, `.../aliases.rs` if the same pattern appears
    there.
- Breaking change: NO (relaxation; current callers that send the
  full request still work — Vectorizer just stops rejecting the
  smaller Qdrant-native requests).
- User benefit: real Qdrant clients talk to the Vectorizer
  qdrant-compat endpoint without modification. Unblocks probe 3.6.
