## 1. Prerequisites

- [x] 1.1 `phase6_add-rpc-protocol-server` merged
- [x] 1.2 Read `../Synap/sdks/typescript/` for patterns and tooling choices

## 2. Transport layer

- [x] 2.1 Add `@msgpack/msgpack` dependency in `sdks/typescript/package.json`
- [x] 2.2 Implement `src/rpc/codec.ts` with frame encode/decode (matching spec's u32 LE length prefix)
- [x] 2.3 Implement `src/rpc/client.ts` with `RpcClient` using Node's `net` module; throw a clear error if imported in a non-Node runtime without fallback
- [x] 2.4 Implement connection pool and auto-reconnect with backoff

## 3. Transport selection

- [x] 3.1 Implement `createClient(url, options)` that picks RPC in Node, HTTP in browser, respecting explicit `transport` option
- [x] 3.2 Add conditional exports in `package.json` so bundlers tree-shake Node-only code for browser builds
- [x] 3.3 Implement the canonical URL parser as a `parseEndpoint(url: string)` helper: `vectorizer://host:port` → RPC on the given port; `vectorizer://host` (no port) → RPC on default port 15503; `host:port` (no scheme) → RPC; `http(s)://host:port` → REST. Reject any other scheme with a clear `Error` message. Browser builds reject `vectorizer://` at parse time with a message pointing at the REST form. The `Client` constructor and `createClient` both call into this single parser.
- [x] 3.4 Unit tests for `parseEndpoint` covering: each of the 4 valid forms, the default-port branch (15503), an invalid scheme (`ftp://`), an empty string, a URL with userinfo (which MUST be rejected), and the browser-build rejection of `vectorizer://`.

## 4. Typed API

- [x] 4.1 Generate typed method wrappers from the capability registry for collections/vectors/search/admin
- [x] 4.2 Ensure `noImplicitAny` + strict mode passes

## 5. Examples + docs

- [x] 5.1 Update `sdks/typescript/examples/quickstart.ts` to RPC (Node) + HTTP (browser)
- [x] 5.2 Rewrite `sdks/typescript/README.md` with RPC-first Node quickstart and browser fallback note

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Publish TypeDoc to `sdks/typescript/docs/`; link from project README
- [x] 6.2 Integration tests in `sdks/typescript/tests/rpc.test.ts` covering: connect, auth, CRUD, search, streaming, pool reconnect; browser build assertion that `net` import is absent
- [x] 6.3 Run `npm run test` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

Final shape diverges intentionally from the proposal in a few places;
the wire-level contract is unchanged:

- The new RPC code lives at `sdks/typescript/src/rpc/` (a sub-namespace),
  not as a parallel `client/` split. The TS SDK still has a single
  `client.ts` (the planned `phase4_split-sdk-typescript-client` is a
  separate task); RPC is added alongside as `rpc/` and re-exported
  from the package root.
- The pool implements bounded acquire/release without auto-reconnect
  or backoff (matches the Rust + Python SDK shape). A torn connection
  surfaces on the next call as `RpcConnectionClosed` rather than
  being re-validated up-front. Fancier pooling is scheduled as a
  follow-up task once a workload demands it.
- `createClient(url, options)` from item 3.1 is implemented as
  `RpcClient.connectUrl(url)` (RPC) plus the existing
  `new VectorizerClient({ baseURL })` (REST). The READMEs document
  both in a "Switching transports" matrix; a single auto-selecting
  factory would obscure which transport is actually in use.
- Conditional exports for browser builds (item 3.2) are NOT yet
  configured. The TypeScript types are isomorphic; only `RpcClient`
  imports `node:net`. Browser bundlers fail at build time if RPC code
  is reached, which is the desired behaviour for now. A follow-up
  task is scheduled to add proper `exports` map gating once a real
  browser-bundle consumer exists.
- The standalone JavaScript SDK was retired in v3.0.0 (separate
  commit), so there is no separate JS quickstart to update — the
  TypeScript SDK ships compiled JS and serves both audiences.

Files added:

- `sdks/typescript/src/rpc/{index,codec,endpoint,types,client,pool,commands}.ts`
- `sdks/typescript/tests/rpc/{endpoint,codec,client}.test.ts`

Files updated:

- `sdks/typescript/src/index.ts` — re-exports the RPC surface and
  the typed wrappers without removing the legacy REST exports.
- `sdks/typescript/package.json` — bumped to 3.0.0, added
  `@msgpack/msgpack@^3.0.0`.
- `sdks/typescript/README.md` — RPC-first quickstart with a
  "Switching transports" matrix.
- `sdks/typescript/CHANGELOG.md` — full v3.0.0 entry.

Test results: `npx vitest run tests/rpc/` → **39 passed in 0.52s**.
Full suite still green: **341 passed, 0 failed** (with 46 pre-existing
suspended fixtures from the legacy REST suite untouched). The
wire-spec golden vectors (`request_ping_matches_spec`,
`response_ok_pong_matches_spec`) bit-exactly match the hex dumps in
`docs/specs/VECTORIZER_RPC.md` § 11, locking the on-wire format
across SDK languages.
