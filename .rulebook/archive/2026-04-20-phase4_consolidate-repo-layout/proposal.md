# Proposal: phase4_consolidate-repo-layout

## Why

The Vectorizer root has **27 items** vs ~10–12 in sibling HiveLLM repos (Synap, Nexus). The disorder shows up in three places:

1. **Loose root files** that don't belong: `coverage.lcov` (1.3 MB committed), `final-test-output.txt`, `test-master-replica.ts`, `create_mcp_key.rs` (a `.rs` script at root), and a `test/` directory next to the real `tests/`. `benchmark/` exists alongside `benches/` with no clear owner.

2. **Config duplication at two levels**: 5 `config*.yml` at the root *and* 4 more inside `config/` — `config.production.yml` is shadowed in both. The new layered loader from `phase5_consolidate-config-files` (`config/modes/<mode>.yml`) already exists but the legacy root files were left behind for the migration window. Time to finish the cutover.

3. **Docker proliferation**: 3 `Dockerfile*` + 4 `docker-compose*.yml` at the root, plus 2 musl Dockerfiles in `docker/`. Synap and Nexus each ship **one** `Dockerfile` + **one** `docker-compose.yml`, with environment variants expressed as compose `profiles`.

The HiveLLM convention (verified across Synap and Nexus): `config/` directory holds every YAML, exactly one Dockerfile + one compose at root, no scratch files at root, and clear single-purpose subdirs (`docs/`, `scripts/`, `helm/`, `sdks/`, `tests/`, `benches/`).

## What Changes

Three independent cleanups bundled because they're all root-level housekeeping with the same blast radius (touch the README and Docker docs, no Rust code paths):

### 1. Root cleanup

- Delete `coverage.lcov` and `final-test-output.txt`.
- Move `test-master-replica.ts` into `scripts/` (or delete if dead).
- Move `create_mcp_key.rs` into `src/bin/` so it's a real `cargo run --bin create_mcp_key` target instead of a stray script.
- Fold `test/file-upload.test.ts` (the only file in `test/`) into `tests/`. Delete the empty `test/` dir.
- Resolve `benchmark/` vs `benches/`: keep `benches/` (Cargo convention), move anything live from `benchmark/` into it, delete the rest.

### 2. Config consolidation

Adopt the Nexus pattern: `config/` is the only home for YAML.

- Move `config.example.yml`, `config.cluster.yml`, `config.hub.yml` from root into `config/modes/` as `cluster.yml`, `hub.yml` (the layered loader already merges them on top of base).
- Delete the root `config.production.yml` (superseded by `config/modes/production.yml`).
- Delete `config/config.production.yml` and `config/config.development.yml` (superseded by `config/modes/{production,dev}.yml`).
- Move `config.yml` into `config/config.yml`. Update bootstrap defaults to read from the new path; keep a thin compatibility shim that warns once and reads `./config.yml` for one release if it still exists, then drop it in v3.1.
- `config/workspace.docker.example.yml` and `config/config.windows.yml` move to `config/modes/windows.yml` if applicable, otherwise stay where they are.
- Update `docs/deployment/configuration.md` and the README to point at the new paths. Add a one-line migration note for operators on the legacy file path.

### 3. Docker consolidation

Adopt the Synap/Nexus pattern: one `Dockerfile` + one `docker-compose.yml` at root, environment-specific behaviour via compose `profiles`.

- Keep `Dockerfile` at root (production build).
- Move `Dockerfile.test` and `Dockerfile.artifacts` into `docker/` (the musl Dockerfiles already live there).
- Collapse `docker-compose.yml`, `docker-compose.dev.yml`, `docker-compose.ha.yml`, `docker-compose.hub.yml` into a single root `docker-compose.yml` using compose v2 `profiles`:
  - `default` — current `docker-compose.yml` services
  - `dev` — adds dev overrides
  - `ha` — adds the HA cluster topology
  - `hub` — adds the Hub services
- Update `docs/deployment/docker.md` (or equivalent) and the README to teach `docker compose --profile <name> up`.

## Impact

- Affected specs: none (organisational only)
- Affected code: zero `src/` changes beyond bootstrap's config-path default. Only file moves, README, deployment docs.
- Breaking change: YES for operators — anyone with scripts pointing at root-level `config*.yml` or `docker-compose.{dev,ha,hub}.yml` paths needs to migrate. The rewrite is mechanical (one path per file). CHANGELOG entry under `Changed` documents the new paths and provides the rewrite table. The compatibility shim covers the most common case (`./config.yml`) for one release.
- User benefit: root drops from 27 items to ~13, matching Synap/Nexus. Operators stop guessing which `config.production.yml` is canonical or which `docker-compose.*.yml` to invoke. New contributors get a familiar HiveLLM layout.

## Out of scope

Cargo workspace split (`vectorizer-core`, `vectorizer-server`, `vectorizer-cli`, `vectorizer-protocol`, `sdks/rust/` as workspace member). That's a much bigger refactor (~1–2 weeks, breaks every `use crate::X`) and belongs in its own task (`phase4_split-vectorizer-workspace`) if/when scheduled.

## Reference

Cross-checked against:
- `e:/HiveLLM/Synap/` — workspace + flat root configs + 1 Dockerfile + 1 compose
- `e:/HiveLLM/Nexus/` — workspace + `config/` subdir + 1 Dockerfile + 1 compose

Both converge on the same pattern; Vectorizer is the outlier.
