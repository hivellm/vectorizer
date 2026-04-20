# VectorizerRPC — Wire Protocol Specification

**Status**: v1 (frozen)
**Default port**: 15503/tcp
**Reference implementation**: ports the production-tested SynapRPC layer
from `../Synap/synap-server/src/protocol/synap_rpc/` (~390 LOC core).

This document is the authoritative byte-level contract between Vectorizer
servers and clients. Every SDK (`phase6_sdk-{rust,go,python,javascript,
typescript,csharp}-rpc`) must conform to this spec; the server enforces
it. Anything not listed here is unspecified and may change without notice.

## Why a new transport?

The project already exposes REST (Axum), MCP (WebSocket JSON-RPC), gRPC
(tonic), and now the capability registry from
`phase4_rest-mcp-parity-tests` makes adding a fourth transport mechanical
— same handlers, new framing.

| Transport | Per-request overhead | Codegen | Best for |
|---|---|---|---|
| REST | HTTP framing + JSON parse + TLS handshake | none | browsers, scripts, ops tooling |
| gRPC | HTTP/2 + protobuf | yes (per language) | strongly-typed RPC across services |
| MCP | WebSocket + JSON-RPC | none | interactive AI tools |
| **RPC (this spec)** | **u32 length + MessagePack** | **none** | **bulk ingest, low-latency search, embedded SDK use** |

RPC is the **default transport** for first-party SDKs starting with
`phase6_make-rpc-default-transport`. REST stays as the universal
fallback.

## 1. Framing

Every frame on the wire — request and response — has identical shape:

```text
┌───────────────────┬──────────────────────────┐
│  length: u32 (LE) │  body: MessagePack bytes  │
└───────────────────┴──────────────────────────┘
    4 bytes              `length` bytes
```

- `length` is the size of the body **only**, in bytes, encoded as a
  little-endian unsigned 32-bit integer.
- `body` is a single MessagePack-encoded value. The server uses
  `rmp-serde` with the default externally-tagged enum representation;
  clients must encode/decode using a compatible MessagePack library.
- **Maximum body size: 64 MiB** (`64 * 1024 * 1024 = 67_108_864` bytes).
  The server will close the connection on a frame that declares a larger
  length to prevent OOM amplification attacks.

A connection is a stream of frames. Frames are not interleaved at the
byte level; the server reads a complete frame, dispatches it (possibly
on its own task), and may write responses out of order.

## 2. Request / Response envelope

Every request is a single MessagePack-encoded `Request` struct; every
response is a single MessagePack-encoded `Response` struct.

```rust
pub struct Request {
    pub id: u32,            // client-chosen monotonic ID
    pub command: String,    // dotted name from the capability registry
    pub args: Vec<Value>,   // positional arguments
}

pub struct Response {
    pub id: u32,                       // echoes Request.id
    pub result: Result<Value, String>, // Ok payload OR error message
}
```

### Multiplexing

Clients **must** treat `id` as opaque and unique per in-flight request
on a connection. The server runs each `Request` on its own
`tokio::spawn` task and emits `Response` frames in completion order, not
arrival order — clients dispatch responses to the originating call by
matching `Response.id` to a pending-call table.

A `u32` gives ~4 billion distinct IDs per connection. SDKs SHOULD wrap
on overflow; collisions on a long-lived connection are vanishingly rare
because the in-flight set is bounded by application backpressure.

### Error encoding (v1)

`result` is a serde `Result<Value, String>` — on success the inner is
`Ok(value)`, on failure it is `Err(message)`. The error string is a
human-readable message; v1 does not carry a structured error code. SDKs
SHOULD parse the string only for display; **do not** branch on it.

A future v2 may upgrade `Err(String)` to `Err(Error { code: u16,
message: String, details: Option<Value> })` once the project's error
enums are unified (see `phase3_unify-error-enums`). The version
negotiated by `HELLO` (§ 5) tells the client which form to expect.

## 3. The `Value` type

The on-wire dynamically-typed value mirrors SynapRPC's `SynapValue`:

```rust
pub enum VectorizerValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),       // raw bytes — NOT base64-encoded
    Str(String),          // valid UTF-8
    Array(Vec<VectorizerValue>),
    Map(Vec<(VectorizerValue, VectorizerValue)>),
}
```

- Encoded with `rmp-serde`'s default externally-tagged representation:
  unit variants are a bare string (`"Null"`), newtype variants are a
  single-key map (`{"Int": 42}`).
- `Bytes` carries raw octets. Unlike the JSON transports there is no
  base64 wrapping — embedding vectors and document payloads on the wire
  is the principal motivation for the binary transport.
- `Map` is an `Vec<(K, V)>` of pairs (not a `HashMap`) because
  MessagePack maps preserve insertion order and keys may be any value,
  not just strings.

### Cross-language mapping

