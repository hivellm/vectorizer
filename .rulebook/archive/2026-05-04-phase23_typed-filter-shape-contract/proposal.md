# Proposal: phase23_typed-filter-shape-contract

## Why

The `delete_by_filter` and `bulk_update_metadata` endpoints (phase13
tier-control primitives) require a typed filter wire shape on the server,
but every SDK signature accepts an opaque `map<string, any>` /
`IDictionary<string, object>` / `dict[str, Any]` filter. Users receive
no compile-time hint about the actual contract, and the runtime error is
generic enough that wrong shapes are misdiagnosed.

Verified against `vectorizer:3.3.0` on 2026-05-04:

```
# Flat shape (most natural for users)
$ POST /collections/test_p20/vectors/delete_by_filter
  body: {"filter":{"topic":"index"}}
→ 400 {"error_type":"validation_error",
       "message":"Invalid filter: empty filter is not allowed; provide
                  at least one condition to prevent accidental wipes"}

# Qdrant-style shape
$ ... body: {"filter":{"must":[{"key":"topic","match":{"value":"index"}}]}}
→ 400 {"error_type":"validation_error",
       "message":"Invalid filter: invalid filter: missing field `type`"}
```

The server expects `{"filter":{"type":"eq","key":"topic","value":"index"}}`
or similar — the actual variant tags are not documented in
`docs/users/api/API_REFERENCE.md` nor in any SDK README. The error
"empty filter" is also misleading: it fires whenever the filter cannot
parse into a typed `Filter` enum, not just when literally empty.

## What Changes

Three independent fixes, packaged together because they share the contract:

1. **Document the filter wire shape** in
   `docs/users/api/API_REFERENCE.md` with a complete reference of every
   variant the server accepts (`eq`, `neq`, `in`, `range`, `and`, `or`,
   `not`, etc.) plus a JSON example per variant.
2. **Improve the server error message**: when the parse fails, return
   `parse_error` with the exact serde error path (e.g. "missing field
   `type` at filter[1].must[0]") instead of the misleading "empty filter".
3. **Add typed Filter helpers per SDK** so users get a discoverable
   builder API instead of typing raw maps:
   - Rust: already has `vectorizer::models::Filter` enum — re-export
     from `vectorizer-sdk` so SDK consumers can construct typed values.
   - TypeScript: add `Filter` discriminated union type in
     `sdks/typescript/src/models/filter.ts`.
   - Python: add `Filter` dataclass tree (typed-dict variants) in
     `sdks/python/models/filter.py`.
   - Go: add a `Filter` struct with builder helpers
     (`vectorizer.FilterEq("topic","index")`,
     `vectorizer.FilterAnd(...)`).
   - C#: add a `Filter` discriminated record tree in
     `sdks/csharp/Models/FilterModels.cs`.
4. The existing `map<string, any>` overloads stay (back-compat) but the
   doc comment recommends the typed builder.

## Impact

- Affected specs:
  - phase13 tier-control spec — already exists; this task is a docs +
    SDK ergonomics extension
- Affected code:
  - `crates/vectorizer-server/src/api/filters.rs` (or wherever the
    serde error is mapped to HTTP) — improve error message
  - `docs/users/api/API_REFERENCE.md` — full filter contract section
  - `sdks/{rust,typescript,python,go,csharp}/...` — typed filter helpers
- Breaking change: NO (additive — error message text is not part of the
  stable contract; raw map filters keep working)
- User benefit: Users discover the filter contract through their IDE's
  autocomplete instead of trial-and-error against the server. Error
  messages name the field that's wrong, not "empty filter".
