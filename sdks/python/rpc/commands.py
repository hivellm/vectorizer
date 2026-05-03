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


# ── Return dataclasses ───────────────────────────────────────────────────────


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
    """One result from ``search.basic`` / ``search.by_text`` etc.

    ``payload`` is an optional JSON string. Decode with ``json.loads``
    if you need structured access.
    """

    id: str
    score: float
    payload: Optional[str] = None


@dataclass
class CreateCollectionResult:
    """Response from ``collections.create``."""

    name: str
    dimension: int
    metric: str
    success: bool


@dataclass
class CleanupEmptyResult:
    """Response from ``collections.cleanup_empty``."""

    removed: int
    dry_run: bool


@dataclass
class VectorWriteResult:
    """Response from ``vectors.insert`` / ``vectors.insert_text`` / ``vectors.update``."""

    id: str
    success: bool


@dataclass
class BatchItemResult:
    """Per-item result inside batch responses."""

    index: int
    id: Optional[str]
    status: str
    error: Optional[str]


@dataclass
class BatchInsertResult:
    """Response from ``vectors.batch_insert`` / ``vectors.batch_insert_texts``."""

    inserted: int
    failed: int
    results: List[BatchItemResult]


@dataclass
class BatchUpdateResult:
    """Response from ``vectors.batch_update``."""

    updated: int
    failed: int
    results: List[BatchItemResult]


@dataclass
class BatchDeleteResult:
    """Response from ``vectors.batch_delete``."""

    deleted: int
    failed: int
    results: List[BatchItemResult]


@dataclass
class BatchSearchResult:
    """One per-query result from ``vectors.batch_search``."""

    index: int
    status: str
    results: List[SearchHit]
    error: Optional[str]


@dataclass
class MoveRpcResult:
    """Response from ``vectors.move``."""

    src: str
    dst: str
    moved: int
    failed: int


@dataclass
class CopyRpcResult:
    """Response from ``vectors.copy``."""

    src: str
    dst: str
    copied: int
    failed: int


@dataclass
class DeleteByFilterRpcResult:
    """Response from ``vectors.delete_by_filter``."""

    scanned: int
    matched: int
    deleted: int


@dataclass
class BulkUpdateMetadataRpcResult:
    """Response from ``vectors.bulk_update_metadata``."""

    scanned: int
    matched: int
    updated: int


@dataclass
class SetExpiryResult:
    """Response from ``vectors.set_expiry``."""

    id: str
    expires_at: int
    success: bool


@dataclass
class EmbedResult:
    """Response from ``vectors.embed``."""

    embedding: List[float]
    model: str
    dimension: int


@dataclass
class VectorListResult:
    """Response from ``vectors.list``."""

    items: List[VectorizerValue]
    total: int
    page: int
    limit: int


@dataclass
class SearchTrace:
    """HNSW traversal trace from ``search.explain``."""

    visited_nodes: int
    ef_search: int
    hnsw_search_ms: float
    total_ms: float


@dataclass
class SearchExplainResult:
    """Response from ``search.explain``."""

    hits: List[SearchHit]
    collection: str
    k: int
    trace: SearchTrace


@dataclass
class DiscoverResult:
    """Summary response from ``discovery.discover``."""

    answer_prompt: str
    sections: int
    bullets: int
    chunks: int


@dataclass
class ScoredCollection:
    """One scored collection from ``discovery.score_collections``."""

    name: str
    score: float
    vector_count: int


@dataclass
class ExpandQueriesResult:
    """Response from ``discovery.expand_queries``."""

    original_query: str
    expanded_queries: List[str]
    count: int


@dataclass
class DiscoveryChunk:
    """One chunk from ``discovery.broad_discovery`` / ``discovery.semantic_focus``."""

    collection: str
    score: float
    content_preview: str


@dataclass
class CompressBullet:
    """One bullet from ``discovery.compress_evidence``."""

    text: str
    source_id: str
    score: float


@dataclass
class AnswerPlanSection:
    """One section inside an answer plan."""

    title: str
    bullets_count: int


@dataclass
class AnswerPlanResult:
    """Response from ``discovery.build_answer_plan``."""

    sections: List[AnswerPlanSection]
    total_bullets: int


@dataclass
class RenderPromptResult:
    """Response from ``discovery.render_llm_prompt``."""

    prompt: str
    length: int
    estimated_tokens: int


@dataclass
class GraphDiscoveryStatus:
    """Response from ``graph.discovery_status``."""

    total_nodes: int
    nodes_with_edges: int
    total_edges: int
    progress_percentage: float


@dataclass
class DiscoverEdgesResult:
    """Response from ``graph.discover_edges``."""

    success: bool
    total_nodes: int
    nodes_processed: int
    nodes_with_edges: int
    total_edges_created: int


@dataclass
class DiscoverEdgesForNodeResult:
    """Response from ``graph.discover_edges_for_node``."""

    success: bool
    node_id: str
    edges_created: int


@dataclass
class AdminStats:
    """Admin stats response from ``admin.stats``."""

    collections_count: int
    total_vectors: int
    version: str


@dataclass
class AdminStatus:
    """Admin status response from ``admin.status``."""

    ready: bool
    collections_count: int
    version: str


@dataclass
class SlowQueryConfigResult:
    """Response from ``admin.slow_queries_config``."""

    threshold_ms: int
    capacity: int
    status: str


@dataclass
class AuthMeResult:
    """Response from ``auth.me``."""

    username: str
    authenticated: bool


@dataclass
class RefreshTokenResult:
    """Response from ``auth.refresh_token``."""

    access_token: str
    token_type: str


@dataclass
class ValidatePasswordResult:
    """Response from ``auth.validate_password``."""

    valid: bool
    errors: List[str]


@dataclass
class ApiKeyCreated:
    """Response from ``auth.api_keys_create`` / ``auth.api_keys_create_scoped``."""

    api_key: str
    id: str
    name: str


@dataclass
class RotatedApiKey:
    """Response from ``auth.api_keys_rotate``."""

    old_key_id: str
    new_key_id: str
    new_token: str
    grace_until: Optional[str]


@dataclass
class ReplicationConfigureResult:
    """Response from ``replication.configure``."""

    success: bool
    role: str
    message: str


@dataclass
class RebalanceStatus:
    """Response from ``cluster.rebalance_status``."""

    status: Optional[str]
    message: Optional[str]


# ── decode helpers ───────────────────────────────────────────────────────────


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


def _need_bool(value: VectorizerValue, key: str, command: str) -> bool:
    v = value.map_get(key)
    b = v.as_bool() if v is not None else None
    if b is None:
        raise RpcServerError(f"{command}: missing bool field '{key}'")
    return b


def _opt_str(value: VectorizerValue, key: str) -> Optional[str]:
    v = value.map_get(key)
    return v.as_str() if v is not None else None


def _opt_int(value: VectorizerValue, key: str, default: int = 0) -> int:
    v = value.map_get(key)
    return (v.as_int() or default) if v is not None else default


def _opt_float(value: VectorizerValue, key: str, default: float = 0.0) -> float:
    v = value.map_get(key)
    return (v.as_float() or default) if v is not None else default


def _opt_bool(value: VectorizerValue, key: str, default: bool = False) -> bool:
    v = value.map_get(key)
    b = v.as_bool() if v is not None else None
    return b if b is not None else default


def _decode_string_array(v: VectorizerValue, cmd: str) -> List[str]:
    arr = v.as_array()
    if arr is None:
        raise RpcServerError(f"{cmd}: expected Array")
    return [item.as_str() for item in arr if item.as_str() is not None]  # type: ignore[misc]


def _decode_search_hits(arr: List[VectorizerValue]) -> List[SearchHit]:
    hits: List[SearchHit] = []
    for entry in arr:
        id_v = entry.map_get("id")
        id_str = id_v.as_str() if id_v is not None else None
        if id_str is None:
            continue
        score = _opt_float(entry, "score", 0.0)
        payload = _opt_str(entry, "payload")
        hits.append(SearchHit(id=id_str, score=score, payload=payload))
    return hits


def _decode_batch_items(arr: List[VectorizerValue]) -> List[BatchItemResult]:
    out: List[BatchItemResult] = []
    for entry in arr:
        out.append(BatchItemResult(
            index=_opt_int(entry, "index", 0),
            id=_opt_str(entry, "id"),
            status=_opt_str(entry, "status") or "unknown",
            error=_opt_str(entry, "error"),
        ))
    return out


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


# ═══════════════════════════════════════════════════════════════════════════════
# Collections — sync
# ═══════════════════════════════════════════════════════════════════════════════


def list_collections_sync(client: RpcClient) -> List[str]:
    """``collections.list`` — return every collection name."""
    return _decode_string_array(client.call("collections.list", []), "collections.list")


def get_collection_info_sync(client: RpcClient, name: str) -> CollectionInfo:
    """``collections.get_info`` — return metadata for one collection."""
    v = client.call("collections.get_info", [VectorizerValue.str_(name)])
    return _decode_collection_info(v)


def create_collection_sync(
    client: RpcClient, name: str, config: VectorizerValue
) -> CreateCollectionResult:
    """``collections.create`` — create a new collection."""
    v = client.call("collections.create", [VectorizerValue.str_(name), config])
    return CreateCollectionResult(
        name=_need_str(v, "name", "collections.create"),
        dimension=_need_int(v, "dimension", "collections.create"),
        metric=_need_str(v, "metric", "collections.create"),
        success=_need_bool(v, "success", "collections.create"),
    )


def delete_collection_sync(client: RpcClient, name: str) -> bool:
    """``collections.delete`` — delete a collection (admin-gated)."""
    v = client.call("collections.delete", [VectorizerValue.str_(name)])
    return _need_bool(v, "success", "collections.delete")


def list_empty_collections_sync(client: RpcClient) -> List[str]:
    """``collections.list_empty`` — list collections with zero vectors."""
    return _decode_string_array(client.call("collections.list_empty", []), "collections.list_empty")


def cleanup_empty_collections_sync(client: RpcClient, dry_run: bool = False) -> CleanupEmptyResult:
    """``collections.cleanup_empty`` — remove empty collections."""
    config = VectorizerValue.map([(VectorizerValue.str_("dry_run"), VectorizerValue.bool_(dry_run))])
    v = client.call("collections.cleanup_empty", [config])
    return CleanupEmptyResult(
        removed=_need_int(v, "removed", "collections.cleanup_empty"),
        dry_run=_need_bool(v, "dry_run", "collections.cleanup_empty"),
    )


def force_save_collection_sync(client: RpcClient, name: str) -> bool:
    """``collections.force_save`` — flush collection to disk."""
    v = client.call("collections.force_save", [VectorizerValue.str_(name)])
    return _need_bool(v, "success", "collections.force_save")


# ═══════════════════════════════════════════════════════════════════════════════
# Vectors — sync
# ═══════════════════════════════════════════════════════════════════════════════


def get_vector_sync(client: RpcClient, collection: str, vector_id: str) -> VectorizerValue:
    """``vectors.get`` — fetch one vector by id."""
    return client.call("vectors.get", [VectorizerValue.str_(collection), VectorizerValue.str_(vector_id)])


def insert_vector_sync(
    client: RpcClient,
    collection: str,
    id: Optional[str],
    data: List[float],
    payload: Optional[VectorizerValue] = None,
) -> VectorWriteResult:
    """``vectors.insert`` — insert one pre-computed vector."""
    id_val = VectorizerValue.str_(id) if id else VectorizerValue.null()
    data_val = VectorizerValue.array([VectorizerValue.float_(f) for f in data])
    args = [VectorizerValue.str_(collection), id_val, data_val]
    if payload is not None:
        args.append(payload)
    v = client.call("vectors.insert", args)
    return VectorWriteResult(id=_need_str(v, "id", "vectors.insert"), success=_need_bool(v, "success", "vectors.insert"))


