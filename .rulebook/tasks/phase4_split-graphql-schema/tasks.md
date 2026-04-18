## 1. Split

- [ ] 1.1 Extract `QueryRoot` + its resolvers into `src/api/graphql/query.rs`.
- [ ] 1.2 Extract `MutationRoot` + its resolvers into `src/api/graphql/mutation.rs`.
- [ ] 1.3 Leave `schema.rs` as the wiring file.

## 2. Dead code audit

- [ ] 2.1 Review the former L1610-1698 block (now distributed across the new files).
- [ ] 2.2 Remove or migrate any resolvers that no longer appear in the active schema.

## 3. Verification

- [ ] 3.1 `cargo check --all-features` clean.
- [ ] 3.2 Existing GraphQL tests + dashboard queries still work.
- [ ] 3.3 The emitted schema text (via the dashboard introspection query) is unchanged.

## 4. Tail (mandatory)

- [ ] 4.1 Update the GraphQL module doc comment.
- [ ] 4.2 Add one schema-shape assertion test if introspection exposure allows it; otherwise rely on existing resolver tests.
- [ ] 4.3 `cargo test --all-features` pass.