| Vectorizer | Rust (rmp-serde) | Python (`msgpack`) | JS (`@msgpack/msgpack`) | Go (`vmihailenco/msgpack`) |
|---|---|---|---|---|
| `Null` | `()` | `None` | `null` | `nil` |
| `Bool` | `bool` | `bool` | `boolean` | `bool` |
| `Int` | `i64` | `int` | `bigint` / `number` | `int64` |
| `Float` | `f64` | `float` | `number` | `float64` |
| `Bytes` | `Vec<u8>` | `bytes` | `Uint8Array` | `[]byte` |
| `Str` | `String` | `str` | `string` | `string` |
| `Array` | `Vec<Value>` | `list` | `array` | `[]interface{}` |
| `Map` | `Vec<(Value, Value)>` | `dict` (string keys) / `list` of tuples | `Map` | `map[interface{}]interface{}` |

## 4. Authentication

Authentication is a **per-connection state**. The server starts a new
connection in `Unauthenticated` state. The client SHOULD issue a
`HELLO` request (§ 5) as the first frame; if `HELLO` carries valid
credentials, the connection transitions to `Authenticated` and stays
there for its lifetime.

Subsequent requests carry no token — auth is implicit in the connection
state. This trades per-frame overhead for a stickier connection model
(connections are now stateful, but on a long-lived TCP socket the
overhead is amortized to zero).

When `auth.enabled = false` server-side (single-user local setups), the
`HELLO` is still expected but credentials are ignored; every command
runs as the implicit local admin. This matches the existing REST/MCP
behaviour.

### Credentials

`HELLO` carries either:

- A bearer JWT (same format as REST `/auth/login` returns), passed in
  the `token` field, OR
- An API key, passed in the `api_key` field.

The server rejects requests that arrive on an `Unauthenticated`
connection (other than `HELLO` itself) with an error `Err("authentication
required: send HELLO first")`.

### Admin role

Commands tagged `Admin` in the capability registry require the
authenticated principal to carry `Role::Admin` claims. The server
returns `Err("admin role required")` and does not advance the request
to the handler.

## 5. The `HELLO` command

`HELLO` is the protocol-version handshake plus the auth handshake in a
single frame. **Every connection MUST start with `HELLO`** before any
data-plane command.

```rust
Request {
    id: <client-chosen>,
    command: "HELLO",
    args: vec![Value::Map(vec![
        (Value::Str("version".into()), Value::Int(1)),
        (Value::Str("token".into()),   Value::Str("<jwt>".into())),  // OR
        (Value::Str("api_key".into()), Value::Str("<api-key>".into())),
        (Value::Str("client_name".into()), Value::Str("vectorizer-rust/2.5.16".into())),
    ])],
}
```

Server reply on success:

```rust
Response {
    id: <echoes Request.id>,
    result: Ok(Value::Map(vec![
        (Value::Str("server_version".into()), Value::Str("2.5.16".into())),
        (Value::Str("protocol_version".into()), Value::Int(1)),
        (Value::Str("capabilities".into()), Value::Array(vec![
            Value::Str("collections.list".into()),
            Value::Str("collections.get_info".into()),
            // … one entry per capability the connection's principal can call
        ])),
        (Value::Str("authenticated".into()), Value::Bool(true)),
        (Value::Str("admin".into()),         Value::Bool(false)),
    ])),
}
```

If the client requests `version > server's max`, the server replies with
its highest supported version and the client SHOULD downgrade or close.
A `HELLO` failure (bad credentials, version unsupported) is delivered as
a normal `Err(message)` response and the connection stays open in
`Unauthenticated` state — the client may retry with corrected
credentials before the server closes for inactivity.

## 6. Command catalog (v1)

The command catalog is the **subset of the capability registry**
(`src/server/capabilities.rs`) where `Transport::Both` or
`Transport::McpOnly` is set AND the operation is reachable from the data
plane. v1 ships read commands first; write/admin commands land in
follow-up tasks but use the same wire format.

| Command | Auth | Args | Returns | Maps to |
|---|---|---|---|---|
| `HELLO` | none | `[Map { version, token?, api_key?, client_name? }]` | `Map { server_version, protocol_version, capabilities, authenticated, admin }` | (handshake — no registry entry) |
| `PING` | any | `[]` | `Str("PONG")` | (health check — no registry entry) |
| `collections.list` | User | `[]` | `Array<Map>` | `collection.list` |
| `collections.get_info` | User | `[Str(name)]` | `Map { dimension, metric, vector_count, … }` | `collection.get_info` |
| `vectors.get` | User | `[Str(collection), Str(vector_id)]` | `Map { id, data, payload? }` | `vector.get` |
| `search.basic` | User | `[Str(collection), Str(query), Int(limit)?, Float(threshold)?]` | `Array<Map { id, score, payload? }>` | `search.basic` |
| `search.intelligent` | User | `[Str(query), Array<Str>?(collections), Int(max_results)?, Bool(domain_expansion)?, Float(threshold)?]` | `Array<Map>` | `search.intelligent` |

### Command name conventions

- All-lowercase, dot-separated. Dot represents a topical group
  (`collections.*`, `vectors.*`, `search.*`, `graph.*`, `file.*`,
  `discovery.*`).
- Names match the registry `id` field exactly. New commands are added by
  appending a registry entry and the dispatch table picks them up.
