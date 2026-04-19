# VectorizerRPC — Operator Guide

VectorizerRPC is the binary transport: length-prefixed MessagePack
frames over raw TCP. It runs on its own port alongside REST/MCP/gRPC,
shares the same capability registry, and is the recommended transport
for first-party SDKs.

The byte-level wire spec lives at
[`docs/specs/VECTORIZER_RPC.md`](../specs/VECTORIZER_RPC.md). This
document is the operator's view: how to enable it, what to monitor, and
how to talk to it from a script.

## Enabling the listener

Off by default in v1 (the SDK matrix is still being rolled out). To
turn it on, add an `rpc:` block to `config.yml`:

```yaml
rpc:
  enabled: true
  host: "0.0.0.0"   # 127.0.0.1 keeps it loopback-only
  port: 15503       # default — Vectorizer's slot in the 15500-range
```

Restart the server. On boot you'll see:

```
INFO  vectorizer::protocol::rpc::server: VectorizerRPC server listening on 0.0.0.0:15503
INFO  vectorizer::server::core::bootstrap: ✅ VectorizerRPC listener bound to 0.0.0.0:15503
```

If the line is missing, check `cargo run` stderr for
`Failed to spawn VectorizerRPC listener`. The most common cause is a
port collision (`EADDRINUSE`) — pick a different `port` and restart.

## Smoke-testing a deployment

The reference test vectors at wire spec § 11 are stable across
versions. From any host with a MessagePack library you can hand-craft a
`HELLO` + `PING` round-trip and verify the listener is reachable.

### Python

```python
import socket
import struct
import msgpack

def call(sock, request_id, command, args):
    body = msgpack.packb([request_id, command, args])
    sock.sendall(struct.pack("<I", len(body)) + body)
    length = struct.unpack("<I", sock.recv(4))[0]
    return msgpack.unpackb(sock.recv(length), raw=False)

with socket.create_connection(("localhost", 15503)) as s:
    print(call(s, 1, "HELLO", [{"version": 1}]))
    print(call(s, 2, "PING", []))
    print(call(s, 3, "collections.list", []))
```

Expected output:

```python
[1, {"Ok": {"Map": [
  ["server_version",   "2.5.16"],
  ["protocol_version", 1],
  ["authenticated",    True],
  ["admin",            True],
  ["capabilities",     ["PING", "collections.list", ...]],
]}}]
[2, {"Ok": {"Str": "PONG"}}]
[3, {"Ok": {"Array": []}}]
```

The double-tagged shape (`{"Ok": {"Str": "PONG"}}` rather than
`{"Ok": "PONG"}`) is documented at wire spec § 11 — both `Result<T,E>`
and `VectorizerValue` use rmp-serde's externally-tagged enum
representation.

### Node.js

```js
import net from "node:net";
import { encode, decode } from "@msgpack/msgpack";

const sock = net.createConnection({ host: "localhost", port: 15503 });

function call(id, command, args) {
  return new Promise((resolve) => {
    const body = encode([id, command, args]);
    const len = Buffer.alloc(4);
    len.writeUInt32LE(body.length, 0);
    sock.write(Buffer.concat([len, body]));
    sock.once("data", (buf) => {
      const length = buf.readUInt32LE(0);
      resolve(decode(buf.subarray(4, 4 + length)));
    });
  });
}

console.log(await call(1, "HELLO", [{ version: 1 }]));
console.log(await call(2, "PING", []));
sock.end();
```

In production a real client would buffer reads and match responses by
`id` rather than rely on `data` event ordering — this snippet is for
ops smoke-testing only.

## What the listener does NOT do (v1)

- **No streaming**: search results that exceed 64 MiB will be rejected
  by the codec's max-frame check. Use REST scroll for very large result
  sets until streaming lands in v2 (wire spec § 7).
- **No pub/sub**: no `SUBSCRIBE` command. Vectorizer doesn't have a
  pub/sub use case yet; the SynapRPC reference shipped one but we
  intentionally dropped it (wire spec § 9).
- **No write commands in the v1 dispatch table**: the server accepts
  read commands (`PING`, `collections.list`, `collections.get_info`,
  `vectors.get`, `search.basic`) plus the `HELLO` handshake. Insert /
  update / delete are wired in a follow-up task slot
  (`phase6_rpc-write-commands`); use REST for writes in the meantime.

## Authentication

Per wire spec § 4, auth is **per-connection sticky**: the first frame
on a connection MUST be `HELLO` carrying a `token` (JWT) or `api_key`
field. The connection then runs as that principal until it closes.
Subsequent frames carry no auth payload.

When server-side `auth.enabled = false` (single-user / loopback dev),
`HELLO` accepts every credential (or none) and the connection runs as
the implicit local admin. This matches the existing REST/MCP behaviour
for single-user setups.

Admin-tagged commands check `Role::Admin` on the authenticated
principal at dispatch time. Failure returns `Err("admin role
required")` and does not terminate the connection — the client may
retry with a different credential by closing and re-opening.

## Metrics

The listener emits the following per-command tracing spans:

- `rpc.conn` (info span, `peer = <addr>`) — one per accepted connection
- `rpc.req` (debug span, `id = <u32>, cmd = <name>`) — one per request

Commands that exceed 1 ms wall-clock are upgraded to a `WARN` log so
operators can spot slow handlers without enabling debug-level tracing.
Prometheus exposition is tracked separately (the global `/metrics`
endpoint will gain `rpc_*` counters in a follow-up alongside the
write-command rollout).

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Connection accepted but every command returns `Err("authentication required")` | Client skipped `HELLO` | Send `HELLO` as the first frame on every new connection |
| `HELLO` returns `Err("invalid JWT: …")` even though the JWT works on REST | Token blacklisted (logged out) or expired | Mint a fresh token via `POST /auth/login` |
| Connection drops mid-frame after a large search | Frame exceeded 64 MiB cap | Page via REST scroll, or wait for v2 streaming |
| `Failed to spawn VectorizerRPC listener: Address already in use` on boot | Port collision | Change `rpc.port` in `config.yml` |
| Unknown-command error for a command that exists on REST | RPC dispatch table is a strict subset; the command lives only on REST today | Either add the dispatch arm or use REST |

## Cross-references

- [Wire spec (byte level)](../specs/VECTORIZER_RPC.md)
- [Capability registry](../architecture/capabilities.md) — single
  source of truth for which operations exist on each transport
- [Route auth matrix](../api/route-auth-matrix.md) — REST companion to
  this guide
