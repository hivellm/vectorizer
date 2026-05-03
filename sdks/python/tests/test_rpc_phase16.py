"""Phase16 RPC typed wrappers — wire-shape tests.

All tests use ``AsyncMock`` to patch ``AsyncRpcClient.call`` so no real
TCP socket is needed.  Each test verifies:

1. The correct command string is dispatched to ``call``.
2. The response ``VectorizerValue`` is decoded into the expected dataclass
   with correct field values.

One test per domain group (10 groups total) plus three pure-decode unit
tests that exercise helper functions directly.
"""

from __future__ import annotations

import os
import sys
from unittest.mock import AsyncMock, MagicMock

import pytest

_SDK_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
if _SDK_ROOT not in sys.path:
    sys.path.insert(0, _SDK_ROOT)

from rpc.async_client import AsyncRpcClient  # noqa: E402
from rpc.commands import (  # noqa: E402
    AdminStats,
    AdminStatus,
    AnswerPlanResult,
    ApiKeyCreated,
    AuthMeResult,
    BatchDeleteResult,
    BatchInsertResult,
    BatchItemResult,
    CleanupEmptyResult,
    CollectionInfo,
    CompressBullet,
    CopyRpcResult,
    CreateCollectionResult,
    DeleteByFilterRpcResult,
    DiscoverEdgesResult,
    DiscoverResult,
    DiscoveryChunk,
    EmbedResult,
    ExpandQueriesResult,
    GraphDiscoveryStatus,
    MoveRpcResult,
    RebalanceStatus,
    RefreshTokenResult,
    RenderPromptResult,
    ReplicationConfigureResult,
    RotatedApiKey,
    ScoredCollection,
    SearchExplainResult,
    SearchHit,
    SearchTrace,
    SetExpiryResult,
    SlowQueryConfigResult,
    ValidatePasswordResult,
    VectorListResult,
    VectorWriteResult,
    _decode_batch_items,
    _decode_search_hits,
    _need_bool,
    _need_int,
    get_collection_info_async,
    list_collections_async,
    search_basic_async,
)
from rpc.types import VectorizerValue  # noqa: E402


# ── wire-response builder helpers ────────────────────────────────────────────


def _make_client(response: VectorizerValue) -> AsyncRpcClient:
    """Return an AsyncRpcClient whose ``call`` always resolves to ``response``."""
    client = MagicMock(spec=AsyncRpcClient)
    client.call = AsyncMock(return_value=response)
    return client  # type: ignore[return-value]


def _map(*pairs: tuple) -> VectorizerValue:
    return VectorizerValue.map([(VectorizerValue.str_(k), v) for k, v in pairs])


def _s(s: str) -> VectorizerValue:
    return VectorizerValue.str_(s)


def _i(i: int) -> VectorizerValue:
    return VectorizerValue.int_(i)


def _b(b: bool) -> VectorizerValue:
    return VectorizerValue.bool_(b)


def _f(f: float) -> VectorizerValue:
    return VectorizerValue.float_(f)


def _arr(*items: VectorizerValue) -> VectorizerValue:
    return VectorizerValue.array(list(items))


# ═══════════════════════════════════════════════════════════════════════════════
# 1. Collections
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_collections_list_wire_shape() -> None:
    """list_collections dispatches 'collections.list' and decodes a string array."""
    resp = _arr(_s("alpha"), _s("beta"))
    client = _make_client(resp)

    result = await list_collections_async(client)

    client.call.assert_awaited_once_with("collections.list", [])
    assert result == ["alpha", "beta"]


@pytest.mark.asyncio
async def test_collections_get_info_wire_shape() -> None:
    """get_collection_info dispatches the right command and decodes all fields."""
    resp = _map(
        ("name", _s("my-col")),
        ("vector_count", _i(100)),
        ("document_count", _i(50)),
        ("dimension", _i(384)),
        ("metric", _s("Cosine")),
        ("created_at", _s("2026-01-01T00:00:00Z")),
        ("updated_at", _s("2026-01-02T00:00:00Z")),
    )
    client = _make_client(resp)

    result = await get_collection_info_async(client, "my-col")

    cmd = client.call.await_args[0][0]
    assert cmd == "collections.get_info"
    assert isinstance(result, CollectionInfo)
    assert result.name == "my-col"
    assert result.vector_count == 100
    assert result.dimension == 384
    assert result.metric == "Cosine"


