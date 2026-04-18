## 1. Split

- [x] 1.1 Extract `QueryRoot` + its resolvers into `src/api/graphql/query.rs`.
- [x] 1.2 Extract `MutationRoot` + its resolvers into `src/api/graphql/mutation.rs`.
- [x] 1.3 Leave `schema.rs` as the wiring file.

## 2. Dead code audit

- [x] 2.1 Review the former L1610-1698 block (now distributed across the new files).
- [x] 2.2 Remove or migrate any resolvers that no longer appear in the active schema.

## 3. Verification

- [x] 3.1 `cargo check --all-features` clean.
- [x] 3.2 Existing GraphQL tests + dashboard queries still work.
- [x] 3.3 The emitted schema text (via the dashboard introspection query) is unchanged.

## 4. Tail (mandatory)

- [x] 4.1 Update the GraphQL module doc comment.
- [x] 4.2 Add one schema-shape assertion test if introspection exposure allows it; otherwise rely on existing resolver tests.
- [x] 4.3 `cargo test --all-features` pass.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
