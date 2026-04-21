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
    file_level_allowed = []  # allow at the very top of the file

    for root, _, files in os.walk("src"):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(root, f)
            with open(p, "r", encoding="utf-8", errors="ignore") as fh:
                content = fh.read()
            lines = content.split("\n")
            unwrap_lines = []
            for i, line in enumerate(lines, start=1):
                if re.search(r"\.unwrap\(\)|\.expect\(", line):
                    unwrap_lines.append(i)
            if not unwrap_lines:
                continue

            # File-level inner attribute (#![allow(...)]) covers the whole file.
            head = "\n".join(lines[:30])
            has_file_allow = bool(
                re.search(
                    r"^#!\s*\[allow\([^\]]*clippy::(unwrap_used|expect_used)",
                    head,
                    re.MULTILINE,
                )
            )
            if has_file_allow:
                file_level_allowed.append((p, len(unwrap_lines)))
                continue

            if is_test_file_by_name(p):
                test_only_files.append((p, len(unwrap_lines)))
                continue

            test_ranges = []
            for m in re.finditer(
                r"^#\[cfg\(test\)\][^\n]*\n(?:#\[[^\n]*\]\n)*mod\s+\w+\s*\{",
                content,
                re.MULTILINE,
            ):
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

            # Function-level allows: walk back from the unwrap line to find
            # the nearest enclosing `fn `/`pub fn`/`async fn` declaration,
            # then check the attribute lines immediately above that fn for
            # an `#[allow(... clippy::unwrap_used ...)]` annotation. Also
            # honor an inline `#[allow(...)]` within 12 lines above the call.
            fn_pattern = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(async\s+)?fn\s+\w+")

            def is_attr_allowed(line_no: int) -> bool:
                # Quick local check
                start = max(0, line_no - 12)
                window = "\n".join(lines[start : line_no - 1])
                if re.search(
                    r"#\s*\[allow\([^\]]*clippy::(unwrap_used|expect_used)",
                    window,
                ):
                    return True
                # Walk back to find enclosing fn line
                for i in range(line_no - 1, -1, -1):
                    if fn_pattern.match(lines[i]):
                        # Collect contiguous attr lines above this fn
                        j = i - 1
                        while j >= 0 and (
                            lines[j].lstrip().startswith("#[")
                            or lines[j].lstrip().startswith("///")
                            or lines[j].strip() == ""
                        ):
                            if re.search(
                                r"#\s*\[allow\([^\]]*clippy::(unwrap_used|expect_used)",
                                lines[j],
                            ):
                                return True
                            j -= 1
                        return False
                return False

            prod_unwraps = [
                line
                for line in unwrap_lines
                if not any(s <= line <= e for s, e in test_ranges)
                and not is_attr_allowed(line)
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