def insert_text_vector_sync(
    client: RpcClient,
    collection: str,
    id: Optional[str],
    text: str,
    payload: Optional[VectorizerValue] = None,
) -> VectorWriteResult:
    """``vectors.insert_text`` — embed text server-side and insert."""
    id_val = VectorizerValue.str_(id) if id else VectorizerValue.null()
    args = [VectorizerValue.str_(collection), id_val, VectorizerValue.str_(text)]
    if payload is not None:
        args.append(payload)
    v = client.call("vectors.insert_text", args)
    return VectorWriteResult(id=_need_str(v, "id", "vectors.insert_text"), success=_need_bool(v, "success", "vectors.insert_text"))


def update_vector_sync(
    client: RpcClient,
    collection: str,
    id: str,
    data: List[float],
    payload: Optional[VectorizerValue] = None,
) -> VectorWriteResult:
    """``vectors.update`` — replace a vector's data and/or payload."""
    data_val = VectorizerValue.array([VectorizerValue.float_(f) for f in data])
    args = [VectorizerValue.str_(collection), VectorizerValue.str_(id), data_val]
    if payload is not None:
        args.append(payload)
    v = client.call("vectors.update", args)
    return VectorWriteResult(id=_need_str(v, "id", "vectors.update"), success=_need_bool(v, "success", "vectors.update"))


def delete_vector_rpc_sync(client: RpcClient, collection: str, id: str) -> bool:
    """``vectors.delete`` — delete one vector by id."""
    v = client.call("vectors.delete", [VectorizerValue.str_(collection), VectorizerValue.str_(id)])
    return _need_bool(v, "success", "vectors.delete")


def list_vectors_sync(client: RpcClient, collection: str, page: int = 0, limit: int = 20) -> VectorListResult:
    """``vectors.list`` — page through vectors in a collection."""
    v = client.call("vectors.list", [VectorizerValue.str_(collection), VectorizerValue.int_(page), VectorizerValue.int_(limit)])
    items_v = v.map_get("items")
    items = items_v.as_array() if items_v is not None else []
    return VectorListResult(
        items=list(items) if items else [],
        total=_need_int(v, "total", "vectors.list"),
        page=_need_int(v, "page", "vectors.list"),
        limit=_need_int(v, "limit", "vectors.list"),
    )


def embed_text_sync(client: RpcClient, text: str, model: Optional[str] = None) -> EmbedResult:
    """``vectors.embed`` — embed text server-side and return the embedding."""
    args = [VectorizerValue.str_(text)]
    if model is not None:
        args.append(VectorizerValue.str_(model))
    v = client.call("vectors.embed", args)
    emb_v = v.map_get("embedding")
    embedding = [x.as_float() for x in (emb_v.as_array() or []) if x.as_float() is not None]  # type: ignore[misc]
    return EmbedResult(
        embedding=embedding,
        model=_opt_str(v, "model") or "bm25",
        dimension=_opt_int(v, "dimension", 0),
    )


def batch_insert_vectors_sync(client: RpcClient, collection: str, items: List[VectorizerValue]) -> BatchInsertResult:
    """``vectors.batch_insert`` — insert multiple pre-computed vectors."""
    v = client.call("vectors.batch_insert", [VectorizerValue.str_(collection), VectorizerValue.array(items)])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchInsertResult(inserted=_opt_int(v, "inserted"), failed=_opt_int(v, "failed"), results=results)


def batch_insert_texts_sync(client: RpcClient, collection: str, items: List[VectorizerValue]) -> BatchInsertResult:
    """``vectors.batch_insert_texts`` — embed and insert multiple text items."""
    v = client.call("vectors.batch_insert_texts", [VectorizerValue.str_(collection), VectorizerValue.array(items)])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchInsertResult(inserted=_opt_int(v, "inserted"), failed=_opt_int(v, "failed"), results=results)


def batch_search_sync(client: RpcClient, requests: List[VectorizerValue]) -> List[BatchSearchResult]:
    """``vectors.batch_search`` — run multiple searches in one round-trip."""
    v = client.call("vectors.batch_search", [VectorizerValue.array(requests)])
    arr = v.as_array()
    if arr is None:
        raise RpcServerError("vectors.batch_search: expected Array")
    out: List[BatchSearchResult] = []
    for entry in arr:
        results_v = entry.map_get("results")
        hits = _decode_search_hits(results_v.as_array() or []) if results_v else []
        out.append(BatchSearchResult(
            index=_opt_int(entry, "index", 0),
            status=_opt_str(entry, "status") or "unknown",
            results=hits,
            error=_opt_str(entry, "error"),
        ))
    return out


def batch_update_vectors_sync(client: RpcClient, collection: str, updates: List[VectorizerValue]) -> BatchUpdateResult:
    """``vectors.batch_update`` — update multiple vectors."""
    v = client.call("vectors.batch_update", [VectorizerValue.str_(collection), VectorizerValue.array(updates)])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchUpdateResult(updated=_opt_int(v, "updated"), failed=_opt_int(v, "failed"), results=results)


def batch_delete_vectors_sync(client: RpcClient, collection: str, ids: List[str]) -> BatchDeleteResult:
    """``vectors.batch_delete`` — delete multiple vectors by id."""
    ids_val = VectorizerValue.array([VectorizerValue.str_(i) for i in ids])
    v = client.call("vectors.batch_delete", [VectorizerValue.str_(collection), ids_val])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchDeleteResult(deleted=_opt_int(v, "deleted"), failed=_opt_int(v, "failed"), results=results)


def move_vectors_rpc_sync(client: RpcClient, src: str, dst: str, ids: List[str]) -> MoveRpcResult:
    """``vectors.move`` — move vectors from ``src`` to ``dst`` collection."""
    ids_val = VectorizerValue.array([VectorizerValue.str_(i) for i in ids])
    v = client.call("vectors.move", [VectorizerValue.str_(src), VectorizerValue.str_(dst), ids_val])
    return MoveRpcResult(
        src=_need_str(v, "src", "vectors.move"),
        dst=_need_str(v, "dst", "vectors.move"),
        moved=_opt_int(v, "moved"),
        failed=_opt_int(v, "failed"),
    )


def copy_vectors_rpc_sync(client: RpcClient, src: str, dst: str, ids: List[str]) -> CopyRpcResult:
    """``vectors.copy`` — copy vectors without deleting source."""
    ids_val = VectorizerValue.array([VectorizerValue.str_(i) for i in ids])
    v = client.call("vectors.copy", [VectorizerValue.str_(src), VectorizerValue.str_(dst), ids_val])
    return CopyRpcResult(
        src=_need_str(v, "src", "vectors.copy"),
        dst=_need_str(v, "dst", "vectors.copy"),
        copied=_opt_int(v, "copied"),
        failed=_opt_int(v, "failed"),
    )


def delete_by_filter_rpc_sync(client: RpcClient, collection: str, filter: VectorizerValue) -> DeleteByFilterRpcResult:
    """``vectors.delete_by_filter`` — delete all vectors matching a filter."""
    v = client.call("vectors.delete_by_filter", [VectorizerValue.str_(collection), filter])
    return DeleteByFilterRpcResult(scanned=_opt_int(v, "scanned"), matched=_opt_int(v, "matched"), deleted=_opt_int(v, "deleted"))


def bulk_update_metadata_rpc_sync(
    client: RpcClient, collection: str, filter: VectorizerValue, patch: VectorizerValue
) -> BulkUpdateMetadataRpcResult:
    """``vectors.bulk_update_metadata`` — apply a JSON-merge-patch to matching vectors."""
    v = client.call("vectors.bulk_update_metadata", [VectorizerValue.str_(collection), filter, patch])
    return BulkUpdateMetadataRpcResult(scanned=_opt_int(v, "scanned"), matched=_opt_int(v, "matched"), updated=_opt_int(v, "updated"))


def set_vector_expiry_sync(client: RpcClient, collection: str, id: str, expires_at: str) -> SetExpiryResult:
    """``vectors.set_expiry`` — attach a TTL to one vector."""
    v = client.call("vectors.set_expiry", [VectorizerValue.str_(collection), VectorizerValue.str_(id), VectorizerValue.str_(expires_at)])
    return SetExpiryResult(id=_need_str(v, "id", "vectors.set_expiry"), expires_at=_need_int(v, "expires_at", "vectors.set_expiry"), success=_need_bool(v, "success", "vectors.set_expiry"))


# ═══════════════════════════════════════════════════════════════════════════════
# Search — sync
# ═══════════════════════════════════════════════════════════════════════════════


def search_basic_sync(client: RpcClient, collection: str, query: str, limit: int = 10) -> List[SearchHit]:
    """``search.basic`` — search a collection and return hits."""
    v = client.call("search.basic", [VectorizerValue.str_(collection), VectorizerValue.str_(query), VectorizerValue.int_(limit)])
    arr = v.as_array()
    if arr is None:
        raise RpcServerError("search.basic: expected Array")
    return _decode_search_hits(arr)


def search_intelligent_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.intelligent`` — multi-collection intelligent search."""
    return client.call("search.intelligent", [request])


def search_by_text_sync(client: RpcClient, collection: str, query: str, limit: int = 10) -> List[SearchHit]:
    """``search.by_text`` — search one collection by text query."""
    v = client.call("search.by_text", [VectorizerValue.str_(collection), VectorizerValue.str_(query), VectorizerValue.int_(limit)])
    results_v = v.map_get("results")
    arr = results_v.as_array() if results_v is not None else None
    if arr is None:
        raise RpcServerError("search.by_text: missing results array")
    return _decode_search_hits(arr)


def search_by_file_sync(client: RpcClient, collection: str, request: VectorizerValue) -> List[SearchHit]:
    """``search.by_file`` — file-content-based search."""
    v = client.call("search.by_file", [VectorizerValue.str_(collection), request])
    results_v = v.map_get("results")
    arr = results_v.as_array() if results_v is not None else None
    if arr is None:
        raise RpcServerError("search.by_file: missing results array")
    return _decode_search_hits(arr)


def search_hybrid_sync(client: RpcClient, collection: str, request: VectorizerValue) -> List[SearchHit]:
    """``search.hybrid`` — RRF / weighted-combination hybrid search."""
    v = client.call("search.hybrid", [VectorizerValue.str_(collection), request])
    results_v = v.map_get("results")
    arr = results_v.as_array() if results_v is not None else None
    if arr is None:
        raise RpcServerError("search.hybrid: missing results array")
    return _decode_search_hits(arr)


def search_semantic_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.semantic`` — semantic re-ranking search."""
    return client.call("search.semantic", [request])


def search_contextual_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.contextual`` — context-filtered semantic search."""
    return client.call("search.contextual", [request])


def search_multi_collection_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.multi_collection`` — fan-out search across multiple collections."""
    return client.call("search.multi_collection", [request])


def search_explain_sync(client: RpcClient, collection: str, request: VectorizerValue) -> SearchExplainResult:
    """``search.explain`` — run a search and return HNSW traversal trace."""
    v = client.call("search.explain", [VectorizerValue.str_(collection), request])
    hits_v = v.map_get("hits")
    hits = _decode_search_hits(hits_v.as_array() or []) if hits_v else []
    trace_v = v.map_get("trace") or VectorizerValue.null()
    trace = SearchTrace(
        visited_nodes=_opt_int(trace_v, "visited_nodes"),
        ef_search=_opt_int(trace_v, "ef_search"),
        hnsw_search_ms=_opt_float(trace_v, "hnsw_search_ms"),
        total_ms=_opt_float(trace_v, "total_ms"),
    )
    return SearchExplainResult(
        hits=hits,
        collection=_opt_str(v, "collection") or "",
        k=_opt_int(v, "k"),
        trace=trace,
    )


# ═══════════════════════════════════════════════════════════════════════════════
# Discovery — sync
# ═══════════════════════════════════════════════════════════════════════════════


def discover_sync(client: RpcClient, request: VectorizerValue) -> DiscoverResult:
    """``discovery.discover`` — full discovery pipeline."""
    v = client.call("discovery.discover", [request])
    return DiscoverResult(
        answer_prompt=_need_str(v, "answer_prompt", "discovery.discover"),
        sections=_opt_int(v, "sections"),
        bullets=_opt_int(v, "bullets"),
        chunks=_opt_int(v, "chunks"),
    )


