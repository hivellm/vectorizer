# Proposal: phase5_handle-whoami-major-bump

## Why

Dependabot PR #241 bumps `whoami` from 1.6.1 → 2.1.1 (major). The rust-tests CI job fails with:

``error[E0277]: `Result<String, whoami::Error>` doesn't implement `Display`.``

Root cause: whoami 2.x changed `realname()` (and related fns) to return `Result<String, whoami::Error>` instead of `String`. Our code formats the return value directly via `Display`, which now fails to compile.

Options:
- Merge as-is: blocked until we update our code.
- Ignore the major: stay on 1.6.1 indefinitely (fine, but Dependabot will keep re-opening).
- Update our code: adopt the new fallible API.

## What Changes

1. Identify every call site of `whoami::realname()`, `whoami::username()`, and similar APIs in `src/`.
2. Update each to handle the `Result`: either `unwrap_or_default()` for best-effort strings or propagate via `?` if we want strict behavior.
3. Accept PR #241 once CI passes after our changes.
4. If we decide to stay on 1.x: close the PR with `@dependabot ignore this major version` and document in CHANGELOG why.

## Impact

- Affected specs: none
- Affected code: any file using `whoami::*` (small, grep-able surface)
- Breaking change: NO (internal)
- User benefit: stay current on transitive-dep fixes; clear stance on a major bump instead of leaving it open.
