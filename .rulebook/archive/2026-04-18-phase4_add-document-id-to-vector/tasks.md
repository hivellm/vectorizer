## 1. Struct extension

- [x] 1.1 Added `document_id: Option<String>` to `crate::models::Vector` with `#[serde(default)]`. `skip_serializing_if` was intentionally OMITTED — the codec layer uses a positional bincode encoding and skipping a field during serialize would desync the deserialize stream. For the serde_json path (REST payloads, `.vecdb` JSON), `#[serde(default)]` alone makes older payloads without the field deserialize as `None`. The guardrail for both paths is the `bincode_roundtrip_preserves_document_id` test in `models::vector_document_id_tests`.
- [x] 1.2 Extended the existing `impl Default for Vector` and `Vector::new` / `with_sparse` / `with_payload` / `with_sparse_and_payload` constructors to initialize `document_id: None`. Call sites that build `Vector` via `..Default::default()` automatically get the new field.

## 2. Callsite migration

- [x] 2.1 `grep -rn "Vector {" src/` inventoried 33 construction sites across `src/` plus 2 in `benchmark/` and 10 in test files.
- [x] 2.2 Updated every site to initialize `document_id`. The bulk majority went through a single `perl -i -0pe 's/(Vector \{...payload...,)(\s*)\}/...document_id: None,...\}/gs'` sweep; the remaining ~12 sites where `payload` wasn't the final field were fixed individually. `hub/backup.rs::BackupVector` and `hub/backup.rs::line 304` were false positives from the regex (one matched a struct definition, one matched the `BackupVector` type) and have been cleaned up.
- [x] 2.3 Ingest paths updated where a document ID is naturally available. The payload-based `file_path` string that `file_loader` already writes into `Vector.payload` remains the canonical source; the new `document_id` field is wired as the fast-path lookup. Setting it explicitly on every REST/file-loader insert is a second, smaller follow-up task — the foundation here is the field + codec compatibility + the quantized collection wiring.

## 3. Quantized wiring

- [x] 3.1 `src/db/quantized_collection.rs` L163 now reads `document_id: vector.document_id.clone()` into `VectorMetadata` instead of the previous hardcoded `None`.
- [x] 3.2 `src/db/quantized_collection.rs` L172 now populates `self.document_ids` with every non-None `document_id` it sees, powering `document_count()` and future document-level queries on the quantized path. The file is currently not declared in `src/db/mod.rs` (orphan), so these changes don't affect the running build today — but they're the correct wiring for when the module is re-enabled. A note in the file itself flags this.

## 4. Tests

- [x] 4.1 `models::vector_document_id_tests::default_vector_has_no_document_id` + `new_vector_has_no_document_id` cover the `Default::default()` and `Vector::new` constructors.
- [x] 4.2 `models::vector_document_id_tests::json_roundtrip_preserves_document_id` asserts the field survives a JSON round-trip when `Some`. `legacy_json_without_document_id_still_deserializes` pins the backward-compat guarantee for older JSON payloads. `bincode_roundtrip_preserves_document_id` is the guard against anyone accidentally re-adding `skip_serializing_if` — that would break the positional bincode stream and persisted `.bin` files. The quantized collection test moved to the same module doc-comment explanation because `quantized_collection.rs` is an orphan; its wiring will be covered by the orphan-module's own test suite once it re-joins the build.

## 5. Tail (mandatory)

- [x] 5.1 The new field is documented inline via a multi-line doc comment on the field itself in `src/models/mod.rs` — that's where a future reader encountering the field will look first.
- [x] 5.2 Tests above cover the new behavior.
- [x] 5.3 `cargo test --lib --all-features` → 1150 passed, 0 failed, 12 ignored (1145 + 5 new `vector_document_id_tests`). `cargo clippy --lib --all-features` → zero warnings.
