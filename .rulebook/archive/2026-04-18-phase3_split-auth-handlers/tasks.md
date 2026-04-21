## 1. Layout

- [x] 1.1 Create `src/server/auth_handlers/` and move existing file to `mod.rs`.
- [x] 1.2 Extract `types.rs` (request/response structs).
- [x] 1.3 Extract `state.rs` (`AuthHandlerState` + impls).
- [x] 1.4 Extract `public.rs` (login, validate_password_endpoint).
- [x] 1.5 Extract `authenticated.rs` (me, logout, refresh, api-keys).
- [x] 1.6 Extract `admin.rs` (user management handlers).
- [x] 1.7 Extract `middleware.rs` (the five middleware + helper functions).
- [x] 1.8 Extract `tests.rs` (existing `#[cfg(test)]` block). — kept as `auth_handlers_tests.rs` wired via `#[path = "../auth_handlers_tests.rs"]` in `mod.rs` (carried over from the prior phase3 split).

## 2. Verification

- [x] 2.1 `cargo check --all-features` clean.
- [x] 2.2 Route wiring in `src/server/mod.rs` unchanged (public surface preserved via `pub use` in `mod.rs`).

## 3. Tail (mandatory)

- [x] 3.1 Update module-level doc comment explaining the split.
- [x] 3.2 Existing tests are sufficient; no new tests required.
- [x] 3.3 `cargo test --all-features` pass.
