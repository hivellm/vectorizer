# A blocked transitive advisory is usually a parent that nobody tried to update

**Category**: dependencies
**Tags**: cargo, security, cargo-audit, dependencies

## Description

`cargo update -p <child> --dry-run` answering `Locking 0 packages to latest compatible versions` means the CHILD cannot move on its own, because a parent's semver requirement pins it. It does NOT mean the advisory is unfixable.

Six of nine vulnerabilities in this repo were filed as "blocked upstream" on that reading. Five cleared with a single `cargo update -p <parent>`:

- `quick-xml` 0.37.5 and `lopdf` 0.34/0.35 were pinned by `transmutation`, which is our own crate. We declared `"0.3.1"`, were resolving 0.3.3, and 0.3.5 was already published. One `cargo update -p transmutation` moved 60 packages and carried `lopdf` to 0.42/0.44, `quick-xml` to 0.41.0, `pdf-extract` to 0.12.0, `umya-spreadsheet` to 3.1.0.
- The last `quick-xml` 0.36.2 came from `docx-rs` 0.4.20; `cargo update -p docx-rs` -> 0.4.22 cleared it.

Procedure before declaring an advisory blocked:
1. `cargo tree -i <crate>` (add `--all-features`; disambiguate duplicated versions with `crate@version`) to find the PARENT.
2. Check the parent's latest on crates.io — a semver-compatible parent release often already carries the fix.
3. `cargo update -p <parent>`, then re-run `cargo audit`.
4. Only if the parent is also capped is it genuinely upstream work.

Two related traps:
- `cargo audit` reads `Cargo.lock`, not the compiled graph. A crate can be listed through an optional feature nothing enables — `cargo tree -i` printing nothing means it is not compiled, and the finding is a lockfile artifact.
- `cargo update -p <crate>` will not advance a PRE-RELEASE dependency. `fastembed` 5.17.3 -> 5.17.4 reported `Locking 0 packages` because it also needs `ort` rc.12 -> rc.13; `--precise 5.17.4` does it. The pre-release bump is the real content of such an update, so read it before taking it.
