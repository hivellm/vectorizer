# Spec: Docker image CVE posture

## ADDED Requirements

### Requirement: Published images MUST carry zero fixable HIGH/CRITICAL CVEs

Every published `hivehub/vectorizer` tag (slim and `-fastembed`)
SHALL scan clean of CRITICAL and HIGH severity CVEs for which the
distribution ships a fixed package version, as reported by
`docker scout cves`.

CVEs with no fixed version available MAY remain, but only when
covered by an OpenVEX statement with a written justification
(see the VEX requirement below).

#### Scenario: Release scan gate

Given a freshly pushed `hivehub/vectorizer:<version>` manifest
When `docker scout cves <tag> --exit-code --only-severity critical,high` runs with the repo VEX applied
Then the command exits 0
And the Scout dashboard shows no unexcepted CRITICAL/HIGH findings

### Requirement: Runtime base image MUST be pinned by digest

The `Dockerfile` runtime stage SHALL reference the base as
`dhi.io/debian-base:trixie@sha256:<digest>` — never by floating
tag alone. The pin line MUST carry a comment with the pin date so
staleness is reviewable in the file itself.

#### Scenario: Build reproducibility

Given two builds of the same git commit executed weeks apart
When both builds pull the runtime base
Then both resolve the identical base layers (same digest)
And any base change appears in git history as an explicit digest bump

#### Scenario: Stale digest surfaces automatically

Given the pinned digest is older than 14 days AND upstream
  `dhi.io/debian-base:trixie` has moved to a newer digest
When the weekly CVE-gate workflow runs
Then it opens (or updates) a tracking issue containing the new
  digest ready for a pin bump

### Requirement: Unfixable CVEs MUST be documented via OpenVEX

CVEs present in the image with no fixed package version in the
distribution SHALL be enumerated in `docker/vex.json` (OpenVEX).
Each statement MUST include `justification` and an
`impact_statement` explaining why the vectorizer runtime is not
affected (e.g. binary never execs system `tar`; no systemd at
runtime; coreutils not invoked by the direct-exec entrypoint).

Bare `not_affected` without reasoning is FORBIDDEN.

#### Scenario: Scanner consumes the VEX

Given `docker/vex.json` lists CVE-2025-45582 as `not_affected`
  with justification `vulnerable_code_not_in_execute_path`
When Docker Scout evaluates the image with the VEX attached
Then CVE-2025-45582 no longer appears as an open finding
And the VEX statement remains auditable in the repo history
