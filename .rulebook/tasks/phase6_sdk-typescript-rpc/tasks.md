## 1. Prerequisites

- [ ] 1.1 `phase6_add-rpc-protocol-server` merged
- [ ] 1.2 Read `../Synap/sdks/typescript/` for patterns and tooling choices

## 2. Transport layer

- [ ] 2.1 Add `@msgpack/msgpack` dependency in `sdks/typescript/package.json`
- [ ] 2.2 Implement `src/rpc/codec.ts` with frame encode/decode (matching spec's u32 LE length prefix)
- [ ] 2.3 Implement `src/rpc/client.ts` with `RpcClient` using Node's `net` module; throw a clear error if imported in a non-Node runtime without fallback
- [ ] 2.4 Implement connection pool and auto-reconnect with backoff

## 3. Transport selection

- [ ] 3.1 Implement `createClient(url, options)` that picks RPC in Node, HTTP in browser, respecting explicit `transport` option
- [ ] 3.2 Add conditional exports in `package.json` so bundlers tree-shake Node-only code for browser builds
- [ ] 3.3 Implement the canonical URL parser as a `parseEndpoint(url: string)` helper: `vectorizer://host:port` → RPC on the given port; `vectorizer://host` (no port) → RPC on default port 15503; `host:port` (no scheme) → RPC; `http(s)://host:port` → REST. Reject any other scheme with a clear `Error` message. Browser builds reject `vectorizer://` at parse time with a message pointing at the REST form. The `Client` constructor and `createClient` both call into this single parser.
- [ ] 3.4 Unit tests for `parseEndpoint` covering: each of the 4 valid forms, the default-port branch (15503), an invalid scheme (`ftp://`), an empty string, a URL with userinfo (which MUST be rejected), and the browser-build rejection of `vectorizer://`.

## 4. Typed API

- [ ] 4.1 Generate typed method wrappers from the capability registry for collections/vectors/search/admin
- [ ] 4.2 Ensure `noImplicitAny` + strict mode passes

## 5. Examples + docs

- [ ] 5.1 Update `sdks/typescript/examples/quickstart.ts` to RPC (Node) + HTTP (browser)
- [ ] 5.2 Rewrite `sdks/typescript/README.md` with RPC-first Node quickstart and browser fallback note

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Publish TypeDoc to `sdks/typescript/docs/`; link from project README
- [ ] 6.2 Integration tests in `sdks/typescript/tests/rpc.test.ts` covering: connect, auth, CRUD, search, streaming, pool reconnect; browser build assertion that `net` import is absent
- [ ] 6.3 Run `npm run test` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
