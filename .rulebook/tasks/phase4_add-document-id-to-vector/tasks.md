## 1. Struct extension

- [ ] 1.1 Add `document_id: Option<String>` to `crate::models::Vector` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- [ ] 1.2 Add a `Default` impl (or extend the existing one) so `..Default::default()` fills the field.

## 2. Callsite migration

- [ ] 2.1 Run `grep -rn "Vector {" src/` to produce the callsite list.
- [ ] 2.2 Update every construction site to initialize `document_id`.
- [ ] 2.3 Populate `document_id` at ingest time in `src/file_loader/`, `src/discovery/`, and REST insert handlers.

## 3. Quantized wiring

- [ ] 3.1 `src/db/quantized_collection.rs` L163 — set `document_id: vector.document_id.clone()`.
- [ ] 3.2 `src/db/quantized_collection.rs` L172 — populate the `document_ids` index.

## 4. Tests

- [ ] 4.1 Round-trip test: `Vector { ..Default::default() }` without `document_id` serializes without the field.
- [ ] 4.2 Quantized collection test: insert two vectors with the same `document_id`; assert the `document_ids` index contains it once.

## 5. Tail (mandatory)

- [ ] 5.1 Update API docs with the new optional field.
- [ ] 5.2 Tests above cover the new behavior.
- [ ] 5.3 Run `cargo test --all-features` and confirm pass.
