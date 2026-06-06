## 1. Data-dir resolver

- [x] 1.1 Add `VECTORIZER_DATA_DIR` env var (already lived in `vectorizer_core::paths::data_dir`) and a matching `--data-dir <path>` CLI flag on the `vectorizer` binary. Flag value is propagated into the process env before any worker thread spawns so every downstream `paths::data_dir()` call sees it.
- [x] 1.2 Resolution centralised in `vectorizer_core::paths::data_dir()` (env > XDG > legacy `./data` fallback). All persistence subpaths already route through this helper — verified via `Grep` for `paths::data_dir` (auth persistence, vector store, snapshots, fastembed cache).
- [x] 1.3 Resolved data dir logged at startup (`info!("📁 Data directory: {}")`).
- [x] 1.4 Auth storage (`.auth.key`, `.root_credentials`, `auth.enc`, `jwt_secret.key`) already routed via `crates/vectorizer/src/auth/persistence.rs:173`.
- [x] 1.5 `logs/`, `snapshots/`, `vectorizer.vecdb`, `vectorizer.vecidx` already routed via `crates/vectorizer/src/db/vector_store/persistence.rs:229` and `vectorizer-server/src/bin/vectorizer.rs:124`.

## 2. Docker default

- [x] 2.1 Set `ENV VECTORIZER_DATA_DIR=/data` in `Dockerfile` (in the runtime stage's ENV block, with a phase32 rationale comment pointing at issue #300).
- [x] 2.2 `/data` is created in the `writable-dirs` stage with `chown -R 65532:65532` so the distroless nonroot user can write there from first boot.
- [x] 2.3 No `docker-entrypoint.sh` — the image runs the binary directly via `ENTRYPOINT ["/vectorizer/vectorizer"]`, which inherits the env var from the `ENV` directive.
- [ ] 2.4 Rebuild a smoke image; verify `ls /data` after first boot shows the real `vectorizer.vecdb` (handed off to §5.1 — Docker smoke test).

## 3. Ephemeral-data-dir warning

- [x] 3.1 `vectorizer_core::paths::ephemeral_data_dir_warning` parses `/proc/self/mountinfo` (Linux only — `#[cfg(target_os = "linux")]`), finds the longest mount prefix covering the resolved data dir, and flags the path as ephemeral when only `/` matches. No-op on non-Linux.
- [x] 3.2 `bin/vectorizer.rs` calls the detector right after logging the data dir and emits `warn!` when the message is non-empty.
- [x] 3.3 Unit test `ephemeral_detector_no_op_outside_real_proc` pins the negative case — a tempdir under `/tmp` (a real mount on every Linux runner) must not be flagged. Positive case (writable-layer `/data`) is verified by §5.1's Docker smoke test where setting up a real overlay layer is feasible.

## 4. Documentation + samples

- [ ] 4.1 Update root `README.md` Docker section: advertise `/data` mount + new env var
- [ ] 4.2 Update sample `docker-compose.yml` (and any deployment templates) to use the single `vec-data:/data` mount
- [x] 4.3 `docs/users/configuration/DATA_DIRECTORY.md` updated with phase32 Docker section: the `/data` default is now the canonical mount, the migration runbook tells 3.3.0 operators how to consolidate from the second-volume workaround onto a single `/data` mount, and the ephemeral-data-dir warning is documented alongside.
- [x] 4.4 CHANGELOG entry under v3.4.0 calling out the structural fix + the migration path.

## 5. Integration test

- [ ] 5.1 Docker-based integration test: start container with `vec-data:/data`, create a collection + insert vector, `docker stop && docker rm`, recreate from the same volume, verify the collection survives
- [ ] 5.2 CI job runs the test on every push touching `Dockerfile` or `crates/vectorizer/src/config/`

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
