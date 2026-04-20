## 1. Root cleanup

- [x] 1.1 Delete `coverage.lcov` (untracked, ~1.3 MB scratch). `final-test-output.txt` was already gone.
- [x] 1.2 `test-master-replica.ts` already absent at task start
- [x] 1.3 `git mv create_mcp_key.rs src/bin/create_mcp_key.rs`; rewrote against current `AuthManager` API (jwt_secret now wrapped in `Secret`, `Permission` enum, `expires_at: Option<u64>`, tuple return). Now reads `VECTORIZER_JWT_SECRET` env var. `cargo check --bin create_mcp_key` clean.
- [x] 1.4 `test/` directory already absent
- [x] 1.5 Merged `benchmark/` into `benches/` instead of deleting (user direction). All topic subdirs preserved under `benches/{comparison,core,embeddings,filter,gpu,grpc,performance,quantization,replication,scripts,search,storage,tests}/`. Loose files (`example_benchmark.rs`, `minimal_benchmark.rs`, `simple_test.rs`, `README.md`, `run_benchmarks.sh`, `benchmark_config.toml`, 10 `*.txt` scratch dumps, `reports/`) moved to `benches/`. Cargo.toml `path = "benchmark/..."` → `path = "benches/..."` (16 entries, 3 active + 13 commented). `scripts/dev/{run-benchmarks.ps1,update-benchmarks.sh}` and `docs/specs/BENCHMARKING.md` paths updated.

## 2. Config consolidation

- [x] 2.1 `git mv config.example.yml config/config.example.yml`
- [x] 2.2 `git mv config.cluster.yml config/presets/cluster.yml` (preserved as a full-file preset rather than treated as a layered override — the original is a complete 652L config, not a sparse delta)
- [x] 2.3 `git mv config.hub.yml config/presets/hub.yml`
- [x] 2.4 `git mv config.production.yml config/presets/production.yml` (preserved instead of deleted — the 180L file focuses on the replication topology and has no equivalent in `config/modes/production.yml`)
- [x] 2.5 `git mv config/config.production.yml config/presets/production-light.yml` and `git mv config/config.development.yml config/presets/development.yml` (preserved instead of deleted — same reason)
- [x] 2.6 `mv config.yml config/config.yml` (untracked, gitignored at root) + bootstrap's default flips to `config/config.yml`. CLI `--config` default also updated. `.gitignore` adds `config/config.yml` to keep the user's working file untracked.
- [x] 2.7 Compat shim added in `bootstrap.rs`: when neither `config/config.yml` nor a CLI override is present, the loader checks `./config.yml` and emits a one-shot deprecation `WARN`. Scheduled for removal in v3.1.
- [x] 2.8 Decided: `config/config.windows.yml` moved to `config/presets/windows.yml` (it's a full-file preset like the others, not a layered override). `config/workspace.docker.example.yml` stayed in `config/` (it's a workspace template, not a server config preset).
- [x] 2.9 Updated `docs/deployment/{configuration,PRODUCTION_GUIDE,docker-compose.production}.md`, `docs/specs/REPLICATION.md`, `docs/users/guides/HA_CLUSTER.md`. Layered loader's `<base_dir>/config/modes/` default also changed to `<base_dir>/modes/` so it resolves correctly under the new layout (rustdoc + `LayeredOptions` doc both updated). Loader header refs in `config/modes/{dev,production}.yml` repointed.
- [x] 2.10 CHANGELOG entry added under `### Changed` with the full path-rewrite table.

## 3. Docker consolidation

- [ ] 3.1 Move `Dockerfile.test` into `docker/Dockerfile.test`
- [ ] 3.2 Move `Dockerfile.artifacts` into `docker/Dockerfile.artifacts`
- [ ] 3.3 Update CI workflows that reference the moved Dockerfiles (search `.github/workflows/`)
- [ ] 3.4 Add the `default` profile to `docker-compose.yml` (no behaviour change)
- [ ] 3.5 Fold `docker-compose.dev.yml` into `docker-compose.yml` under `profiles: [dev]`
- [ ] 3.6 Fold `docker-compose.ha.yml` into `docker-compose.yml` under `profiles: [ha]`
- [ ] 3.7 Fold `docker-compose.hub.yml` into `docker-compose.yml` under `profiles: [hub]`
- [ ] 3.8 Delete the merged `docker-compose.{dev,ha,hub}.yml` files
- [ ] 3.9 Update `docs/deployment/docker.md` (or equivalent) and the README to teach `docker compose --profile <name> up`

## 4. Verification

- [ ] 4.1 `cargo build --release` succeeds with the new config-path default
- [ ] 4.2 `cargo test --lib` passes unchanged
- [ ] 4.3 `docker compose config` parses; `docker compose --profile dev config` and `--profile ha config` and `--profile hub config` each parse
- [ ] 4.4 `docker build -f docker/Dockerfile.test .` succeeds (smoke test of the moved test Dockerfile)
- [ ] 4.5 Manual: start the server with the moved `config/config.yml`, hit `/health`, confirm 200

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 README, `docs/deployment/configuration.md`, `docs/deployment/docker.md` reflect the new paths
- [ ] 5.2 Tests above (4.1–4.5) cover the new behaviour
- [ ] 5.3 Run the verification block in section 4 and confirm pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