def filter_collections_sync(client: RpcClient, request: VectorizerValue) -> List[str]:
    """``discovery.filter_collections`` — filter collection list by query relevance."""
    v = client.call("discovery.filter_collections", [request])
    fc_v = v.map_get("filtered_collections")
    if fc_v is None:
        raise RpcServerError("discovery.filter_collections: missing filtered_collections")
    arr = fc_v.as_array() or []
    out: List[str] = []
    for entry in arr:
        n = entry.map_get("name")
        if n is not None:
            s = n.as_str()
            if s:
                out.append(s)
    return out


def score_collections_sync(client: RpcClient, request: VectorizerValue) -> List[ScoredCollection]:
    """``discovery.score_collections`` — score all collections for a query."""
    v = client.call("discovery.score_collections", [request])
    sc_v = v.map_get("scored_collections")
    if sc_v is None:
        raise RpcServerError("discovery.score_collections: missing scored_collections")
    arr = sc_v.as_array() or []
    return [ScoredCollection(
        name=_opt_str(entry, "name") or "",
        score=_opt_float(entry, "score"),
        vector_count=_opt_int(entry, "vector_count"),
    ) for entry in arr]


def expand_queries_sync(client: RpcClient, request: VectorizerValue) -> ExpandQueriesResult:
    """``discovery.expand_queries`` — generate query variants."""
    v = client.call("discovery.expand_queries", [request])
    eq_v = v.map_get("expanded_queries")
    expanded = [x.as_str() for x in (eq_v.as_array() or []) if x.as_str() is not None]  # type: ignore[misc]
    return ExpandQueriesResult(
        original_query=_need_str(v, "original_query", "discovery.expand_queries"),
        expanded_queries=expanded,
        count=_opt_int(v, "count"),
    )


def broad_discovery_sync(client: RpcClient, request: VectorizerValue) -> List[DiscoveryChunk]:
    """``discovery.broad_discovery`` — multi-query broad search."""
    v = client.call("discovery.broad_discovery", [request])
    chunks_v = v.map_get("chunks")
    if chunks_v is None:
        raise RpcServerError("discovery.broad_discovery: missing chunks")
    return [DiscoveryChunk(
        collection=_opt_str(entry, "collection") or "",
        score=_opt_float(entry, "score"),
        content_preview=_opt_str(entry, "content_preview") or "",
    ) for entry in (chunks_v.as_array() or [])]


def semantic_focus_sync(client: RpcClient, request: VectorizerValue) -> List[DiscoveryChunk]:
    """``discovery.semantic_focus`` — deep semantic search within one collection."""
    v = client.call("discovery.semantic_focus", [request])
    chunks_v = v.map_get("chunks")
    if chunks_v is None:
        raise RpcServerError("discovery.semantic_focus: missing chunks")
    return [DiscoveryChunk(
        collection=_opt_str(entry, "collection") or "",
        score=_opt_float(entry, "score"),
        content_preview=_opt_str(entry, "content_preview") or "",
    ) for entry in (chunks_v.as_array() or [])]


def promote_readme_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``discovery.promote_readme`` — promote README chunks to the top."""
    return client.call("discovery.promote_readme", [request])


def compress_evidence_sync(client: RpcClient, request: VectorizerValue) -> List[CompressBullet]:
    """``discovery.compress_evidence`` — compress a chunk set into ranked bullets."""
    v = client.call("discovery.compress_evidence", [request])
    bullets_v = v.map_get("bullets")
    if bullets_v is None:
        raise RpcServerError("discovery.compress_evidence: missing bullets")
    return [CompressBullet(
        text=_opt_str(entry, "text") or "",
        source_id=_opt_str(entry, "source_id") or "",
        score=_opt_float(entry, "score"),
    ) for entry in (bullets_v.as_array() or [])]


def build_answer_plan_sync(client: RpcClient, request: VectorizerValue) -> AnswerPlanResult:
    """``discovery.build_answer_plan`` — organise bullets into a structured answer plan."""
    v = client.call("discovery.build_answer_plan", [request])
    sections_v = v.map_get("sections")
    sections = [AnswerPlanSection(
        title=_opt_str(entry, "title") or "",
        bullets_count=_opt_int(entry, "bullets_count"),
    ) for entry in (sections_v.as_array() or [])] if sections_v else []
    return AnswerPlanResult(sections=sections, total_bullets=_opt_int(v, "total_bullets"))


def render_llm_prompt_sync(client: RpcClient, request: VectorizerValue) -> RenderPromptResult:
    """``discovery.render_llm_prompt`` — render an answer plan into an LLM prompt."""
    v = client.call("discovery.render_llm_prompt", [request])
    return RenderPromptResult(
        prompt=_need_str(v, "prompt", "discovery.render_llm_prompt"),
        length=_opt_int(v, "length"),
        estimated_tokens=_opt_int(v, "estimated_tokens"),
    )


# ═══════════════════════════════════════════════════════════════════════════════
# File ops — sync
# ═══════════════════════════════════════════════════════════════════════════════


def file_content_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.content`` — retrieve raw file content stored in a collection."""
    return client.call("file.content", [request])


def file_list_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.list`` — list files indexed in a collection."""
    return client.call("file.list", [request])


def file_summary_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.summary`` — extractive or structural summary of one file."""
    return client.call("file.summary", [request])


def file_chunks_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.chunks`` — retrieve ordered chunks for one file."""
    return client.call("file.chunks", [request])


def file_outline_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.outline`` — directory-tree outline of a collection's files."""
    return client.call("file.outline", [request])


def file_related_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.related`` — find files semantically related to a given file."""
    return client.call("file.related", [request])


def file_search_by_type_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.search_by_type`` — search within files of specific extension types."""
    return client.call("file.search_by_type", [request])


# ═══════════════════════════════════════════════════════════════════════════════
# Graph — sync
# ═══════════════════════════════════════════════════════════════════════════════


def graph_list_nodes_sync(client: RpcClient, collection: str) -> VectorizerValue:
    """``graph.list_nodes`` — list all graph nodes in a collection."""
    return client.call("graph.list_nodes", [VectorizerValue.str_(collection)])


def graph_neighbors_sync(client: RpcClient, collection: str, node_id: str) -> VectorizerValue:
    """``graph.neighbors`` — fetch direct neighbors of a graph node."""
    return client.call("graph.neighbors", [VectorizerValue.str_(collection), VectorizerValue.str_(node_id)])


def graph_find_related_sync(client: RpcClient, collection: str, node_id: str, max_hops: int) -> VectorizerValue:
    """``graph.find_related`` — find nodes reachable within ``max_hops``."""
    return client.call("graph.find_related", [VectorizerValue.str_(collection), VectorizerValue.str_(node_id), VectorizerValue.int_(max_hops)])


def graph_find_path_sync(client: RpcClient, collection: str, from_id: str, to_id: str) -> VectorizerValue:
    """``graph.find_path`` — shortest path between two graph nodes."""
    return client.call("graph.find_path", [VectorizerValue.str_(collection), VectorizerValue.str_(from_id), VectorizerValue.str_(to_id)])


def graph_create_edge_sync(client: RpcClient, collection: str, edge: VectorizerValue) -> VectorizerValue:
    """``graph.create_edge`` — create a directed edge between two nodes."""
    return client.call("graph.create_edge", [VectorizerValue.str_(collection), edge])


def graph_delete_edge_sync(client: RpcClient, collection: str, edge_id: str) -> VectorizerValue:
    """``graph.delete_edge`` — remove an edge by its id."""
    return client.call("graph.delete_edge", [VectorizerValue.str_(collection), VectorizerValue.str_(edge_id)])


def graph_list_edges_sync(client: RpcClient, collection: str) -> VectorizerValue:
    """``graph.list_edges`` — list all edges in a collection's graph."""
    return client.call("graph.list_edges", [VectorizerValue.str_(collection)])


def graph_discover_edges_sync(client: RpcClient, collection: str, request: VectorizerValue) -> DiscoverEdgesResult:
    """``graph.discover_edges`` — auto-discover edges by vector similarity."""
    v = client.call("graph.discover_edges", [VectorizerValue.str_(collection), request])
    return DiscoverEdgesResult(
        success=_opt_bool(v, "success"),
        total_nodes=_opt_int(v, "total_nodes"),
        nodes_processed=_opt_int(v, "nodes_processed"),
        nodes_with_edges=_opt_int(v, "nodes_with_edges"),
        total_edges_created=_opt_int(v, "total_edges_created"),
    )


def graph_discover_edges_for_node_sync(
    client: RpcClient, collection: str, node_id: str, request: VectorizerValue
) -> DiscoverEdgesForNodeResult:
    """``graph.discover_edges_for_node`` — auto-discover edges for one node."""
    v = client.call("graph.discover_edges_for_node", [VectorizerValue.str_(collection), VectorizerValue.str_(node_id), request])
    return DiscoverEdgesForNodeResult(
        success=_opt_bool(v, "success"),
        node_id=_opt_str(v, "node_id") or node_id,
        edges_created=_opt_int(v, "edges_created"),
    )


def graph_discovery_status_sync(client: RpcClient, collection: str) -> GraphDiscoveryStatus:
    """``graph.discovery_status`` — percentage of nodes that have edges."""
    v = client.call("graph.discovery_status", [VectorizerValue.str_(collection)])
    return GraphDiscoveryStatus(
        total_nodes=_opt_int(v, "total_nodes"),
        nodes_with_edges=_opt_int(v, "nodes_with_edges"),
        total_edges=_opt_int(v, "total_edges"),
        progress_percentage=_opt_float(v, "progress_percentage"),
    )


# ═══════════════════════════════════════════════════════════════════════════════
# Admin — sync
# ═══════════════════════════════════════════════════════════════════════════════


def admin_stats_sync(client: RpcClient) -> AdminStats:
    """``admin.stats`` — aggregate vector/collection counts."""
    v = client.call("admin.stats", [])
    return AdminStats(
        collections_count=_opt_int(v, "collections_count"),
        total_vectors=_opt_int(v, "total_vectors"),
        version=_opt_str(v, "version") or "",
    )


def admin_status_sync(client: RpcClient) -> AdminStatus:
    """``admin.status`` — readiness probe and basic counts."""
    v = client.call("admin.status", [])
    return AdminStatus(
        ready=_opt_bool(v, "ready"),
        collections_count=_opt_int(v, "collections_count"),
        version=_opt_str(v, "version") or "",
    )


def admin_logs_sync(client: RpcClient, request: Optional[VectorizerValue] = None) -> VectorizerValue:
    """``admin.logs`` — in-process log entries."""
    args = [request] if request else []
    return client.call("admin.logs", args)


def admin_indexing_progress_sync(client: RpcClient) -> VectorizerValue:
    """``admin.indexing_progress`` — how many collections have been indexed."""
    return client.call("admin.indexing_progress", [])


def admin_config_get_sync(client: RpcClient) -> VectorizerValue:
    """``admin.config_get`` — read the server's config."""
    return client.call("admin.config_get", [])


def admin_config_update_sync(client: RpcClient, patch: VectorizerValue) -> bool:
    """``admin.config_update`` — write a patch map to config."""
    v = client.call("admin.config_update", [patch])
    return _need_bool(v, "success", "admin.config_update")


def admin_backups_list_sync(client: RpcClient) -> VectorizerValue:
    """``admin.backups_list`` — list available backup files."""
    return client.call("admin.backups_list", [])


def admin_backups_create_sync(client: RpcClient, request: VectorizerValue) -> str:
    """``admin.backups_create`` — create a backup."""
    v = client.call("admin.backups_create", [request])
    return _need_str(v, "backup_id", "admin.backups_create")


def admin_backups_restore_sync(client: RpcClient, request: VectorizerValue) -> bool:
    """``admin.backups_restore`` — restore a backup by id."""
    v = client.call("admin.backups_restore", [request])
    return _need_bool(v, "success", "admin.backups_restore")


