## 1. Decision

- [ ] 1.1 Inspect current state of `sdks/javascript/` (CJS-only? published separately to npm?); record findings in `design.md`
- [ ] 1.2 Record Option A (deprecate) or Option B (CJS wrapper) decision via `rulebook_decision_create`

## 2. Option A — Deprecate

- [ ] 2.1 Update `sdks/javascript/README.md` to a redirect pointing at `sdks/typescript/`
- [ ] 2.2 Remove the separate npm package (publish a final version with a deprecation message via `npm deprecate`)
- [ ] 2.3 Delete the source files; keep only the README stub

## 3. Option B — CJS wrapper

- [ ] 3.1 Configure `sdks/typescript/` to emit both ESM and CJS bundles
- [ ] 3.2 Make `sdks/javascript/index.js` a thin `require('@vectorizer/sdk')` re-export
- [ ] 3.3 Publish synchronized versions of both packages

## 4. Examples + docs

- [ ] 4.1 Update `sdks/javascript/examples/` (if retained) to RPC-first usage
- [ ] 4.2 Rewrite `sdks/javascript/README.md` with chosen-option guidance

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Cross-link docs between `sdks/javascript/` and `sdks/typescript/`; update project-root README's SDK table
- [ ] 5.2 Add a smoke test (if Option B) that imports via CJS and connects over RPC
- [ ] 5.3 Run `npm test` in both SDKs and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
