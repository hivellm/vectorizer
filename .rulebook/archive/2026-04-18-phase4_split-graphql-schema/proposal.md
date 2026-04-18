# Proposal: phase4_split-graphql-schema

## Why

`src/api/graphql/schema.rs` is **1,698 lines** mixing `QueryRoot`, `MutationRoot`, and (per the original audit) a ~90-line tail at L1610-1698 that looks like dead code from an earlier schema iteration. The two root types have no shared code beyond the async-graphql trait machinery; keeping them together is pure accretion.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

1. Split into `src/api/graphql/query.rs` (`QueryRoot` + resolvers) and `mutation.rs` (`MutationRoot` + resolvers).
2. Keep `src/api/graphql/schema.rs` as a thin file that wires the two roots into a `Schema<Query, Mutation, _>` builder.
3. Audit L1610-1698 of the pre-split file for dead code; delete what is unreachable.

## Impact

- Affected specs: none.
- Affected code: `src/api/graphql/schema.rs`, new `src/api/graphql/{query,mutation}.rs`.
- Breaking change: NO — the generated GraphQL schema string is identical (same roots, same resolvers, same type graph).
- User benefit: GraphQL contributors can focus on reads or writes without scrolling past the other; dead code gets actively pruned.