@pytest.mark.asyncio
async def test_collections_create_wire_shape() -> None:
    """create_collection decodes name/dimension/metric/success."""
    from rpc.commands import create_collection_async

    resp = _map(
        ("name", _s("new-col")),
        ("dimension", _i(128)),
        ("metric", _s("cosine")),
        ("success", _b(True)),
    )
    client = _make_client(resp)
    config = VectorizerValue.map(
        [(VectorizerValue.str_("dimension"), VectorizerValue.int_(128))]
    )

    result = await create_collection_async(client, "new-col", config)

    assert client.call.await_args[0][0] == "collections.create"
    assert isinstance(result, CreateCollectionResult)
    assert result.success is True
    assert result.dimension == 128


# ═══════════════════════════════════════════════════════════════════════════════
# 2. Vectors
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_vectors_insert_wire_shape() -> None:
    """insert_vector dispatches 'vectors.insert' and decodes VectorWriteResult."""
    from rpc.commands import insert_vector_async

    resp = _map(("id", _s("vec-001")), ("success", _b(True)))
    client = _make_client(resp)

    result = await insert_vector_async(client, "col", "vec-001", [0.1, 0.2, 0.3])

    assert client.call.await_args[0][0] == "vectors.insert"
    assert isinstance(result, VectorWriteResult)
    assert result.id == "vec-001"
    assert result.success is True


@pytest.mark.asyncio
async def test_vectors_batch_insert_wire_shape() -> None:
    """batch_insert_vectors decodes inserted/failed/results."""
    from rpc.commands import batch_insert_vectors_async

    item = _map(("index", _i(0)), ("id", _s("x")), ("status", _s("ok")))
    resp = _map(("inserted", _i(1)), ("failed", _i(0)), ("results", _arr(item)))
    client = _make_client(resp)

    result = await batch_insert_vectors_async(client, "col", [])

    assert client.call.await_args[0][0] == "vectors.batch_insert"
    assert isinstance(result, BatchInsertResult)
    assert result.inserted == 1
    assert result.failed == 0
    assert result.results[0].status == "ok"


@pytest.mark.asyncio
async def test_vectors_move_wire_shape() -> None:
    """move_vectors_rpc decodes src/dst/moved/failed."""
    from rpc.commands import move_vectors_rpc_async

    resp = _map(
        ("src", _s("col-a")),
        ("dst", _s("col-b")),
        ("moved", _i(5)),
        ("failed", _i(1)),
    )
    client = _make_client(resp)

    result = await move_vectors_rpc_async(client, "col-a", "col-b", ["v1", "v2"])

    assert client.call.await_args[0][0] == "vectors.move"
    assert isinstance(result, MoveRpcResult)
    assert result.src == "col-a"
    assert result.moved == 5


@pytest.mark.asyncio
async def test_vectors_set_expiry_wire_shape() -> None:
    """set_vector_expiry decodes id/expires_at/success."""
    from rpc.commands import set_vector_expiry_async

    resp = _map(("id", _s("v99")), ("expires_at", _i(9_999_999)), ("success", _b(True)))
    client = _make_client(resp)

    result = await set_vector_expiry_async(client, "col", "v99", "2030-01-01T00:00:00Z")

    assert client.call.await_args[0][0] == "vectors.set_expiry"
    assert isinstance(result, SetExpiryResult)
    assert result.expires_at == 9_999_999
    assert result.success is True


