# Proposal: phase35_fix-docker-image-cves

Source: Docker Scout scan of `hivehub/vectorizer:3.4.0-fastembed`
(digest `87098f2bebbd`, and the same base underlies `:3.4.0` /
`:latest`) — 31 vulnerabilities: 4 HIGH, 2 MEDIUM, 25 LOW.

## Why

Every one of the 31 CVEs comes from the runtime base image
(`dhi.io/debian-base:trixie` as pulled at build time in June 2026),
none from the Rust binary or the JS assets. Verified with
`docker scout cves hivehub/vectorizer:3.4.0-fastembed`:

| Package | Version in image | CVEs | Fix available? |
|---|---|---|---|
| openssl | 3.5.6-1~deb13u1+dhi2 | **4 HIGH** (CVE-2026-45447 8.8, CVE-2026-7383 8.1, CVE-2026-9076 7.5, CVE-2026-34180 7.5) + 1 MED (CVE-2026-42766) + 11 LOW | **Yes — 3.5.6-1~deb13u2** |
| tar | 1.35+dfsg-3.1+dhi1 | 1 MED (CVE-2025-45582 file-smuggling) + 1 LOW (CVE-2005-2541) | **No upstream fix** |
| glibc | 2.41-12+deb13u3+dhi1 | 7 LOW (2010–2019, mostly disputed) | No fix in trixie |
| systemd | 257.13-1~deb13u1+dhi1 | 4 LOW (2013/2023) | No fix in trixie |
| coreutils | 9.7-3+dhi3 | 2 LOW | No fix in trixie |

The image has been re-scanning red for a month. The openssl HIGHs
are the material risk: the vectorizer binary dynamically links
`libssl.so.3` from the base (reqwest/native-tls indirect deps), so
the vulnerable code IS on the hot path for outbound TLS
(HiveHub SDK, hf-hub model downloads, webhook calls).

Two structural problems keep this recurring:

1. **The base is pulled by floating tag** (`dhi.io/debian-base:trixie`),
   so "what's in the image" depends on the build date, and a CVE
   fixed upstream doesn't reach us until someone happens to rebuild.
2. **No CI gate fails the build on known-fixable HIGHs**, so red
   scans accumulate silently.

## What Changes

1. **Rebuild against the current DHI base.** DHI rebuilds weekly;
   the openssl `deb13u2` fix is in the current
   `dhi.io/debian-base:trixie`. A plain rebuild + push of `:3.4.x`
   and `:3.4.x-fastembed` clears 16 of 31 CVEs (all 4 HIGH, the
   openssl MED, 11 LOW).
2. **Pin the runtime base by digest** in the `Dockerfile`
   (`FROM dhi.io/debian-base:trixie@sha256:<digest>`) with a
   comment carrying the human-readable date. Reproducible builds;
   bumping the digest becomes an explicit, reviewable diff. A
   scheduled CI job (weekly) checks whether the pinned digest is
   stale against the upstream tag and opens an issue when it is.
3. **VEX / Scout exceptions for the not-fixable set.** tar
   CVE-2025-45582 + CVE-2005-2541, the 7 glibc lows, 4 systemd
   lows, 2 coreutils lows have no fixed version in trixie. Ship a
   `vex.json` (OpenVEX) alongside the image declaring
   `not_affected` / `under_investigation` with justification per
   CVE — the vectorizer binary does not shell out to system `tar`
   (it uses the Rust `tar` crate compiled in), does not run
   systemd, and does not invoke coreutils at runtime (distroless
   entrypoint). Scout consumes the VEX and the scan goes green
   without hiding real signal.
4. **CI CVE gate.** New scheduled + on-release workflow running
   `docker scout cves --exit-code --only-severity critical,high`
   against the pushed tags, with the VEX applied. New fixable
   HIGH/CRITICAL blocks release; the weekly schedule catches CVEs
   published between releases.
5. **Evaluate dropping the deb tar/coreutils surface.** DHI base
   ships bash + coreutils + tar for debuggability. Investigate the
   `dhi.io/debian-base:trixie-minimal` (or static) variant — if the
   HEALTHCHECK (busybox) and `docker exec` debugging still work,
   moving to the smaller variant removes the unfixable-CVE packages
   entirely instead of VEX-ing them. Decision recorded in
   design.md; if minimal breaks operational debugging, stay on the
   current base + VEX.

## Impact

- Affected specs: `specs/phase35_fix-docker-image-cves/`
- Affected code:
  - `Dockerfile` (digest pin, possibly base variant swap)
  - `.github/workflows/docker-image-smoke.yml` (extend) or new
    `docker-cve-gate.yml` (scout gate + stale-digest check)
  - `docker/vex.json` (new, OpenVEX)
  - `docs/development/docker-builds.md` (digest-bump runbook)
- Breaking change: NO. Runtime behaviour identical; images get
  rebuilt with patched openssl.
- User benefit:
  - The 4 HIGH openssl CVEs leave the TLS hot path immediately.
  - Scout dashboard goes green and STAYS green (gate + weekly
    schedule + digest pin).
  - Unfixable base CVEs are documented with explicit justification
    instead of sitting as permanent red noise that trains everyone
    to ignore the scanner.
