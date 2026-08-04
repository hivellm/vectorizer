#!/usr/bin/env bash
# check-no-credential-logs.sh
#
# Enforces the rule that log macros in credential-handling code must not
# reference `password` / `secret` / `api_key` in their format string.
#
# Rationale: `Secret<T>` protects against typing a raw secret into `{:?}`,
# but a developer can still hand-roll a leak with `info!("password = {}", pwd)`.
# This gate catches the pattern at review time before it ships.
#
# Scope: the credential-handling trees — `crates/vectorizer/src/auth/` and
# `crates/vectorizer-server/src/server/auth_handlers/` (plus its sibling test
# file). Other modules are out of scope.
#
# These paths used to read `src/auth/` and `src/server/auth_handlers.rs`, which
# stopped existing when the code moved into the workspace crates: `grep` failed
# on the missing paths and the gate printed "clean" without reading a line.
#
# Allowed:
#   * Lines carrying a trailing `// logging-allow(<reason>): ...` sentinel
#     (for unavoidable labels like `"Failed to hash password: {}"` where the
#     `{}` slot holds the bcrypt error, not the password).
#
# Exit 0 = clean; exit 1 = violations (prints them).

set -euo pipefail

pattern='(println|info|debug|warn|error|trace)!\([^)]*\b(password|secret|api_key)\b'

SCAN_PATHS=(
  crates/vectorizer/src/auth/
  crates/vectorizer-server/src/server/auth_handlers/
  crates/vectorizer-server/src/server/auth_handlers_tests.rs
)

for path in "${SCAN_PATHS[@]}"; do
  if [[ ! -e "$path" ]]; then
    echo "::error::scan path '$path' does not exist — the gate would pass without reading anything"
    exit 1
  fi
done

hits=$(grep -rnE "$pattern" "${SCAN_PATHS[@]}" \
  | grep -vE 'logging-allow\(' \
  || true)

if [[ -n "$hits" ]]; then
  echo "::error::Log macro in credential-handling code references password/secret/api_key:"
  echo "$hits"
  echo
  echo "Fix: either drop the reference from the format string, or add a trailing"
  echo "     // logging-allow(<reason>): <why it is safe>"
  echo "     sentinel on the same line if the reference is a label (not a value)."
  exit 1
fi

echo "Credential log-leakage gate: clean."