def admin_workspaces_list_sync(client: RpcClient) -> VectorizerValue:
    """``admin.workspaces_list`` — list configured workspaces."""
    return client.call("admin.workspaces_list", [])


def admin_workspace_get_sync(client: RpcClient) -> VectorizerValue:
    """``admin.workspace_get`` — read workspace.yml."""
    return client.call("admin.workspace_get", [])


def admin_workspace_add_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``admin.workspace_add`` — register a new workspace directory."""
    return client.call("admin.workspace_add", [request])


def admin_workspace_remove_sync(client: RpcClient, name: str) -> bool:
    """``admin.workspace_remove`` — remove a workspace by name."""
    v = client.call("admin.workspace_remove", [VectorizerValue.str_(name)])
    return _need_bool(v, "success", "admin.workspace_remove")


def admin_restart_sync(client: RpcClient) -> bool:
    """``admin.restart`` — schedule a server restart."""
    v = client.call("admin.restart", [])
    return _need_bool(v, "success", "admin.restart")


def admin_slow_queries_list_sync(client: RpcClient) -> VectorizerValue:
    """``admin.slow_queries_list`` — retrieve the slow-query ring buffer."""
    return client.call("admin.slow_queries_list", [])


def admin_slow_queries_config_sync(client: RpcClient, config: VectorizerValue) -> SlowQueryConfigResult:
    """``admin.slow_queries_config`` — configure slow-query threshold."""
    v = client.call("admin.slow_queries_config", [config])
    return SlowQueryConfigResult(
        threshold_ms=_opt_int(v, "threshold_ms"),
        capacity=_opt_int(v, "capacity"),
        status=_opt_str(v, "status") or "ok",
    )


# ═══════════════════════════════════════════════════════════════════════════════
# Auth / RBAC — sync
# ═══════════════════════════════════════════════════════════════════════════════


def auth_me_sync(client: RpcClient) -> AuthMeResult:
    """``auth.me`` — return the authenticated principal's identity."""
    v = client.call("auth.me", [])
    return AuthMeResult(
        username=_opt_str(v, "username") or "unknown",
        authenticated=_opt_bool(v, "authenticated"),
    )


def auth_logout_sync(client: RpcClient, token: str) -> VectorizerValue:
    """``auth.logout`` — blacklist the supplied JWT."""
    return client.call("auth.logout", [VectorizerValue.str_(token)])


def auth_refresh_token_sync(client: RpcClient, token: str) -> RefreshTokenResult:
    """``auth.refresh_token`` — exchange a valid JWT for a fresh one."""
    v = client.call("auth.refresh_token", [VectorizerValue.str_(token)])
    return RefreshTokenResult(
        access_token=_need_str(v, "access_token", "auth.refresh_token"),
        token_type=_opt_str(v, "token_type") or "Bearer",
    )


def auth_validate_password_sync(client: RpcClient, password: str) -> ValidatePasswordResult:
    """``auth.validate_password`` — check a password against policy."""
    v = client.call("auth.validate_password", [VectorizerValue.str_(password)])
    errors_v = v.map_get("errors")
    errors = [x.as_str() for x in (errors_v.as_array() or []) if x.as_str() is not None]  # type: ignore[misc]
    return ValidatePasswordResult(valid=_opt_bool(v, "valid"), errors=errors)


def auth_api_keys_create_sync(client: RpcClient, request: VectorizerValue) -> ApiKeyCreated:
    """``auth.api_keys_create`` — create a new API key."""
    v = client.call("auth.api_keys_create", [request])
    return ApiKeyCreated(
        api_key=_need_str(v, "api_key", "auth.api_keys_create"),
        id=_need_str(v, "id", "auth.api_keys_create"),
        name=_need_str(v, "name", "auth.api_keys_create"),
    )


def auth_api_keys_list_sync(client: RpcClient) -> VectorizerValue:
    """``auth.api_keys_list`` — list API keys for the current principal."""
    return client.call("auth.api_keys_list", [])


def auth_api_keys_revoke_sync(client: RpcClient, key_id: str) -> bool:
    """``auth.api_keys_revoke`` — permanently revoke an API key by id."""
    v = client.call("auth.api_keys_revoke", [VectorizerValue.str_(key_id)])
    return _need_bool(v, "success", "auth.api_keys_revoke")


def rotate_api_key_rpc_sync(client: RpcClient, key_id: str) -> RotatedApiKey:
    """``auth.api_keys_rotate`` — rotate an API key (5-minute grace period)."""
    v = client.call("auth.api_keys_rotate", [VectorizerValue.str_(key_id)])
    return RotatedApiKey(
        old_key_id=_need_str(v, "old_key_id", "auth.api_keys_rotate"),
        new_key_id=_need_str(v, "new_key_id", "auth.api_keys_rotate"),
        new_token=_need_str(v, "new_token", "auth.api_keys_rotate"),
        grace_until=_opt_str(v, "grace_until"),
    )


def auth_api_keys_create_scoped_sync(client: RpcClient, request: VectorizerValue) -> ApiKeyCreated:
    """``auth.api_keys_create_scoped`` — create a collection-scoped API key."""
    v = client.call("auth.api_keys_create_scoped", [request])
    return ApiKeyCreated(
        api_key=_need_str(v, "api_key", "auth.api_keys_create_scoped"),
        id=_need_str(v, "id", "auth.api_keys_create_scoped"),
        name=_need_str(v, "name", "auth.api_keys_create_scoped"),
    )


def auth_introspect_sync(client: RpcClient, token: str) -> VectorizerValue:
    """``auth.introspect`` — inspect a token's claims and blacklist status."""
    return client.call("auth.introspect", [VectorizerValue.str_(token)])


