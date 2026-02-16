#!/usr/bin/env bash
# Run the same checks as CI (Formatter and linter workflow) before pushing.
# Usage: ./scripts/check-before-push.sh
# Optional: skip tests (format + clippy only): ./scripts/check-before-push.sh --no-test

set -e
cd "$(git rev-parse --show-toplevel)"

RUN_TESTS=true
for arg in "$@"; do
  case "$arg" in
    --no-test) RUN_TESTS=false ;;
  esac
done

echo "=== 1. Check code formatting (cargo fmt --check) ==="
if command -v rustup &>/dev/null; then
  cargo +nightly fmt --all -- --check 2>/dev/null || cargo fmt --all -- --check
else
  cargo fmt --all -- --check
fi
echo "OK"
echo ""

echo "=== 2. Clippy (workspace) ==="
cargo clippy --workspace -- -D warnings
echo "OK"
echo ""

echo "=== 3. Clippy (all-targets) ==="
cargo clippy --workspace --all-targets -- -D warnings
echo "OK"
echo ""

if [ "$RUN_TESTS" = true ]; then
  echo "=== 4. Tests (workspace) ==="
  cargo test --workspace --no-fail-fast
  echo "OK"
else
  echo "=== 4. Tests skipped (--no-test) ==="
fi

echo ""
echo "All checks passed. Safe to push."
