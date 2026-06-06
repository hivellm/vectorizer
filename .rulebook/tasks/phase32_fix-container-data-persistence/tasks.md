## 1. Data-dir resolver

- [ ] 1.1 Add `VECTORIZER_DATA_DIR` env var and `--data-dir <path>` CLI flag to `vectorizer` + `vectorizer-cli`
- [ ] 1.2 Centralise resolution in a `data_dir()` helper (env > CLI > XDG > default), used by every persistence subpath
- [ ] 1.3 Log the resolved data dir at startup (`info!`)
- [ ] 1.4 Route auth storage (`.auth.key`, `.root_credentials`, `auth.enc`, `jwt_secret.key`) through the resolver
- [ ] 1.5 Route `logs/`, `snapshots/`, `vectorizer.vecdb`, `vectorizer.vecidx` through the resolver

## 2. Docker default

- [ ] 2.1 Set `ENV VECTORIZER_DATA_DIR=/data` in `Dockerfile`
- [ ] 2.2 Ensure `/data` is created with the runtime user's permissions in the image
- [ ] 2.3 Confirm `docker-entrypoint.sh` (if any) honours the env var
- [ ] 2.4 Rebuild a smoke image; verify `ls /data` after first boot shows the real `vectorizer.vecdb`

## 3. Ephemeral-data-dir warning

- [ ] 3.1 Detect when the resolved data dir is on the container's writable layer via `/proc/self/mountinfo` (Linux only; no-op elsewhere)
- [ ] 3.2 Emit `warn!("data dir at {} is ephemeral; recommend mounting a volume", path)` on startup when detected
- [ ] 3.3 Unit test the detector with synthetic mountinfo input

## 4. Documentation + samples

- [ ] 4.1 Update root `README.md` Docker section: advertise `/data` mount + new env var
- [ ] 4.2 Update sample `docker-compose.yml` (and any deployment templates) to use the single `vec-data:/data` mount
- [ ] 4.3 Add `docs/deployment/docker.md` (or update existing) with the mount-points table and migration steps for operators currently mounting `/.local/share/vectorizer`
- [ ] 4.4 Add a CHANGELOG entry under v3.4.0

## 5. Integration test

- [ ] 5.1 Docker-based integration test: start container with `vec-data:/data`, create a collection + insert vector, `docker stop && docker rm`, recreate from the same volume, verify the collection survives
- [ ] 5.2 CI job runs the test on every push touching `Dockerfile` or `crates/vectorizer/src/config/`

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
