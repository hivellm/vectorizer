## 1. Layout

- [ ] 1.1 Create `sdks/javascript/src/_base.js` with transport helpers.
- [ ] 1.2 Extract per-surface files (collections, vectors, search, graph, admin, auth).
- [ ] 1.3 Rewrite `client.js` as a facade.

## 2. Public API

- [ ] 2.1 Default export `VectorizerClient` unchanged.
- [ ] 2.2 Add named exports for the sub-clients.

## 3. Verification

- [ ] 3.1 `npm test` in `sdks/javascript/` passes unchanged.
- [ ] 3.2 No sub-file exceeds 500 lines.

## 4. Tail (mandatory)

- [ ] 4.1 Update `sdks/javascript/README.md`.
- [ ] 4.2 Existing tests sufficient.
- [ ] 4.3 Run SDK tests and confirm pass.
