## 1. Implementation

- [x] 1.1 Add the scheme guard to `HttpTransport::new`
      (`sdks/rust/src/http_transport.rs`): reject a `base_url` whose scheme is
      neither `http` nor `https`, returning `VectorizerError::configuration`.
      `vectorizer://` gets the redirect to `rpc::RpcClient::connect_url`,
      mirroring the wording the RPC client already uses in the other
      direction; any other scheme gets the generic "expected `http(s)://`"
      message naming what was passed. Scheme-less input must keep working.
      Implemented without touching `rpc::endpoint`.
      **Correction to the plan:** the stated reason was that `http_transport`
      compiles without the `rpc` module. That is backwards — `pub mod rpc` is
      unconditional in `lib.rs:43`, and `http_transport` is the gated one
      (`feature = "http"`). The real reason is behavioural: `parse_endpoint`
      maps a scheme-less `host:port` to **RPC**, so delegating would reject
      `localhost:15002`, a form this transport accepts today.
      **Done when:** `HttpTransport::new("vectorizer://h:15503", None, 30)`
      returns `Err`, `http://` and `https://` and `localhost:15002` return
      `Ok`, and the crate compiles with `--all-features` and with
      `--no-default-features --features rpc`. Both verified.
- [x] 1.2 Point the caller at the right client: doc comment on
      `VectorizerClient::new` (`sdks/rust/src/client/mod.rs`) stating that
      `base_url` is an `http(s)://` URL and that `vectorizer://` belongs to
      `rpc::RpcClient::connect_url`. No behavior change here — the guard in
      1.1 already covers all four `HttpTransport::new` call sites, including
      the two replication ones.
      **Done when:** `cargo doc` renders it and the note names both clients.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Update or create documentation covering the implementation: the
      SDK README / transport section stating which scheme belongs to which
      client, and that mixing them now fails at construction.
      Added under the existing URL/constructor table, showing both directions
      with their actual messages, and stating that scheme-less base URLs stay
      valid for the HTTP client.
- [x] 2.2 Write tests covering the new behavior: unit tests in
      `http_transport.rs` for `vectorizer://` (asserting the message names
      `RpcClient`), for an unrelated scheme, for `http`/`https`, and for
      scheme-less input; plus one asserting the failure arrives from
      `VectorizerClient::new` rather than at first request, which is the
      actual complaint in #392.
      5 unit tests (including an uppercase-scheme case, since schemes are
      case-insensitive per RFC 3986) plus `tests/rest_client_rejects_rpc_url.rs`
      with 3 driving the public constructors — the mirror of
      `rpc_integration.rs::connect_url_rejects_http_scheme_with_clear_error`.
      The integration file exists because the guard sits one layer below the
      public API: a refactor that stopped routing construction through
      `HttpTransport::new` would keep the unit tests green while regressing
      the behaviour #392 reported.
- [x] 2.3 Run tests and confirm they pass (`cargo nextest run` for the SDK
      package, plus clippy and fmt).
      `sdks/rust` is a workspace member (`members = ["crates/*", "sdks/rust"]`),
      so the workspace gate covers it: **2037 passed, 0 failed, 9 skipped**
      (2029 before this task — the 8 new tests), clippy exit 0, `fmt --check`
      clean. Also checked `--all-features` and
      `--no-default-features --features rpc`, since `http_transport` is gated
      behind `feature = "http"` and the guard must not break an rpc-only build.
