## 1. Implementation

- [ ] 1.1 Remove unused `ClusterNode` import at `tests/integration/cluster_ha.rs:14`
- [ ] 1.2 Inline `id` into the format string at `tests/integration/cluster_ha.rs:318-321`
- [ ] 1.3 Inline `elapsed` into the format string at `tests/integration/cluster_ha.rs:734-738`
- [ ] 1.4 Run `cargo clippy --all-targets -- -D warnings` locally and confirm zero warnings
- [ ] 1.5 Run `cargo fmt --all -- --check` to confirm formatting stays clean

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
- [ ] 2.2 Write tests covering the new behavior
- [ ] 2.3 Run tests and confirm they pass
