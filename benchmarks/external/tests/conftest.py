"""Put the materialised harness on `sys.path` for tests that need it.

The engine client subclasses upstream base classes (`BaseUploader`,
`BaseSearcher`) and takes upstream types (`Record`, `Query`), so a test of our
id mapping can only import it once `setup.py` has produced `.work/`. Tests that
need it ask for the `harness` fixture and skip when it is absent, so
`pytest tests/` stays runnable on a fresh checkout — `test_report_gate.py`
exercises `report.py`, which has no such dependency.

Run these under the harness venv, which has upstream's dependencies:

    .work/.venv/Scripts/python.exe -m pytest tests/
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

WORK = Path(__file__).resolve().parent.parent / ".work"


@pytest.fixture(scope="session")
def harness():
    """The materialised upstream tree, or a skip explaining how to get one."""
    if not (WORK / "engine" / "base_client").is_dir():
        pytest.skip(
            "harness not materialised — run `python setup.py` and "
            "`python setup.py --venv` in benchmarks/external"
        )
    if str(WORK) not in sys.path:
        sys.path.insert(0, str(WORK))
    try:
        import engine.base_client.upload  # noqa: F401
    except ImportError as exc:
        pytest.skip(f"harness dependencies missing ({exc}); run `setup.py --venv`")
    return WORK
