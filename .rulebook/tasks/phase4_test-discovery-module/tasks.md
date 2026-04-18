## 1. Scaffolding

- [ ] 1.1 Create `tests/discovery/` with module file and `fixtures/` subdir
- [ ] 1.2 Add `proptest` as a dev-dependency in `Cargo.toml` if not already present

## 2. Test files

- [ ] 2.1 Write `tests/discovery/basics.rs` — happy path
- [ ] 2.2 Write `tests/discovery/edge_cases.rs` — symlinks, unreadable files, empty dirs
- [ ] 2.3 Write `tests/discovery/path_traversal.rs` — adversarial inputs (coordinates with `phase2_sanitize-discovery-paths`)
- [ ] 2.4 Write `tests/discovery/manifest_parse.rs` — YAML validity + proptest
- [ ] 2.5 Write `tests/discovery/concurrent.rs` — concurrent re-index + watcher events

## 3. Test seams (as needed)

- [ ] 3.1 Expose a `Discovery::builder()` or similar test-friendly constructor in `src/discovery/`
- [ ] 3.2 Parameterize base-dir and manifest path so tests don't require absolute paths

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Document discovery test strategy in `docs/development/testing.md`
- [ ] 4.2 Confirm new tests produce adequate coverage of `src/discovery/` (target ≥80%)
- [ ] 4.3 Run `cargo test --all-features -- discovery` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
