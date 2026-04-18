## 1. Prerequisites

- [ ] 1.1 Coordinate with `phase4_reenable-go-sdk-ci` — this task assumes re-enable is chosen
- [ ] 1.2 Read `../Synap/sdks/go/` for prior-art patterns

## 2. Transport layer

- [ ] 2.1 Add `github.com/vmihailenco/msgpack/v5` to `go.mod`
- [ ] 2.2 Implement `rpc/codec.go` with frame encode/decode
- [ ] 2.3 Implement `rpc/client.go` with `Client`, `Connect`, `Call`, `Close`; use `net.Conn`
- [ ] 2.4 Implement `rpc/pool.go` with configurable pool size

## 3. Typed API

- [ ] 3.1 Generate method wrappers from capability registry for collections/vectors/search/admin
- [ ] 3.2 Every method takes `ctx context.Context` as first arg

## 4. Top-level API

- [ ] 4.1 Export `vectorizer.NewClient(addr, ...opts)` defaulting to RPC
- [ ] 4.2 Keep `vectorizer.NewHTTPClient` available for HTTP opt-in

## 5. Examples + docs

- [ ] 5.1 Update `sdks/go/examples/quickstart/main.go` to RPC
- [ ] 5.2 Add `sdks/go/examples/http_legacy/main.go`
- [ ] 5.3 Rewrite `sdks/go/README.md` with RPC-first quickstart

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Publish pkg.go.dev docs via proper comments; update project README SDK table
- [ ] 6.2 Integration tests in `sdks/go/rpc_test.go` covering CRUD, search, streaming, context cancellation, pool
- [ ] 6.3 Run `go test ./...` in `sdks/go/` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
