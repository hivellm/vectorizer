"""VectorizerRPC client for Python.

This package implements the binary VectorizerRPC transport (port 15503/tcp)
documented in `docs/specs/VECTORIZER_RPC.md`. It is the default transport
in v3.x; the legacy `aiohttp`-based REST client stays available for
browsers, scripting, and ops tooling that already targets HTTP.

The shapes mirror the Rust SDK at `sdks/rust/src/rpc/` so that polyglot
codebases can switch language without re-learning the surface:

- `parse_endpoint(url)` — canonical URL parser shared with every SDK.
- `RpcClient` / `AsyncRpcClient` — single-connection clients that do
  HELLO + multiplexed call/response by `Request.id`.
- `RpcPool` / `AsyncRpcPool` — minimal bounded connection pools.
- Typed wrappers: `list_collections`, `get_collection_info`,
  `get_vector`, `search_basic`.

Quickstart::

    from rpc import AsyncRpcClient, HelloPayload

    async def main():
        client = await AsyncRpcClient.connect_url(
            "vectorizer://127.0.0.1:15503"
        )
        await client.hello(HelloPayload(client_name="my-app"))
        results = await client.search_basic("my-collection", "query", limit=5)
        for hit in results:
            print(hit.id, hit.score)
"""

from rpc.endpoint import (
    DEFAULT_HTTP_PORT,
    DEFAULT_RPC_PORT,
    Endpoint,
    EndpointParseError,
    parse_endpoint,
)
from rpc.async_client import AsyncRpcClient
from rpc.pool import AsyncPooledClient, AsyncRpcPool, PooledClient, RpcPool, RpcPoolConfig
from rpc.sync_client import (
    HelloPayload,
    HelloResponse,
    RpcClient,
    RpcClientError,
    RpcConnectionClosed,
    RpcNotAuthenticated,
    RpcServerError,
)
from rpc.types import Request, Response, VectorizerValue

# Import for the side effect of attaching typed wrappers as methods on
# RpcClient and AsyncRpcClient. Must come AFTER the client imports.
from rpc.commands import CollectionInfo, SearchHit  # noqa: E402  (intentional ordering)

__all__ = [
    "DEFAULT_HTTP_PORT",
    "DEFAULT_RPC_PORT",
    "AsyncPooledClient",
    "AsyncRpcClient",
    "AsyncRpcPool",
    "CollectionInfo",
    "Endpoint",
    "EndpointParseError",
    "HelloPayload",
    "HelloResponse",
    "PooledClient",
    "Request",
    "Response",
    "RpcClient",
    "RpcClientError",
    "RpcConnectionClosed",
    "RpcNotAuthenticated",
    "RpcPool",
    "RpcPoolConfig",
    "RpcServerError",
    "SearchHit",
    "VectorizerValue",
    "parse_endpoint",
]