def auth_audit_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``auth.audit`` — query the auth audit log."""
    return client.call("auth.audit", [request])


# ═══════════════════════════════════════════════════════════════════════════════
# Replication — sync
# ═══════════════════════════════════════════════════════════════════════════════


def replication_status_sync(client: RpcClient) -> VectorizerValue:
    """``replication.status`` — current replication role and replica list."""
    return client.call("replication.status", [])


def replication_configure_sync(client: RpcClient, config: VectorizerValue) -> ReplicationConfigureResult:
    """``replication.configure`` — set the replication role for this node."""
    v = client.call("replication.configure", [config])
    return ReplicationConfigureResult(
        success=_need_bool(v, "success", "replication.configure"),
        role=_need_str(v, "role", "replication.configure"),
        message=_opt_str(v, "message") or "",
    )


def replication_stats_sync(client: RpcClient) -> VectorizerValue:
    """``replication.stats`` — replication throughput and lag statistics."""
    return client.call("replication.stats", [])


def replication_replicas_list_sync(client: RpcClient) -> VectorizerValue:
    """``replication.replicas_list`` — list connected replicas (master only)."""
    return client.call("replication.replicas_list", [])


# ═══════════════════════════════════════════════════════════════════════════════
# Cluster — sync
# ═══════════════════════════════════════════════════════════════════════════════


def cluster_failover_sync(client: RpcClient, replica_id: str) -> VectorizerValue:
    """``cluster.failover`` — promote a replica to master."""
    return client.call("cluster.failover", [VectorizerValue.str_(replica_id)])


def cluster_replica_resync_sync(client: RpcClient, replica_id: str) -> VectorizerValue:
    """``cluster.replica_resync`` — force a replica to resync from master."""
    return client.call("cluster.replica_resync", [VectorizerValue.str_(replica_id)])


def cluster_peer_add_sync(client: RpcClient, request: VectorizerValue) -> VectorizerValue:
    """``cluster.peer_add`` — add a new peer to the cluster."""
    return client.call("cluster.peer_add", [request])


def cluster_rebalance_sync(client: RpcClient) -> VectorizerValue:
    """``cluster.rebalance`` — trigger a shard rebalance across peers."""
    return client.call("cluster.rebalance", [])


def cluster_rebalance_status_sync(client: RpcClient) -> RebalanceStatus:
    """``cluster.rebalance_status`` — check the status of an in-progress rebalance."""
    v = client.call("cluster.rebalance_status", [])
    return RebalanceStatus(status=_opt_str(v, "status"), message=_opt_str(v, "message"))


# ═══════════════════════════════════════════════════════════════════════════════
# Async wrappers (mirror every sync wrapper above)
# ═══════════════════════════════════════════════════════════════════════════════


# ── Collections async ────────────────────────────────────────────────────────


async def list_collections_async(client: AsyncRpcClient) -> List[str]:
    """``collections.list``."""
    return _decode_string_array(await client.call("collections.list", []), "collections.list")


async def get_collection_info_async(client: AsyncRpcClient, name: str) -> CollectionInfo:
    """``collections.get_info``."""
    v = await client.call("collections.get_info", [VectorizerValue.str_(name)])
    return _decode_collection_info(v)


async def create_collection_async(client: AsyncRpcClient, name: str, config: VectorizerValue) -> CreateCollectionResult:
    """``collections.create``."""
    v = await client.call("collections.create", [VectorizerValue.str_(name), config])
    return CreateCollectionResult(name=_need_str(v, "name", "collections.create"), dimension=_need_int(v, "dimension", "collections.create"), metric=_need_str(v, "metric", "collections.create"), success=_need_bool(v, "success", "collections.create"))


async def delete_collection_async(client: AsyncRpcClient, name: str) -> bool:
    """``collections.delete``."""
    v = await client.call("collections.delete", [VectorizerValue.str_(name)])
    return _need_bool(v, "success", "collections.delete")


async def list_empty_collections_async(client: AsyncRpcClient) -> List[str]:
    """``collections.list_empty``."""
    return _decode_string_array(await client.call("collections.list_empty", []), "collections.list_empty")


async def cleanup_empty_collections_async(client: AsyncRpcClient, dry_run: bool = False) -> CleanupEmptyResult:
    """``collections.cleanup_empty``."""
    config = VectorizerValue.map([(VectorizerValue.str_("dry_run"), VectorizerValue.bool_(dry_run))])
    v = await client.call("collections.cleanup_empty", [config])
    return CleanupEmptyResult(removed=_need_int(v, "removed", "collections.cleanup_empty"), dry_run=_need_bool(v, "dry_run", "collections.cleanup_empty"))


async def force_save_collection_async(client: AsyncRpcClient, name: str) -> bool:
    """``collections.force_save``."""
    v = await client.call("collections.force_save", [VectorizerValue.str_(name)])
    return _need_bool(v, "success", "collections.force_save")


# ── Vectors async ─────────────────────────────────────────────────────────────


async def get_vector_async(client: AsyncRpcClient, collection: str, vector_id: str) -> VectorizerValue:
    """``vectors.get``."""
    return await client.call("vectors.get", [VectorizerValue.str_(collection), VectorizerValue.str_(vector_id)])


async def insert_vector_async(client: AsyncRpcClient, collection: str, id: Optional[str], data: List[float], payload: Optional[VectorizerValue] = None) -> VectorWriteResult:
    """``vectors.insert``."""
    id_val = VectorizerValue.str_(id) if id else VectorizerValue.null()
    data_val = VectorizerValue.array([VectorizerValue.float_(f) for f in data])
    args = [VectorizerValue.str_(collection), id_val, data_val]
    if payload is not None:
        args.append(payload)
    v = await client.call("vectors.insert", args)
    return VectorWriteResult(id=_need_str(v, "id", "vectors.insert"), success=_need_bool(v, "success", "vectors.insert"))


async def insert_text_vector_async(client: AsyncRpcClient, collection: str, id: Optional[str], text: str, payload: Optional[VectorizerValue] = None) -> VectorWriteResult:
    """``vectors.insert_text``."""
    id_val = VectorizerValue.str_(id) if id else VectorizerValue.null()
    args = [VectorizerValue.str_(collection), id_val, VectorizerValue.str_(text)]
    if payload is not None:
        args.append(payload)
    v = await client.call("vectors.insert_text", args)
    return VectorWriteResult(id=_need_str(v, "id", "vectors.insert_text"), success=_need_bool(v, "success", "vectors.insert_text"))


async def update_vector_async(client: AsyncRpcClient, collection: str, id: str, data: List[float], payload: Optional[VectorizerValue] = None) -> VectorWriteResult:
    """``vectors.update``."""
    data_val = VectorizerValue.array([VectorizerValue.float_(f) for f in data])
    args = [VectorizerValue.str_(collection), VectorizerValue.str_(id), data_val]
    if payload is not None:
        args.append(payload)
    v = await client.call("vectors.update", args)
    return VectorWriteResult(id=_need_str(v, "id", "vectors.update"), success=_need_bool(v, "success", "vectors.update"))


async def delete_vector_rpc_async(client: AsyncRpcClient, collection: str, id: str) -> bool:
    """``vectors.delete``."""
    v = await client.call("vectors.delete", [VectorizerValue.str_(collection), VectorizerValue.str_(id)])
    return _need_bool(v, "success", "vectors.delete")


async def list_vectors_async(client: AsyncRpcClient, collection: str, page: int = 0, limit: int = 20) -> VectorListResult:
    """``vectors.list``."""
    v = await client.call("vectors.list", [VectorizerValue.str_(collection), VectorizerValue.int_(page), VectorizerValue.int_(limit)])
    items_v = v.map_get("items")
    items = list(items_v.as_array()) if items_v is not None and items_v.as_array() is not None else []
    return VectorListResult(items=items, total=_need_int(v, "total", "vectors.list"), page=_need_int(v, "page", "vectors.list"), limit=_need_int(v, "limit", "vectors.list"))


async def embed_text_async(client: AsyncRpcClient, text: str, model: Optional[str] = None) -> EmbedResult:
    """``vectors.embed``."""
    args = [VectorizerValue.str_(text)]
    if model is not None:
        args.append(VectorizerValue.str_(model))
    v = await client.call("vectors.embed", args)
    emb_v = v.map_get("embedding")
    embedding = [x.as_float() for x in (emb_v.as_array() or []) if x.as_float() is not None]  # type: ignore[misc]
    return EmbedResult(embedding=embedding, model=_opt_str(v, "model") or "bm25", dimension=_opt_int(v, "dimension"))


async def batch_insert_vectors_async(client: AsyncRpcClient, collection: str, items: List[VectorizerValue]) -> BatchInsertResult:
    """``vectors.batch_insert``."""
    v = await client.call("vectors.batch_insert", [VectorizerValue.str_(collection), VectorizerValue.array(items)])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchInsertResult(inserted=_opt_int(v, "inserted"), failed=_opt_int(v, "failed"), results=results)


async def batch_insert_texts_async(client: AsyncRpcClient, collection: str, items: List[VectorizerValue]) -> BatchInsertResult:
    """``vectors.batch_insert_texts``."""
    v = await client.call("vectors.batch_insert_texts", [VectorizerValue.str_(collection), VectorizerValue.array(items)])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchInsertResult(inserted=_opt_int(v, "inserted"), failed=_opt_int(v, "failed"), results=results)


async def batch_search_async(client: AsyncRpcClient, requests: List[VectorizerValue]) -> List[BatchSearchResult]:
    """``vectors.batch_search``."""
    v = await client.call("vectors.batch_search", [VectorizerValue.array(requests)])
    arr = v.as_array()
    if arr is None:
        raise RpcServerError("vectors.batch_search: expected Array")
    out: List[BatchSearchResult] = []
    for entry in arr:
        results_v = entry.map_get("results")
        hits = _decode_search_hits(results_v.as_array() or []) if results_v else []
        out.append(BatchSearchResult(index=_opt_int(entry, "index"), status=_opt_str(entry, "status") or "unknown", results=hits, error=_opt_str(entry, "error")))
    return out


async def batch_update_vectors_async(client: AsyncRpcClient, collection: str, updates: List[VectorizerValue]) -> BatchUpdateResult:
    """``vectors.batch_update``."""
    v = await client.call("vectors.batch_update", [VectorizerValue.str_(collection), VectorizerValue.array(updates)])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchUpdateResult(updated=_opt_int(v, "updated"), failed=_opt_int(v, "failed"), results=results)


async def batch_delete_vectors_async(client: AsyncRpcClient, collection: str, ids: List[str]) -> BatchDeleteResult:
    """``vectors.batch_delete``."""
    ids_val = VectorizerValue.array([VectorizerValue.str_(i) for i in ids])
    v = await client.call("vectors.batch_delete", [VectorizerValue.str_(collection), ids_val])
    results_v = v.map_get("results")
    results = _decode_batch_items(results_v.as_array() or []) if results_v else []
    return BatchDeleteResult(deleted=_opt_int(v, "deleted"), failed=_opt_int(v, "failed"), results=results)


async def move_vectors_rpc_async(client: AsyncRpcClient, src: str, dst: str, ids: List[str]) -> MoveRpcResult:
    """``vectors.move``."""
    ids_val = VectorizerValue.array([VectorizerValue.str_(i) for i in ids])
    v = await client.call("vectors.move", [VectorizerValue.str_(src), VectorizerValue.str_(dst), ids_val])
    return MoveRpcResult(src=_need_str(v, "src", "vectors.move"), dst=_need_str(v, "dst", "vectors.move"), moved=_opt_int(v, "moved"), failed=_opt_int(v, "failed"))


async def copy_vectors_rpc_async(client: AsyncRpcClient, src: str, dst: str, ids: List[str]) -> CopyRpcResult:
    """``vectors.copy``."""
    ids_val = VectorizerValue.array([VectorizerValue.str_(i) for i in ids])
    v = await client.call("vectors.copy", [VectorizerValue.str_(src), VectorizerValue.str_(dst), ids_val])
    return CopyRpcResult(src=_need_str(v, "src", "vectors.copy"), dst=_need_str(v, "dst", "vectors.copy"), copied=_opt_int(v, "copied"), failed=_opt_int(v, "failed"))


async def delete_by_filter_rpc_async(client: AsyncRpcClient, collection: str, filter: VectorizerValue) -> DeleteByFilterRpcResult:
    """``vectors.delete_by_filter``."""
    v = await client.call("vectors.delete_by_filter", [VectorizerValue.str_(collection), filter])
    return DeleteByFilterRpcResult(scanned=_opt_int(v, "scanned"), matched=_opt_int(v, "matched"), deleted=_opt_int(v, "deleted"))


async def bulk_update_metadata_rpc_async(client: AsyncRpcClient, collection: str, filter: VectorizerValue, patch: VectorizerValue) -> BulkUpdateMetadataRpcResult:
    """``vectors.bulk_update_metadata``."""
    v = await client.call("vectors.bulk_update_metadata", [VectorizerValue.str_(collection), filter, patch])
    return BulkUpdateMetadataRpcResult(scanned=_opt_int(v, "scanned"), matched=_opt_int(v, "matched"), updated=_opt_int(v, "updated"))


async def set_vector_expiry_async(client: AsyncRpcClient, collection: str, id: str, expires_at: str) -> SetExpiryResult:
    """``vectors.set_expiry``."""
    v = await client.call("vectors.set_expiry", [VectorizerValue.str_(collection), VectorizerValue.str_(id), VectorizerValue.str_(expires_at)])
    return SetExpiryResult(id=_need_str(v, "id", "vectors.set_expiry"), expires_at=_need_int(v, "expires_at", "vectors.set_expiry"), success=_need_bool(v, "success", "vectors.set_expiry"))


# ── Search async ──────────────────────────────────────────────────────────────


async def search_basic_async(client: AsyncRpcClient, collection: str, query: str, limit: int = 10) -> List[SearchHit]:
    """``search.basic``."""
    v = await client.call("search.basic", [VectorizerValue.str_(collection), VectorizerValue.str_(query), VectorizerValue.int_(limit)])
    arr = v.as_array()
    if arr is None:
        raise RpcServerError("search.basic: expected Array")
    return _decode_search_hits(arr)


async def search_intelligent_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.intelligent``."""
    return await client.call("search.intelligent", [request])


async def search_by_text_async(client: AsyncRpcClient, collection: str, query: str, limit: int = 10) -> List[SearchHit]:
    """``search.by_text``."""
    v = await client.call("search.by_text", [VectorizerValue.str_(collection), VectorizerValue.str_(query), VectorizerValue.int_(limit)])
    results_v = v.map_get("results")
    arr = results_v.as_array() if results_v is not None else None
    if arr is None:
        raise RpcServerError("search.by_text: missing results array")
    return _decode_search_hits(arr)


async def search_by_file_async(client: AsyncRpcClient, collection: str, request: VectorizerValue) -> List[SearchHit]:
    """``search.by_file``."""
    v = await client.call("search.by_file", [VectorizerValue.str_(collection), request])
    results_v = v.map_get("results")
    arr = results_v.as_array() if results_v is not None else None
    if arr is None:
        raise RpcServerError("search.by_file: missing results array")
    return _decode_search_hits(arr)


async def search_hybrid_async(client: AsyncRpcClient, collection: str, request: VectorizerValue) -> List[SearchHit]:
    """``search.hybrid``."""
    v = await client.call("search.hybrid", [VectorizerValue.str_(collection), request])
    results_v = v.map_get("results")
    arr = results_v.as_array() if results_v is not None else None
    if arr is None:
        raise RpcServerError("search.hybrid: missing results array")
    return _decode_search_hits(arr)


async def search_semantic_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.semantic``."""
    return await client.call("search.semantic", [request])


async def search_contextual_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.contextual``."""
    return await client.call("search.contextual", [request])


async def search_multi_collection_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``search.multi_collection``."""
    return await client.call("search.multi_collection", [request])


async def search_explain_async(client: AsyncRpcClient, collection: str, request: VectorizerValue) -> SearchExplainResult:
    """``search.explain``."""
    v = await client.call("search.explain", [VectorizerValue.str_(collection), request])
    hits_v = v.map_get("hits")
    hits = _decode_search_hits(hits_v.as_array() or []) if hits_v else []
    trace_v = v.map_get("trace") or VectorizerValue.null()
    trace = SearchTrace(visited_nodes=_opt_int(trace_v, "visited_nodes"), ef_search=_opt_int(trace_v, "ef_search"), hnsw_search_ms=_opt_float(trace_v, "hnsw_search_ms"), total_ms=_opt_float(trace_v, "total_ms"))
    return SearchExplainResult(hits=hits, collection=_opt_str(v, "collection") or "", k=_opt_int(v, "k"), trace=trace)


# ── Discovery async ───────────────────────────────────────────────────────────


async def discover_async(client: AsyncRpcClient, request: VectorizerValue) -> DiscoverResult:
    """``discovery.discover``."""
    v = await client.call("discovery.discover", [request])
    return DiscoverResult(answer_prompt=_need_str(v, "answer_prompt", "discovery.discover"), sections=_opt_int(v, "sections"), bullets=_opt_int(v, "bullets"), chunks=_opt_int(v, "chunks"))


async def filter_collections_async(client: AsyncRpcClient, request: VectorizerValue) -> List[str]:
    """``discovery.filter_collections``."""
    v = await client.call("discovery.filter_collections", [request])
    fc_v = v.map_get("filtered_collections")
    if fc_v is None:
        raise RpcServerError("discovery.filter_collections: missing filtered_collections")
    return [n.as_str() for entry in (fc_v.as_array() or []) if (n := entry.map_get("name")) is not None and n.as_str() is not None]  # type: ignore[misc]


async def score_collections_async(client: AsyncRpcClient, request: VectorizerValue) -> List[ScoredCollection]:
    """``discovery.score_collections``."""
    v = await client.call("discovery.score_collections", [request])
    sc_v = v.map_get("scored_collections")
    if sc_v is None:
        raise RpcServerError("discovery.score_collections: missing scored_collections")
    return [ScoredCollection(name=_opt_str(entry, "name") or "", score=_opt_float(entry, "score"), vector_count=_opt_int(entry, "vector_count")) for entry in (sc_v.as_array() or [])]


async def expand_queries_async(client: AsyncRpcClient, request: VectorizerValue) -> ExpandQueriesResult:
    """``discovery.expand_queries``."""
    v = await client.call("discovery.expand_queries", [request])
    eq_v = v.map_get("expanded_queries")
    expanded = [x.as_str() for x in (eq_v.as_array() or []) if x.as_str() is not None]  # type: ignore[misc]
    return ExpandQueriesResult(original_query=_need_str(v, "original_query", "discovery.expand_queries"), expanded_queries=expanded, count=_opt_int(v, "count"))


async def broad_discovery_async(client: AsyncRpcClient, request: VectorizerValue) -> List[DiscoveryChunk]:
    """``discovery.broad_discovery``."""
    v = await client.call("discovery.broad_discovery", [request])
    chunks_v = v.map_get("chunks")
    if chunks_v is None:
        raise RpcServerError("discovery.broad_discovery: missing chunks")
    return [DiscoveryChunk(collection=_opt_str(entry, "collection") or "", score=_opt_float(entry, "score"), content_preview=_opt_str(entry, "content_preview") or "") for entry in (chunks_v.as_array() or [])]


async def semantic_focus_async(client: AsyncRpcClient, request: VectorizerValue) -> List[DiscoveryChunk]:
    """``discovery.semantic_focus``."""
    v = await client.call("discovery.semantic_focus", [request])
    chunks_v = v.map_get("chunks")
    if chunks_v is None:
        raise RpcServerError("discovery.semantic_focus: missing chunks")
    return [DiscoveryChunk(collection=_opt_str(entry, "collection") or "", score=_opt_float(entry, "score"), content_preview=_opt_str(entry, "content_preview") or "") for entry in (chunks_v.as_array() or [])]


async def promote_readme_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``discovery.promote_readme``."""
    return await client.call("discovery.promote_readme", [request])


