"""The id contract: an integer goes in, the same integer comes back.

This is the only logic in the engine client that is ours, and the only one that
fails *plausibly*. The framework scores a run with

    precision = len(returned_ids & query.expected_result[:top]) / top

so an id that does not survive the round trip scores 0.00 on every query — at
full speed, because the engine still answered. That reads like a fast database
with terrible recall rather than a broken client, which is exactly how the
retracted `qdrant_comparison_2025-11-24_*` report came to declare a 5.31x search
win at 0.00% recall.

Vectorizer's vector ids are strings and the dataset's are integers, so
`upload.py` writes `str(record.id)` and `search.py` reads `int(hit["id"])`. The
two are a pair; a test that only checked one half would pass while the round
trip was broken.

The transport is stubbed rather than pointed at a server, so these run in CI and
fail for one reason only. The end-to-end check is a `random-100` run, whose
ground truth makes precision exactly 1.0 — see the runbook.
"""

from __future__ import annotations

import pytest


@pytest.fixture
def client_module(harness, monkeypatch):
    """The overlaid engine client, with its HTTP transport replaced by a spy."""
    from engine.clients.vectorizer import search as search_mod
    from engine.clients.vectorizer import upload as upload_mod

    class SpyClient:
        """Captures request bodies and replays a canned response."""

        def __init__(self):
            self.requests: list[tuple[str, str, dict]] = []
            self.response: dict = {}

        def request(self, method, path, body=None):
            self.requests.append((method, path, body))
            return self.response

        def close(self):
            pass

    return upload_mod, search_mod, SpyClient


def test_integer_ids_are_stringified_on_the_way_in(client_module):
    from dataset_reader.base_reader import Record

    upload_mod, _, SpyClient = client_module
    spy = SpyClient()
    spy.response = {"inserted": 3, "failed": 0}
    upload_mod.VectorizerUploader.client = spy

    upload_mod.VectorizerUploader.upload_batch(
        [Record(id=i, vector=[0.1, 0.2], sparse_vector=None, metadata=None) for i in (0, 7, 42)]
    )

    _, path, body = spy.requests[0]
    assert path == "/insert_vectors"
    sent = [v["id"] for v in body["vectors"]]
    assert sent == ["0", "7", "42"], (
        "the server rejects non-string vector ids; sending the raw integer "
        f"fails the upload outright, got {sent!r}"
    )


def test_string_ids_are_parsed_back_to_integers_on_the_way_out(client_module):
    from dataset_reader.base_reader import Query

    _, search_mod, SpyClient = client_module
    spy = SpyClient()
    # What the server actually answers: ids as strings.
    spy.response = {
        "results": [
            {"id": "42", "score": 0.99},
            {"id": "7", "score": 0.51},
        ]
    }
    search_mod.VectorizerSearcher.client = spy

    got = search_mod.VectorizerSearcher.search_one(
        Query(vector=[0.1, 0.2], sparse_vector=None, meta_conditions=None,
              expected_result=[42, 7]), top=2
    )

    assert got == [(42, 0.99), (7, 0.51)], (
        "the framework intersects these ids with integer ground truth; strings "
        f"never match and score 0.00 at full speed, got {got!r}"
    )
    assert all(isinstance(hit_id, int) for hit_id, _ in got)


def test_ids_survive_the_full_trip_without_losing_precision(client_module):
    """Upload, then read back what the server would actually store.

    The stub echoes the *stored* id as a string, which is what the server does
    — it has no other representation. An echo that replayed the uploaded object
    unchanged would pass even with the stringify removed, so it would prove
    nothing about the pair.

    The large id is the point of this test rather than a bigger version of the
    two above: 2**53 + 1 is the first integer a float cannot represent, so any
    step that routes an id through a float — a JSON parser configured for
    floats, a careless `float(hit["id"])` — comes back off by one and scores
    zero against ground truth while looking entirely reasonable.
    """
    from dataset_reader.base_reader import Query, Record

    upload_mod, search_mod, SpyClient = client_module
    ids = [0, 1, 999, 2**53 + 1]

    upload_spy = SpyClient()
    upload_spy.response = {"inserted": len(ids), "failed": 0}
    upload_mod.VectorizerUploader.client = upload_spy
    upload_mod.VectorizerUploader.upload_batch(
        [Record(id=i, vector=[0.1], sparse_vector=None, metadata=None) for i in ids]
    )
    stored = [v["id"] for v in upload_spy.requests[0][2]["vectors"]]

    search_spy = SpyClient()
    search_spy.response = {
        "results": [{"id": str(s), "score": 1.0} for s in stored]
    }
    search_mod.VectorizerSearcher.client = search_spy

    returned = [
        hit_id
        for hit_id, _ in search_mod.VectorizerSearcher.search_one(
            Query(vector=[0.1], sparse_vector=None, meta_conditions=None,
                  expected_result=ids),
            top=len(ids),
        )
    ]
    assert returned == ids
    assert returned[-1] == 2**53 + 1, "an id that went through a float"


def test_a_partial_upload_raises_instead_of_lowering_recall(client_module):
    from dataset_reader.base_reader import Record

    upload_mod, _, SpyClient = client_module
    spy = SpyClient()
    # `/insert_vectors` answers 200 with a per-row breakdown, so a partial
    # failure is a *successful* HTTP call. Uploading fewer vectors than the
    # dataset holds lowers recall on every query afterwards, and the run would
    # report that as search quality.
    spy.response = {"inserted": 1, "failed": 1, "results": [{"error": "boom"}]}
    upload_mod.VectorizerUploader.client = spy

    with pytest.raises(RuntimeError, match="rejected by the server"):
        upload_mod.VectorizerUploader.upload_batch(
            [Record(id=i, vector=[0.1], sparse_vector=None, metadata=None) for i in (0, 1)]
        )


def test_a_top_above_the_server_cap_raises_instead_of_being_truncated(client_module):
    from dataset_reader.base_reader import Query

    _, search_mod, SpyClient = client_module
    spy = SpyClient()
    spy.response = {"results": []}
    search_mod.VectorizerSearcher.client = spy

    # The server clamps `limit` to MAX_SEARCH_LIMIT silently. A top-1000
    # dataset would be scored against 100 results and report ~10% recall — a
    # truncation misread as a search-quality finding.
    with pytest.raises(ValueError, match="clamps search"):
        search_mod.VectorizerSearcher.search_one(
            Query(vector=[0.1], sparse_vector=None, meta_conditions=None,
                  expected_result=[]),
            top=1000,
        )
    assert spy.requests == [], "the request must not be sent at all"
