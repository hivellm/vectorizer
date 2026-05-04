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

- [ ] 4.1 Create `sdks/typescript/src/models/filter.ts` with a discriminated union `type Filter = EqFilter | InFilter | AndFilter | ...`
- [ ] 4.2 Export a `filter` namespace with builder helpers (`filter.eq("topic","index")`, `filter.and(...)`)
- [ ] 4.3 Update `deleteByFilter` / `bulkUpdateMetadata` signatures to accept `Filter | Record<string, unknown>`
- [ ] 4.4 Add a vitest unit test exercising the typed path

## 5. Python SDK typed filter

- [ ] 5.1 Create `sdks/python/models/filter.py` with `Filter` dataclasses (`FilterEq`, `FilterAnd`, etc.) and a `to_dict()` serializer
- [ ] 5.2 Update `delete_by_filter` / `bulk_update_metadata` to accept `Filter | dict`
- [ ] 5.3 Add a pytest unit test for the typed path

## 6. Go SDK typed filter

- [ ] 6.1 Add `sdks/go/filter.go` with a `Filter` struct + builder helpers (`FilterEq(k, v)`, `FilterAnd(filters ...Filter)`)
- [ ] 6.2 Add an `EncodeJSON()` method that produces the typed wire shape
- [ ] 6.3 Update `DeleteByFilter` / `BulkUpdateMetadata` to accept `Filter` (keep a `*Raw` variant for back-compat)
- [ ] 6.4 Add a wire-shape test against `httptest.NewServer`

## 7. C# SDK typed filter

- [ ] 7.1 Add `sdks/csharp/Models/FilterModels.cs` with a sealed-record discriminated union mirroring the wire shape
- [ ] 7.2 Update `DeleteByFilterAsync` / `BulkUpdateMetadataAsync` overloads to accept `Filter`
- [ ] 7.3 Add an xUnit test for the typed path

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update each SDK's CHANGELOG with the typed filter addition under [Unreleased] (or a new patch version after this task lands)
- [ ] 8.2 Run all SDK test suites and confirm green
- [x] 8.3 Run `cargo test -p vectorizer-server --lib error_middleware` and confirm pass (replaced the planned external integration test with inline unit tests in `error_middleware.rs` because the in-process server bootstrap proved heavier than the value of the test — phase24 / phase25 will add HTTP-level coverage when they wire CSRF echo + new metric endpoints)
