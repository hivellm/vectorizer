#!/usr/bin/env python3
"""Turn benchmark result files into a comparison — or refuse to.

The rule this exists to enforce: **a latency number is meaningless below a
recall floor.** The report this work replaces declared Vectorizer the search
winner at 5.31x with Precision@10 of 0.00%. A search that returns nothing
relevant is arbitrarily fast; comparing its latency to an engine that actually
found the neighbours compares two different operations.

So the gate is not a column in the output, it is a precondition for producing
output at all. A run below the floor prints what failed and exits non-zero.

    python benchmarks/external/report.py                       # gate + table
    python benchmarks/external/report.py --min-recall 0.95
    python benchmarks/external/report.py --results-dir some/dir

Reads the JSON the framework writes to `.work/results/`:
`{"params": {engine, dataset, parallel, ...}, "results": {mean_precisions,
rps, mean_time, p95_time, p99_time, ...}}`.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

HERE = Path(__file__).parent.resolve()
DEFAULT_RESULTS = HERE / ".work" / "results"

# Below this, a latency comparison is not a slower or faster engine, it is a
# broken measurement. 0.9 is deliberately lenient — real ANN configurations
# trade recall for speed, and this floor is meant to catch "the client is
# wrong", not "this config favours throughput".
DEFAULT_MIN_RECALL = 0.9


@dataclass(frozen=True)
class Run:
    engine: str
    experiment: str
    dataset: str
    parallel: int
    recall: float
    rps: float
    mean_ms: float
    p95_ms: float
    p99_ms: float
    source: Path

    @property
    def label(self) -> str:
        return f"{self.experiment} (parallel={self.parallel})"


def load_runs(results_dir: Path) -> list[Run]:
    runs: list[Run] = []
    for path in sorted(results_dir.glob("*-search-*.json")):
        try:
            blob = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise SystemExit(f"{path.name}: not valid JSON ({exc})") from exc

        params = blob.get("params", {})
        results = blob.get("results", {})

        missing = [
            key for key in ("mean_precisions", "rps", "mean_time") if key not in results
        ]
        if missing:
            # A result file without recall cannot be gated, and a run that
            # cannot be gated must not be reported — that is the whole point.
            raise SystemExit(
                f"{path.name}: missing {', '.join(missing)} in `results`. "
                "This file cannot be checked against the recall floor, so it "
                "will not be summarised."
            )

        runs.append(
            Run(
                engine=params.get("engine", "?"),
                experiment=params.get("experiment", path.stem),
                dataset=params.get("dataset", "?"),
                parallel=int(params.get("parallel", 1)),
                recall=float(results["mean_precisions"]),
                rps=float(results["rps"]),
                mean_ms=float(results["mean_time"]) * 1000,
                p95_ms=float(results.get("p95_time", 0.0)) * 1000,
                p99_ms=float(results.get("p99_time", 0.0)) * 1000,
                source=path,
            )
        )
    return runs


def check_recall(runs: Iterable[Run], floor: float) -> list[Run]:
    """Return the runs that fall below the floor."""
    return [run for run in runs if run.recall < floor]


def format_table(runs: list[Run]) -> str:
    header = f"| {'engine':<12} | {'config':<34} | {'recall':>7} | {'rps':>10} | {'mean ms':>8} | {'p95 ms':>8} | {'p99 ms':>8} |"
    sep = f"|{'-' * 14}|{'-' * 36}|{'-' * 9}|{'-' * 12}|{'-' * 10}|{'-' * 10}|{'-' * 10}|"
    lines = [header, sep]
    for run in sorted(runs, key=lambda r: (r.dataset, -r.rps)):
        lines.append(
            f"| {run.engine:<12} | {run.label:<34} | {run.recall:>6.2%} | "
            f"{run.rps:>10.1f} | {run.mean_ms:>8.3f} | {run.p95_ms:>8.3f} | {run.p99_ms:>8.3f} |"
        )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results-dir", type=Path, default=DEFAULT_RESULTS)
    parser.add_argument("--min-recall", type=float, default=DEFAULT_MIN_RECALL)
    args = parser.parse_args()

    if not args.results_dir.exists():
        raise SystemExit(
            f"no results at {args.results_dir} — run the benchmark first "
            "(see benchmarks/external/README.md)"
        )

    runs = load_runs(args.results_dir)
    if not runs:
        raise SystemExit(f"no search results found in {args.results_dir}")

    datasets = {run.dataset for run in runs}
    engines = {run.engine for run in runs}

    failures = check_recall(runs, args.min_recall)
    if failures:
        print(
            f"REFUSING to publish a comparison: {len(failures)} run(s) below the "
            f"{args.min_recall:.0%} recall floor.\n",
            file=sys.stderr,
        )
        for run in failures:
            print(
                f"  {run.engine:<12} {run.label:<34} recall={run.recall:.2%}  "
                f"({run.source.name})",
                file=sys.stderr,
            )
        print(
            "\nA latency number from these runs would not mean the engine is fast.\n"
            "Recall this low usually means the client is wrong rather than the\n"
            "engine — check the id round-trip first: the framework scores with\n"
            "len(returned_ids & expected_result[:top]) / top, so returning the\n"
            "engine's own ids instead of the dataset's yields exactly 0.00%.",
            file=sys.stderr,
        )
        return 1

    print(f"dataset(s): {', '.join(sorted(datasets))}")
    print(f"engines   : {', '.join(sorted(engines))}")
    print(f"recall floor: {args.min_recall:.0%} — all {len(runs)} run(s) pass\n")
    print(format_table(runs))

    if len(engines) == 1:
        print(
            f"\nNote: only one engine present ({next(iter(engines))}). This is a "
            "baseline, not a comparison — run the others before quoting it."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
