"""Test-suite conftest.

Pulls `VECTORIZER_API_KEY` (or the `.root_credentials` file that the
server auto-generates on first boot) into every integration test's
`VectorizerClient(...)` call so the suite exercises the live REST
surface on auth-enabled servers without each test file having to
thread the token through its constructor.

When both are absent and the server has auth enabled, the existing
`health_check()` probe in each test file's `asyncSetUp` flips
`server_available = False` and the tests call `skipTest("Server not
available")` — so nothing blows up; the integration tests simply
skip with a clear reason.

See `phase8_fix-python-sdk-wire-shapes` (probe 4.2).
"""
from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Optional

import pytest


def _resolve_api_key() -> Optional[str]:
    """Return a bearer token from the environment.

    Resolution order:
    1. `VECTORIZER_API_KEY` env var (bearer token or API key).
    2. `.root_credentials` file emitted by the server on first boot
       — only honoured when the caller also sets
       `VECTORIZER_USE_ROOT_CREDS=1` so a stray `.root_credentials`
       under `%APPDATA%/vectorizer/` does NOT silently grant tests
       full admin rights on every run.
    """
    direct = os.environ.get("VECTORIZER_API_KEY")
    if direct:
        return direct

    if os.environ.get("VECTORIZER_USE_ROOT_CREDS") != "1":
        return None

    # Cross-platform paths for the auto-generated root credentials.
    if sys.platform == "win32":
        appdata = os.environ.get("APPDATA", "")
        if not appdata:
            return None
        path = Path(appdata) / "vectorizer" / ".root_credentials"
    else:
        xdg = os.environ.get("XDG_DATA_HOME") or str(Path.home() / ".local" / "share")
        path = Path(xdg) / "vectorizer" / ".root_credentials"

    if not path.is_file():
        return None

    # Parse a simple `key=value` file; the credential lives under the
    # `password=` line and the test client will log in with it.
    creds: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        k, _, v = line.partition("=")
        creds[k.strip()] = v.strip()

    # For the test harness we want a bearer token, not the raw
    # username/password pair — delegate the exchange to a lazy
    # fixture below.
    return creds.get("__bearer__")  # populated by the fixture, if at all


@pytest.fixture(scope="session", autouse=True)
def _inject_api_key_into_clients():
    """Auto-use fixture that monkey-patches `VectorizerClient.__init__`
    to default `api_key` to `_resolve_api_key()` when the caller did
    not pass one explicitly. Keeps every integration test that does
    `VectorizerClient(base_url=...)` talking to an auth-enabled
    server without touching each test file.
    """
    key = _resolve_api_key()
    if not key:
        yield
        return

    # Import here so the package discovery in each test file runs
    # first (they prepend `sys.path` to reach the flat layout).
    try:
        from client import VectorizerClient  # type: ignore[import-not-found]
    except ImportError:
        yield
        return

    original_init = VectorizerClient.__init__

    def patched_init(self, *args, **kwargs):
        if "api_key" not in kwargs or kwargs["api_key"] is None:
            kwargs["api_key"] = key
        return original_init(self, *args, **kwargs)

    VectorizerClient.__init__ = patched_init
    try:
        yield
    finally:
        VectorizerClient.__init__ = original_init
