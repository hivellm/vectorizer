//! Structural checks for the Docker CVE posture artifacts
//! (phase35_fix-docker-image-cves).
//!
//! Two repo files are load-bearing for the CVE gate:
//!
//! 1. `docker/vex.json` — the OpenVEX document the gate applies via
//!    `--vex-location`. A malformed statement (missing justification
//!    or impact_statement) silently weakens the exception audit trail
//!    the image-security spec requires.
//! 2. The `Dockerfile` runtime base pin — the freshness job in
//!    `docker-cve-gate.yml` parses both the `@sha256:` digest and the
//!    `# Pinned YYYY-MM-DD` comment; if either drifts out of the
//!    expected shape the staleness check dies quietly.
//!
//! These tests pin the shapes so a drive-by edit fails CI instead of
//! disabling the gate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("..");
    p
}

#[test]
fn vex_document_is_well_formed_openvex() {
    let raw = fs::read_to_string(repo_root().join("docker").join("vex.json"))
        .expect("docker/vex.json must exist — the CVE gate applies it via --vex-location");
    let doc: serde_json::Value =
        serde_json::from_str(&raw).expect("docker/vex.json must be valid JSON");

    assert_eq!(
        doc["@context"], "https://openvex.dev/ns/v0.2.0",
        "VEX @context must be OpenVEX 0.2.0"
    );
    assert!(
        doc["author"].as_str().is_some_and(|a| !a.is_empty()),
        "VEX author must be set — docker-cve-gate.yml passes it via --vex-author"
    );

    let statements = doc["statements"]
        .as_array()
        .expect("VEX must carry a statements array");
    assert!(
        !statements.is_empty(),
        "VEX with zero statements means the unfixable-CVE set lost its exceptions"
    );

    for st in statements {
        let cve = st["vulnerability"]["name"]
            .as_str()
            .expect("every statement needs vulnerability.name");
        assert!(
            cve.starts_with("CVE-"),
            "vulnerability.name must be a CVE id, got {cve}"
        );
        assert_eq!(
            st["status"], "not_affected",
            "{cve}: only not_affected exceptions are allowed in this file"
        );
        // Spec: bare not_affected without reasoning is FORBIDDEN.
        assert!(
            st["justification"].as_str().is_some_and(|j| !j.is_empty()),
            "{cve}: missing justification"
        );
        assert!(
            st["impact_statement"]
                .as_str()
                .is_some_and(|i| i.len() >= 40),
            "{cve}: impact_statement missing or too thin to be an audit trail"
        );
        let products = st["products"].as_array();
        assert!(
            products.is_some_and(|p| !p.is_empty()),
            "{cve}: statement must name its product"
        );
        assert_eq!(
            products.unwrap()[0]["@id"],
            "pkg:docker/hivehub/vectorizer",
            "{cve}: product must be the published image purl"
        );
    }
}

#[test]
fn dockerfile_runtime_base_is_digest_pinned_with_date() {
    let dockerfile = fs::read_to_string(repo_root().join("Dockerfile")).expect("Dockerfile");

    // The freshness job greps this exact shape:
    //   FROM dhi.io/debian-base:trixie@sha256:<64 hex> AS vectorizer
    let pin = dockerfile.lines().find(|l| {
        l.starts_with("FROM dhi.io/debian-base:trixie@sha256:") && l.contains(" AS vectorizer")
    });
    let pin = pin.expect(
        "runtime base must be digest-pinned (image-security spec); \
         a floating tag makes image contents depend on build date",
    );
    let digest = pin
        .split("@sha256:")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    assert_eq!(digest.len(), 64, "digest must be full sha256 hex");
    assert!(
        digest.chars().all(|c| c.is_ascii_hexdigit()),
        "digest must be hex"
    );

    // The staleness check parses `# Pinned YYYY-MM-DD` to compute age.
    let dated = dockerfile.lines().any(|l| {
        l.trim_start().starts_with("# Pinned ")
            && l.split("# Pinned ").nth(1).is_some_and(|d| {
                let d = &d[..d.len().min(10)];
                d.len() == 10
                    && d.as_bytes()[4] == b'-'
                    && d.as_bytes()[7] == b'-'
                    && d.chars().enumerate().all(|(i, c)| {
                        if i == 4 || i == 7 {
                            c == '-'
                        } else {
                            c.is_ascii_digit()
                        }
                    })
            })
    });
    assert!(
        dated,
        "Dockerfile must carry a `# Pinned YYYY-MM-DD` comment next to the base pin — \
         docker-cve-gate.yml's freshness job parses that date"
    );
}