- Server returns `Err("unknown command 'foo'")` for any command not in
  the dispatch table — no dynamic invocation, no reflection.

## 7. Streaming (deferred to v2)

Search results that exceed a single 64 MiB frame would need chunking.
v1 does not implement streaming; instead, the server caps the result
set at the number of items that fit comfortably in 64 MiB (~250k
1024-dim vectors with payloads). Larger result sets must page via the
existing REST scroll API or wait for v2.

When v2 lands, streaming will use a `last: bool` field in the response
envelope: the server emits multiple `Response` frames with the same
`id`, each containing a partial array; the final frame sets `last:
true`. SDKs MUST handle a single-frame response (`last` absent or
`true`) for v1 compatibility.

## 8. Versioning

The wire spec is versioned by an integer `protocol_version` returned
from `HELLO`. v1 is the only version. Breaking changes to framing,
envelope shape, or the `Value` type bump the major version; additive
changes (new commands, new fields with `#[serde(default)]`) do not.

A server that receives a `HELLO` declaring `version > server.max`
replies with `Ok(Map { protocol_version: <highest supported>, … })` and
the client SHOULD speak the older version. Clients SHOULD NOT
preemptively probe versions; declare what you can speak, accept what
the server returns.

## 9. Comparison with SynapRPC

| Aspect | SynapRPC | VectorizerRPC v1 |
|---|---|---|
| Framing | `[u32 LE len][msgpack body]` | identical |
| Request shape | `{id, command, args}` | identical |
| Response shape | `{id, result: Result<Value, String>}` | identical |
| Value enum | `SynapValue` | renamed `VectorizerValue`; identical variants |
| Auth | bearer in `HELLO`, sticky | identical |
| Pub/Sub | `SUBSCRIBE` + `_push` frames with `id: u32::MAX` | **not in v1** (no use case yet) |
| Default port | 15500 | **15503** (Vectorizer's port range) |
| Max frame | 16 MiB | **64 MiB** (we ship larger payloads) |

Wire-level parity with SynapRPC is intentional — a Synap-compatible
client can talk to a Vectorizer server with only command-name changes.
This shrinks the SDK matrix because the framing/codec layer is shared.

## 10. Security model

- **TLS**: optional, controlled by `config.rpc.tls.cert_path` /
  `tls.key_path`. When enabled, the server wraps the TCP listener with
  `tokio-rustls`. Clients connect to the same port over TLS. There is
  no STARTTLS — TLS is decided at connection time by the server config.
- **Origin pinning**: not applicable (this is a back-end transport, no
  cross-origin requests).
- **Rate limiting**: per-connection. The server caps in-flight requests
  per connection at `config.rpc.max_in_flight` (default 256). A
  connection that exceeds the limit will pause reading until a slot
  frees; clients SHOULD respect server-side backpressure rather than
  open a second connection.
- **Replay**: there is no replay protection at the protocol layer. If
  you need it, use a JWT with a short expiry plus client-side nonces in
  payload metadata.
- **Admin commands**: the server checks `Role::Admin` on the
  authenticated principal before dispatching admin-tagged registry
  entries. Failure does not terminate the connection.

## 11. Reference test vectors

These vectors are stable across server versions. SDKs SHOULD include
them as fixtures.

### Encode `Request { id: 1, command: "PING", args: [] }`

```
08 00 00 00                    # length = 8
93                             # array(3)
01                             #   id = 1
a4 50 49 4e 47                 #   command = "PING"
90                             #   args = array(0)
```

### Encode `Response { id: 1, result: Ok(Str("PONG")) }`

Both `Result<T, E>` *and* `VectorizerValue` use rmp-serde's default
externally-tagged enum representation, so an `Ok(Str("PONG"))` round-trips
as **two** nested one-key maps (one for the `Result` variant, one for
the `Value` variant). Clients MUST decode through both layers.

```
10 00 00 00                    # length = 16
92                             # array(2)
01                             #   id = 1
81                             #   result = map(1)
a2 4f 6b                       #     key = "Ok"
81                             #     value = map(1)
a3 53 74 72                    #       key = "Str"
a4 50 4f 4e 47                 #       value = "PONG"
```

### Decode failure cases

- `length` larger than 64 MiB → server closes connection, no response.
- `body` is not a valid MessagePack value → server emits `Err("frame
  decode failed: <rmp_serde error>")` with `id: u32::MAX` and closes
  the connection.
- `command` empty string → `Err("missing command")`.
- `args` arity mismatch → `Err("command '<name>' expects N args, got M")`.

## 12. Glossary

- **Frame**: the smallest unit on the wire — `[u32 LE len][body]`.
- **Envelope**: the `Request` or `Response` struct, MessagePack-encoded
  inside a frame's body.
- **Capability**: an operation registered in `src/server/capabilities.rs`.
- **Sticky auth**: per-connection auth state established by `HELLO`,
  not re-sent per request.
- **Default port 15503**: Vectorizer's RPC listener. The 15500-range is
  reserved for binary transports across the HiveLLM family
  (Synap=15500, Vectorizer=15503, future=15504+).
