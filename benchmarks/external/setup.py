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
import tomllib
from pathlib import Path

HERE = Path(__file__).parent.resolve()
WORK = HERE / ".work"
OVERLAY = HERE / "overlay"
VENV = WORK / ".venv"
UPSTREAM = json.loads((HERE / "upstream.json").read_text(encoding="utf-8"))

# Upstream declares `python = ">=3.10,<3.13"`. Honour it rather than forcing a
# newer interpreter: the harness is what produces the numbers, and running it
# outside its supported range makes any surprise unattributable.
PYTHON_VERSION = "3.12"

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


def locked_requirements() -> list[str]:
    """Every runtime dependency, at the exact version `poetry.lock` pins.

    Read from the lock rather than `pyproject.toml` on purpose. The manifest
    asks for `qdrant-client` from a git *branch*, which resolves to whatever
    that branch points at today — the same silent drift that pinning the
    upstream commit exists to prevent. The lock records the commit the harness
    was actually developed against, so this reads that instead.

    The dev group is skipped except for pytest, which `tests/` needs: the id
    round-trip tests import the engine client, and that import only resolves
    once upstream's own dependencies are present. pre-commit is left out.
    """
    lock = tomllib.loads((WORK / "poetry.lock").read_text(encoding="utf-8"))
    requirements: list[str] = []
    for package in lock["package"]:
        if "main" not in package.get("groups", ["main"]):
            continue
        source = package.get("source", {})
        if source.get("type") == "git":
            requirement = (
                f"{package['name']} @ git+{source['url']}@{source['resolved_reference']}"
            )
        else:
            requirement = f"{package['name']}=={package['version']}"

        # A package can appear more than once, one row per interpreter range —
        # numpy is locked at 2.2.6 for 3.10 and 2.4.1 for 3.11+. Dropping the
        # marker makes those rows contradict each other and the install fails
        # to resolve, so carry it through.
        #
        # `markers` is a bare string for a package that belongs to one group
        # and a per-group mapping for one that spans several (colorama is in
        # both main and dev, with a different marker in each). Take the main
        # group's, since that is the only group installed here.
        marker = package.get("markers")
        if isinstance(marker, dict):
            marker = marker.get("main")
        if marker:
            requirement = f"{requirement} ; {marker}"
        requirements.append(requirement)

    # From the dev group, and only this: tests/test_id_roundtrip.py imports the
    # engine client, which pulls upstream's runtime deps, so it has to run in
    # this venv rather than the ambient interpreter.
    for package in lock["package"]:
        if package["name"] == "pytest" and "dev" in package.get("groups", []):
            requirements.append(f"pytest=={package['version']}")
            break

    return requirements


def build_venv() -> None:
    """Create `.work/.venv` and install the locked dependencies into it.

    Lives here rather than in a README instruction because a run that installs
    a different dependency set is a different benchmark. `uv` is used for
    speed; `poetry` would work too, but it is not what this repo has.
    """
    if not (WORK / "poetry.lock").exists():
        raise SystemExit(f"missing {WORK / 'poetry.lock'} — materialise the workspace first")

    # `--clear` so a rerun is a rebuild, not an error. This is a build
    # directory; a half-installed venv left over from a failed run is exactly
    # what a rerun is trying to escape.
    #
    # But `--clear` deletes before it recreates, and on Windows the delete
    # fails partway through if anything is running out of this venv — leaving
    # it *worse* than before. That happened once mid-benchmark: the run's
    # upload had finished, the delete stripped `certifi` out from under it, and
    # the search workers then spawned, died on import, and hung the run with
    # every engine sitting idle and no error anywhere. Say what happened rather
    # than surfacing a bare "Access denied".
    print(f"creating venv at {VENV} (python {PYTHON_VERSION})")
    try:
        run("uv", "venv", "--clear", "--python", PYTHON_VERSION, str(VENV))
    except SystemExit as exc:
        raise SystemExit(
            f"{exc}\n\n"
            "The venv could not be replaced, most likely because a benchmark "
            "run is still using it. It may now be PARTIALLY DELETED and unable "
            "to import its dependencies — a run started against it will fail in "
            "spawned workers and appear to hang.\n"
            "Stop every process using it, then run `setup.py --venv` again."
        ) from exc

    requirements = locked_requirements()
    requirements_file = WORK / ".locked-requirements.txt"
    requirements_file.write_text("\n".join(requirements) + "\n", encoding="utf-8")

    print(f"installing {len(requirements)} locked package(s)")
    run(
        "uv",
        "pip",
        "install",
        "--python",
        str(venv_python()),
        "-r",
        str(requirements_file),
    )
    print(f"venv ready: {venv_python()}")


def venv_python() -> Path:
    """The interpreter inside `.work/.venv`, on either platform."""
    if sys.platform == "win32":
        return VENV / "Scripts" / "python.exe"
    return VENV / "bin" / "python"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify the workspace without cloning or writing",
    )
    parser.add_argument(
        "--venv",
        action="store_true",
        help="create .work/.venv and install the locked dependencies into it",
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
        print(f"venv            : {'yes' if venv_python().exists() else 'no'}")
        return 0 if (pinned and registered) else 1

    if args.venv:
        build_venv()
        return 0

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
