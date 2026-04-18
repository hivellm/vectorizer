## 1. Authorization

- [ ] 1.1 Confirm with the user that deletion of both `.bak` files is approved (per `AGENTS.md` Tier-1 rule #3)

## 2. Implementation

- [ ] 2.1 Delete `src/db/vector_store.rs.bak`
- [ ] 2.2 Delete `tests/integration/sharding_validation.rs.bak`
- [ ] 2.3 Add `*.bak` and `*.orig` entries to `.gitignore`
- [ ] 2.4 Add the same entries to `.dockerignore`
- [ ] 2.5 Optionally add a pre-commit hook in `.git/hooks/pre-commit` (or a script in `scripts/`) that rejects `*.bak` in staged files

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Note the cleanup in CHANGELOG under "Chore"
- [ ] 3.2 Run `cargo check --all-targets` to confirm nothing references the deleted files
- [ ] 3.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
