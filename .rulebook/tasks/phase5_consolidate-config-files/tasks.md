## 1. Analysis

- [ ] 1.1 Diff all five config files; build a per-key matrix showing which values differ across modes
- [ ] 1.2 Decide the canonical default value for each key (document in `design.md`)

## 2. Implementation

- [ ] 2.1 Rewrite `config.example.yml` to hold all keys with canonical defaults and rich comments
- [ ] 2.2 Create `config/modes/production.yml`, `config/modes/cluster.yml`, `config/modes/hub.yml`, `config/modes/dev.yml` containing only delta values
- [ ] 2.3 Extend `src/config/loader.rs` (or equivalent) to support layering: base → mode → env → CLI
- [ ] 2.4 Delete the old `config.production.yml`, `config.cluster.yml`, `config.hub.yml`
- [ ] 2.5 Update `docker-compose.*.yml`, `helm/`, and `k8s/` references to the new layout

## 3. Migration

- [ ] 3.1 Write `scripts/migrate_config.sh` taking an old mode file and producing the new base + override split
- [ ] 3.2 Document migration steps in CHANGELOG under "Breaking"

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Publish `docs/deployment/configuration.md` explaining the layered model
- [ ] 4.2 Add tests loading each mode override and validating the merged config
- [ ] 4.3 Run `cargo test --all-features -- config` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
