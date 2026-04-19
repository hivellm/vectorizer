# `scripts/` — operator and developer helpers

Small task-oriented scripts grouped by purpose. Anything that ships with
the binary (build / install) lives at the repo root or in [`build/`](build/);
anything that operates on the running service lives in [`service/`](service/);
quality / CI gates live in [`ci/`](ci/); and codemods used during the
phase4 sweeps live in [`codemods/`](codemods/).

## Top-level entry points

| File | Purpose |
|------|---------|
| [`install.sh`](install.sh) | One-line installer for Linux / macOS — clones the repo, installs Rust if missing, builds in release mode, drops the binary on `$PATH`. Documented under `docs/users/getting-started/`. |
| [`install.ps1`](install.ps1) | Windows PowerShell equivalent of `install.sh`. |

The installers are deliberately at the top level so the canonical
`curl … | bash` URL stays short:

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash

# Windows
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

## Subdirectories

### [`build/`](build/) — compile, link, package, release

| File | Purpose |
|------|---------|
| `build.sh` / `build.bat` | Standard release build wrappers. |
| `build-gpu.sh` | Release build with the `hive-gpu` feature. |
| `build-windows-safe.ps1` | BSOD-safe Windows build (limits parallelism, gates on driver state — see `docs/specs/GUARDRAILS_SUMMARY.md`). |
| `pre-build-check.ps1` | Windows pre-flight checks (memory, disk, GPU drivers, virtual memory, recent BSODs). |
| `install-lld.sh` | Install LLD on Linux / WSL for ~3× faster linking. |
| `setup-sccache.sh` / `setup-sccache.ps1` | Configure sccache compilation cache. |
| `release.sh` / `release.bat` | Build + tag + publish the release artefacts. |

### [`service/`](service/) — start / stop / restart / status

| File | Purpose |
|------|---------|
| `start.sh` / `start.bat` | Bring the server up with the standard config. |
| `stop.sh` / `stop.bat` | Graceful shutdown. |
| `restart.sh` | Stop-then-start. |
| `status.sh` / `status.bat` | Print health and PID information. |

### [`docker/`](docker/) — image build, push, verify

| File | Purpose |
|------|---------|
| `build.ps1` / `build-push.ps1` / `push.ps1` | Build / build-and-push / push the Docker image to the registry. |
| `hub-up.ps1` | Bring up the local Docker stack. |
| `pull-and-verify.sh` / `pull-and-verify.ps1` | Pull the published image and run smoke checks. |

### [`ci/`](ci/) — quality gates wired into `.github/workflows/`

| File | Purpose |
|------|---------|
| `check-no-tier1-markers.sh` | Fails CI on `TODO`/`FIXME`/`HACK`/`XXX` outside the `TASK(phaseN_<slug>)` allow-list. Wired into `rust-lint.yml`. |
| `check-no-credential-logs.sh` | Fails CI when `password`/`secret`/`api_key` appears in a log macro format string under `src/auth/` without an explicit allow-sentinel. Wired into `rust-lint.yml`. |
| `check-before-push.sh` | Local pre-push gate — runs the cheap diagnostic checks (clippy, fmt, type-check) before allowing a push. |
| `run-ci-local.sh` | Replays the CI matrix locally so a `git push` is not the first time CI runs. |

### [`cluster/`](cluster/) — multi-node integration helpers

| File | Purpose |
|------|---------|
| `simulate-cluster.sh` | Spin up an N-node cluster on `127.0.0.1` for manual testing. |
| `test-cluster.sh` | Drive a cluster through the standard read / write workload and assert convergence. |
| `test-local-cluster.sh` | Lighter-weight local-cluster integration smoke. |

### [`codemods/`](codemods/) — one-off codebase rewrites + audits

| File | Purpose |
|------|---------|
| `classify_unwraps.py` | Walks `src/` and splits every `.unwrap()` / `.expect(...)` site into test-only vs production (honouring file-level `#![allow]` and function-level `#[allow]`). Output is the input for the no-unwrap sweep. |
| `add_test_unwrap_allow.py` | Idempotent codemod that prepends `#[allow(clippy::unwrap_used, clippy::expect_used)]` to every `#[cfg(test)] mod <name> { ... }` block whose body contains a `.unwrap()` / `.unwrap_err()` / `.expect(...)` call. |
| `add_file_unwrap_allow.py` | File-level variant for `*_tests.rs` source files and every `.rs` file under `tests/`. |
| `design-unwrap-audit.txt` | Latest dump of `grep -rnE '\.unwrap\(\)\|\.expect\(' src/` — input for the classifier. Regenerable. |
| `design-ok-audit.txt` | Same dump for `.ok()` chains; companion to the unwrap audit (every hit was inspected and classified during phase4_enforce-no-unwrap-policy). |

### [`dev/`](dev/) — local-dev one-shots

| File | Purpose |
|------|---------|
| `test_routes.rs` | `cargo run --bin test_routes` — quick smoke-test of every REST route against `localhost:15002`. Wired as a binary in `Cargo.toml`. |
| `replace-println.ps1` | Historical helper from the `println!` → `tracing` migration. |
| `run-benchmarks.ps1` / `update-benchmarks.sh` | Run the benchmark harness and refresh the published numbers. |
| `fix-dimension-mismatch.sh` / `fix-gpu-collections.sh` | One-off data-fix helpers for known historical regressions. |
| `export.sh` | Dump live collections to portable JSON for backups / migration tests. |

## Conventions

- **Bash scripts** target `bash 4+` and POSIX coreutils; on Windows run them
  via Git Bash or WSL.
- **PowerShell scripts** target PowerShell 5.1+ (Windows-shipped) and 7.x
  (cross-platform).
- **Codemods** are Python 3.9+, pure stdlib — no `pip install` required.
- **Quality gates** in `ci/` are pinned by the CI workflow and never invoked
  by hand outside `check-before-push.sh`.
- Temporary one-shot scripts go in `dev/` and SHOULD be deleted after the
  one-shot is no longer needed (see project rule
  `temporary files must live in /scripts and be removed immediately after use`).
