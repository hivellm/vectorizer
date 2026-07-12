## 1. Base image refresh (clears the 4 HIGH + openssl MED + 11 LOW)

- [ ] 1.1 Pull the current `dhi.io/debian-base:trixie` and confirm it carries `openssl 3.5.6-1~deb13u2` (`docker run --rm --entrypoint dpkg <img> -l openssl` or inspect via scout)
- [ ] 1.2 Record the new digest and pin it in `Dockerfile`: `FROM dhi.io/debian-base:trixie@sha256:<digest>` with a dated comment
- [ ] 1.3 Rebuild `local/vectorizer:3.5.0` (slim) + `:3.5.0-fastembed` and re-scan with `docker scout cves` — assert 0 CRITICAL, 0 HIGH, 0 fixable MEDIUM

## 2. Base variant evaluation (removes tar/coreutils/systemd CVE surface)

- [ ] 2.1 Enumerate available DHI trixie variants (`minimal` / `static` / `nonroot`) and diff their package lists against the current base
- [ ] 2.2 Test-build the runtime stage against the smallest variant that still satisfies: glibc + libssl3 + libstdc++ copy target dir + busybox HEALTHCHECK + `docker exec` shell for ops debugging
- [ ] 2.3 Record the go/no-go decision in `design.md` (if no-go, the VEX in §3 covers the residual packages; if go, re-run §1.3 scan on the new variant)

## 3. VEX for the not-fixable set

- [ ] 3.1 Author `docker/vex.json` (OpenVEX) covering: tar CVE-2025-45582 + CVE-2005-2541 (binary uses the compiled-in Rust `tar` crate, never shells out), glibc CVE-2019-9192/1010022-25/2018-20796/2010-4756 (regex/DoS in APIs the binary doesn't call via system glibc paths), systemd CVE-2023-31437/38/39 + CVE-2013-4392 (no systemd at runtime — direct-exec entrypoint), coreutils CVE-2025-5278 + CVE-2017-18018 (not invoked at runtime)
- [ ] 3.2 Attach the VEX to the pushed image (`docker scout attestation add` or sidecar in repo, per what the org's Scout plan supports) and verify Scout consumes it (dashboard drops the excepted findings)
- [ ] 3.3 Each VEX statement carries `justification` + `impact_statement` — no bare `not_affected` without reasoning

## 4. CI CVE gate + digest freshness

- [ ] 4.1 New `.github/workflows/docker-cve-gate.yml`: on release publish + weekly cron, run `docker scout cves hivehub/vectorizer:<latest-release> --exit-code --only-severity critical,high` (VEX applied); failure opens/updates a pinned issue
- [ ] 4.2 Same workflow: compare the Dockerfile's pinned base digest against the live `dhi.io/debian-base:trixie` tag; when stale ≥14 days, open/update a "base digest stale" issue with the new digest ready to paste
- [ ] 4.3 Document the digest-bump runbook in `docs/development/docker-builds.md` (who bumps, how to verify, how the gate reacts)

## 5. Release the patched images

- [ ] 5.1 Build + push `hivehub/vectorizer:3.5.0` and `:3.5.0-fastembed` (multi-arch, SBOM + provenance, buildx registry cache) from the pinned base
- [ ] 5.2 Post-push scan both tags; paste the Scout summary (expect 0C/0H/0 fixable-M) into the PR
- [ ] 5.3 Update `latest` tag; confirm Scout dashboard reflects the new digests

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
