#!/usr/bin/env bash
# check-audit-gate.sh
#
# Runs the dependency audit, and first proves the audit can actually fail.
#
# phase7_dependency-security-audit §2.2. The thing under test here is the
# *gate*, not the dependencies. A security check that cannot fail is worse than
# no check, because it reports "clean" forever and everyone believes it — which
# is exactly what this repository had:
#
#   * `audit.toml` sat at the repository root, where cargo-audit never looks,
#     so its recorded exceptions did nothing. Running with the file and with it
#     deleted produced byte-identical output.
#   * Moved to `.cargo/`, it turned out the schema was invalid too, and
#     cargo-audit rejects a bad config with exit code **1** — indistinguishable
#     from "vulnerabilities found" to anything that only reads the exit code.
#
# So this script asserts three things in order, and each one has failed for
# real at some point:
#
#   1. The policy file is where cargo-audit reads it, and parses.
#   2. cargo-audit still detects a known-vulnerable lockfile.
#   3. Our own lockfile is clean.
#
# Step 2 is the one that keeps the other two honest. Without it, a future
# change that quietly stops the tool from working would leave step 3 passing.
#
# Exit 0 = clean; exit 1 = the gate is broken, or a vulnerability was found.

set -euo pipefail

POLICY=".cargo/audit.toml"
LOCKFILE="Cargo.lock"

for required in "$LOCKFILE" "$POLICY"; do
  if [[ ! -f "$required" ]]; then
    echo "::error::$required not found — this gate would not be auditing what it claims to"
    exit 1
  fi
done

if ! command -v cargo-audit >/dev/null 2>&1 && ! cargo audit --version >/dev/null 2>&1; then
  echo "::error::cargo-audit is not installed — install it with 'cargo install cargo-audit --locked'"
  exit 1
fi

# ── 1. the policy parses ─────────────────────────────────────────────────────
# `--no-fetch` skips the advisory download so this asks one question only.
if cargo audit --no-fetch 2>&1 | grep -q "fatal error: parse error"; then
  echo "::error::$POLICY does not parse. cargo-audit exits 1 on this, which looks exactly like a failing audit."
  cargo audit --no-fetch 2>&1 | tail -8
  exit 1
fi

# ── 2. the tool still detects something ──────────────────────────────────────
# A hand-written lockfile naming a crate with a long-standing advisory
# (RUSTSEC-2020-0071, `time` segfault). Hermetic: cargo-audit reads the
# lockfile directly, so nothing is resolved or downloaded for the fixture
# itself.
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
cat > "$fixture/Cargo.lock" <<'FIXTURE'
version = 3

[[package]]
name = "time"
version = "0.1.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
FIXTURE

if cargo audit -f "$fixture/Cargo.lock" >/dev/null 2>&1; then
  echo "::error::the audit reported a known-vulnerable lockfile as clean."
  echo "  Expected RUSTSEC-2020-0071 against time 0.1.44 and got a pass."
  echo "  Something is silencing findings — an over-broad ignore in $POLICY,"
  echo "  a stale advisory database, or a cargo-audit that is not running."
  echo "  Every result from this gate is meaningless until that is fixed."
  exit 1
fi

# ── 3. our lockfile is clean ─────────────────────────────────────────────────
if ! cargo audit; then
  echo
  echo "::error::vulnerabilities found in $LOCKFILE."
  echo "  Fix by updating the crate, or — if it is pinned by a parent — update"
  echo "  the PARENT. \`cargo update -p <child> --dry-run\` answering"
  echo "  'Locking 0 packages' means the child cannot move on its own; it does"
  echo "  not mean the advisory is unfixable. See"
  echo "  docs/analysis/dependency-security-2026-08/02-rust-advisories.md."
  echo "  If there is genuinely no fix, add it to $POLICY with the reasoning."
  exit 1
fi

echo "Dependency audit gate: policy parses, detection works, lockfile clean."
