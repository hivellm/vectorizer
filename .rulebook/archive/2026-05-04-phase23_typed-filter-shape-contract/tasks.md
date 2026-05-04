## 1. Server-side error clarity

- [x] 1.1 Locate the filter serde-deserialize path in `crates/vectorizer-server/src/api/filters.rs` (or equivalent) and capture the underlying serde error
- [x] 1.2 Map the serde error to a structured response: `{"error_type":"parse_error","message":"<serde-error-with-path>","details":{"field":"filter","reason":"<concrete>"}}` instead of the generic "empty filter is not allowed"
- [x] 1.3 Keep the legacy `validation_error` for the genuinely-empty case (zero conditions) but rename the message to "filter has no conditions" for clarity

## 2. Documentation

- [x] 2.1 Add a `### Filter shape` section to `docs/users/api/API_REFERENCE.md` enumerating every variant the server accepts, with JSON example + matched semantics for each
- [x] 2.2 Add a "Common mistakes" subsection covering: flat shape (`{key:value}`), Qdrant-style shape (`{must:[...]}`), missing `type` field
- [x] 2.3 Cross-link the filter section from `delete_by_filter` and `bulk_update_metadata` rows in the existing tier-control table

## 3. Rust SDK typed filter

- [x] 3.1 Re-export `vectorizer::models::Filter` (or move it to a public location) from `sdks/rust/src/models/mod.rs`
- [x] 3.2 Update `delete_by_filter` and `bulk_update_metadata` doc comments in `sdks/rust/src/client/vectors.rs` to recommend the typed `Filter` value
- [x] 3.3 Add a unit test that round-trips a typed `Filter::Eq` / `Filter::And([...])` through the SDK against a hermetic mock server

## 4. TypeScript SDK typed filter

- [x] 4.1 Create `sdks/typescript/src/models/filter.ts` with `QdrantFilter` + `FilterCondition` + `FilterMatch` + `FilterRange` mirroring the Rust SDK wire shape
- [x] 4.2 Export a `filter` namespace with builder helpers (`filter.eq("topic","index")`, `filter.must(...)`, etc.)
- [x] 4.3 Update `deleteByFilter` / `bulkUpdateMetadata` signatures to accept `QdrantFilter | Record<string, unknown>`
- [x] 4.4 vitest unit tests — 20/20 passing

## 5. Python SDK typed filter

- [x] 5.1 Create `sdks/python/models/filter.py` with `Filter` dataclasses (`FilterEq`, `FilterAnd`, etc.) and a `to_dict()` serializer
- [x] 5.2 Update `delete_by_filter` / `bulk_update_metadata` to accept `Filter | dict`
- [x] 5.3 Add a pytest unit test for the typed path

## 6. Go SDK typed filter

- [x] 6.1 Add `sdks/go/filter.go` with a `Filter` struct + builder helpers (`FilterEq(k, v)`, `FilterAnd(filters ...Filter)`)
- [x] 6.2 Add an `EncodeJSON()` method that produces the typed wire shape
- [x] 6.3 Update `DeleteByFilter` / `BulkUpdateMetadata` to accept `Filter` (keep a `*Raw` variant for back-compat)
- [x] 6.4 Add a wire-shape test against `httptest.NewServer`

## 7. C# SDK typed filter

- [x] 7.1 Add `sdks/csharp/Models/FilterModels.cs` with a sealed-record discriminated union mirroring the wire shape
- [x] 7.2 Update `DeleteByFilterAsync` / `BulkUpdateMetadataAsync` overloads to accept `Filter`
- [x] 7.3 Add an xUnit test for the typed path

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 SDK CHANGELOGs (Rust, TS, Python, Go, C#) all carry a phase23 typed-Filter bullet under their `## [3.3.0]` entries
- [x] 8.2 Per-SDK test suites all green: Rust 8/8, TS 20/20, Python 17/17, Go 9/9, C# 10/10
- [x] 8.3 Run `cargo test -p vectorizer-server --lib error_middleware` and confirm pass (replaced the planned external integration test with inline unit tests; phase24/phase25 will add HTTP-level coverage)
- [x] 8.4 Update or create documentation covering the implementation
- [x] 8.5 Write tests covering the new behavior
- [x] 8.6 Run tests and confirm they pass
