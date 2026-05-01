"""Retry-After header parser tests for the Python SDK (issue #263, phase9 §7).

The full retry loop is exercised end-to-end at the server level by
``crates/vectorizer-server/tests/backpressure_429.rs``; here we only
lock in the value-parsing edges that determine how aggressively the
SDK backs off.

These constants are kept in sync with ``utils/http_client.py``:

  - missing/unparseable header  -> 1 s default
  - ``Retry-After: 0``          -> 1 s default (never busy-loop)
  - capped at 30 s so a misconfigured server cannot pin the client
"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from utils.http_client import _parse_retry_after  # type: ignore[import-not-found]


class TestRetryAfterParse(unittest.TestCase):
    def test_missing_header_returns_default(self) -> None:
        self.assertEqual(_parse_retry_after(None), 1)

    def test_empty_or_whitespace_returns_default(self) -> None:
        self.assertEqual(_parse_retry_after(""), 1)
        self.assertEqual(_parse_retry_after("   "), 1)

    def test_zero_returns_default_to_avoid_busy_loop(self) -> None:
        self.assertEqual(_parse_retry_after("0"), 1)

    def test_unparseable_string_returns_default(self) -> None:
        self.assertEqual(_parse_retry_after("not-a-number"), 1)

    def test_small_values_pass_through_verbatim(self) -> None:
        self.assertEqual(_parse_retry_after("3"), 3)
        self.assertEqual(_parse_retry_after("7"), 7)
        self.assertEqual(_parse_retry_after(" 5 "), 5)

    def test_large_values_are_capped_at_30s(self) -> None:
        # If this assertion ever flips, audit _RETRY_AFTER_MAX_SECONDS
        # in utils/http_client.py first.
        self.assertEqual(_parse_retry_after("3600"), 30)
        self.assertEqual(_parse_retry_after("31"), 30)


if __name__ == "__main__":
    unittest.main()
