# Spec: container data persistence

## ADDED Requirements

### Requirement: Configurable data directory

The system SHALL resolve its persistent data directory in the
following precedence order: (1) `--data-dir <path>` CLI flag,
(2) `VECTORIZER_DATA_DIR` environment variable, (3) XDG data home
(`$XDG_DATA_HOME/vectorizer` or `~/.local/share/vectorizer`).

All persistent files (`.auth.key`, `.root_credentials`, `auth.enc`,
`jwt_secret.key`, `logs/`, `snapshots/`, `vectorizer.vecdb`,
`vectorizer.vecidx`) MUST live under the resolved directory.

The resolved path MUST be logged at startup at `info` level so
operators can confirm where state will land.

#### Scenario: CLI flag wins over env var

Given `VECTORIZER_DATA_DIR=/var/lib/vz` is set
And the binary is started with `--data-dir /srv/vz`
When the server resolves its data directory
Then the resolved directory is `/srv/vz`
And `/srv/vz/vectorizer.vecdb` is the on-disk store

#### Scenario: env var wins over XDG default

Given `VECTORIZER_DATA_DIR=/data` is set
And no `--data-dir` flag is passed
When the server resolves its data directory
Then the resolved directory is `/data`
And `/data/.auth.key`, `/data/vectorizer.vecdb`, `/data/snapshots/`
are the canonical paths

#### Scenario: XDG fallback when no override is provided

Given neither `--data-dir` nor `VECTORIZER_DATA_DIR` is set
When the server resolves its data directory
Then the resolved directory is `$XDG_DATA_HOME/vectorizer`
Or `$HOME/.local/share/vectorizer` if `XDG_DATA_HOME` is unset

### Requirement: Container image defaults to `/data`

The official `hivehub/vectorizer` Docker image MUST set
`ENV VECTORIZER_DATA_DIR=/data` so that the documented
`/data` volume mount holds the real persistent state without any
extra configuration.

The image MUST create `/data` at build time with the runtime
user's permissions so a fresh container boot does not fail on
file creation.

#### Scenario: Single-volume mount survives recreate

Given a Docker container started with `--volume vec-data:/data`
And a collection `c1` was created with vectors inserted
When the container is stopped, removed, and recreated from the
   same volume
Then GET `/collections` MUST return `c1`
And vector search against `c1` returns the inserted vectors

### Requirement: Warn on ephemeral data directory

On Linux, when the resolved data directory has no underlying
mount (i.e., it lives on the container's writable layer per
`/proc/self/mountinfo`), the system SHALL emit a `warn!`-level
log at startup of the form `data dir at <path> is ephemeral;
recommend mounting a volume`.

On non-Linux hosts and when `/proc/self/mountinfo` is not
available, the detector MUST be a no-op (no false-positive
warnings on bare-metal Windows / macOS).

#### Scenario: Container with no /data mount triggers a warning

Given a container where `/data` is on the writable layer (no
   bind mount or named volume)
And `VECTORIZER_DATA_DIR=/data`
When the server starts
Then a warning `data dir at /data is ephemeral; recommend
   mounting a volume` MUST appear in the startup log

#### Scenario: Properly-mounted volume does not warn

Given a container started with `--volume vec-data:/data`
And `VECTORIZER_DATA_DIR=/data`
When the server starts
Then no ephemeral-data-dir warning is emitted
