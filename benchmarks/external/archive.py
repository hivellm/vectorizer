#!/usr/bin/env python3
"""Snapshot a benchmark run into `results/` with what it takes to read it.

A latency number is only interpretable next to the things that produced it:
which build of each engine, on what hardware, under which caps. Recorded
automatically rather than typed into a commit message, because the one detail
nobody writes down by hand is the one that turns out to matter — the retracted
`qdrant_comparison_2025-11-24_*` report named neither engine version nor any
resource limit, so there is now no way to tell what it actually measured.

    python benchmarks/external/archive.py --label glove-4-engines

Writes `results/<timestamp>-<label>/` containing every raw result file plus a
`metadata.json`. It refuses to run if the recall gate would refuse: an archived
run is a published run, and the gate is not advisory.
"""

from __future__ import annotations

import argparse
import json
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

HERE = Path(__file__).parent.resolve()
WORK_RESULTS = HERE / ".work" / "results"
ARCHIVE_ROOT = HERE / "results"
COMPOSE = HERE / "docker-compose.yml"


def engine_versions() -> dict[str, str]:
    """The image each engine actually ran, read from the live containers.

    From `docker compose ps`, not from the compose file: the file says what was
    requested, the containers say what is running. A stack left up from an
    earlier edit would otherwise be recorded as whatever the file says today.
    """
    try:
        out = subprocess.run(
            ["docker", "compose", "-f", str(COMPOSE), "ps", "--format", "json"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True,
        ).stdout
    except (subprocess.CalledProcessError, FileNotFoundError) as exc:
        return {"error": f"could not read running containers: {exc}"}

    versions: dict[str, str] = {}
    for line in out.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        service = row.get("Service")
        if service:
            versions[service] = row.get("Image", "unknown")
    return versions


def resource_limits() -> dict[str, str]:
    """The caps every engine ran under, resolved the way compose resolves them."""
    import os

    return {
        "cpus": os.getenv("BENCH_CPUS", "4"),
        "memory": os.getenv("BENCH_MEMORY", "8g"),
        "note": (
            "Applied to all four services through one YAML anchor. Unequal caps "
            "invalidate the comparison before it starts."
        ),
    }


def host_specs() -> dict[str, object]:
    return {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "python": platform.python_version(),
        "cpu_count": __import__("os").cpu_count(),
    }


def upstream_pin() -> dict[str, str]:
    return json.loads((HERE / "upstream.json").read_text(encoding="utf-8"))


def gate_passes(min_recall: float) -> tuple[bool, str]:
    """Run the reporter and report whether it would publish."""
    proc = subprocess.run(
        [sys.executable, str(HERE / "report.py"), "--min-recall", str(min_recall)],
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    return proc.returncode == 0, (proc.stdout or "") + (proc.stderr or "")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--label", required=True, help="short name for this run")
    parser.add_argument("--min-recall", type=float, default=0.9)
    args = parser.parse_args()

    results = sorted(WORK_RESULTS.glob("*.json"))
    if not results:
        raise SystemExit(f"no results in {WORK_RESULTS} — run the benchmark first")

    passes, report_text = gate_passes(args.min_recall)
    if not passes:
        print(report_text, file=sys.stderr)
        raise SystemExit(
            "\nRefusing to archive: the recall gate would refuse to publish this "
            "run. Archiving it would put the numbers in the repository anyway, "
            "where the gate no longer stands between them and a reader."
        )

    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H-%M-%SZ")
    target = ARCHIVE_ROOT / f"{stamp}-{args.label}"
    (target / "raw").mkdir(parents=True, exist_ok=True)

    for path in results:
        shutil.copy2(path, target / "raw" / path.name)

    metadata = {
        "label": args.label,
        "recorded_at": stamp,
        "recall_floor": args.min_recall,
        "engines": engine_versions(),
        "resource_limits": resource_limits(),
        "host": host_specs(),
        "upstream_harness": upstream_pin(),
        "raw_result_files": len(results),
    }
    (target / "metadata.json").write_text(
        json.dumps(metadata, indent=2) + "\n", encoding="utf-8"
    )
    (target / "report.md").write_text(report_text, encoding="utf-8")

    print(f"archived {len(results)} result file(s) to {target}")
    print("  metadata.json  — engine images, host, limits, harness pin")
    print("  report.md      — the gated comparison as it stood")
    return 0


if __name__ == "__main__":
    sys.exit(main())
