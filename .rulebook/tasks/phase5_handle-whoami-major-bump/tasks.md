## 1. Investigation

- [ ] 1.1 Grep `src/` for `whoami::` call sites; list each in `design.md`
- [ ] 1.2 Decide whether to upgrade or pin via `rulebook_decision_create`

## 2. Upgrade path (preferred)

- [ ] 2.1 Update each `whoami` call to handle the `Result<String, whoami::Error>` return type (prefer `unwrap_or_default()` for display strings, `?` where strictness matters)
- [ ] 2.2 Rebase PR #241 via `@dependabot rebase` and confirm CI passes
- [ ] 2.3 Squash-merge PR #241

## 3. Pin path (fallback)

- [ ] 3.1 Comment `@dependabot ignore this major version` on PR #241
- [ ] 3.2 Document the decision in CHANGELOG under "Chore/Deps"

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update any user-facing docs that reference the old behavior
- [ ] 4.2 Add a regression test exercising the whoami call path (even if trivial) so future major bumps catch breakage in CI
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
