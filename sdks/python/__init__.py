"""
Hive Vectorizer Python SDK

A Python client library for the Hive Vectorizer service. Starting with
v3.0.0, the **default transport is VectorizerRPC** (binary MessagePack
over TCP, port 15503) — see :mod:`rpc` and ``docs/specs/VECTORIZER_RPC.md``.
The legacy ``VectorizerClient`` (REST over ``aiohttp``) stays available
for browsers, scripting, and ops tooling that already targets HTTP.

Quickstart (sync, RPC default)::

    import vectorizer_sdk
    client = vectorizer_sdk.connect("vectorizer://127.0.0.1:15503")
    client.hello(vectorizer_sdk.HelloPayload(client_name="my-app"))
    print(client.list_collections())

Quickstart (async)::

    import asyncio, vectorizer_sdk

    async def main():
        client = await vectorizer_sdk.connect_async(
            "vectorizer://127.0.0.1:15503"
        )
        await client.hello(vectorizer_sdk.HelloPayload(client_name="my-app"))
        print(await client.list_collections())

    asyncio.run(main())

Author: HiveLLM Team
Version: 3.0.0
License: Apache-2.0
"""

from typing import Optional

from client import VectorizerClient
from exceptions import (
    VectorizerError,
    AuthenticationError,
    CollectionNotFoundError,
    ValidationError,
    NetworkError,
    ServerError,
)
from models import (
    Vector,
    Collection,
    SearchResult,
    EmbeddingRequest,
    SearchRequest,
    CollectionInfo,
    # Hybrid search models
    HybridSearchRequest,
    HybridSearchResponse,
    HybridSearchResult,
    SparseVector,
    # Replication/routing models
    ReadPreference,
    HostConfig,
    ReadOptions,
    # File upload models
    FileUploadRequest,
    FileUploadResponse,
    FileUploadConfig,
)
from rpc import (
    AsyncRpcClient,
    Endpoint,
    HelloPayload,
    HelloResponse,
    RpcClient,
    RpcClientError,
    RpcConnectionClosed,
    RpcNotAuthenticated,
    RpcPool,
    RpcPoolConfig,
    RpcServerError,
    SearchHit,
    VectorizerValue,
    parse_endpoint,
)
# The RPC ``CollectionInfo`` carries different fields than the legacy
# REST one (`models.CollectionInfo` keeps backward compat). Alias the
# new dataclass so both shapes are accessible without shadowing.
from rpc import CollectionInfo as RpcCollectionInfo


def connect(url: str, *, timeout: Optional[float] = None) -> RpcClient:
    """Open a synchronous RPC connection to ``url``.

    ``url`` is parsed by :func:`rpc.parse_endpoint` so any of these
    work: ``vectorizer://host:port``, ``vectorizer://host`` (default
    port 15503), ``host:port`` (no scheme). REST URLs are rejected
    here — use :class:`VectorizerClient` for those.

    Does NOT issue HELLO; the caller MUST call ``client.hello(...)``
    before any data-plane command.
    """
    return RpcClient.connect_url(url, timeout=timeout)


async def connect_async(url: str, *, timeout: Optional[float] = None) -> AsyncRpcClient:
    """Open an asyncio RPC connection to ``url``.

    Same URL contract as :func:`connect`. Does NOT issue HELLO.
    """
    return await AsyncRpcClient.connect_url(url, timeout=timeout)


__version__ = "3.0.0"
__author__ = "HiveLLM Team"
__email__ = "team@hivellm.org"

__all__ = [
    # RPC (default transport in v3.x)
    "AsyncRpcClient",
    "Endpoint",
    "HelloPayload",
    "HelloResponse",
    "RpcClient",
    "RpcClientError",
    "RpcConnectionClosed",
    "RpcNotAuthenticated",
    "RpcPool",
    "RpcPoolConfig",
    "RpcServerError",
    "RpcCollectionInfo",
    "SearchHit",
    "VectorizerValue",
    "connect",
    "connect_async",
    "parse_endpoint",
    # Legacy REST client
    "VectorizerClient",
    "VectorizerError",
    "AuthenticationError",
    "CollectionNotFoundError",
    "ValidationError",
    "NetworkError",
    "ServerError",
    "Vector",
    "Collection",
    "SearchResult",
    "EmbeddingRequest",
    "SearchRequest",
    "CollectionInfo",
    # Hybrid search
    "HybridSearchRequest",
    "HybridSearchResponse",
    "HybridSearchResult",
    "SparseVector",
    # Replication/routing
    "ReadPreference",
    "HostConfig",
    "ReadOptions",
    # File upload
    "FileUploadRequest",
    "FileUploadResponse",
    "FileUploadConfig",
]
