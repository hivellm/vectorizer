## 1. Base image refresh (clears the 4 HIGH + openssl MED + 11 LOW)

- [x] 1.1 Pulled current `dhi.io/debian-base:trixie` — SBOM confirms `openssl 3.5.6-1~deb13u2+dhi0` (dpkg absent in distroless base; used `docker scout sbom`)
- [x] 1.2 Digest `sha256:17dc256ec746f1168765cab1fc552418b60d09de8337d03ffa92cc529ed2ea7a` pinned in `Dockerfile` with dated comment + CVE list
- [x] 1.3 Rebuilt `local/vectorizer:3.5.0-scan` and rescanned: **0 CRITICAL, 0 HIGH, 1 MEDIUM (tar, not fixed upstream), 15 LOW** — 31→16; the only MEDIUM is unfixable and VEX-covered; gate command exits 0

## 2. Base variant evaluation (removes tar/coreutils/systemd CVE surface)

- [x] 2.1 Enumerated: only `trixie` + `trixie-dev` exist in the org's DHI catalog; no minimal/static/nonroot/slim variants (probed 9 tag/repo combinations)
- [x] 2.2 No smaller variant exists to test-build against; static base non-viable anyway (binary dynamically links libssl3/libstdc++/glibc)
- [x] 2.3 NO-GO decision recorded in `design.md` (D1) — stay on `debian-base:trixie` + VEX

## 3. VEX for the not-fixable set

- [x] 3.1 `docker/vex.json` (OpenVEX 0.2.0) authored: tar (2), glibc (7), systemd (4), coreutils (2) = 15 statements
- [x] 3.2 Verified Scout consumes it: requires `--vex-author 'HiveLLM Vectorizer maintainers'` (CLI default only trusts `<.*@docker.com>` — discovered by testing; recorded in design.md D3 + runbook); attestation-attach path documented as D3, gate uses `--vex-location` which works on any Scout plan
- [x] 3.3 Every statement carries `justification` + `impact_statement`

## 4. CI CVE gate + digest freshness

- [x] 4.1 `.github/workflows/docker-cve-gate.yml`: release publish + weekly cron + manual dispatch; scans slim and -fastembed tags with VEX applied; failure opens/updates `Docker CVE gate failing` issue
- [x] 4.2 `base-digest-freshness` job compares pinned digest vs live tag; stale ≥14 days → opens/updates `Base digest stale` issue with new digest
- [x] 4.3 Runbook added to `docs/development/docker-builds.md` § "CVE posture (phase35)": digest-bump procedure, VEX maintenance, gate behaviour

## 5. Release the patched images

- [x] 5.1 Validation build + push done as `hivehub/vectorizer:3.5.0-dev` (multi-arch amd64+arm64, SBOM + provenance, digest `c9e01af5...`, commit 615cb605) — user-authorized dev release to verify the CVE posture on the Hub. The final `:3.5.0` / `:3.5.0-fastembed` tags are produced by the repo's established release mechanism (`release-artifacts.yml` on GitHub release publish), which builds from the same pinned base
- [x] 5.2 Post-push scan of `:3.5.0-dev`: **0 CRITICAL, 0 HIGH**, 1 unfixable MEDIUM + 15 LOW all covered by the attached OpenVEX attestation (was 31 CVEs / 4 HIGH on 3.4.0); gate command exits 0; Hub dashboard dropped to 0 visible after the attestation (CVE-2010-0928 added when the deb13u2 openssl surfaced it)
- [x] 5.3 `latest` moves with the release publish per release-artifacts.yml; docker-cve-gate.yml scans the released tags weekly + on publish and tracks digest staleness

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation — `docs/development/docker-builds.md` CVE-posture section (digest bump runbook + VEX + gate)
- [x] 6.2 Write tests covering the new behavior — `crates/vectorizer/tests/docker_vex.rs` validates docker/vex.json structure (OpenVEX context, every statement has vulnerability name + status + justification + impact_statement, product purl) and the Dockerfile digest-pin format the freshness check parses
- [x] 6.3 Run tests and confirm they pass — `cargo test --test docker_vex` 2 passed, 0 failed
