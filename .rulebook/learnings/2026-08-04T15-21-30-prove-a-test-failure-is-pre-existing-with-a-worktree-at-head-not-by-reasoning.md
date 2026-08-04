# Prove a test failure is pre-existing with a worktree at HEAD, not by reasoning
**Source**: manual
**Date**: 2026-08-04
**Related Task**: phase1_bump-openraft-alpha30
**Tags**: testing, git, flaky, baseline
The openraft bump left two failures in the full `vectorizer` suite. Both were unrelated to consensus, and the tempting move was to argue them away ("openraft is not on the cache path"). Reasoning is not evidence when the claim is "I did not break this".

What actually settled it, without any destructive git operation (no stash, no reset):

    git worktree add <scratch>/head-wt HEAD
    cp -r dashboard/dist <scratch>/head-wt/dashboard/dist   # untracked build artifact
    cd <scratch>/head-wt && CARGO_TARGET_DIR=<scratch>/head-wt/target cargo test ...
    git worktree remove --force <scratch>/head-wt

`prometheus_counter_increments_on_every_cache_get` failed identically at HEAD → pre-existing, and it got its own follow-up task instead of a hand-wave.

Two details worth keeping:
- A fresh worktree does not compile here until `dashboard/dist` is copied in: `#[derive(RustEmbed)]` needs that folder and it is a build artifact, not tracked.
- The second failure (`test_hybrid_search_pure_dense`) appeared once and never again — three subsequent full runs were clean, and it passes in isolation. Distinguishing "flake" from "regression" needs repetition, not a single run. Consensus tests deserve the same treatment: the new live three-node HA test was run five times before being trusted.

Baseline discipline: record the pass/fail counts of the suite before and after (925+1 at HEAD, 926+1 after — the extra pass being the new test). A count comparison catches a silently-skipped or silently-added test that a green summary hides.