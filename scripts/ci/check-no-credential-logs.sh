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
# Scope: src/auth/ and src/server/auth_handlers.rs — files that legitimately
# handle credential material. Other modules are out of scope.
#
# Allowed:
#   * Lines carrying a trailing `// logging-allow(<reason>): ...` sentinel
#     (for unavoidable labels like `"Failed to hash password: {}"` where the
#     `{}` slot holds the bcrypt error, not the password).
#
# Exit 0 = clean; exit 1 = violations (prints them).

set -euo pipefail

pattern='(println|info|debug|warn|error|trace)!\([^)]*\b(password|secret|api_key)\b'

hits=$(grep -rnE "$pattern" src/auth/ src/server/auth_handlers.rs \
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
