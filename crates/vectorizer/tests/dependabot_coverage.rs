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

/// Does `.gitmodules` declare `dir` as a submodule?
///
/// `sdks/go` lives in its own repository (`hivellm/vectorizer-go`) and is
/// vendored here as a submodule, so a checkout that does not fetch submodules
/// leaves the directory empty — `actions/checkout` skips them by default. In
/// that case the parent repo still pins the path through `.gitmodules` plus the
/// gitlink, which is the proof this test needs: the SDK has not moved, it is
/// simply not materialised in this working tree.
fn declares_submodule(dir: &str) -> bool {
    let expected = format!("path = {dir}");
    fs::read_to_string(repo_root().join(".gitmodules"))
        .map(|config| config.lines().any(|line| line.trim() == expected))
        .unwrap_or(false)
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
            manifest_path.exists() || declares_submodule(dir),
            "expected manifest {manifest} in {dir}, and {dir} is not declared \
             as a submodule either — if the SDK moved, update this test AND \
             dependabot.yml"
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
    // Front-end manifests are covered too — their advisories were invisible in
    // practice until they got entries, because Dependabot only opens updates
    // (security ones included) for directories listed in the config.
    for directory in ["/dashboard", "/gui"] {
        assert!(
            config.contains(&format!("directory: \"{directory}\"")),
            "dependabot.yml must watch {directory} — npm advisories there get \
             no PR otherwise"
        );
    }
    assert!(
        !config.contains("directory: \"/sdks/rust\""),
        "sdks/rust must NOT have its own cargo entry — it shares the \
         root Cargo.lock; a separate entry opens duplicate PRs"
    );
}

/// Pins both branches of the submodule escape hatch above, so the coverage
/// test cannot silently start accepting a genuinely missing SDK.
///
/// Without this, `manifest.exists() || declares_submodule(dir)` would be
/// satisfied by the manifest alone in a full checkout and the submodule branch
/// would never be exercised until CI failed again.
#[test]
fn only_the_go_sdk_is_declared_as_a_submodule() {
    assert!(
        declares_submodule("sdks/go"),
        ".gitmodules must declare `path = sdks/go` — the coverage test relies \
         on it when a checkout skips submodules and leaves the directory empty"
    );
    assert!(
        !declares_submodule("sdks/typescript"),
        "sdks/typescript is vendored in-tree; if it ever became a submodule, \
         its manifest guard would need the same escape hatch"
    );
}
