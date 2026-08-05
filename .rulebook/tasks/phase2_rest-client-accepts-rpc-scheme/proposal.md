# Proposal: phase2_rest-client-accepts-rpc-scheme

Fixes [#392](https://github.com/hivellm/vectorizer/issues/392).

## Why

The Rust SDK's REST facade accepts a `vectorizer://` base URL without
complaint and only fails later, inside reqwest:

```
Network error: HTTP request failed:
builder error for url (vectorizer://127.0.0.1:15503/auth/login)
```

`VectorizerClient::new` returns `Ok`, so the mistake surfaces at the first
request — far from its cause — and the message names neither the scheme nor
the client that would have worked.

The RPC side already handles the mirror-image mistake well
(`sdks/rust/src/rpc/client.rs:268`):

> RpcClient cannot dial REST URL '...'; use the HTTP client
> (`vectorizer_sdk::VectorizerClient`) instead, or pass a `vectorizer://` URL

So the SDK teaches the user in one direction and abandons them in the other.
That asymmetry is what this task closes.

Severity is low — both transports work correctly once each gets its own
scheme (the issue reports PING, HELLO/AUTH with JWT, `collection.list` and
`search.by_text` all green over `vectorizer://`). This is purely about the
misuse diagnostic, which is why it is cheap to fix and easy to keep fixed.

## What Changes

Validate the scheme in **`HttpTransport::new`**
(`sdks/rust/src/http_transport.rs:43`), not in `VectorizerClient::new`.

Two reasons that placement is the right one:

- It is the single choke point. `client/mod.rs` builds an `HttpTransport` at
  four sites — the plain HTTP path (185), the auto-detect path (159), and both
  replication paths (222, 227). A guard in `VectorizerClient::new` would miss
  the replica URLs, which take the same kind of value from a different field.
- `ClientConfig.base_url` is not always an HTTP URL: the UMICP path
  legitimately produces `umicp://host:port` as its `base_url` (171). A blanket
  check on the config field would reject a working configuration.

The guard rejects any base URL whose scheme is not `http` / `https` and
returns `VectorizerError::configuration` — the error type this function
already uses for bad credentials — naming the offending scheme and the client
to use instead. `vectorizer://` gets the specific redirect to
`rpc::RpcClient::connect_url`; other schemes get the generic
"expected `http(s)://`" form. Scheme-less input keeps working, since that is a
form callers pass today.

Deliberately self-contained: it does **not** reuse
`rpc::endpoint::parse_endpoint`, because `http_transport` compiles without the
`rpc` module and must not gain that dependency.

## Impact

- Affected specs: none.
- Affected code: `sdks/rust/src/http_transport.rs` (guard + unit tests);
  doc comment on `sdks/rust/src/client/mod.rs::VectorizerClient::new` pointing
  at the RPC client for `vectorizer://`.
- Breaking change: NO in practice — it converts a guaranteed runtime failure
  into an immediate, explained construction failure. Code that "worked" while
  passing `vectorizer://` to the REST client did not work.
- User benefit: the mistake is caught at construction with a message that
  names the fix, matching what the RPC client already does.
