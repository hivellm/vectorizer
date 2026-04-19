"""Prepend `#![allow(clippy::unwrap_used, clippy::expect_used)]` to each
test-by-name source file so tests can keep using unwrap idiomatically once
the `unwrap_used = "deny"` workspace lint is on. Idempotent.

Targets *_tests.rs and tests.rs files plus any file passed on the CLI.
"""
import os
import re
import sys

ATTR = "#![allow(clippy::unwrap_used, clippy::expect_used)]\n"


def patch(path: str) -> bool:
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    if "clippy::unwrap_used" in content[:600]:
        return False
    # Find the first non-comment, non-blank line — insert the inner attribute
    # immediately above it, after any leading `//!` doc comments.
    lines = content.split("\n")
    insert_at = 0
    in_inner_doc = False
    for i, line in enumerate(lines):
        s = line.lstrip()
        if s.startswith("//!"):
            in_inner_doc = True
            insert_at = i + 1
            continue
        if in_inner_doc and s == "":
            insert_at = i + 1
            continue
        # First substantive line — stop searching.
        break
    new_lines = lines[:insert_at] + [ATTR.rstrip("\n"), ""] + lines[insert_at:]
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(new_lines))
    return True


def main(argv: list[str]) -> None:
    targets: list[str] = []
    if len(argv) > 1:
        targets = argv[1:]
    else:
        # Test-by-name files in src/.
        for root, _, files in os.walk("src"):
            for f in files:
                name = f.lower()
                if name == "tests.rs" or name.endswith("_tests.rs"):
                    targets.append(os.path.join(root, f))
        # Every .rs file under tests/ (integration tests).
        for root, _, files in os.walk("tests"):
            for f in files:
                if f.endswith(".rs"):
                    targets.append(os.path.join(root, f))

    patched = []
    for p in targets:
        if patch(p):
            patched.append(p)
    print(f"Patched {len(patched)} file(s):")
    for p in patched:
        print(f"  {p}")


if __name__ == "__main__":
    main(sys.argv)
