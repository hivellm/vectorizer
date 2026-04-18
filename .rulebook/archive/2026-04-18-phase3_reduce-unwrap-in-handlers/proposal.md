# Proposal: phase3_reduce-unwrap-in-handlers

## Why

Audit found **257 `.unwrap()` / `.expect()` calls in non-test `src/` code** and **257 `.ok()` chains** silencing errors. The highest concentration is in request-handling paths:

- `src/server/rest_handlers.rs` — 15 unsafe parses of UUIDs/floats/usizes via `.parse::<T>().ok()` then `.unwrap()`
- `src/db/vector_store.rs` — 6 `.ok()` on `store.get_collection()` swallowing auth/not-found errors silently
- `src/server/mod.rs` — 34 misc unwraps in bootstrap/routing

`AGENTS.md` forbids `unwrap/expect` in non-test code unless the invariant is obvious within 5 lines. Current state makes a malformed request path crash the server (DoS) instead of returning 400/404.

## What Changes

1. Categorize every `.unwrap()` / `.expect()` in `src/` into:
   - **Replace with `?`** and proper error propagation (most handlers)
   - **Replace with `map_or(default, ...)`** when a sane fallback exists
   - **Keep with justification** (invariant obvious from surrounding 5 lines) — add a `// SAFE:` comment explaining why
2. Replace `.ok()` on Results in handlers with explicit `.map_err(VectorizerError::from)?` so errors propagate to the HTTP layer.
3. Drop the `.parse::<T>().ok().unwrap()` anti-pattern: use `.parse::<T>().map_err(|e| VectorizerError::BadRequest(...))?`.
4. Add clippy `unwrap_used = "deny"` and `expect_used = "deny"` to `clippy.toml` with `#[allow]` on genuinely-safe test modules.

## Impact

- Affected specs: `/.rulebook/specs/RUST.md` (unwrap policy)
- Affected code: mainly `src/server/rest_handlers.rs`, `src/db/vector_store.rs`, `src/server/mod.rs`; scattered smaller sites
- Breaking change: NO behavior change except some paths now return 400/404 instead of 500/crash — strictly an improvement.
- User benefit: server can't be DoS'd by malformed input; errors surface as proper HTTP status codes; spec compliance.
