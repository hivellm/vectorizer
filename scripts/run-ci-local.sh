#!/usr/bin/env bash
# Run the same checks as CI (Formatter and linter + Rust tests) locally.
# Usage: ./scripts/run-ci-local.sh
# Skip tests (lint only): ./scripts/run-ci-local.sh --lint-only
# Skip dashboard build (if already built): ./scripts/run-ci-local.sh --no-dashboard

set -e
cd "$(git rev-parse --show-toplevel)"

LINT_ONLY=false
NO_DASHBOARD=false
for arg in "$@"; do
  case "$arg" in
    --lint-only)    LINT_ONLY=true ;;
    --no-dashboard) NO_DASHBOARD=true ;;
  esac
done

echo "=============================================="
echo "  CI/CD local - Formatter and linter + Tests"
echo "=============================================="
echo ""

# --- Dashboard (required for rust-embed) ---
if [ "$NO_DASHBOARD" = false ]; then
  echo "=== 1. Build dashboard (required for rust-embed) ==="
  (cd dashboard && pnpm install --frozen-lockfile && pnpm build)
  echo "OK"
  echo ""
else
  echo "=== 1. Dashboard build skipped (--no-dashboard) ==="
  echo ""
fi

# --- Formatter and linter (rust-lint.yml) ---
echo "=== 2. Check code formatting (cargo +nightly fmt --check) ==="
if command -v rustup &>/dev/null && rustup run nightly cargo fmt --all -- --check 2>/dev/null; then
  echo "OK (nightly)"
elif cargo +nightly fmt --all -- --check 2>/dev/null; then
  echo "OK (nightly)"
else
  echo "Nightly not available, using stable fmt --check..."
  cargo fmt --all -- --check
  echo "OK (stable - CI uses nightly, format may differ)"
fi
echo ""

echo "=== 3. Clippy (workspace) ==="
cargo clippy --workspace -- -D warnings
echo "OK"
echo ""

echo "=== 4. Clippy (all-targets) ==="
cargo clippy --workspace --all-targets -- -D warnings
echo "OK"
echo ""

if [ "$LINT_ONLY" = true ]; then
  echo "=============================================="
  echo "  Lint only: all checks passed."
  echo "=============================================="
  exit 0
fi

# --- Rust tests (rust.yml) ---
echo "=== 5. Build (tests) ==="
cargo build --tests --workspace
echo "OK"
echo ""

echo "=== 6. Run tests (nextest or cargo test) ==="
if command -v cargo-nextest &>/dev/null; then
  NEXTEST_TIMEOUT=300 cargo nextest run --workspace --all-targets --test-threads 4
  NEXTEST_TIMEOUT=600 cargo nextest run --test all_tests --workspace
else
  cargo test --workspace --no-fail-fast
  cargo test --test all_tests --workspace --no-fail-fast
fi
echo "OK"
echo ""

echo "=== 7. Doc tests ==="
cargo test --workspace --doc
echo "OK"
echo ""

echo "=============================================="
echo "  All CI checks passed locally."
echo "=============================================="
