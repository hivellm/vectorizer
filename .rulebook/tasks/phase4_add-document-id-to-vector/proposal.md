# Proposal: phase4_add-document-id-to-vector

## Why

`src/db/quantized_collection.rs` at L163 and L172 records `document_id: None` with a marker noting the `crate::models::Vector` struct has no `document_id` field. The quantized collection already has a `document_ids` index ready to be populated but cannot be because the data isn't on the `Vector` struct.

Without a first-class `document_id`, the quantized path cannot do document-level operations (listing vectors per document, deleting-by-document, reverse lookups). The current workaround — encoding document IDs inside `Vector.payload.data` JSON — works for the unquantized path but can't be extracted cheaply in the quantized hot path.

## What Changes

1. Add `document_id: Option<String>` as a new field on `crate::models::Vector` in `src/models/mod.rs`, with `#[serde(default, skip_serializing_if = "Option::is_none")]` so existing serialized payloads still deserialize.
2. Update all ~50 construction sites across the codebase to either initialize `document_id: None` explicitly or use `..Default::default()` where available.
3. Wire `quantized_collection.rs` to populate `VectorMetadata::document_id` from `vector.document_id` and populate the `document_ids` index.
4. Audit the ingest paths (file_loader, discovery, REST insert endpoints) to set `document_id` when a vector belongs to a known document.

## Impact

- Affected specs: `/.rulebook/specs/` — any doc mentioning the `Vector` struct shape.
- Affected code: `src/models/mod.rs` plus every file that constructs `Vector` (run `grep -rn "Vector {" src/`).
- Breaking change: NO — the new field is optional and defaults to `None`. Serialization stays compatible.
- User benefit: the quantized path gains document-level operations parity with the dense path.
