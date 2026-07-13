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

- [ ] 5.1 Build + push `hivehub/vectorizer:3.5.0` and `:3.5.0-fastembed` (multi-arch, SBOM + provenance, buildx registry cache) from the pinned base
- [ ] 5.2 Post-push scan both tags; paste the Scout summary (expect 0C/0H/0 fixable-M) into the PR
- [ ] 5.3 Update `latest` tag; confirm Scout dashboard reflects the new digests

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Documentation: `docs/development/docker-builds.md` CVE-posture section (digest bump runbook + VEX + gate)
- [x] 6.2 Tests: `crates/vectorizer/tests/docker_vex.rs` — validates docker/vex.json structure (OpenVEX context, every statement has vulnerability name + status + justification + impact_statement, product purl present) and Dockerfile digest-pin format (pinned FROM + dated comment the freshness check parses)
- [x] 6.3 `cargo test --test docker_vex` — 2 passed, 0 failed

## 7. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 7.1 Update or create documentation covering the implementation
- [ ] 7.2 Write tests covering the new behavior
- [ ] 7.3 Run tests and confirm they pass
