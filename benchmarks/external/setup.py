#!/usr/bin/env python3
"""Materialise the benchmark workspace: upstream at a pinned commit + our overlay.

`qdrant/vector-db-benchmark` is the framework Qdrant, Redis and Weaviate all
publish against. Rather than vendoring it (a fork rots) or writing another
in-house harness (the previous one published a 5.31x "win" at 0.00% recall),
this clones it at the commit pinned in `upstream.json` and copies our engine
client on top.

Everything we author lives in `overlay/` and is committed. `.work/` is the
materialised clone and is gitignored — treat it as a build directory.

    python benchmarks/external/setup.py            # clone/refresh + overlay
    python benchmarks/external/setup.py --check    # verify without changing

Registration into `engine/clients/client_factory.py` is a scripted edit rather
than a vendored copy of that file. If upstream restructures the registry, this
fails loudly at setup instead of silently running against a stale fork.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent.resolve()
WORK = HERE / ".work"
OVERLAY = HERE / "overlay"
UPSTREAM = json.loads((HERE / "upstream.json").read_text(encoding="utf-8"))

ENGINE_NAME = "vectorizer"

# (dict name, class name) — the three registries in client_factory.py.
REGISTRY_ENTRIES = [
    ("ENGINE_CONFIGURATORS", "VectorizerConfigurator"),
    ("ENGINE_UPLOADERS", "VectorizerUploader"),
    ("ENGINE_SEARCHERS", "VectorizerSearcher"),
]

IMPORT_BLOCK = (
    "from engine.clients.vectorizer import (\n"
    "    VectorizerConfigurator,\n"
    "    VectorizerSearcher,\n"
    "    VectorizerUploader,\n"
    ")\n"
)

# Anchor the import insertion on an upstream import that has been stable and is
# alphabetically last, so our block lands after the existing ones.
IMPORT_ANCHOR = "from engine.clients.weaviate import ("


def run(*args: str, cwd: Path | None = None) -> str:
    """Run a command, raising with its stderr attached when it fails."""
    proc = subprocess.run(
        args, cwd=cwd, capture_output=True, text=True, encoding="utf-8"
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"command failed: {' '.join(args)}\n"
            f"  exit: {proc.returncode}\n"
            f"  stderr: {proc.stderr.strip()}"
        )
    return proc.stdout.strip()


def materialise_upstream() -> None:
    """Clone (or fetch) the framework and check out the pinned commit."""
    commit = UPSTREAM["commit"]

    if not (WORK / ".git").exists():
        if WORK.exists():
            shutil.rmtree(WORK)
        print(f"cloning {UPSTREAM['repository']}")
        run("git", "clone", "--filter=blob:none", UPSTREAM["repository"], str(WORK))

    current = run("git", "rev-parse", "HEAD", cwd=WORK)
    if current == commit:
        print(f"upstream already at pinned commit {commit[:8]}")
        return

    print(f"checking out pinned commit {commit[:8]} (was {current[:8]})")
    run("git", "fetch", "origin", commit, cwd=WORK)
    run("git", "checkout", "--force", commit, cwd=WORK)


def copy_overlay() -> list[Path]:
    """Copy every file under `overlay/` into the workspace, preserving layout."""
    copied = []
    for src in sorted(OVERLAY.rglob("*")):
        if src.is_dir() or "__pycache__" in src.parts:
            continue
        dest = WORK / src.relative_to(OVERLAY)
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
        copied.append(dest.relative_to(WORK))
    return copied


def register_engine(check_only: bool = False) -> bool:
    """Add our engine to the three registries in client_factory.py.

    Returns True when the file already carried the registration.
    """
    factory = WORK / "engine" / "clients" / "client_factory.py"
    if not factory.exists():
        raise SystemExit(
            f"missing {factory.relative_to(WORK)} — upstream layout changed; "
            "the pin in upstream.json needs revisiting"
        )

    source = factory.read_text(encoding="utf-8")
    if f'"{ENGINE_NAME}"' in source:
        return True
    if check_only:
        return False

    if IMPORT_ANCHOR not in source:
        raise SystemExit(
            f"could not find the import anchor {IMPORT_ANCHOR!r} in "
            "client_factory.py. Upstream restructured its imports: re-read the "
            "file and update IMPORT_ANCHOR rather than loosening this check."
        )

    # Insert our import after the whole anchor statement, not merely after the
    # matched line — the upstream imports are multi-line parenthesised blocks.
    anchor_at = source.index(IMPORT_ANCHOR)
    close_at = source.index(")\n", anchor_at) + len(")\n")
    source = source[:close_at] + IMPORT_BLOCK + source[close_at:]

    for dict_name, class_name in REGISTRY_ENTRIES:
        marker = f"{dict_name} = {{"
        if marker not in source:
            raise SystemExit(
                f"could not find {marker!r} in client_factory.py — upstream "
                "renamed a registry; update REGISTRY_ENTRIES."
            )
        at = source.index(marker) + len(marker)
        source = source[:at] + f'\n    "{ENGINE_NAME}": {class_name},' + source[at:]

    factory.write_text(source, encoding="utf-8")
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify the workspace without cloning or writing",
    )
    args = parser.parse_args()

    if args.check:
        if not (WORK / ".git").exists():
            print("workspace not materialised — run without --check")
            return 1
        at = run("git", "rev-parse", "HEAD", cwd=WORK)
        pinned = at == UPSTREAM["commit"]
        registered = register_engine(check_only=True)
        print(f"upstream commit : {at[:8]} ({'pinned' if pinned else 'DRIFTED'})")
        print(f"engine registered: {'yes' if registered else 'NO'}")
        return 0 if (pinned and registered) else 1

    if not OVERLAY.exists():
        raise SystemExit(f"missing {OVERLAY} — nothing to overlay")

    materialise_upstream()
    copied = copy_overlay()
    already = register_engine()

    print(f"\noverlaid {len(copied)} file(s):")
    for path in copied:
        print(f"  {path.as_posix()}")
    print(f"engine registration: {'already present' if already else 'applied'}")
    print(
        f"\nworkspace ready at {WORK}\n"
        "next: install deps there (poetry install) and run, e.g.\n"
        f"  python run.py --engines '{ENGINE_NAME}-*' --datasets glove-100-angular"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
