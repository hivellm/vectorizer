## 1. Implementation

Strictly ordered: nothing below 1.1 can start until the commits are on
GitHub, and 1.5 cannot start until the registries actually serve 3.6.1.

- [ ] 1.1 **Push `main`.** Blocked on the user — this shell has no SSH key,
      and the HTTPS + `gh auth git-credential` route hung on a non-interactive
      credential prompt. As of the phase3 handoff the remote sits at
      `ead8c55b` and everything from `33fd22be` onward is local.
      **Done when:** `gh api repos/hivellm/vectorizer/commits/main --jq .sha`
      matches local `HEAD`.
- [ ] 1.2 **Push the `sdks/go` submodule.** It is a separate repository with
      its own remote and carries five unpushed commits, the newest being the
      3.6.1 version bump. Until this lands, CI still cannot add
      `submodules: true` to checkout without failing outright.
      **Done when:** the submodule's remote HEAD matches its local HEAD.
- [ ] 1.3 **Tag `v3.6.1` on the superproject and push the tag.** This is what
      drives the SDK publish workflows.
      **Done when:** the tag exists on the remote and
      `release-artifacts.yml` has started.
- [ ] 1.4 **Tag `v3.6.1` in the `sdks/go` repository too.** Go resolves a
      module version from a tag in that repo; the superproject's tag does
      nothing for it. The `Version` constant bumped in phase3 has to agree
      with this tag, or `go get` and the client's self-reported version
      disagree.
      **Done when:** `go list -m github.com/hivellm/vectorizer-go@v3.6.1`
      resolves.
- [ ] 1.5 **Verify all five SDKs actually landed** — crates.io, npm, PyPI,
      NuGet, and the Go module proxy. Do not infer from a green workflow: the
      v3.6.0 run reported success while publishing no Docker image at all,
      because the jobs had been deleted and there was nothing left to fail.
      **Done when:** each registry serves 3.6.1, checked by query rather than
      by reading CI.
- [ ] 1.6 **Point the GUI at the published SDK** (`gui/package.json`:
      `@hivehub/vectorizer-sdk` → `^3.6.1`), then `pnpm install` to refresh
      the lockfile. Must come after 1.5 — a `package.json` whose lockfile
      disagrees breaks `pnpm install --frozen-lockfile` in CI.
      **Done when:** `pnpm install` resolves and `pnpm type-check` passes.
- [ ] 1.7 **Publish the GitHub release notes** for `v3.6.1` from the
      CHANGELOG's 3.6.1 section.
      **Done when:** the release is visible and not a draft.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Update or create documentation covering the implementation.
      Candidate for a waiver: phase3 already moved every install snippet and
      the Docker Hub readme to 3.6.x. Re-check only if a registry ends up
      serving something other than 3.6.1.
      **Worth doing instead of waiving:** `deploy/docker/dockerhub-readme.md`
      is documented as shipping to Docker Hub on every release, and nothing
      does that any more — the job went with the deleted Docker CI. Either
      push the description by hand while publishing, or record that the file
      is now decorative.
- [ ] 2.2 Write tests covering the new behavior — waive; this task performs a
      publish and changes no behaviour. The guard that matters already exists:
      `crates/vectorizer/tests/version_carriers_agree.rs` fails if any carrier
      drifts, and it was verified by sabotage rather than by passing.
- [ ] 2.3 Run tests and confirm they pass — after 1.6 only, since that is the
      one step here that touches the tree (`pnpm type-check` in `gui/`).

## Already done, listed so the verification lives in one place

Published from phase3, because CI cannot: `hivehub/vectorizer:3.6.1`,
`:3.6.1-fastembed` and `:latest`, multi-arch with SBOM + provenance
attestations. Both tags boot-tested on `linux/amd64` and `linux/arm64` and
scanned with the VEX applied.

Two notes for whoever audits the images:

- The two images of the same version carry **different** `GIT_COMMIT_ID`
  values — the default was built at `c610c2cb`, the fastembed variant at
  `3d0cf879`, because the version-carrier test landed between the two builds.
  Both are release commits and neither changes the binary, but a provenance
  diff will show it.
- They were built from commits that were not yet on GitHub, the same
  situation the 3.6.0 images were built under. Once 1.1 lands, the images and
  the remote agree.
