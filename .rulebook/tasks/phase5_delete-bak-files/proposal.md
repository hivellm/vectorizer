# Proposal: phase5_delete-bak-files

## Why

Two `.bak` files are committed to the repository:

- `src/db/vector_store.rs.bak` (~63 KB, ~1598 LOC — an older GPU collection variant)
- `tests/integration/sharding_validation.rs.bak`

Consequences:

- Confuses new contributors: which is the "real" file?
- IDE search returns results from the `.bak` file, leading to edits in the wrong place.
- Gets included in `cargo` source distribution (unless explicitly excluded).
- `.gitignore` does not block `*.bak`, so future backups will recur.

`AGENTS.md` Tier-1 rule #3 requires explicit user authorization to delete files, which is exactly the kind of confirmation this task provides — delete with authorization, not silently.

## What Changes

1. Get explicit user approval to delete `src/db/vector_store.rs.bak` and `tests/integration/sharding_validation.rs.bak`.
2. Delete both files.
3. Add `*.bak` and `*.orig` patterns to `.gitignore` and `.dockerignore`.
4. Optionally add a pre-commit hook refusing `*.bak` files in the staging area.

## Impact

- Affected specs: none
- Affected code: two file deletions, `.gitignore` update
- Breaking change: NO (files are not referenced by `cargo build`)
- User benefit: clean workspace; navigation/search returns no stale matches; future backups can't accidentally be committed.
