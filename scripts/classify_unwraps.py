"""Classify unwrap/expect calls in src/ into test-only vs production code.

Output:
- test-only count + file list (need crate-level cfg_attr or per-file allow)
- prod count + file list (need actual fixes or per-call SAFE comments)
"""
import os
import re


def is_test_file_by_name(p: str) -> bool:
    name = os.path.basename(p)
    norm = p.replace("\\", "/")
    return name == "tests.rs" or name.endswith("_tests.rs") or "/tests/" in norm


def main() -> None:
    test_only_files = []
    prod_files = []

    for root, _, files in os.walk("src"):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(root, f)
            with open(p, "r", encoding="utf-8", errors="ignore") as fh:
                content = fh.read()
            unwrap_lines = []
            for i, line in enumerate(content.split("\n"), start=1):
                if re.search(r"\.unwrap\(\)|\.expect\(", line):
                    unwrap_lines.append(i)
            if not unwrap_lines:
                continue
            if is_test_file_by_name(p):
                test_only_files.append((p, len(unwrap_lines)))
                continue

            test_ranges = []
            for m in re.finditer(r"^#\[cfg\(test\)\][^\n]*\n(?:#\[[^\n]*\]\n)*mod\s+tests\s*\{", content, re.MULTILINE):
                test_start = content[: m.start()].count("\n") + 1
                after = content[m.end():]
                depth = 1
                for j, ch in enumerate(after):
                    if ch == "{":
                        depth += 1
                    elif ch == "}":
                        depth -= 1
                        if depth == 0:
                            test_end = content[: m.end() + j].count("\n") + 1
                            test_ranges.append((test_start, test_end))
                            break

            prod_unwraps = [
                line
                for line in unwrap_lines
                if not any(s <= line <= e for s, e in test_ranges)
            ]
            if not prod_unwraps:
                test_only_files.append((p, len(unwrap_lines)))
            else:
                prod_files.append((p, len(unwrap_lines), len(prod_unwraps)))

    print("TEST-ONLY FILES (count):", sum(n for _, n in test_only_files), "in", len(test_only_files), "files")
    print()
    print("PROD FILES with unwraps (need fixes):")
    for p, total, prod in sorted(prod_files, key=lambda x: -x[2])[:30]:
        print(f"  {prod:4} prod / {total:4} total  {p}")
    print()
    print("Total prod unwraps:", sum(p for _, _, p in prod_files))
    print("Total prod files:", len(prod_files))


if __name__ == "__main__":
    main()
