#!/usr/bin/env bash
# check-no-tier1-markers.sh
#
# Enforces AGENTS.md Tier-1 rule #1: no deferral markers in source code.
#
# Forbidden patterns (case-sensitive): TODO, FIXME, HACK, XXX.
#
# Allowed forms:
#   * Lines that carry a `TASK(phaseN_<slug>)` reference to a tracked rulebook task.
#   * Lines that carry the `grep-ignore(tier1-markers)` sentinel (used by the
#     marker-detection feature in file_operations/operations.rs — it must
#     keep the literal tokens to do its job).
#
# Scan roots default to the workspace crates' source trees. Before the
# workspace split the default was a bare `src`, which stopped existing when
# the code moved under `crates/*/src` — `grep` then failed on a missing path
# and the gate reported "clean" without having read anything. Tests and docs
# stay out of scope, matching what the pre-split default covered.
#
# Exit 0 = clean; exit 1 = violations (prints them).

set -euo pipefail

if [[ $# -gt 0 ]]; then
  SCAN_ROOTS=("$@")
else
  SCAN_ROOTS=()
  for crate_src in crates/*/src; do
    [[ -d "$crate_src" ]] && SCAN_ROOTS+=("$crate_src")
  done
  if [[ ${#SCAN_ROOTS[@]} -eq 0 ]]; then
    echo "::error::no scan root found — expected crates/*/src to exist"
    exit 1
  fi
fi

for root in "${SCAN_ROOTS[@]}"; do
  if [[ ! -d "$root" ]]; then
    echo "::error::scan root '$root' does not exist"
    exit 1
  fi
done

hits=$(grep -rEn '\b(TODO|FIXME|HACK|XXX)\b' "${SCAN_ROOTS[@]}" \
  | grep -vE 'TASK\(phase[0-9]+_[a-z0-9-]+\)' \
  | grep -vE 'grep-ignore\(tier1-markers\)' \
  || true)

if [[ -n "$hits" ]]; then
  echo "::error::Tier-1 marker(s) found outside the TASK(phaseN_<slug>) allow-list:"
  echo "$hits"
  echo
  echo "Fix: convert each marker to '// TASK(phaseN_<slug>): ...' pointing at a rulebook task,"
  echo "or, if the literal is required by a detection feature, add 'grep-ignore(tier1-markers)' on the same line."
  exit 1
fi

echo "Tier-1 marker gate: clean."