async def compress_evidence_async(client: AsyncRpcClient, request: VectorizerValue) -> List[CompressBullet]:
    """``discovery.compress_evidence``."""
    v = await client.call("discovery.compress_evidence", [request])
    bullets_v = v.map_get("bullets")
    if bullets_v is None:
        raise RpcServerError("discovery.compress_evidence: missing bullets")
    return [CompressBullet(text=_opt_str(entry, "text") or "", source_id=_opt_str(entry, "source_id") or "", score=_opt_float(entry, "score")) for entry in (bullets_v.as_array() or [])]


async def build_answer_plan_async(client: AsyncRpcClient, request: VectorizerValue) -> AnswerPlanResult:
    """``discovery.build_answer_plan``."""
    v = await client.call("discovery.build_answer_plan", [request])
    sections_v = v.map_get("sections")
    sections = [AnswerPlanSection(title=_opt_str(entry, "title") or "", bullets_count=_opt_int(entry, "bullets_count")) for entry in (sections_v.as_array() or [])] if sections_v else []
    return AnswerPlanResult(sections=sections, total_bullets=_opt_int(v, "total_bullets"))


async def render_llm_prompt_async(client: AsyncRpcClient, request: VectorizerValue) -> RenderPromptResult:
    """``discovery.render_llm_prompt``."""
    v = await client.call("discovery.render_llm_prompt", [request])
    return RenderPromptResult(prompt=_need_str(v, "prompt", "discovery.render_llm_prompt"), length=_opt_int(v, "length"), estimated_tokens=_opt_int(v, "estimated_tokens"))


# ── File ops async ────────────────────────────────────────────────────────────


async def file_content_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.content``."""
    return await client.call("file.content", [request])


async def file_list_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.list``."""
    return await client.call("file.list", [request])


async def file_summary_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.summary``."""
    return await client.call("file.summary", [request])


async def file_chunks_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.chunks``."""
    return await client.call("file.chunks", [request])


async def file_outline_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.outline``."""
    return await client.call("file.outline", [request])


async def file_related_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.related``."""
    return await client.call("file.related", [request])


async def file_search_by_type_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``file.search_by_type``."""
    return await client.call("file.search_by_type", [request])


# ── Graph async ───────────────────────────────────────────────────────────────


async def graph_list_nodes_async(client: AsyncRpcClient, collection: str) -> VectorizerValue:
    """``graph.list_nodes``."""
    return await client.call("graph.list_nodes", [VectorizerValue.str_(collection)])


async def graph_neighbors_async(client: AsyncRpcClient, collection: str, node_id: str) -> VectorizerValue:
    """``graph.neighbors``."""
    return await client.call("graph.neighbors", [VectorizerValue.str_(collection), VectorizerValue.str_(node_id)])


async def graph_find_related_async(client: AsyncRpcClient, collection: str, node_id: str, max_hops: int) -> VectorizerValue:
    """``graph.find_related``."""
    return await client.call("graph.find_related", [VectorizerValue.str_(collection), VectorizerValue.str_(node_id), VectorizerValue.int_(max_hops)])


async def graph_find_path_async(client: AsyncRpcClient, collection: str, from_id: str, to_id: str) -> VectorizerValue:
    """``graph.find_path``."""
    return await client.call("graph.find_path", [VectorizerValue.str_(collection), VectorizerValue.str_(from_id), VectorizerValue.str_(to_id)])


async def graph_create_edge_async(client: AsyncRpcClient, collection: str, edge: VectorizerValue) -> VectorizerValue:
    """``graph.create_edge``."""
    return await client.call("graph.create_edge", [VectorizerValue.str_(collection), edge])


async def graph_delete_edge_async(client: AsyncRpcClient, collection: str, edge_id: str) -> VectorizerValue:
    """``graph.delete_edge``."""
    return await client.call("graph.delete_edge", [VectorizerValue.str_(collection), VectorizerValue.str_(edge_id)])


async def graph_list_edges_async(client: AsyncRpcClient, collection: str) -> VectorizerValue:
    """``graph.list_edges``."""
    return await client.call("graph.list_edges", [VectorizerValue.str_(collection)])


async def graph_discover_edges_async(client: AsyncRpcClient, collection: str, request: VectorizerValue) -> DiscoverEdgesResult:
    """``graph.discover_edges``."""
    v = await client.call("graph.discover_edges", [VectorizerValue.str_(collection), request])
    return DiscoverEdgesResult(success=_opt_bool(v, "success"), total_nodes=_opt_int(v, "total_nodes"), nodes_processed=_opt_int(v, "nodes_processed"), nodes_with_edges=_opt_int(v, "nodes_with_edges"), total_edges_created=_opt_int(v, "total_edges_created"))


async def graph_discover_edges_for_node_async(client: AsyncRpcClient, collection: str, node_id: str, request: VectorizerValue) -> DiscoverEdgesForNodeResult:
    """``graph.discover_edges_for_node``."""
    v = await client.call("graph.discover_edges_for_node", [VectorizerValue.str_(collection), VectorizerValue.str_(node_id), request])
    return DiscoverEdgesForNodeResult(success=_opt_bool(v, "success"), node_id=_opt_str(v, "node_id") or node_id, edges_created=_opt_int(v, "edges_created"))


async def graph_discovery_status_async(client: AsyncRpcClient, collection: str) -> GraphDiscoveryStatus:
    """``graph.discovery_status``."""
    v = await client.call("graph.discovery_status", [VectorizerValue.str_(collection)])
    return GraphDiscoveryStatus(total_nodes=_opt_int(v, "total_nodes"), nodes_with_edges=_opt_int(v, "nodes_with_edges"), total_edges=_opt_int(v, "total_edges"), progress_percentage=_opt_float(v, "progress_percentage"))


# ── Admin async ───────────────────────────────────────────────────────────────


async def admin_stats_async(client: AsyncRpcClient) -> AdminStats:
    """``admin.stats``."""
    v = await client.call("admin.stats", [])
    return AdminStats(collections_count=_opt_int(v, "collections_count"), total_vectors=_opt_int(v, "total_vectors"), version=_opt_str(v, "version") or "")


async def admin_status_async(client: AsyncRpcClient) -> AdminStatus:
    """``admin.status``."""
    v = await client.call("admin.status", [])
    return AdminStatus(ready=_opt_bool(v, "ready"), collections_count=_opt_int(v, "collections_count"), version=_opt_str(v, "version") or "")


async def admin_logs_async(client: AsyncRpcClient, request: Optional[VectorizerValue] = None) -> VectorizerValue:
    """``admin.logs``."""
    args = [request] if request else []
    return await client.call("admin.logs", args)


async def admin_indexing_progress_async(client: AsyncRpcClient) -> VectorizerValue:
    """``admin.indexing_progress``."""
    return await client.call("admin.indexing_progress", [])


async def admin_config_get_async(client: AsyncRpcClient) -> VectorizerValue:
    """``admin.config_get``."""
    return await client.call("admin.config_get", [])


async def admin_config_update_async(client: AsyncRpcClient, patch: VectorizerValue) -> bool:
    """``admin.config_update``."""
    v = await client.call("admin.config_update", [patch])
    return _need_bool(v, "success", "admin.config_update")


async def admin_backups_list_async(client: AsyncRpcClient) -> VectorizerValue:
    """``admin.backups_list``."""
    return await client.call("admin.backups_list", [])


async def admin_backups_create_async(client: AsyncRpcClient, request: VectorizerValue) -> str:
    """``admin.backups_create``."""
    v = await client.call("admin.backups_create", [request])
    return _need_str(v, "backup_id", "admin.backups_create")


async def admin_backups_restore_async(client: AsyncRpcClient, request: VectorizerValue) -> bool:
    """``admin.backups_restore``."""
    v = await client.call("admin.backups_restore", [request])
    return _need_bool(v, "success", "admin.backups_restore")


async def admin_workspaces_list_async(client: AsyncRpcClient) -> VectorizerValue:
    """``admin.workspaces_list``."""
    return await client.call("admin.workspaces_list", [])


async def admin_workspace_get_async(client: AsyncRpcClient) -> VectorizerValue:
    """``admin.workspace_get``."""
    return await client.call("admin.workspace_get", [])


async def admin_workspace_add_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``admin.workspace_add``."""
    return await client.call("admin.workspace_add", [request])


async def admin_workspace_remove_async(client: AsyncRpcClient, name: str) -> bool:
    """``admin.workspace_remove``."""
    v = await client.call("admin.workspace_remove", [VectorizerValue.str_(name)])
    return _need_bool(v, "success", "admin.workspace_remove")


async def admin_restart_async(client: AsyncRpcClient) -> bool:
    """``admin.restart``."""
    v = await client.call("admin.restart", [])
    return _need_bool(v, "success", "admin.restart")


async def admin_slow_queries_list_async(client: AsyncRpcClient) -> VectorizerValue:
    """``admin.slow_queries_list``."""
    return await client.call("admin.slow_queries_list", [])


async def admin_slow_queries_config_async(client: AsyncRpcClient, config: VectorizerValue) -> SlowQueryConfigResult:
    """``admin.slow_queries_config``."""
    v = await client.call("admin.slow_queries_config", [config])
    return SlowQueryConfigResult(threshold_ms=_opt_int(v, "threshold_ms"), capacity=_opt_int(v, "capacity"), status=_opt_str(v, "status") or "ok")


# ── Auth async ────────────────────────────────────────────────────────────────


async def auth_me_async(client: AsyncRpcClient) -> AuthMeResult:
    """``auth.me``."""
    v = await client.call("auth.me", [])
    return AuthMeResult(username=_opt_str(v, "username") or "unknown", authenticated=_opt_bool(v, "authenticated"))


async def auth_logout_async(client: AsyncRpcClient, token: str) -> VectorizerValue:
    """``auth.logout``."""
    return await client.call("auth.logout", [VectorizerValue.str_(token)])


async def auth_refresh_token_async(client: AsyncRpcClient, token: str) -> RefreshTokenResult:
    """``auth.refresh_token``."""
    v = await client.call("auth.refresh_token", [VectorizerValue.str_(token)])
    return RefreshTokenResult(access_token=_need_str(v, "access_token", "auth.refresh_token"), token_type=_opt_str(v, "token_type") or "Bearer")


