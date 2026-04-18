## 1. Layout

- [ ] 1.1 Create `src/server/auth_handlers/` and move existing file to `mod.rs`.
- [ ] 1.2 Extract `types.rs` (request/response structs).
- [ ] 1.3 Extract `state.rs` (`AuthHandlerState` + impls).
- [ ] 1.4 Extract `public.rs` (login, validate_password_endpoint).
- [ ] 1.5 Extract `authenticated.rs` (me, logout, refresh, api-keys).
- [ ] 1.6 Extract `admin.rs` (user management handlers).
- [ ] 1.7 Extract `middleware.rs` (the five middleware + helper functions).
- [ ] 1.8 Extract `tests.rs` (existing `#[cfg(test)]` block).

## 2. Verification

- [ ] 2.1 `cargo check --all-features` clean.
- [ ] 2.2 Route wiring in `src/server/mod.rs` unchanged (public surface preserved via `pub use` in `mod.rs`).

## 3. Tail (mandatory)

- [ ] 3.1 Update module-level doc comment explaining the split.
- [ ] 3.2 Existing tests are sufficient; no new tests required.
- [ ] 3.3 `cargo test --all-features` pass.
