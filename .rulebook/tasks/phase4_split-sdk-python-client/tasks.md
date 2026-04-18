## 1. Layout

- [ ] 1.1 Create `sdks/python/vectorizer/_base.py` with transport, retry, auth header helpers.
- [ ] 1.2 Extract `collections.py` + `vectors.py` + `search.py`.
- [ ] 1.3 Extract `graph.py` + `admin.py` + `auth.py`.
- [ ] 1.4 Rewrite `client.py` as a facade composing the above.

## 2. Public surface

- [ ] 2.1 `from vectorizer import VectorizerClient` keeps working identically.
- [ ] 2.2 Sub-clients (`from vectorizer.collections import CollectionsClient`) are also exposed for advanced users.

## 3. Verification

- [ ] 3.1 `pytest sdks/python/tests` passes unchanged.
- [ ] 3.2 No sub-module exceeds 700 lines.

## 4. Tail (mandatory)

- [ ] 4.1 Update `sdks/python/README.md` with the new layout + import examples.
- [ ] 4.2 Existing tests sufficient; no new tests required.
- [ ] 4.3 Run the SDK test suite and confirm pass.
