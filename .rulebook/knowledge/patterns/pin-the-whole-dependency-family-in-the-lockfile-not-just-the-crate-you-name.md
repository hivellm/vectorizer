# Pin the whole dependency family in the lockfile, not just the crate you name

**Category**: dependencies
**Tags**: dependencies, cargo, openraft, pinning

## Description

openraft is pinned with `=0.10.0-alpha.NN` in both Cargo.tomls so the consensus layer cannot drift between alphas. That pin is weaker than it looks: openraft's own manifest declares its siblings (`openraft-macros`, `openraft-rt`, `openraft-rt-tokio`) with a plain caret, and for pre-release versions Cargo happily resolves `0.10.0-alpha.30` to `0.10.0-alpha.32`. After bumping openraft to alpha.30 the lock held alpha.30 + siblings at alpha.32 — a combination upstream never ships or tests together.

Fix: pin the siblings in the lockfile too, without adding them as direct dependencies:

    cargo update -p openraft-rt-tokio --precise 0.10.0-alpha.30
    cargo update -p openraft-rt        --precise 0.10.0-alpha.30
    cargo update -p openraft-macros    --precise 0.10.0-alpha.30

Order matters: downgrading a leaf first fails, because the sibling still at the newer version imposes `^0.10.0-alpha.32` on it. Start with the crate nothing else constrains (`-rt-tokio`), then walk inward. The committed lockfile then freezes the whole family for every checkout.

Verify the result with `grep -A 2 -E '^name = "openraft' Cargo.lock` — every version in the family should read the same, and `cargo tree -i <crate>` names whoever is still pushing a version up.

## When to Use

Bumping any dependency that is deliberately pinned for behavioural stability and that ships as a family of crates (consensus, runtime, codec stacks).
