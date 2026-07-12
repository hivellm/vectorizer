# §5 — Operations & Developer Experience

> Scope: config loading (`crates/vectorizer/src/config/`),
> observability (`monitoring/`, logging), deployment (`Dockerfile`,
> `docker-compose.yml`, `k8s/`, `.github/workflows/`), dashboard
> (high level), `docs/` structure, release process.

## 5.1 Config sprawl

**Two competing `config.yml` schemas coexist.** Root `/config.yml`
(5 KB, generated, flat/legacy shape) has no `api`, `performance`,
`normalization`, or `backpressure` sections — yet
`config/modes/production.yml` overrides exactly those. The canonical
base is `config/config.yml` + `config/config.example.yml`; the root
file is a legacy shim.

Total surfaces: root `config.yml`, `config/config.yml`,
`config/config.example.yml`, 2 `config/modes/*`, 6
`config/presets/*`, `workspace.yml`, env vars (`VECTORIZER_MODE`,
`VECTORIZER_AUTH_ENABLED`, `VECTORIZER_JWT_SECRET`,
`VECTORIZER_AUTO_GEN_JWT_SECRET`, …), CLI flags — **~11 surfaces**.

**Validation gap (confirmed):** `layered.rs` module docs state
unknown keys are tolerated; serde uses `#[serde(default)]` without
`deny_unknown_fields`, so **config typos are silently ignored**.
Worse, `bootstrap.rs` bypasses the layered loader — it re-reads
`config.yml` with `serde_yaml::from_str(...).ok()` **four separate
times** (max_request_size `:1341`, auth `:1362`, hub `:1438`,
backpressure in `setup_handlers.rs:26`), each swallowing parse errors
into silent defaults. Mode overrides never reach auth/hub config.

**Committed pollution:** `workspace.yml` contains 7 throwaway
`E:/test/workspace-*` entries with 2026-04 timestamps.

## 5.2 Observability gaps

- Prometheus is real: `GET /metrics` (`meta.rs:302`); search has a
  per-collection latency histogram; per-collection insert counter
  exists (`insert.rs:642`). **Gap:** `insert_latency_seconds` is a
  single global histogram — no per-collection label
  (`metrics.rs:49`).
- **Traces are a stub.** `telemetry::try_init` (`telemetry.rs:45`)
  never creates a provider — logs "prepared but not enabled" and
  returns Ok. OTLP fully unwired despite the deps and a docstring
  implying export. Called at `bootstrap.rs:228` as a no-op.
- **Log noise:** `bootstrap.rs` emits ~50 emoji `info!` lines
  including a `🔍 STEP 1…7` narration. Suppressed in production
  mode, but pollutes the default/dev first-run.

## 5.3 Startup UX

Hard-fail on `0.0.0.0` without auth (`routing.rs:87-108`) is correct
security but **traps first-run**: default `config.yml` ships
`host: 0.0.0.0` + `auth.enabled: true` + `jwt_secret: ''`. The empty
secret makes `AuthManager::new` yield None (`bootstrap.rs:1424`),
collapsing to no-auth → boot rejected. The escape is the non-obvious
`--auto-generate-jwt-secret` / `VECTORIZER_AUTO_GEN_JWT_SECRET`. A
setup wizard exists (`setup_handlers.rs`) but the credential
bootstrap is not auto-wired to first boot.

## 5.4 CI health

16 workflows, ~61 jobs. **Dashboard is rebuilt 4× independently**
(`rust.yml`, `rust-lint.yml`, `rust-docs.yml`,
`rust-all-features.yml` each run `pnpm build` for rust-embed) — only
`release-artifacts.yml` shares via artifact.
**`continue-on-error: true` ×6** (5 in `rust.yml`, 1 in
`rust-all-features.yml`) — failures masked. `release-artifacts.yml`
is the slow path (9 jobs, multi-target cross-compile).

## 5.5 Dashboard debt

`vite.config.ts:90` `chunkSizeWarningLimit: 500` with only coarse
`manualChunks` (monaco/visx/vendor) — the vendor chunk trips the
warning (995 kB observed in CI builds). **27 `eslint-disable`**, 1
`@ts-*`, 1 `as any`, **26 TODO/FIXME/HACK** markers across
`dashboard/src`.

## 5.6 Docs structure

267 markdown files. **Duplicates:** `simd-benchmarks.md` and
`simd.md` each exist in `docs/architecture/` **and**
`docs/superpowers/plans/.../reference/uploads/`; ~22 duplicated
basenames (`MONITORING.md`, `REPLICATION.md`, `CLUSTER.md`,
`CONFIGURATION.md`, …) split across `specs/`, `users/`,
`architecture/`. `docs/superpowers/plans/` holds a single migration
plan, orphaned from any index.

## 5.7 Release process

`scripts/docker/*` is a mix of `.ps1`/`.sh` with no single
entrypoint. `release-artifacts.yml` triggers only on
`release: published` (manual draft-publish). **Workflow-scope
problem:** tokens without the GitHub `workflow` scope cannot merge
PRs touching `.github/workflows/` (bit this release twice: #334 and
the dependabot actions bumps) — undocumented in-repo.

## 5.8 Findings table

| Sev | Location | Description | Fix |
|-----|----------|-------------|-----|
| **HIGH** | `config/vectorizer.rs`, `layered.rs` | Config typos silently ignored (no `deny_unknown_fields`) | `deny_unknown_fields` or a merge-layer key allowlist |
| **HIGH** | `bootstrap.rs:1341-1458`, `setup_handlers.rs:26` | Config re-parsed 4× with `.ok()`, bypassing the layered loader + modes | Load once via `load_layered`; inject `VectorizerConfig` |
| **HIGH** | `routing.rs:87` + default `config.yml` | Default `0.0.0.0`+auth+empty secret → first boot fails | Default `host: 127.0.0.1`, or auto-gen secret on by default |
| **MED** | `telemetry.rs:45` | OTLP tracing is a no-op stub | Wire a real OTLP pipeline or delete the dep + docstring |
| **MED** | `metrics.rs:49` | `insert_latency_seconds` not per-collection | `HistogramVec` with a collection label |
| **MED** | `rust*.yml` | Dashboard built 4× | Build once, share via artifact/cache |
| **MED** | `rust.yml` | 6× `continue-on-error` masking failures | Remove or demote to non-required jobs |
| **MED** | root `config.yml` vs `config/config.yml` | Two divergent schemas | Delete the legacy root file |
| LOW | `bootstrap.rs` | ~50 emoji + `🔍 STEP` logs at info | Demote to `debug!` |
| LOW | `workspace.yml` | 7 committed `E:/test/*` entries | Remove; gitignore local workspace state |
| LOW | `docs/` | simd docs duplicated; ~22 dup basenames | Dedup; add a docs index/linter |
| LOW | `vite.config.ts:90` + `dashboard/src` | 995 kB vendor chunk; 27 eslint-disable; 26 TODO | Finer `manualChunks`; triage suppressions |
| LOW | `scripts/docker/*`, release flow | Manual publish; `workflow`-scope block undocumented | Document PAT scope; unify release script |

(→ phase40 for config/observability hardening items; docs/CI hygiene
items are follow-up chores)
