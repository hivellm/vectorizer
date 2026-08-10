# A green release pipeline is not evidence that a package published

**Category**: release
**Tags**: release, ci, registries, go-modules, anti-pattern

## Description

Twice now a release has reported complete success while shipping nothing for one of its targets, because CI cannot fail at a step it does not have.

- **v3.6.0**: the run was green and Docker Hub received no image. The `publish-docker` jobs had been deleted from `release-artifacts.yml` — the workflow contains zero occurrences of "docker" — so there was nothing left to fail.
- **v3.6.1**: all six release workflows succeeded while the Go SDK was unresolvable. No workflow publishes Go; the module is tagged in its own repository, and nothing in this repo's CI touches it.

Verify a release by **asking the registries**, never by reading the pipeline:

    crates.io   https://crates.io/api/v1/crates/<name>            .crate.max_version
    PyPI        https://pypi.org/pypi/<name>/json                 .info.version
    npm         https://registry.npmjs.org/<@scope%2Fname>        .'dist-tags'.latest
    NuGet       https://api.nuget.org/v3-flatcontainer/<id>/index.json
    Go          https://proxy.golang.org/<module>/@v/list

The Go case is worth its own note, because a tag can exist and still resolve to nothing. `go.mod` declared `github.com/hivellm/vectorizer-sdk-go` while the code lives in `vectorizer-go` — Go fetches the URL the *module path* names, not the repo you tagged — and a major version at or above 2 additionally requires a `/vN` path suffix, so a `v3.6.1` tag on a suffix-less path is not a version the proxy will serve. Both were true for years; the module had never been installable. The symptom is the proxy answering an empty list (path exists, no valid versions) versus 404 (path unknown) — the empty answer is the one people misread as "not indexed yet".

Related: the in-tree guard `crates/vectorizer/tests/version_carriers_agree.rs` catches carriers that disagree with each other, and could not have caught this. Every carrier read 3.6.1 correctly; the module was still unfetchable. Declaring a version and publishing one are different claims.

## When to Use

Verifying that a release actually shipped, or diagnosing a package that CI claims to have published but consumers cannot install.
