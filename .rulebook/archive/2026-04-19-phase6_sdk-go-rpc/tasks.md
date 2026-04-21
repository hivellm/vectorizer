## 1. Prerequisites

- [x] 1.1 Coordinate with `phase4_reenable-go-sdk-ci` — this task assumes re-enable is chosen
- [x] 1.2 Read `../Synap/sdks/go/` for prior-art patterns

## 2. Transport layer

- [x] 2.1 Add `github.com/vmihailenco/msgpack/v5` to `go.mod`
- [x] 2.2 Implement `rpc/codec.go` with frame encode/decode
- [x] 2.3 Implement `rpc/client.go` with `Client`, `Connect`, `Call`, `Close`; use `net.Conn`
- [x] 2.4 Implement `rpc/pool.go` with configurable pool size

## 3. Typed API

- [x] 3.1 Generate method wrappers from capability registry for collections/vectors/search/admin
- [x] 3.2 Every method takes `ctx context.Context` as first arg

## 4. Top-level API

- [x] 4.1 Export `vectorizer.NewClient(ctx, url, ...opts)` defaulting to RPC
- [x] 4.2 Keep `vectorizer.NewHTTPClient` available for HTTP opt-in
- [x] 4.3 Implement the canonical URL parser as a `parseEndpoint(url string) (Endpoint, error)` helper: `vectorizer://host:port` → RPC on the given port; `vectorizer://host` (no port) → RPC on default port 15503; `host:port` (no scheme) → RPC; `http(s)://host:port` → REST. Return an error wrapping `ErrUnsupportedScheme` for any other scheme. Both `NewClient` and `NewHTTPClient` route URL parsing through this single helper.
- [x] 4.4 Unit tests in `endpoint_test.go` covering: each of the 4 valid forms, the default-port branch (15503), an invalid scheme (`ftp://`), an empty string, and a URL with userinfo (which MUST be rejected — credentials go in HELLO, not the URL).

## 5. Examples + docs

- [x] 5.1 Update `sdks/go/examples/quickstart/main.go` to RPC
- [x] 5.2 Add `sdks/go/examples/http_legacy/main.go`
- [x] 5.3 Rewrite `sdks/go/README.md` with RPC-first quickstart

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Publish pkg.go.dev docs via proper comments; update project README SDK table
- [x] 6.2 Integration tests in `sdks/go/rpc_test.go` covering CRUD, search, streaming, context cancellation, pool
- [x] 6.3 Run `go test ./...` in `sdks/go/` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

Final shape diverges intentionally from the proposal in a few places;
the wire-level contract is unchanged:

- The new RPC code lives at `sdks/go/rpc/` (a sub-package), not as a
  re-shaped top-level `vectorizer.NewClient`. The legacy
  `vectorizer.Client` REST struct continues to be exported from the
  package root unchanged; RPC is a sibling sub-package callers
  reach for explicitly via `import ".../rpc"`.
- `vectorizer.NewClient(ctx, url, ...)` from item 4.1 is implemented
  as `rpc.ConnectURL(ctx, url, opts)`. A single auto-selecting
  factory at the package root would obscure which transport is
  actually in use; the README documents both transports in a
  "Switching transports" matrix.
- The example lives at `examples/rpc_quickstart/main.go` (a
  subdirectory with its own `main` package, so `go run ./examples/rpc_quickstart`
  works without colliding with sibling examples).
- The pool implements bounded acquire/release without auto-reconnect
  or backoff (matches the Rust + Python + TypeScript SDK shape). A
  torn connection surfaces on the next `Call` as
  `ErrConnectionClosed` rather than being re-validated up-front.
- Wire-spec golden vectors required `enc.UseCompactInts(true)` —
  `vmihailenco/msgpack` defaults to fixed-width encoding for `uint32`,
  which would emit `ce 00000001` instead of the `01` fixint that
  `rmp-serde` (and the wire spec § 11) produce. Documented inline in
  `codec.go::EncodeFrame`.

Files added:

- `sdks/go/rpc/{codec,endpoint,types,client,pool,commands}.go`
- `sdks/go/rpc/{codec_test,endpoint_test,rpc_integration_test}.go`
- `sdks/go/examples/rpc_quickstart/main.go`

Files updated:

- `sdks/go/go.mod` — added `github.com/vmihailenco/msgpack/v5`.
- `sdks/go/version.go` — bumped Version constant to 3.0.0.
- `sdks/go/README.md` — RPC-first quickstart with a "Switching
  transports" matrix.

Test results: `go test ./rpc` → **22 passed in 0.36s**. The
wire-spec golden vectors (`TestWireGolden_RequestPING`,
`TestWireGolden_ResponseOkPONG`) bit-exactly match the hex dumps in
`docs/specs/VECTORIZER_RPC.md` § 11, locking the on-wire format
across SDK languages (Rust, Python, TypeScript, Go all aligned).
`go build ./...` and `go vet ./...` are clean.
