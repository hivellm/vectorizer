## 1. Investigation

- [ ] 1.1 Run `grep -rn '#\[cfg(feature = "cluster")' src/` and enumerate every site in `design.md`
- [ ] 1.2 Decide Option A (define feature in Cargo.toml) vs Option B (delete gate) and record the decision via `rulebook_decision_create`

## 2. Implementation (Option A path)

- [ ] 2.1 Add `cluster = []` (or with optional deps) to `Cargo.toml [features]`
- [ ] 2.2 Move `openraft` / `openraft-memstore` under `[dependencies]` with `optional = true` if Option A chose to gate them
- [ ] 2.3 Run `cargo build --no-default-features --features cluster` and fix compile errors
- [ ] 2.4 Run `cargo build --features full` and confirm green
- [ ] 2.5 Add a CI matrix row in `.github/workflows/rust.yml` building with `--features cluster`

## 3. Implementation (Option B path — only if A rejected)

- [ ] 3.1 Delete every `#[cfg(feature = "cluster")]` wrapper, keeping the gated code
- [ ] 3.2 Delete every `#[cfg(not(feature = "cluster"))]` branch
- [ ] 3.3 Run `cargo check --all-targets` and fix any leftover references

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `README.md` / `docs/deployment.md` to document the cluster feature flag status
- [ ] 4.2 Add tests proving cluster behavior works under the chosen compilation path
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