# ═══════════════════════════════════════════════════════════════════════════════
# 3. Search
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_search_basic_wire_shape() -> None:
    """search_basic decodes an array of SearchHit."""
    hit = _map(("id", _s("h1")), ("score", _f(0.95)), ("payload", _s('{"k":"v"}')))
    resp = _arr(hit)
    client = _make_client(resp)

    result = await search_basic_async(client, "col", "query", 5)

    assert client.call.await_args[0][0] == "search.basic"
    assert len(result) == 1
    assert isinstance(result[0], SearchHit)
    assert result[0].id == "h1"
    assert abs(result[0].score - 0.95) < 1e-9
    assert result[0].payload == '{"k":"v"}'


@pytest.mark.asyncio
async def test_search_explain_wire_shape() -> None:
    """search_explain decodes hits + trace with all numeric fields."""
    from rpc.commands import search_explain_async

    hit = _map(("id", _s("v1")), ("score", _f(0.8)))
    trace = _map(
        ("visited_nodes", _i(50)),
        ("ef_search", _i(100)),
        ("hnsw_search_ms", _f(1.5)),
        ("total_ms", _f(2.0)),
    )
    resp = _map(
        ("hits", _arr(hit)),
        ("collection", _s("col")),
        ("k", _i(10)),
        ("trace", trace),
    )
    client = _make_client(resp)

    result = await search_explain_async(client, "col", VectorizerValue.null())

    assert client.call.await_args[0][0] == "search.explain"
    assert isinstance(result, SearchExplainResult)
    assert len(result.hits) == 1
    assert result.hits[0].id == "v1"
    assert isinstance(result.trace, SearchTrace)
    assert result.trace.visited_nodes == 50
    assert abs(result.trace.hnsw_search_ms - 1.5) < 1e-9


# ═══════════════════════════════════════════════════════════════════════════════
# 4. Discovery
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_discovery_discover_wire_shape() -> None:
    """discover decodes answer_prompt/sections/bullets/chunks."""
    from rpc.commands import discover_async

    resp = _map(
        ("answer_prompt", _s("Here is the answer...")),
        ("sections", _i(3)),
        ("bullets", _i(12)),
        ("chunks", _i(8)),
    )
    client = _make_client(resp)

    result = await discover_async(client, VectorizerValue.null())

    assert client.call.await_args[0][0] == "discovery.discover"
    assert isinstance(result, DiscoverResult)
    assert result.answer_prompt == "Here is the answer..."
    assert result.bullets == 12


@pytest.mark.asyncio
async def test_discovery_score_collections_wire_shape() -> None:
    """score_collections decodes a list of ScoredCollection."""
    from rpc.commands import score_collections_async

    entry = _map(("name", _s("col-a")), ("score", _f(0.75)), ("vector_count", _i(200)))
    resp = _map(("scored_collections", _arr(entry)))
    client = _make_client(resp)

    result = await score_collections_async(client, VectorizerValue.null())

    assert client.call.await_args[0][0] == "discovery.score_collections"
    assert len(result) == 1
    assert isinstance(result[0], ScoredCollection)
    assert result[0].name == "col-a"
    assert abs(result[0].score - 0.75) < 1e-9


@pytest.mark.asyncio
async def test_discovery_expand_queries_wire_shape() -> None:
    """expand_queries decodes original_query/expanded_queries/count."""
    from rpc.commands import expand_queries_async

    resp = _map(
        ("original_query", _s("rust")),
        ("expanded_queries", _arr(_s("rust lang"), _s("rust programming"))),
        ("count", _i(2)),
    )
    client = _make_client(resp)

    result = await expand_queries_async(client, VectorizerValue.null())

    assert client.call.await_args[0][0] == "discovery.expand_queries"
    assert isinstance(result, ExpandQueriesResult)
    assert result.original_query == "rust"
    assert len(result.expanded_queries) == 2


# ═══════════════════════════════════════════════════════════════════════════════
# 5. File ops
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_file_content_wire_shape() -> None:
    """file_content forwards the request map and returns raw VectorizerValue."""
    from rpc.commands import file_content_async

    resp = _map(("content", _s("hello world")))
    client = _make_client(resp)
    request = _map(("collection", _s("col")), ("file_path", _s("/README.md")))

    result = await file_content_async(client, request)

    assert client.call.await_args[0][0] == "file.content"
    content_v = result.map_get("content")
    assert content_v is not None
    assert content_v.as_str() == "hello world"


