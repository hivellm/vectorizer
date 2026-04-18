# Proposal: phase4_split-sdk-python-client

## Why

`sdks/python/client.py` is **2,907 lines** — the largest SDK client in the repo. All API surfaces live on one `VectorizerClient` class: collections, vectors, search, graph, admin, auth. New Python users reading the source to understand the SDK have to scroll through 20+ methods unrelated to whatever they're trying to do.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Split the module under `sdks/python/vectorizer/`:

- `_base.py` — transport, error handling, retry, auth header.
- `collections.py`, `vectors.py`, `search.py`, `graph.py`, `admin.py`, `auth.py` — one module per API surface.
- `client.py` — thin facade that composes the sub-clients into the public `VectorizerClient`.

Public API is preserved: `from vectorizer import VectorizerClient` keeps working; individual sub-clients are also importable for advanced users.

## Impact

- Affected specs: none.
- Affected code: `sdks/python/` — `client.py`, new submodules, `__init__.py`.
- Breaking change: NO — the flat `VectorizerClient` API is preserved as a facade.
- User benefit: 6×smaller review per change; Python users can browse the module most relevant to their use case without reading 2,900 lines of unrelated surfaces.
