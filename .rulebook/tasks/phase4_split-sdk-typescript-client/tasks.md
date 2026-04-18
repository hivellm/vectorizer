## 1. Layout

- [ ] 1.1 Create `sdks/typescript/src/client/` with `_base.ts` + per-surface files.
- [ ] 1.2 Rewrite `index.ts` (or `client.ts`) as a facade.

## 2. Verification

- [ ] 2.1 `tsc --noEmit` clean.
- [ ] 2.2 Existing Jest/Vitest tests pass unchanged.
- [ ] 2.3 No sub-file exceeds 500 lines.

## 3. Tail (mandatory)

- [ ] 3.1 Update `sdks/typescript/README.md` with both the flat and per-surface import paths.
- [ ] 3.2 No new tests required.
- [ ] 3.3 Run SDK tests and confirm pass.
