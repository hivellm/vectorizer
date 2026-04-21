"""Typed wrappers around the v1 RPC command catalog.

Each function corresponds to one entry in the wire spec's command
catalog (§ 6). The wrapper:

1. Builds the positional ``args`` array per the spec.
2. Calls :meth:`RpcClient.call` (sync) or :meth:`AsyncRpcClient.call`
   (async).
3. Decodes the :class:`VectorizerValue` response into a typed Python
   value with explicit field handling.

Wrappers are exposed two ways:

- As **methods** on :class:`RpcClient` / :class:`AsyncRpcClient` (the
  ergonomic path used by the README quickstart).
- As **module-level functions** that take a client argument (handy
  when threading a client through helpers without subclassing).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import List, Optional

from rpc.async_client import AsyncRpcClient
from rpc.sync_client import RpcClient, RpcServerError
from rpc.types import VectorizerValue


@dataclass
class CollectionInfo:
    """Collection metadata returned by ``collections.get_info``."""

    name: str
    vector_count: int
    document_count: int
    dimension: int
    metric: str
    created_at: str
    updated_at: str


@dataclass
class SearchHit:
    """One result from ``search.basic``.

    ``payload`` is an optional JSON string. The server stores payloads
    as ``serde_json::Value``; the RPC layer ships them as a string
    because the wire ``VectorizerValue`` enum doesn't model JSON
    directly. Decode with ``json.loads`` if you need structured access.
    """

    id: str
    score: float
    payload: Optional[str] = None


# ── decode helpers (shared between sync + async) ────────────────────────────


def _need_str(value: VectorizerValue, key: str, command: str) -> str:
    v = value.map_get(key)
    s = v.as_str() if v is not None else None
    if s is None:
        raise RpcServerError(f"{command}: missing string field '{key}'")
    return s


def _need_int(value: VectorizerValue, key: str, command: str) -> int:
    v = value.map_get(key)
    i = v.as_int() if v is not None else None
    if i is None:
        raise RpcServerError(f"{command}: missing int field '{key}'")
    return i


def _decode_collection_info(v: VectorizerValue) -> CollectionInfo:
    return CollectionInfo(
        name=_need_str(v, "name", "collections.get_info"),
        vector_count=_need_int(v, "vector_count", "collections.get_info"),
        document_count=_need_int(v, "document_count", "collections.get_info"),
        dimension=_need_int(v, "dimension", "collections.get_info"),
        metric=_need_str(v, "metric", "collections.get_info"),
        created_at=_need_str(v, "created_at", "collections.get_info"),
        updated_at=_need_str(v, "updated_at", "collections.get_info"),
    )


def _decode_collections_list(v: VectorizerValue) -> List[str]:
    arr = v.as_array()
    if arr is None:
        raise RpcServerError("collections.list: expected Array")
    out: List[str] = []
    for item in arr:
        s = item.as_str()
        if s is not None:
            out.append(s)
    return out


def _decode_search_basic(v: VectorizerValue) -> List[SearchHit]:
    arr = v.as_array()
    if arr is None:
        raise RpcServerError("search.basic: expected Array")
    hits: List[SearchHit] = []
    for entry in arr:
        id_v = entry.map_get("id")
        id_str = id_v.as_str() if id_v is not None else None
        if id_str is None:
            raise RpcServerError("search.basic: hit missing 'id'")
        score_v = entry.map_get("score")
        score = score_v.as_float() if score_v is not None else None
        if score is None:
            raise RpcServerError("search.basic: hit missing 'score'")
        payload_v = entry.map_get("payload")
        payload = payload_v.as_str() if payload_v is not None else None
        hits.append(SearchHit(id=id_str, score=score, payload=payload))
    return hits


def _basic_search_args(collection: str, query: str, limit: int) -> List[VectorizerValue]:
    return [
        VectorizerValue.str_(collection),
        VectorizerValue.str_(query),
        VectorizerValue.int_(limit),
    ]


# ── Sync wrappers (module-level functions + monkey-patched onto RpcClient) ──


def list_collections_sync(client: RpcClient) -> List[str]:
    return _decode_collections_list(client.call("collections.list", []))


def get_collection_info_sync(client: RpcClient, name: str) -> CollectionInfo:
    v = client.call("collections.get_info", [VectorizerValue.str_(name)])
    return _decode_collection_info(v)


def get_vector_sync(
    client: RpcClient, collection: str, vector_id: str
) -> VectorizerValue:
    return client.call(
        "vectors.get",
        [VectorizerValue.str_(collection), VectorizerValue.str_(vector_id)],
    )


def search_basic_sync(
    client: RpcClient, collection: str, query: str, limit: int = 10
) -> List[SearchHit]:
    v = client.call("search.basic", _basic_search_args(collection, query, limit))
    return _decode_search_basic(v)


# ── Async wrappers ──────────────────────────────────────────────────────────


async def list_collections_async(client: AsyncRpcClient) -> List[str]:
    return _decode_collections_list(await client.call("collections.list", []))


async def get_collection_info_async(client: AsyncRpcClient, name: str) -> CollectionInfo:
    v = await client.call("collections.get_info", [VectorizerValue.str_(name)])
    return _decode_collection_info(v)


async def get_vector_async(
    client: AsyncRpcClient, collection: str, vector_id: str
) -> VectorizerValue:
    return await client.call(
        "vectors.get",
        [VectorizerValue.str_(collection), VectorizerValue.str_(vector_id)],
    )


async def search_basic_async(
    client: AsyncRpcClient, collection: str, query: str, limit: int = 10
) -> List[SearchHit]:
    v = await client.call("search.basic", _basic_search_args(collection, query, limit))
    return _decode_search_basic(v)


# Attach as methods so callers can write ``client.list_collections()``
# instead of ``list_collections_sync(client)``. This mirrors the Rust
# SDK where every wrapper is an ``impl RpcClient`` method.
RpcClient.list_collections = list_collections_sync  # type: ignore[attr-defined]
RpcClient.get_collection_info = get_collection_info_sync  # type: ignore[attr-defined]
RpcClient.get_vector = get_vector_sync  # type: ignore[attr-defined]
RpcClient.search_basic = search_basic_sync  # type: ignore[attr-defined]

AsyncRpcClient.list_collections = list_collections_async  # type: ignore[attr-defined]
AsyncRpcClient.get_collection_info = get_collection_info_async  # type: ignore[attr-defined]
AsyncRpcClient.get_vector = get_vector_async  # type: ignore[attr-defined]
AsyncRpcClient.search_basic = search_basic_async  # type: ignore[attr-defined]


__all__ = [
    "CollectionInfo",
    "SearchHit",
    "get_collection_info_async",
    "get_collection_info_sync",
    "get_vector_async",
    "get_vector_sync",
    "list_collections_async",
    "list_collections_sync",
    "search_basic_async",
    "search_basic_sync",
]
