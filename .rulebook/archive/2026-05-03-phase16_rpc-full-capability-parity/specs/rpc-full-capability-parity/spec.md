# Spec: RPC full capability parity

Targets the native VectorizerRPC transport (length-prefixed MessagePack
over TCP), specified in `docs/specs/VECTORIZER_RPC.md`. Not gRPC.

## ADDED Requirements

### Requirement: RPC dispatch covers every Transport::Both capability
The server's RPC dispatch table (`crates/vectorizer-server/src/protocol/rpc/dispatch.rs`)
SHALL contain a match arm for every operation flagged
`Transport::Both` in the capability registry
(`crates/vectorizer-server/src/server/capabilities.rs`). No registered
capability MAY return `unknown command` from the v1 dispatch.

#### Scenario: Every registered capability is dispatchable
Given the capability registry lists N entries with `Transport::Both`
When a client iterates each capability and sends a well-formed `Request` with that command name
Then NO response carries the error `"unknown command '<name>'"`
And EVERY response is either `Ok(...)` or a typed domain error (e.g. `not_found`, `forbidden`)

#### Scenario: search.intelligent is wired
Given a connected, authenticated RPC client
When the client sends `Request { command: "search.intelligent", args: [Map(query)] }`
Then the response MUST NOT contain the string `"not yet wired"`
And the response MUST be `Ok(Array<Map>)` with intelligent-search results

### Requirement: HELLO capabilities array reflects the dispatch table
The `capabilities` array returned by `HELLO` SHALL list exactly the
commands wired in the dispatch table — no missing entries, no entries
that are not actually wired.

#### Scenario: Capability advertisement matches dispatch
Given a server with N wired RPC commands
When a client completes a successful `HELLO` handshake
Then the `capabilities` array in the reply contains exactly N entries
And every entry corresponds to a match arm that returns `Ok(...)` for at least one valid input

#### Scenario: New commands added to dispatch are advertised automatically
Given a developer adds a new command arm to dispatch
When `cargo test --package vectorizer-server protocol::rpc::dispatch::tests::capabilities_match_dispatch` runs
Then the test MUST fail unless `rpc_capability_names()` is updated to include the new command

### Requirement: Argument shape consistency with the wire spec
For each wired command, the positional `args` array SHALL match the
catalog declared in `docs/specs/VECTORIZER_RPC.md` § 6. Map-typed
arguments MUST carry the same field names as the equivalent REST
request body.

#### Scenario: REST-equivalent fields cross-decode
Given a REST request body for `POST /collections/{n}/vectors/move` with `{ destination, ids }`
When the same operation is invoked via RPC `vectors.move` with `args = [Str(src), Str(dst), Array<Str>(ids)]`
Then both produce identical server-side state changes
And both responses carry the same per-id `results` shape (`status: ok | missing_in_src | dst_insert_failed | src_delete_failed`)

### Requirement: Admin-only commands enforce the admin role
Every RPC command flagged as admin in the registry SHALL check
`ConnectionAuth::admin` before executing, mirroring REST's
`require_admin_middleware`.

#### Scenario: Non-admin user receives forbidden on admin command
Given an authenticated RPC client whose principal does NOT have `Role::Admin`
When the client sends `Request { command: "admin.restart", args: [] }`
Then the response MUST be `Err("forbidden: admin role required")`
And the server MUST NOT perform the restart

#### Scenario: Admin user succeeds on admin command
Given an authenticated RPC client whose principal HAS `Role::Admin`
When the client sends the same `admin.restart` request
Then the response MUST be `Ok(...)` and the server begins the restart sequence

### Requirement: Frame-size cap surfaces a typed error
A response that would exceed the 64 MiB v1 frame cap SHALL return a
typed `frame_too_large` error rather than panicking the transport or
silently truncating.

#### Scenario: Oversized batch_search response
Given a `vectors.batch_search` request whose result set exceeds 64 MiB encoded
When the server attempts to respond
Then the response MUST be `Err("frame_too_large: response would exceed 64 MiB; use REST scroll API or wait for v2 streaming")`
And the connection MUST remain open and ready for further requests

### Requirement: All five SDKs expose typed wrappers per command
For every wired RPC command, the Rust, TypeScript, Python, Go, and C#
SDKs SHALL each provide a typed client wrapper. The wrapper SHALL
build the positional `args`, call the underlying `RpcClient::call`,
and decode the response into a typed struct (no `serde_json::from_value`
detour — the wire is MessagePack).

#### Scenario: Rust wrapper for vectors.move
Given a connected Rust `RpcClient`
When the caller invokes `client.move_vectors_rpc("src", "dst", &["vec-1"]).await`
Then the SDK sends `Request { command: "vectors.move", args: [Str("src"), Str("dst"), Array([Str("vec-1")])] }`
And returns a typed `MoveReport` whose `results` mirror per-id status

#### Scenario: TypeScript wrapper for search.intelligent
Given a connected TS `RpcClient`
When the caller invokes `client.searchIntelligent(request)`
Then the SDK sends `Request { command: "search.intelligent", args: [Map(request)] }`
And returns a typed `IntelligentSearchResponse` matching the server response

### Requirement: Additive backward compatibility
This phase SHALL be additive. The five existing v1 commands (`HELLO`,
`PING`, `collections.list`, `collections.get_info`, `vectors.get`,
`search.basic`) MUST keep their exact signatures and semantics. The
`protocol_version` stays at 1; clients negotiate via the
`capabilities` array.

#### Scenario: Pre-phase16 client works against post-phase16 server
Given a 3.6 SDK that only knows the original five commands
When the client connects to a 3.7 server
Then `HELLO` succeeds, `PING` returns `"PONG"`, and the original commands return identical responses
And the new commands are simply absent from the client's call surface (no breakage)
