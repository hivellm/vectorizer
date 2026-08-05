## 1. Implementation

- [ ] 1.1 Add the scheme guard to `HttpTransport::new`
      (`sdks/rust/src/http_transport.rs`): reject a `base_url` whose scheme is
      neither `http` nor `https`, returning `VectorizerError::configuration`.
      `vectorizer://` gets the redirect to `rpc::RpcClient::connect_url`,
      mirroring the wording the RPC client already uses in the other
      direction; any other scheme gets the generic "expected `http(s)://`"
      message naming what was passed. Scheme-less input must keep working.
      Implemented without touching `rpc::endpoint` — this module compiles
      without the `rpc` feature.
      **Done when:** `HttpTransport::new("vectorizer://h:15503", None, 30)`
      returns `Err`, `http://` and `https://` and `localhost:15002` return
      `Ok`, and the workspace compiles with and without the `rpc` feature.
- [ ] 1.2 Point the caller at the right client: doc comment on
      `VectorizerClient::new` (`sdks/rust/src/client/mod.rs`) stating that
      `base_url` is an `http(s)://` URL and that `vectorizer://` belongs to
      `rpc::RpcClient::connect_url`. No behavior change here — the guard in
      1.1 already covers all four `HttpTransport::new` call sites, including
      the two replication ones.
      **Done when:** `cargo doc` renders it and the note names both clients.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Update or create documentation covering the implementation: the
      SDK README / transport section stating which scheme belongs to which
      client, and that mixing them now fails at construction.
- [ ] 2.2 Write tests covering the new behavior: unit tests in
      `http_transport.rs` for `vectorizer://` (asserting the message names
      `RpcClient`), for an unrelated scheme, for `http`/`https`, and for
      scheme-less input; plus one asserting the failure arrives from
      `VectorizerClient::new` rather than at first request, which is the
      actual complaint in #392.
- [ ] 2.3 Run tests and confirm they pass (`cargo nextest run` for the SDK
      package, plus clippy and fmt).
