#!/usr/bin/env bash
# sweep-target.sh — phase34 / issue #320
#
# Garbage-collects stale artifacts in `target/` without nuking the
# incremental hot set. Wraps `cargo-sweep` so the next build is still
# fast — only rlibs / object files / docs that haven't been touched
# in $1 days (default 14) are removed.
#
# Use this instead of `cargo clean` for routine hygiene. Reach for
# `cargo clean` only when you need a fully cold rebuild (debugging a
# build-script regression, switching toolchains, etc.).
#
# Usage:
#   bash scripts/sweep-target.sh [days]
#
# Schedules (Linux / macOS):
#   # ~/.config/cron.d/vectorizer-sweep — daily at 03:00
#   0 3 * * * cd /path/to/vectorizer && bash scripts/sweep-target.sh 14
#
# Full operator runbook: docs/development/rust-target-hygiene.md

set -euo pipefail

DAYS="${1:-14}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ ! -f "${ROOT}/Cargo.toml" ]; then
  echo "sweep-target.sh: ${ROOT}/Cargo.toml not found — am I in a Cargo workspace?" >&2
  exit 1
fi

cd "${ROOT}"

if ! command -v cargo-sweep >/dev/null 2>&1; then
  echo "==> installing cargo-sweep (one-time)"
  cargo install --locked cargo-sweep
fi

size_before=$(du -sb target 2>/dev/null | awk '{print $1}' || echo 0)

echo "==> sweeping artifacts not accessed in the last ${DAYS} day(s)"
cargo sweep --time "${DAYS}"

size_after=$(du -sb target 2>/dev/null | awk '{print $1}' || echo 0)
reclaimed=$(( size_before - size_after ))

# Pretty-print both numbers in MiB; `du -sh` would re-walk the tree.
human() {
  awk -v b="$1" 'BEGIN {
    if (b >= 1073741824) printf "%.2f GiB", b/1073741824;
    else if (b >= 1048576) printf "%.2f MiB", b/1048576;
    else if (b >= 1024) printf "%.2f KiB", b/1024;
    else printf "%d B", b;
  }'
}

echo
echo "target/  before: $(human "${size_before}")"
echo "target/   after: $(human "${size_after}")"
echo "      reclaimed: $(human "${reclaimed}")"