async def auth_validate_password_async(client: AsyncRpcClient, password: str) -> ValidatePasswordResult:
    """``auth.validate_password``."""
    v = await client.call("auth.validate_password", [VectorizerValue.str_(password)])
    errors_v = v.map_get("errors")
    errors = [x.as_str() for x in (errors_v.as_array() or []) if x.as_str() is not None]  # type: ignore[misc]
    return ValidatePasswordResult(valid=_opt_bool(v, "valid"), errors=errors)


async def auth_api_keys_create_async(client: AsyncRpcClient, request: VectorizerValue) -> ApiKeyCreated:
    """``auth.api_keys_create``."""
    v = await client.call("auth.api_keys_create", [request])
    return ApiKeyCreated(api_key=_need_str(v, "api_key", "auth.api_keys_create"), id=_need_str(v, "id", "auth.api_keys_create"), name=_need_str(v, "name", "auth.api_keys_create"))


async def auth_api_keys_list_async(client: AsyncRpcClient) -> VectorizerValue:
    """``auth.api_keys_list``."""
    return await client.call("auth.api_keys_list", [])


async def auth_api_keys_revoke_async(client: AsyncRpcClient, key_id: str) -> bool:
    """``auth.api_keys_revoke``."""
    v = await client.call("auth.api_keys_revoke", [VectorizerValue.str_(key_id)])
    return _need_bool(v, "success", "auth.api_keys_revoke")


async def rotate_api_key_rpc_async(client: AsyncRpcClient, key_id: str) -> RotatedApiKey:
    """``auth.api_keys_rotate``."""
    v = await client.call("auth.api_keys_rotate", [VectorizerValue.str_(key_id)])
    return RotatedApiKey(old_key_id=_need_str(v, "old_key_id", "auth.api_keys_rotate"), new_key_id=_need_str(v, "new_key_id", "auth.api_keys_rotate"), new_token=_need_str(v, "new_token", "auth.api_keys_rotate"), grace_until=_opt_str(v, "grace_until"))


async def auth_api_keys_create_scoped_async(client: AsyncRpcClient, request: VectorizerValue) -> ApiKeyCreated:
    """``auth.api_keys_create_scoped``."""
    v = await client.call("auth.api_keys_create_scoped", [request])
    return ApiKeyCreated(api_key=_need_str(v, "api_key", "auth.api_keys_create_scoped"), id=_need_str(v, "id", "auth.api_keys_create_scoped"), name=_need_str(v, "name", "auth.api_keys_create_scoped"))


async def auth_introspect_async(client: AsyncRpcClient, token: str) -> VectorizerValue:
    """``auth.introspect``."""
    return await client.call("auth.introspect", [VectorizerValue.str_(token)])


