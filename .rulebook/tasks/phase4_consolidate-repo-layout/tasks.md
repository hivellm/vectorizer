## 1. Root cleanup

- [x] 1.1 Delete `coverage.lcov` (untracked, ~1.3 MB scratch). `final-test-output.txt` was already gone.
- [x] 1.2 `test-master-replica.ts` already absent at task start
- [x] 1.3 `git mv create_mcp_key.rs src/bin/create_mcp_key.rs`; rewrote against current `AuthManager` API (jwt_secret now wrapped in `Secret`, `Permission` enum, `expires_at: Option<u64>`, tuple return). Now reads `VECTORIZER_JWT_SECRET` env var. `cargo check --bin create_mcp_key` clean.
- [x] 1.4 `test/` directory already absent
- [x] 1.5 Merged `benchmark/` into `benches/` instead of deleting (user direction). All topic subdirs preserved under `benches/{comparison,core,embeddings,filter,gpu,grpc,performance,quantization,replication,scripts,search,storage,tests}/`. Loose files (`example_benchmark.rs`, `minimal_benchmark.rs`, `simple_test.rs`, `README.md`, `run_benchmarks.sh`, `benchmark_config.toml`, 10 `*.txt` scratch dumps, `reports/`) moved to `benches/`. Cargo.toml `path = "benchmark/..."` → `path = "benches/..."` (16 entries, 3 active + 13 commented). `scripts/dev/{run-benchmarks.ps1,update-benchmarks.sh}` and `docs/specs/BENCHMARKING.md` paths updated.

## 2. Config consolidation

- [ ] 2.1 Move `config.example.yml` from root into `config/config.example.yml`
- [ ] 2.2 Move `config.cluster.yml` into `config/modes/cluster.yml`
- [ ] 2.3 Move `config.hub.yml` into `config/modes/hub.yml`
- [ ] 2.4 Delete the root `config.production.yml` (already superseded by `config/modes/production.yml`)
- [ ] 2.5 Delete `config/config.production.yml` and `config/config.development.yml` (superseded by `config/modes/{production,dev}.yml`)
- [ ] 2.6 Move `config.yml` into `config/config.yml`; update bootstrap default path
- [ ] 2.7 Add a one-release compatibility shim: bootstrap warns once and reads `./config.yml` if it still exists, then exits the shim in v3.1
- [ ] 2.8 Decide on `config/config.windows.yml` and `config/workspace.docker.example.yml` placement; document under `config/README.md` if kept
- [ ] 2.9 Update `docs/deployment/configuration.md` and the README to point at the new paths
- [ ] 2.10 Add a CHANGELOG entry under `Changed` with the rewrite table for operators

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
