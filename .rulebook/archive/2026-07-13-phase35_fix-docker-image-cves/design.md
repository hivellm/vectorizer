# Design notes — phase35_fix-docker-image-cves

## D1 — Base variant evaluation (tasks §2): NO-GO, stay on `debian-base:trixie` + VEX

Probed 2026-07-11 against the org's DHI registry:

| Ref | Result |
|---|---|
| `dhi.io/debian-base:trixie` | EXISTS (current base) |
| `dhi.io/debian-base:trixie-dev` | EXISTS (dev/build variant — larger, wrong direction) |
| `dhi.io/debian-base:trixie-minimal` | ABSENT |
| `dhi.io/debian-base:trixie-static` / `static-trixie` | ABSENT |
| `dhi.io/debian-base:trixie-nonroot` / `trixie-slim` | ABSENT |
| `dhi.io/static:*`, `dhi.io/glibc-base:trixie`, `dhi.io/base:trixie` | ABSENT |

Conclusion: the DHI catalog available to this org ships exactly two
debian-base tags — the runtime base we already use and a `-dev`
variant. There is **no smaller glibc variant** to move to, and a
fully-static base is not viable anyway: the vectorizer binary links
`libssl.so.3`, `libstdc++`, and glibc dynamically (native-tls +
onnxruntime), so removing the C runtime would require a musl static
relink — out of scope for a CVE-hygiene task.

Consequence: the unfixable tar/glibc/systemd/coreutils CVE surface
stays in the image and is handled via `docker/vex.json` (tasks §3),
as the proposal's fallback path prescribed.

## D2 — Digest pin

Pinned `dhi.io/debian-base:trixie@sha256:17dc256ec746f1168765cab1fc552418b60d09de8337d03ffa92cc529ed2ea7a`
(pulled 2026-07-11). SBOM confirms `openssl 3.5.6-1~deb13u2+dhi0` —
the deb13u2 build that fixes CVE-2026-45447, CVE-2026-7383,
CVE-2026-9076, CVE-2026-34180 plus the openssl MEDIUM/LOWs from the
3.4.0 scan.

## D3 — VEX distribution

`docker/vex.json` lives in-repo (auditable history) and is passed to
the scan via `docker scout cves --vex-location docker/vex.json`. The
`docker scout attestation add` path requires Scout enrollment writes;
the CI gate uses `--vex-location` so the gate works regardless of the
org's Scout plan. If/when attestation attach is enabled for the repo,
the same file can be attached without changes.