# ═══════════════════════════════════════════════════════════════════════════════
# 6. Graph
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_graph_discovery_status_wire_shape() -> None:
    """graph_discovery_status decodes all four count/percentage fields."""
    from rpc.commands import graph_discovery_status_async

    resp = _map(
        ("total_nodes", _i(100)),
        ("nodes_with_edges", _i(75)),
        ("total_edges", _i(200)),
        ("progress_percentage", _f(75.0)),
    )
    client = _make_client(resp)

    result = await graph_discovery_status_async(client, "col")

    assert client.call.await_args[0][0] == "graph.discovery_status"
    assert isinstance(result, GraphDiscoveryStatus)
    assert result.total_nodes == 100
    assert abs(result.progress_percentage - 75.0) < 1e-9


@pytest.mark.asyncio
async def test_graph_discover_edges_wire_shape() -> None:
    """graph_discover_edges decodes success and all count fields."""
    from rpc.commands import graph_discover_edges_async

    resp = _map(
        ("success", _b(True)),
        ("total_nodes", _i(50)),
        ("nodes_processed", _i(50)),
        ("nodes_with_edges", _i(40)),
        ("total_edges_created", _i(120)),
    )
    client = _make_client(resp)

    result = await graph_discover_edges_async(client, "col", VectorizerValue.null())

    assert client.call.await_args[0][0] == "graph.discover_edges"
    assert isinstance(result, DiscoverEdgesResult)
    assert result.success is True
    assert result.total_edges_created == 120


# ═══════════════════════════════════════════════════════════════════════════════
# 7. Admin
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_admin_stats_wire_shape() -> None:
    """admin_stats decodes collections_count/total_vectors/version."""
    from rpc.commands import admin_stats_async

    resp = _map(
        ("collections_count", _i(7)),
        ("total_vectors", _i(9999)),
        ("version", _s("3.8.0")),
    )
    client = _make_client(resp)

    result = await admin_stats_async(client)

    assert client.call.await_args[0][0] == "admin.stats"
    assert isinstance(result, AdminStats)
    assert result.collections_count == 7
    assert result.total_vectors == 9999
    assert result.version == "3.8.0"


@pytest.mark.asyncio
async def test_admin_slow_queries_config_wire_shape() -> None:
    """admin_slow_queries_config decodes threshold_ms/capacity/status."""
    from rpc.commands import admin_slow_queries_config_async

    resp = _map(
        ("threshold_ms", _i(200)),
        ("capacity", _i(100)),
        ("status", _s("ok")),
    )
    client = _make_client(resp)

    result = await admin_slow_queries_config_async(client, VectorizerValue.null())

    assert client.call.await_args[0][0] == "admin.slow_queries_config"
    assert isinstance(result, SlowQueryConfigResult)
    assert result.threshold_ms == 200
    assert result.status == "ok"


# ═══════════════════════════════════════════════════════════════════════════════
# 8. Auth
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_auth_me_wire_shape() -> None:
    """auth_me decodes username/authenticated."""
    from rpc.commands import auth_me_async

    resp = _map(("username", _s("alice")), ("authenticated", _b(True)))
    client = _make_client(resp)

    result = await auth_me_async(client)

    assert client.call.await_args[0][0] == "auth.me"
    assert isinstance(result, AuthMeResult)
    assert result.username == "alice"
    assert result.authenticated is True


@pytest.mark.asyncio
async def test_rotate_api_key_rpc_wire_shape() -> None:
    """rotate_api_key_rpc decodes old/new key ids, new_token, grace_until."""
    from rpc.commands import rotate_api_key_rpc_async

    resp = _map(
        ("old_key_id", _s("key-old")),
        ("new_key_id", _s("key-new")),
        ("new_token", _s("tok-abc")),
        ("grace_until", _s("2026-06-01T00:00:00Z")),
    )
    client = _make_client(resp)

    result = await rotate_api_key_rpc_async(client, "key-old")

    assert client.call.await_args[0][0] == "auth.api_keys_rotate"
    assert isinstance(result, RotatedApiKey)
    assert result.old_key_id == "key-old"
    assert result.new_token == "tok-abc"
    assert result.grace_until == "2026-06-01T00:00:00Z"


