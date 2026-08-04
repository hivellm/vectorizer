# Proposal: phase1_python-sdk-return-type-contract

## Why

The v3.6.0 PyPI publish failed at its test gate. The test-plumbing repair fixed
34 of the 43 failures (a deprecated `asyncio.get_event_loop` helper, and mocks
aimed at a seam that had moved). The remaining 9 are not test problems — they
are the Python SDK contradicting its own type annotations, and they block the
PyPI publish.

Several methods declare a parsed return type and then return the transport's
raw dict. `vectorizer/search.py`:

```python
async def search_vectors(...) -> List[SearchResult]:
    ...
    return await self._transport.post(f"/collections/{collection}/search", data=payload)
```

The caller gets `{"results": [...]}`, so `results[0].id` raises `KeyError: 0`
and `len(results)` counts dict keys. The same shape shows up in `get_vector`
(`'dict' object has no attribute 'id'`), `embed_text` (`1 != 512` — the length
of a dict, not of the embedding) and `delete_vectors` (`KeyError: 'collection'`).

Two more mismatches in the same 9:

- 404 responses surface as a generic `ServerError: Resource not found`, while
  the tests expect a not-found type (`test_get_vector_not_found`,
  `test_search_vectors_collection_not_found`).
- Two tests assert error strings the SDK never emits: `'Health check failed'`
  and `'Failed to connect to service'`.

This is a published-behaviour question, not a cleanup: `vectorizer-sdk` 3.5.0
on PyPI returns those dicts today. Anyone who wrote `data["results"]` against
the shipped package breaks when the methods start returning typed objects, and
anyone who trusted the annotations is already broken. That is why the fix was
left out of the test-repair commit.

Why this went unnoticed is separate and worth fixing in the same pass:
`sdk-python-test.yml` ends its pytest chain with `|| echo "Some tests may have
failed"`, so the Python suite cannot fail that workflow. The release-time gate
in `sdk-publish-python.yml` was the first honest run of the suite.

## What Changes

Decide the contract, then make code, annotations and tests agree:

- **Option A — honour the annotations.** Parse the responses into
  `SearchResult`, `Vector`, embedding lists, etc. Correct, matches the other
  four SDKs, and breaks callers of the published dict shape, so it needs a
  CHANGELOG breaking-change note and a migration line in the SDK README.
- **Option B — honour the current behaviour.** Change the annotations to
  `Dict[str, Any]` and adjust the tests. Non-breaking and honest, but leaves
  the Python SDK less typed than its siblings, which do parse.

Whichever is chosen, also:

- Map 404 to the specific not-found exceptions the tests expect, or drop the
  expectation if `ServerError` is the intended contract.
- Reconcile the two error-message assertions with what the transport emits.
- Remove the `|| echo "Some tests may have failed"` fallback chain from
  `sdk-python-test.yml` so the suite can fail its own workflow, and align its
  pytest invocation with the publish gate's.

## Impact

- Affected specs: none (SDK client contract)
- Affected code: `sdks/python/vectorizer/search.py`,
  `sdks/python/vectorizer/vectors.py`, `sdks/python/vectorizer/admin.py`,
  `sdks/python/utils/http_client.py` (error mapping),
  `sdks/python/tests/test_sdk_comprehensive.py`,
  `.github/workflows/sdk-python-test.yml`
- Breaking change: YES under Option A — the published 3.5.0 returns dicts
- User benefit: the PyPI publish stops failing, and the SDK's annotations stop
  lying about what callers receive
