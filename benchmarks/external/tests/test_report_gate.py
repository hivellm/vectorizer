"""The recall gate refuses to publish a comparison it cannot trust.

These tests encode the failure this whole task exists to prevent: the previous
in-tree report declared a 5.31x search win alongside Precision@10 of 0.00%.
The gate has to reject that shape, and reject it by *not producing output*,
not by adding a caveat somewhere in the output.

Run from the repository root:

    python -m pytest benchmarks/external/tests -q
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

REPORT = Path(__file__).resolve().parents[1] / "report.py"


def write_result(
    directory: Path,
    *,
    engine: str,
    recall: float,
    rps: float = 1000.0,
    mean_time: float = 0.001,
    parallel: int = 1,
    dataset: str = "glove-100-angular",
) -> Path:
    # `parallel` is part of the name because a single engine legitimately
    # produces one result per concurrency level. Keying only on engine+dataset
    # made a second write silently overwrite the first, so a test that set up
    # three runs would quietly assert against two.
    path = (
        directory
        / f"{engine}-{dataset}-search-p{parallel}-2026-08-10-00-00-00.json"
    )
    path.write_text(
        json.dumps(
            {
                "params": {
                    "engine": engine,
                    "experiment": f"{engine}-default",
                    "dataset": dataset,
                    "parallel": parallel,
                },
                "results": {
                    "mean_precisions": recall,
                    "rps": rps,
                    "mean_time": mean_time,
                    "p95_time": mean_time * 2,
                    "p99_time": mean_time * 3,
                },
            }
        ),
        encoding="utf-8",
    )
    return path


def run_report(results_dir: Path, *extra: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(REPORT), "--results-dir", str(results_dir), *extra],
        capture_output=True,
        text=True,
        encoding="utf-8",
    )


def test_zero_recall_is_refused_however_fast(tmp_path: Path):
    """The exact shape of the report being retracted: fast and finding nothing."""
    write_result(tmp_path, engine="vectorizer", recall=0.0, rps=6285.0, mean_time=0.00016)
    write_result(tmp_path, engine="qdrant", recall=1.0, rps=1183.0, mean_time=0.00084)

    proc = run_report(tmp_path)

    assert proc.returncode != 0, "a 0% recall run must not produce a comparison"
    assert "REFUSING" in proc.stderr
    assert "vectorizer" in proc.stderr
    # The table must not appear at all — a caveat next to a published number is
    # what produced the misleading report in the first place.
    assert "rps" not in proc.stdout.lower()


def test_borderline_recall_below_floor_is_refused(tmp_path: Path):
    write_result(tmp_path, engine="vectorizer", recall=0.89)
    proc = run_report(tmp_path)
    assert proc.returncode != 0
    assert "89" in proc.stderr or "89.00%" in proc.stderr


def test_passing_runs_produce_a_table(tmp_path: Path):
    write_result(tmp_path, engine="vectorizer", recall=0.97, rps=4000.0)
    write_result(tmp_path, engine="qdrant", recall=0.99, rps=3500.0)

    proc = run_report(tmp_path)

    assert proc.returncode == 0, proc.stderr
    assert "vectorizer" in proc.stdout
    assert "qdrant" in proc.stdout
    assert "recall" in proc.stdout


def test_floor_is_configurable(tmp_path: Path):
    write_result(tmp_path, engine="vectorizer", recall=0.80)

    assert run_report(tmp_path).returncode != 0
    assert run_report(tmp_path, "--min-recall", "0.75").returncode == 0


def test_single_engine_is_labelled_a_baseline_not_a_comparison(tmp_path: Path):
    write_result(tmp_path, engine="vectorizer", recall=0.97)
    proc = run_report(tmp_path)
    assert proc.returncode == 0
    assert "baseline" in proc.stdout.lower(), (
        "one engine is not a comparison; the output has to say so or it will be "
        "quoted as one"
    )


def test_result_without_recall_is_rejected_rather_than_summarised(tmp_path: Path):
    path = tmp_path / "vectorizer-glove-100-angular-search-0-2026-08-10-00-00-00.json"
    path.write_text(
        json.dumps(
            {
                "params": {"engine": "vectorizer", "dataset": "glove-100-angular"},
                "results": {"rps": 9999.0, "mean_time": 0.0001},
            }
        ),
        encoding="utf-8",
    )

    proc = run_report(tmp_path)

    # No recall means the run cannot be gated, and an ungateable run must not
    # be summarised — otherwise the gate is trivially bypassed by a result file
    # that simply omits the field.
    assert proc.returncode != 0
    assert "mean_precisions" in proc.stderr


def test_no_results_directory_is_an_error_not_an_empty_table(tmp_path: Path):
    proc = run_report(tmp_path / "does-not-exist")
    assert proc.returncode != 0
    assert "no results" in proc.stderr.lower()


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-q"]))


def test_engines_at_different_concurrency_are_not_ranked_against_each_other(
    tmp_path: Path,
):
    """The other way to publish a true-but-fictional comparison.

    Throughput scales with concurrency, so a single list ordered by rps puts
    whichever engine was measured at the highest parallelism on top — and the
    top row reads as the winner. Every number in it is real; the comparison is
    not. Grouping by parallelism is what keeps the reader from making it.
    """
    write_result(tmp_path, engine="vectorizer", recall=0.95, rps=9000.0, parallel=16)
    write_result(tmp_path, engine="qdrant", recall=0.99, rps=4000.0, parallel=8)

    result = run_report(tmp_path)
    assert result.returncode == 0, result.stderr

    # Each concurrency level gets its own table, so the fast-at-16 row is never
    # printed directly above the slower-at-8 one as though they were rivals.
    assert "@ parallel=16" in result.stdout
    assert "@ parallel=8" in result.stdout

    lines = [ln for ln in result.stdout.splitlines() if ln.startswith("| vectorizer") or ln.startswith("| qdrant")]
    assert len(lines) == 2
    vector_at, qdrant_at = (result.stdout.index(ln) for ln in lines)
    between = result.stdout[min(vector_at, qdrant_at) : max(vector_at, qdrant_at)]
    assert "parallel=" in between, (
        "the two engines were printed in one uninterrupted ranking; a reader "
        "comparing adjacent rows would be comparing different concurrencies"
    )


def test_an_incomplete_concurrency_group_says_so(tmp_path: Path):
    """A level only one engine ran at is a partial field, not a comparison."""
    write_result(tmp_path, engine="vectorizer", recall=0.95, rps=9000.0, parallel=16)
    write_result(tmp_path, engine="vectorizer", recall=0.95, rps=5000.0, parallel=8)
    write_result(tmp_path, engine="qdrant", recall=0.99, rps=4000.0, parallel=8)

    result = run_report(tmp_path)
    assert result.returncode == 0, result.stderr
    assert "incomplete" in result.stdout, (
        "parallel=16 has no qdrant run; presenting it as a comparison table "
        "without saying so is how a solo number gets quoted as a win"
    )
    assert "no run for: qdrant" in result.stdout