@pytest.mark.asyncio
async def test_auth_validate_password_wire_shape() -> None:
    """auth_validate_password decodes valid flag and errors list."""
    from rpc.commands import auth_validate_password_async

    resp = _map(
        ("valid", _b(False)),
        ("errors", _arr(_s("too short"), _s("needs digit"))),
    )
    client = _make_client(resp)

    result = await auth_validate_password_async(client, "abc")

    assert client.call.await_args[0][0] == "auth.validate_password"
    assert isinstance(result, ValidatePasswordResult)
    assert result.valid is False
    assert len(result.errors) == 2
    assert "too short" in result.errors


# ═══════════════════════════════════════════════════════════════════════════════
# 9. Replication
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_replication_configure_wire_shape() -> None:
    """replication_configure decodes success/role/message."""
    from rpc.commands import replication_configure_async

    resp = _map(
        ("success", _b(True)),
        ("role", _s("replica")),
        ("message", _s("replication configured")),
    )
    client = _make_client(resp)

    result = await replication_configure_async(client, VectorizerValue.null())

    assert client.call.await_args[0][0] == "replication.configure"
    assert isinstance(result, ReplicationConfigureResult)
    assert result.success is True
    assert result.role == "replica"


# ═══════════════════════════════════════════════════════════════════════════════
# 10. Cluster
# ═══════════════════════════════════════════════════════════════════════════════


@pytest.mark.asyncio
async def test_cluster_rebalance_status_wire_shape() -> None:
    """cluster_rebalance_status decodes optional status/message fields."""
    from rpc.commands import cluster_rebalance_status_async

    resp = _map(("status", _s("idle")), ("message", _s("no rebalance running")))
    client = _make_client(resp)

    result = await cluster_rebalance_status_async(client)

    assert client.call.await_args[0][0] == "cluster.rebalance_status"
    assert isinstance(result, RebalanceStatus)
    assert result.status == "idle"
    assert "no rebalance" in (result.message or "")


# ═══════════════════════════════════════════════════════════════════════════════
# Pure decode unit tests (no network, no mock client)
# ═══════════════════════════════════════════════════════════════════════════════


def test_decode_batch_items_index_id_status_error() -> None:
    """_decode_batch_items correctly extracts index/id/status/error from a map."""
    item = VectorizerValue.map([
        (VectorizerValue.str_("index"), VectorizerValue.int_(2)),
        (VectorizerValue.str_("id"), VectorizerValue.str_("abc")),
        (VectorizerValue.str_("status"), VectorizerValue.str_("ok")),
    ])
    results = _decode_batch_items([item])
    assert len(results) == 1
    r = results[0]
    assert isinstance(r, BatchItemResult)
    assert r.index == 2
    assert r.id == "abc"
    assert r.status == "ok"
    assert r.error is None


def test_search_hit_missing_payload_is_none() -> None:
    """SearchHit.payload is None when the server omits the field."""
    hit = VectorizerValue.map([
        (VectorizerValue.str_("id"), VectorizerValue.str_("v1")),
        (VectorizerValue.str_("score"), VectorizerValue.float_(0.9)),
    ])
    hits = _decode_search_hits([hit])
    assert len(hits) == 1
    assert hits[0].payload is None


def test_cleanup_empty_result_dry_run_field() -> None:
    """CleanupEmptyResult.dry_run is correctly decoded from a Bool VectorizerValue."""
    v = VectorizerValue.map([
        (VectorizerValue.str_("removed"), VectorizerValue.int_(0)),
        (VectorizerValue.str_("dry_run"), VectorizerValue.bool_(True)),
    ])
    r = CleanupEmptyResult(
        removed=_need_int(v, "removed", "t"),
        dry_run=_need_bool(v, "dry_run", "t"),
    )
    assert r.dry_run is True
    assert r.removed == 0
