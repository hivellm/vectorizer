# Migration: RPC is now the default transport (v3.x)

Starting with v3.x the Vectorizer server bundle ships with the
**VectorizerRPC** binary transport enabled by default and treats it as
the recommended path for first-party clients. REST and MCP stay
available and fully functional — the change is to defaults and
documentation, not to the transport surface itself.

## What changed

| Surface | Before (≤ v2.x) | After (v3.x) |
|---|---|---|
| `RpcConfig::default()` | `enabled: false` | `enabled: true` |
| `config.example.yml` | RPC section absent | `rpc:` block first, with `enabled: true` and a "recommended default" comment |
| `Dockerfile` | `EXPOSE 15002` | `EXPOSE 15503 15002` (RPC first) |
| `docker-compose.yml` | publishes `15002` and `15003` | publishes `15503`, `15002`, `15003` |
| Helm chart | `service.port: 15002` only | `service.rpcPort: 15503` + `service.port: 15002` (both exposed) |
| k8s manifests (`k8s/service.yaml`, `statefulset.yaml`, `statefulset-ha.yaml`) | port `15002` only | named ports `rpc` (15503), `http` (15002), and existing replication / grpc |

## What did NOT change

- **REST stays available** on the same port (default 15002). The
  dashboard, every existing browser/curl/HTTP-SDK consumer, and ops
  tooling continue to work unchanged.
- **MCP stays available** on the same port (15002 by default; the MCP
  endpoint is `/mcp` over WebSocket).
- **gRPC stays available** when configured.
- **Wire format** of every existing transport is byte-identical to v2.x.
- The capability registry (`src/server/capabilities.rs`) drives all
  transports — adding an operation lands it on every surface at once.

## What you need to do

### Default deployments — nothing

If you run with `config.example.yml` defaults, the upgrade is
transparent. The server now also listens on port `15503` for binary
RPC. If your environment doesn't expose 15503, the listener simply
isn't reachable from outside; nothing else changes.

### Restricted environments — opt out

If your firewall or security policy can't expose an additional port,
flip RPC off in `config.yml`:

```yaml
rpc:
  enabled: false
```

The server boot will skip the listener entirely (`debug!
"VectorizerRPC listener disabled in config"` in the boot log).

### Containerized deployments — open port 15503

The Dockerfile now `EXPOSE`s both `15503` and `15002`. Update your
deployment to publish both:

- **docker-compose**: `docker-compose.yml` already maps both ports.
  If you maintain a custom compose file, add `"15503:15503"` to the
  `ports:` list.
- **Helm**: the chart sets `service.rpcPort: 15503` by default. To
  disable the RPC port for a chart deployment, override
  `--set service.rpcPort=""` (or drop the port from a values file
  copy). Templates skip the port if rpcPort is unset.
- **k8s raw manifests**: the bundled `k8s/service.yaml` and the
  StatefulSet specs include the `rpc` named port. Apply the new
  manifests or merge the additional port into your existing copies.

## How to verify the cutover landed

```bash
# Health: REST endpoint still works, dashboard is up.
curl -fs http://localhost:15002/health

# RPC liveness: HELLO + PING round-trip from any MessagePack-aware
# language. The Python snippet below uses only the stdlib + msgpack.
python3 - <<'PY'
import socket, struct, msgpack
def call(sock, request_id, command, args):
    body = msgpack.packb([request_id, command, args])
    sock.sendall(struct.pack("<I", len(body)) + body)
    length = struct.unpack("<I", sock.recv(4))[0]
    return msgpack.unpackb(sock.recv(length), raw=False)
with socket.create_connection(("localhost", 15503)) as s:
    print(call(s, 1, "HELLO", [{"version": 1}]))
    print(call(s, 2, "PING", []))
PY
```

A healthy server replies with the protocol version (`1`),
authenticated/admin flags, capability list, and `PONG`. Full operator
guide: [`docs/deployment/rpc.md`](../deployment/rpc.md). Byte-level
contract: [`docs/specs/VECTORIZER_RPC.md`](../specs/VECTORIZER_RPC.md).

## SDK readiness

This cutover ships the **server** defaults. First-party SDK constructors
(Rust, Python, TypeScript, Go, C#) flip to RPC as their default
transport in the matching `phase6_sdk-{lang}-rpc` task slots, which
have not been merged at the time of this cutover. Until the SDKs ship:

- The `vectorizer://host:15503` URL scheme is the canonical default
  (see each SDK proposal under `.rulebook/tasks/phase6_sdk-*-rpc/`).
- Existing HTTP-SDK code keeps working — REST is the v2.x default and
  remains available.

When a given SDK ships its RPC client, the corresponding README's
quickstart will show RPC first (per each task's "Default URL scheme"
section). The framework integrations (langchain, langflow, n8n, etc.)
inherit the URL parsing from the underlying SDK and will follow.

## Rollback

Set `rpc.enabled: false` in `config.yml`, restart the server, and
optionally close port `15503` at the firewall. No data is touched and
no other transport changes behaviour. The boot log line
`VectorizerRPC listener disabled in config` confirms the rollback took
effect.
