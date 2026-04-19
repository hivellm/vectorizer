"""Add `#[allow(clippy::unwrap_used, clippy::expect_used)]` to every
`#[cfg(test)] mod tests { ... }` block whose body contains at least one
`.unwrap()` or `.expect(...)` call. Idempotent.

Skips files where the allow attribute is already present immediately above
the `mod tests` declaration.

Run from repo root: `python scripts/add_test_unwrap_allow.py`.
"""
import os
import re

ATTR = "#[allow(clippy::unwrap_used, clippy::expect_used)]"


def patch_file(path: str) -> bool:
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    pattern = re.compile(
        r"(^#\[cfg\(test\)\][^\n]*\n)((?:#\[[^\n]*\]\n)*)(mod\s+\w+\s*\{)",
        re.MULTILINE,
    )
    changed = False

    def replacer(m):
        nonlocal changed
        prefix, attrs, mod_decl = m.group(1), m.group(2), m.group(3)
        if "clippy::unwrap_used" in attrs:
            return m.group(0)
        # Only add if the test mod actually contains an unwrap/expect call.
        # Find body extents by brace counting.
        start = m.end()
        depth = 1
        i = start
        while i < len(content) and depth > 0:
            c = content[i]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
            i += 1
        body = content[start:i]
        if not re.search(r"\.unwrap\(\)|\.expect\(", body):
            return m.group(0)
        changed = True
        return f"{prefix}{attrs}{ATTR}\n{mod_decl}"

    new_content = pattern.sub(replacer, content)
    if changed and new_content != content:
        with open(path, "w", encoding="utf-8") as f:
            f.write(new_content)
        return True
    return False


def main() -> None:
    touched = []
    for root, _, files in os.walk("src"):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(root, f)
            if patch_file(p):
                touched.append(p)
    print(f"Patched {len(touched)} files:")
    for p in touched:
        print(f"  {p}")


if __name__ == "__main__":
    main()
