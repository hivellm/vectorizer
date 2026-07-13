//! Dependabot ecosystem-coverage check (phase36 dependency-refresh).
//!
//! The dependency-refresh spec requires dependabot to watch every SDK
//! ecosystem so client deps don't silently rot between releases. This
//! test pins that invariant: each SDK directory that carries a package
//! manifest must have a matching `updates:` entry in
//! `.github/dependabot.yml` — except `sdks/rust`, which is a workspace
//! member sharing the root `Cargo.lock` and is covered by the root
//! `cargo` entry (a second entry would open duplicate PRs).

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
fn dependabot_covers_every_sdk_ecosystem() {
    let config = fs::read_to_string(repo_root().join(".github").join("dependabot.yml"))
        .expect(".github/dependabot.yml must exist");

    // (sdk dir, manifest that proves the ecosystem exists, required
    //  package-ecosystem, required directory value)
    let required = [
        ("sdks/typescript", "package.json", "npm", "/sdks/typescript"),
        ("sdks/python", "pyproject.toml", "pip", "/sdks/python"),
        ("sdks/go", "go.mod", "gomod", "/sdks/go"),
        ("sdks/csharp", "Vectorizer.csproj", "nuget", "/sdks/csharp"),
    ];

    for (dir, manifest, ecosystem, expected_directory) in required {
        let manifest_path = repo_root().join(dir).join(manifest);
        assert!(
            manifest_path.exists(),
            "expected manifest {manifest} in {dir} — if the SDK moved, update this test AND dependabot.yml"
        );

        let ecosystem_line = format!("package-ecosystem: \"{ecosystem}\"");
        assert!(
            config.contains(&ecosystem_line),
            "dependabot.yml missing a `{ecosystem}` ecosystem entry for {dir}"
        );
        let directory_line = format!("directory: \"{expected_directory}\"");
        assert!(
            config.contains(&directory_line),
            "dependabot.yml `{ecosystem}` entry must point at {expected_directory}"
        );
    }

    // Root cargo entry covers the workspace (incl. sdks/rust).
    assert!(
        config.contains("package-ecosystem: \"cargo\""),
        "root cargo ecosystem entry missing"
    );
    assert!(
        !config.contains("directory: \"/sdks/rust\""),
        "sdks/rust must NOT have its own cargo entry — it shares the \
         root Cargo.lock; a separate entry opens duplicate PRs"
    );
}
