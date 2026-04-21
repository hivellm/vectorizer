## 1. Layout

- [x] 1.1 Create `sdks/typescript/src/client/` with `_base.ts` + per-surface files. → 9 surface files (`core`, `collections`, `vectors`, `search`, `discovery`, `files`, `graph`, `qdrant`, `admin`) + `_base.ts` + `index.ts`. Largest is `qdrant.ts` at 385 lines, well under the 500-line cap.
- [x] 1.2 Rewrite `index.ts` (or `client.ts`) as a facade. → `client/index.ts` declares `VectorizerClient extends BaseClient` and uses the standard TS mixin recipe (`applyMixins`) to copy every per-surface prototype onto it. Old monolithic `client.ts` (1,879 lines) removed.
- [x] 1.3 Define `_base.ts::Transport` as a TypeScript `interface`. → `Transport` is exported as an alias of `ITransport` (already an interface in `utils/transport.ts`). Per-surface classes only call `this.transport.{get,post,put,delete,postFormData}` — never `fetch` directly. The new `transport` field on `VectorizerClientConfig` is the injection point that lets tests (and the upcoming `RpcTransport`) bypass `TransportFactory`.

## 2. Verification

- [x] 2.1 `tsc --noEmit` clean. → Run from `sdks/typescript/`, exit 0.
- [x] 2.2 Existing Jest/Vitest tests pass unchanged. → 17 test files, 352 passing, 46 require a live server (unchanged behaviour vs. baseline).
- [x] 2.3 No sub-file exceeds 500 lines. → `_base.ts` 262, `qdrant.ts` 385, `vectors.ts` 240, `search.ts` 225, `files.ts` 164, `admin.ts` 127, `collections.ts` 100, `graph.ts` 99, `index.ts` 98, `core.ts` 57, `discovery.ts` 57.
- [x] 2.4 Add a unit test that supplies a `MockTransport implements Transport` to every per-surface client. → `tests/mock-transport.test.ts` (11 cases): 9 per-surface tests + facade composition test + a `BaseClient`-only test that confirms `HttpClient` is never instantiated when `transport` is injected.

## 3. Tail (mandatory)

- [x] 3.1 Update `sdks/typescript/README.md` with the per-surface layout. → New "Package layout (per-surface clients)" section after the Quick Start. Documents the file tree, the per-surface import path, the `MockTransport` injection seam, and points forward to `phase6_sdk-typescript-rpc` and the canonical `vectorizer://host:15503` URL scheme.
- [x] 3.2 Add the `MockTransport` test from item 2.4. → See `tests/mock-transport.test.ts`. The `afterEach` asserts `HttpClient` was never constructed — guarantees the per-surface classes are not coupled to the REST transport.
- [x] 3.3 Run SDK tests and confirm pass. → `pnpm exec vitest run` → 17 files / 352 passing / 46 require a live server. `pnpm exec tsc --noEmit` clean.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
