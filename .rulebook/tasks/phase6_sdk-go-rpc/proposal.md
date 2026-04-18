# Proposal: phase6_sdk-go-rpc

## Why

`sdks/go/` needs RPC support — and also needs its CI re-enabled (`phase4_reenable-go-sdk-ci`). Either we fix both or deprecate the Go SDK. This task assumes Option A (re-enable) is chosen; if Option B (deprecate) wins, this task closes as WONTFIX.

Reference: `../Synap/sdks/go/`.

## What Changes

Inside `sdks/go/`:

1. New package `github.com/hivellm/vectorizer-go/rpc` with `Client` using `net.Dial` + `vmihailenco/msgpack`.
2. Connection pool with sync.Mutex guarding a slice.
3. Typed API methods: `client.CollectionsCreate(ctx, ...)`, `client.VectorsInsert(ctx, ...)`, etc.
4. Context cancellation support on every call.
5. Go 1.21+ minimum.
6. Update `README.md` quickstart to RPC.
7. Module version bumped via a git tag.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/go/rpc/` (new), existing HTTP client, `go.mod`, README, `examples/`, `tests/`
- Breaking change: YES (default transport changes) — major tag bump
- User benefit: idiomatic Go client on fast transport; context-aware.
