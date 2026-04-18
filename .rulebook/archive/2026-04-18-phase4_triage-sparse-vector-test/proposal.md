# Proposal: phase4_triage-sparse-vector-test

## Why

`tests/integration/sparse_vector.rs::test_sparse_vector_search` was ignored with a reason of "parallel-test collision on shared store". With the test muted there's no coverage on the sparse-vector search path at the integration level, which means a regression in sparse-vector handling would ship silently.

## What Changes

1. Identify what shared state causes the parallel collision (most likely a process-wide `VectorStore` or a fixed collection name that two tests both try to create).
2. Refactor the test to use a unique collection name per run and its own `VectorStore` instance, matching the pattern used by sibling integration tests.
3. Remove the `#[ignore]`.

## Impact

- Affected specs: sparse-vector / hybrid-search spec
- Affected code: `tests/integration/sparse_vector.rs`, possibly a small tempdir helper
- Breaking change: NO
- User benefit: ongoing regression coverage on sparse-vector search.
