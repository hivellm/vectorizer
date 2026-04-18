## 1. Prerequisites

- [ ] 1.1 `phase6_add-rpc-protocol-server` merged
- [ ] 1.2 Read `../Synap/sdks/typescript/` for patterns and tooling choices

## 2. Transport layer

- [ ] 2.1 Add `@msgpack/msgpack` dependency in `sdks/typescript/package.json`
- [ ] 2.2 Implement `src/rpc/codec.ts` with frame encode/decode (matching spec's u32 LE length prefix)
- [ ] 2.3 Implement `src/rpc/client.ts` with `RpcClient` using Node's `net` module; throw a clear error if imported in a non-Node runtime without fallback
- [ ] 2.4 Implement connection pool and auto-reconnect with backoff

## 3. Transport selection

- [ ] 3.1 Implement `createClient(addr, options)` that picks RPC in Node, HTTP in browser, respecting explicit `transport` option
- [ ] 3.2 Add conditional exports in `package.json` so bundlers tree-shake Node-only code for browser builds

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
