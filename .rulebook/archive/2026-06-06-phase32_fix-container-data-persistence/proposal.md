# Proposal: phase32_fix-container-data-persistence

Source: [issue #300](https://github.com/hivellm/vectorizer/issues/300)

## Why

The `hivehub/vectorizer:3.3.0` image writes its persistent state
(collections, vectors, auth keys, JWT secret, snapshots) to the
XDG data dir `/.local/share/vectorizer/` even though the image
declares `WORKDIR /data` and the README + sample compose templates
advertise `/data` as the volume mount point. Every
`docker compose up -d --force-recreate vectorizer` wipes every
collection because the XDG path lives in the container's writable
layer, not on any persistent volume.

The trap was hit in production on 2026-05-27 on a `hivellm/cortex`
deployment after a routine restart: post-recreate coverage dropped
from full to **0/567 collections** with no error from Vectorizer
itself. The `/data` volume kept the `config.yml` and an empty
placeholder `vectorizer.vecdb`, masking the loss until search
queries returned nothing.

Today the only workaround is mounting a second volume on
`/.local/share/vectorizer`, which is undocumented and easy to miss.
This task closes the trap structurally so a single `volumes: -
vec-data:/data` mount is sufficient, and adds defence in depth via
a CLI/env override, a runtime warning when the data dir is on a
writable layer, and a documentation refresh.

## What Changes

1. **Configurable data dir** (`VECTORIZER_DATA_DIR` env var +
   `--data-dir` CLI flag) — resolves before falling back to the
   XDG default. Logged at startup. Applies to every persistent
   subpath (`.auth.key`, `.root_credentials`, `auth.enc`,
   `jwt_secret.key`, `logs/`, `snapshots/`, `vectorizer.vecdb`,
   `vectorizer.vecidx`).
2. **Docker image defaults `VECTORIZER_DATA_DIR=/data`** so the
   documented mount holds the real data without extra config.
   Existing deployments that explicitly mounted
   `/.local/share/vectorizer` keep working because the resolver
   still honours the env var when set.
3. **Startup warning** when the resolved data dir is on the
   container's writable layer (detected via `/proc/self/mountinfo`).
   Emits `WARN data dir at <path> is ephemeral; recommend mounting
   a volume` so the trap surfaces on first boot.
4. **Documentation refresh** — README's "Docker" section, the
   sample `docker-compose.yml`, and `docs/deployment/docker.md`
   advertise the `/data` mount and the new env var, and document
   the migration path for operators who mounted the XDG path.

## Impact

- Affected specs: `specs/phase32_fix-container-data-persistence/`
- Affected code:
  - `crates/vectorizer/src/config/` (data-dir resolver)
  - `crates/vectorizer-cli/src/main.rs` (`--data-dir`)
  - `crates/vectorizer-server/src/main.rs` (`--data-dir`)
  - `crates/vectorizer/src/persistence/` (path resolution)
  - `crates/vectorizer/src/auth/storage/` (key paths)
  - `Dockerfile` + `docker-compose.yml` samples
  - `docs/deployment/docker.md` (or equivalent)
- Breaking change: NO. Existing XDG path stays as fallback;
  operators who mounted `/.local/share/vectorizer` keep their data
  during the upgrade. The default for fresh containers shifts to
  `/data` only via env var in the image, not via code default.
- User benefit:
  - `docker compose up -d --force-recreate` no longer wipes data
    on a single-volume mount.
  - Bare-metal operators can pick any data dir via flag or env.
  - Misconfiguration surfaces at startup instead of silent loss.