async def auth_audit_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``auth.audit``."""
    return await client.call("auth.audit", [request])


# ── Replication async ─────────────────────────────────────────────────────────


async def replication_status_async(client: AsyncRpcClient) -> VectorizerValue:
    """``replication.status``."""
    return await client.call("replication.status", [])


async def replication_configure_async(client: AsyncRpcClient, config: VectorizerValue) -> ReplicationConfigureResult:
    """``replication.configure``."""
    v = await client.call("replication.configure", [config])
    return ReplicationConfigureResult(success=_need_bool(v, "success", "replication.configure"), role=_need_str(v, "role", "replication.configure"), message=_opt_str(v, "message") or "")


async def replication_stats_async(client: AsyncRpcClient) -> VectorizerValue:
    """``replication.stats``."""
    return await client.call("replication.stats", [])


async def replication_replicas_list_async(client: AsyncRpcClient) -> VectorizerValue:
    """``replication.replicas_list``."""
    return await client.call("replication.replicas_list", [])


# ── Cluster async ─────────────────────────────────────────────────────────────


async def cluster_failover_async(client: AsyncRpcClient, replica_id: str) -> VectorizerValue:
    """``cluster.failover``."""
    return await client.call("cluster.failover", [VectorizerValue.str_(replica_id)])


async def cluster_replica_resync_async(client: AsyncRpcClient, replica_id: str) -> VectorizerValue:
    """``cluster.replica_resync``."""
    return await client.call("cluster.replica_resync", [VectorizerValue.str_(replica_id)])


async def cluster_peer_add_async(client: AsyncRpcClient, request: VectorizerValue) -> VectorizerValue:
    """``cluster.peer_add``."""
    return await client.call("cluster.peer_add", [request])


async def cluster_rebalance_async(client: AsyncRpcClient) -> VectorizerValue:
    """``cluster.rebalance``."""
    return await client.call("cluster.rebalance", [])


async def cluster_rebalance_status_async(client: AsyncRpcClient) -> RebalanceStatus:
    """``cluster.rebalance_status``."""
    v = await client.call("cluster.rebalance_status", [])
    return RebalanceStatus(status=_opt_str(v, "status"), message=_opt_str(v, "message"))


# ═══════════════════════════════════════════════════════════════════════════════
# Monkey-patch sync + async clients so callers write ``client.method(...)``
# ═══════════════════════════════════════════════════════════════════════════════

# Collections
RpcClient.list_collections = list_collections_sync  # type: ignore[attr-defined]
RpcClient.get_collection_info = get_collection_info_sync  # type: ignore[attr-defined]
RpcClient.create_collection = create_collection_sync  # type: ignore[attr-defined]
RpcClient.delete_collection = delete_collection_sync  # type: ignore[attr-defined]
RpcClient.list_empty_collections = list_empty_collections_sync  # type: ignore[attr-defined]
RpcClient.cleanup_empty_collections = cleanup_empty_collections_sync  # type: ignore[attr-defined]
RpcClient.force_save_collection = force_save_collection_sync  # type: ignore[attr-defined]
# Vectors
RpcClient.get_vector = get_vector_sync  # type: ignore[attr-defined]
RpcClient.insert_vector = insert_vector_sync  # type: ignore[attr-defined]
RpcClient.insert_text_vector = insert_text_vector_sync  # type: ignore[attr-defined]
RpcClient.update_vector = update_vector_sync  # type: ignore[attr-defined]
RpcClient.delete_vector_rpc = delete_vector_rpc_sync  # type: ignore[attr-defined]
RpcClient.list_vectors = list_vectors_sync  # type: ignore[attr-defined]
RpcClient.embed_text = embed_text_sync  # type: ignore[attr-defined]
RpcClient.batch_insert_vectors = batch_insert_vectors_sync  # type: ignore[attr-defined]
RpcClient.batch_insert_texts = batch_insert_texts_sync  # type: ignore[attr-defined]
RpcClient.batch_search = batch_search_sync  # type: ignore[attr-defined]
RpcClient.batch_update_vectors = batch_update_vectors_sync  # type: ignore[attr-defined]
RpcClient.batch_delete_vectors = batch_delete_vectors_sync  # type: ignore[attr-defined]
RpcClient.move_vectors_rpc = move_vectors_rpc_sync  # type: ignore[attr-defined]
RpcClient.copy_vectors_rpc = copy_vectors_rpc_sync  # type: ignore[attr-defined]
RpcClient.delete_by_filter_rpc = delete_by_filter_rpc_sync  # type: ignore[attr-defined]
RpcClient.bulk_update_metadata_rpc = bulk_update_metadata_rpc_sync  # type: ignore[attr-defined]
RpcClient.set_vector_expiry = set_vector_expiry_sync  # type: ignore[attr-defined]
# Search
RpcClient.search_basic = search_basic_sync  # type: ignore[attr-defined]
RpcClient.search_intelligent = search_intelligent_sync  # type: ignore[attr-defined]
RpcClient.search_by_text = search_by_text_sync  # type: ignore[attr-defined]
RpcClient.search_by_file = search_by_file_sync  # type: ignore[attr-defined]
RpcClient.search_hybrid = search_hybrid_sync  # type: ignore[attr-defined]
RpcClient.search_semantic = search_semantic_sync  # type: ignore[attr-defined]
RpcClient.search_contextual = search_contextual_sync  # type: ignore[attr-defined]
RpcClient.search_multi_collection = search_multi_collection_sync  # type: ignore[attr-defined]
RpcClient.search_explain = search_explain_sync  # type: ignore[attr-defined]
# Discovery
RpcClient.discover = discover_sync  # type: ignore[attr-defined]
RpcClient.filter_collections = filter_collections_sync  # type: ignore[attr-defined]
RpcClient.score_collections = score_collections_sync  # type: ignore[attr-defined]
RpcClient.expand_queries = expand_queries_sync  # type: ignore[attr-defined]
RpcClient.broad_discovery = broad_discovery_sync  # type: ignore[attr-defined]
RpcClient.semantic_focus = semantic_focus_sync  # type: ignore[attr-defined]
RpcClient.promote_readme = promote_readme_sync  # type: ignore[attr-defined]
RpcClient.compress_evidence = compress_evidence_sync  # type: ignore[attr-defined]
RpcClient.build_answer_plan = build_answer_plan_sync  # type: ignore[attr-defined]
RpcClient.render_llm_prompt = render_llm_prompt_sync  # type: ignore[attr-defined]
# File ops
RpcClient.file_content = file_content_sync  # type: ignore[attr-defined]
RpcClient.file_list = file_list_sync  # type: ignore[attr-defined]
RpcClient.file_summary = file_summary_sync  # type: ignore[attr-defined]
RpcClient.file_chunks = file_chunks_sync  # type: ignore[attr-defined]
RpcClient.file_outline = file_outline_sync  # type: ignore[attr-defined]
RpcClient.file_related = file_related_sync  # type: ignore[attr-defined]
RpcClient.file_search_by_type = file_search_by_type_sync  # type: ignore[attr-defined]
# Graph
RpcClient.graph_list_nodes = graph_list_nodes_sync  # type: ignore[attr-defined]
RpcClient.graph_neighbors = graph_neighbors_sync  # type: ignore[attr-defined]
RpcClient.graph_find_related = graph_find_related_sync  # type: ignore[attr-defined]
RpcClient.graph_find_path = graph_find_path_sync  # type: ignore[attr-defined]
RpcClient.graph_create_edge = graph_create_edge_sync  # type: ignore[attr-defined]
RpcClient.graph_delete_edge = graph_delete_edge_sync  # type: ignore[attr-defined]
RpcClient.graph_list_edges = graph_list_edges_sync  # type: ignore[attr-defined]
RpcClient.graph_discover_edges = graph_discover_edges_sync  # type: ignore[attr-defined]
RpcClient.graph_discover_edges_for_node = graph_discover_edges_for_node_sync  # type: ignore[attr-defined]
RpcClient.graph_discovery_status = graph_discovery_status_sync  # type: ignore[attr-defined]
# Admin
RpcClient.admin_stats = admin_stats_sync  # type: ignore[attr-defined]
RpcClient.admin_status = admin_status_sync  # type: ignore[attr-defined]
RpcClient.admin_logs = admin_logs_sync  # type: ignore[attr-defined]
RpcClient.admin_indexing_progress = admin_indexing_progress_sync  # type: ignore[attr-defined]
RpcClient.admin_config_get = admin_config_get_sync  # type: ignore[attr-defined]
RpcClient.admin_config_update = admin_config_update_sync  # type: ignore[attr-defined]
RpcClient.admin_backups_list = admin_backups_list_sync  # type: ignore[attr-defined]
RpcClient.admin_backups_create = admin_backups_create_sync  # type: ignore[attr-defined]
RpcClient.admin_backups_restore = admin_backups_restore_sync  # type: ignore[attr-defined]
RpcClient.admin_workspaces_list = admin_workspaces_list_sync  # type: ignore[attr-defined]
RpcClient.admin_workspace_get = admin_workspace_get_sync  # type: ignore[attr-defined]
RpcClient.admin_workspace_add = admin_workspace_add_sync  # type: ignore[attr-defined]
RpcClient.admin_workspace_remove = admin_workspace_remove_sync  # type: ignore[attr-defined]
RpcClient.admin_restart = admin_restart_sync  # type: ignore[attr-defined]
RpcClient.admin_slow_queries_list = admin_slow_queries_list_sync  # type: ignore[attr-defined]
RpcClient.admin_slow_queries_config = admin_slow_queries_config_sync  # type: ignore[attr-defined]
# Auth
RpcClient.auth_me = auth_me_sync  # type: ignore[attr-defined]
RpcClient.auth_logout = auth_logout_sync  # type: ignore[attr-defined]
RpcClient.auth_refresh_token = auth_refresh_token_sync  # type: ignore[attr-defined]
RpcClient.auth_validate_password = auth_validate_password_sync  # type: ignore[attr-defined]
RpcClient.auth_api_keys_create = auth_api_keys_create_sync  # type: ignore[attr-defined]
RpcClient.auth_api_keys_list = auth_api_keys_list_sync  # type: ignore[attr-defined]
RpcClient.auth_api_keys_revoke = auth_api_keys_revoke_sync  # type: ignore[attr-defined]
RpcClient.rotate_api_key_rpc = rotate_api_key_rpc_sync  # type: ignore[attr-defined]
RpcClient.auth_api_keys_create_scoped = auth_api_keys_create_scoped_sync  # type: ignore[attr-defined]
RpcClient.auth_introspect = auth_introspect_sync  # type: ignore[attr-defined]
RpcClient.auth_audit = auth_audit_sync  # type: ignore[attr-defined]
# Replication
RpcClient.replication_status = replication_status_sync  # type: ignore[attr-defined]
RpcClient.replication_configure = replication_configure_sync  # type: ignore[attr-defined]
RpcClient.replication_stats = replication_stats_sync  # type: ignore[attr-defined]
RpcClient.replication_replicas_list = replication_replicas_list_sync  # type: ignore[attr-defined]
# Cluster
RpcClient.cluster_failover = cluster_failover_sync  # type: ignore[attr-defined]
RpcClient.cluster_replica_resync = cluster_replica_resync_sync  # type: ignore[attr-defined]
RpcClient.cluster_peer_add = cluster_peer_add_sync  # type: ignore[attr-defined]
RpcClient.cluster_rebalance = cluster_rebalance_sync  # type: ignore[attr-defined]
RpcClient.cluster_rebalance_status = cluster_rebalance_status_sync  # type: ignore[attr-defined]

# ── AsyncRpcClient methods ────────────────────────────────────────────────────

# Collections
AsyncRpcClient.list_collections = list_collections_async  # type: ignore[attr-defined]
AsyncRpcClient.get_collection_info = get_collection_info_async  # type: ignore[attr-defined]
AsyncRpcClient.create_collection = create_collection_async  # type: ignore[attr-defined]
AsyncRpcClient.delete_collection = delete_collection_async  # type: ignore[attr-defined]
AsyncRpcClient.list_empty_collections = list_empty_collections_async  # type: ignore[attr-defined]
AsyncRpcClient.cleanup_empty_collections = cleanup_empty_collections_async  # type: ignore[attr-defined]
AsyncRpcClient.force_save_collection = force_save_collection_async  # type: ignore[attr-defined]
# Vectors
AsyncRpcClient.get_vector = get_vector_async  # type: ignore[attr-defined]
AsyncRpcClient.insert_vector = insert_vector_async  # type: ignore[attr-defined]
AsyncRpcClient.insert_text_vector = insert_text_vector_async  # type: ignore[attr-defined]
AsyncRpcClient.update_vector = update_vector_async  # type: ignore[attr-defined]
AsyncRpcClient.delete_vector_rpc = delete_vector_rpc_async  # type: ignore[attr-defined]
AsyncRpcClient.list_vectors = list_vectors_async  # type: ignore[attr-defined]
AsyncRpcClient.embed_text = embed_text_async  # type: ignore[attr-defined]
AsyncRpcClient.batch_insert_vectors = batch_insert_vectors_async  # type: ignore[attr-defined]
AsyncRpcClient.batch_insert_texts = batch_insert_texts_async  # type: ignore[attr-defined]
AsyncRpcClient.batch_search = batch_search_async  # type: ignore[attr-defined]
AsyncRpcClient.batch_update_vectors = batch_update_vectors_async  # type: ignore[attr-defined]
AsyncRpcClient.batch_delete_vectors = batch_delete_vectors_async  # type: ignore[attr-defined]
AsyncRpcClient.move_vectors_rpc = move_vectors_rpc_async  # type: ignore[attr-defined]
AsyncRpcClient.copy_vectors_rpc = copy_vectors_rpc_async  # type: ignore[attr-defined]
AsyncRpcClient.delete_by_filter_rpc = delete_by_filter_rpc_async  # type: ignore[attr-defined]
AsyncRpcClient.bulk_update_metadata_rpc = bulk_update_metadata_rpc_async  # type: ignore[attr-defined]
AsyncRpcClient.set_vector_expiry = set_vector_expiry_async  # type: ignore[attr-defined]
# Search
AsyncRpcClient.search_basic = search_basic_async  # type: ignore[attr-defined]
AsyncRpcClient.search_intelligent = search_intelligent_async  # type: ignore[attr-defined]
AsyncRpcClient.search_by_text = search_by_text_async  # type: ignore[attr-defined]
AsyncRpcClient.search_by_file = search_by_file_async  # type: ignore[attr-defined]
AsyncRpcClient.search_hybrid = search_hybrid_async  # type: ignore[attr-defined]
AsyncRpcClient.search_semantic = search_semantic_async  # type: ignore[attr-defined]
AsyncRpcClient.search_contextual = search_contextual_async  # type: ignore[attr-defined]
AsyncRpcClient.search_multi_collection = search_multi_collection_async  # type: ignore[attr-defined]
AsyncRpcClient.search_explain = search_explain_async  # type: ignore[attr-defined]
# Discovery
AsyncRpcClient.discover = discover_async  # type: ignore[attr-defined]
AsyncRpcClient.filter_collections = filter_collections_async  # type: ignore[attr-defined]
AsyncRpcClient.score_collections = score_collections_async  # type: ignore[attr-defined]
AsyncRpcClient.expand_queries = expand_queries_async  # type: ignore[attr-defined]
AsyncRpcClient.broad_discovery = broad_discovery_async  # type: ignore[attr-defined]
AsyncRpcClient.semantic_focus = semantic_focus_async  # type: ignore[attr-defined]
AsyncRpcClient.promote_readme = promote_readme_async  # type: ignore[attr-defined]
AsyncRpcClient.compress_evidence = compress_evidence_async  # type: ignore[attr-defined]
AsyncRpcClient.build_answer_plan = build_answer_plan_async  # type: ignore[attr-defined]
AsyncRpcClient.render_llm_prompt = render_llm_prompt_async  # type: ignore[attr-defined]
# File ops
AsyncRpcClient.file_content = file_content_async  # type: ignore[attr-defined]
AsyncRpcClient.file_list = file_list_async  # type: ignore[attr-defined]
AsyncRpcClient.file_summary = file_summary_async  # type: ignore[attr-defined]
AsyncRpcClient.file_chunks = file_chunks_async  # type: ignore[attr-defined]
AsyncRpcClient.file_outline = file_outline_async  # type: ignore[attr-defined]
AsyncRpcClient.file_related = file_related_async  # type: ignore[attr-defined]
AsyncRpcClient.file_search_by_type = file_search_by_type_async  # type: ignore[attr-defined]
# Graph
AsyncRpcClient.graph_list_nodes = graph_list_nodes_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_neighbors = graph_neighbors_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_find_related = graph_find_related_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_find_path = graph_find_path_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_create_edge = graph_create_edge_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_delete_edge = graph_delete_edge_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_list_edges = graph_list_edges_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_discover_edges = graph_discover_edges_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_discover_edges_for_node = graph_discover_edges_for_node_async  # type: ignore[attr-defined]
AsyncRpcClient.graph_discovery_status = graph_discovery_status_async  # type: ignore[attr-defined]
# Admin
AsyncRpcClient.admin_stats = admin_stats_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_status = admin_status_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_logs = admin_logs_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_indexing_progress = admin_indexing_progress_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_config_get = admin_config_get_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_config_update = admin_config_update_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_backups_list = admin_backups_list_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_backups_create = admin_backups_create_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_backups_restore = admin_backups_restore_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_workspaces_list = admin_workspaces_list_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_workspace_get = admin_workspace_get_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_workspace_add = admin_workspace_add_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_workspace_remove = admin_workspace_remove_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_restart = admin_restart_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_slow_queries_list = admin_slow_queries_list_async  # type: ignore[attr-defined]
AsyncRpcClient.admin_slow_queries_config = admin_slow_queries_config_async  # type: ignore[attr-defined]
# Auth
AsyncRpcClient.auth_me = auth_me_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_logout = auth_logout_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_refresh_token = auth_refresh_token_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_validate_password = auth_validate_password_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_api_keys_create = auth_api_keys_create_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_api_keys_list = auth_api_keys_list_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_api_keys_revoke = auth_api_keys_revoke_async  # type: ignore[attr-defined]
AsyncRpcClient.rotate_api_key_rpc = rotate_api_key_rpc_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_api_keys_create_scoped = auth_api_keys_create_scoped_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_introspect = auth_introspect_async  # type: ignore[attr-defined]
AsyncRpcClient.auth_audit = auth_audit_async  # type: ignore[attr-defined]
# Replication
AsyncRpcClient.replication_status = replication_status_async  # type: ignore[attr-defined]
AsyncRpcClient.replication_configure = replication_configure_async  # type: ignore[attr-defined]
AsyncRpcClient.replication_stats = replication_stats_async  # type: ignore[attr-defined]
AsyncRpcClient.replication_replicas_list = replication_replicas_list_async  # type: ignore[attr-defined]
# Cluster
AsyncRpcClient.cluster_failover = cluster_failover_async  # type: ignore[attr-defined]
AsyncRpcClient.cluster_replica_resync = cluster_replica_resync_async  # type: ignore[attr-defined]
AsyncRpcClient.cluster_peer_add = cluster_peer_add_async  # type: ignore[attr-defined]
AsyncRpcClient.cluster_rebalance = cluster_rebalance_async  # type: ignore[attr-defined]
AsyncRpcClient.cluster_rebalance_status = cluster_rebalance_status_async  # type: ignore[attr-defined]


__all__ = [
    # Dataclasses
    "AdminStats",
    "AdminStatus",
    "AnswerPlanResult",
    "AnswerPlanSection",
    "ApiKeyCreated",
    "AuthMeResult",
    "BatchDeleteResult",
    "BatchInsertResult",
    "BatchItemResult",
    "BatchSearchResult",
    "BatchUpdateResult",
    "BulkUpdateMetadataRpcResult",
    "CleanupEmptyResult",
    "CollectionInfo",
    "CompressBullet",
    "CopyRpcResult",
    "CreateCollectionResult",
    "DeleteByFilterRpcResult",
    "DiscoverEdgesForNodeResult",
    "DiscoverEdgesResult",
    "DiscoverResult",
    "DiscoveryChunk",
    "EmbedResult",
    "ExpandQueriesResult",
    "GraphDiscoveryStatus",
    "MoveRpcResult",
    "RebalanceStatus",
    "RefreshTokenResult",
    "RenderPromptResult",
    "ReplicationConfigureResult",
    "RotatedApiKey",
    "ScoredCollection",
    "SearchExplainResult",
    "SearchHit",
    "SearchTrace",
    "SetExpiryResult",
    "SlowQueryConfigResult",
    "ValidatePasswordResult",
    "VectorListResult",
    "VectorWriteResult",
    # Sync module-level functions (selection; clients prefer method form)
    "list_collections_sync",
    "get_collection_info_sync",
    "get_vector_sync",
    "search_basic_sync",
    # Async module-level functions
    "list_collections_async",
    "get_collection_info_async",
    "get_vector_async",
    "search_basic_async",
]
