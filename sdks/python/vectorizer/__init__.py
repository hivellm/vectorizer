"""Vectorizer Python SDK — per-surface client package.

This package is the new home for the Python SDK client, split by API
surface: :mod:`collections`, :mod:`vectors`, :mod:`search`,
:mod:`graph`, :mod:`admin`, :mod:`auth`. Legacy imports
(``from client import VectorizerClient``) are preserved by
``sdks/python/client.py`` acting as a compatibility shim.

Basic usage::

    from vectorizer import VectorizerClient

    async with VectorizerClient(base_url="http://localhost:15002") as client:
        info = await client.list_collections()

Advanced usage (direct sub-client access)::

    from vectorizer.collections import CollectionsClient
    from vectorizer import RestTransport

    transport = RestTransport("http://localhost:15002", api_key="...")
    collections = CollectionsClient(transport)
    info = await collections.list_collections()

The :class:`Transport` base class is abstract: the default
:class:`RestTransport` ships here and the ``RpcTransport`` from the
forthcoming ``phase6_sdk-python-rpc`` work plugs in by subclassing the
same ABC. See ``docs/specs/VECTORIZER_RPC.md`` for the RPC URL scheme
``vectorizer://host:15503``.
"""

from ._base import (
    Transport,
    RestTransport,
    TransportRouter,
    AuthState,
)
from .collections import CollectionsClient
from .vectors import VectorsClient
from .search import SearchClient
from .graph import GraphClient
from .admin import AdminClient
from .auth import AuthClient
from .replication import ReplicationClient
from .hub import HubClient
from .discovery import DiscoveryClient
from .client import VectorizerClient

try:
    from ..models import (
        AddPeerRequest,
        ApiKeyScope,
        ApiKeyUsageBucket,
        ApiKeyUsageReport,
        ApiKeyView,
        AuditEntry,
        AuditQuery,
        BulkUpdateReport,
        CopyReport,
        CreateScopedApiKeyRequest,
        DeleteByFilterReport,
        ExplainResponse,
        ExplainTrace,
        FailoverReport,
        NativeSnapshotInfo,
        PeerInfo,
        RebalanceJob,
        ReencodeJob,
        ReindexJob,
        ReindexParams,
        ResyncJob,
        RotatedKey,
        RouteStats,
        RuntimeMetrics,
        SlowQueryConfig,
        SlowQueryEntry,
        TokenIntrospection,
        TokenScope,
        UpdateApiKeyPermissionsRequest,
        VectorCountSample,
        WalSnapshot,
    )
except ImportError:  # pragma: no cover
    from models import (  # type: ignore[import-not-found]
        AddPeerRequest,
        ApiKeyScope,
        ApiKeyUsageBucket,
        ApiKeyUsageReport,
        ApiKeyView,
        AuditEntry,
        AuditQuery,
        BulkUpdateReport,
        CopyReport,
        CreateScopedApiKeyRequest,
        DeleteByFilterReport,
        ExplainResponse,
        ExplainTrace,
        FailoverReport,
        NativeSnapshotInfo,
        PeerInfo,
        RebalanceJob,
        ReencodeJob,
        ReindexJob,
        ReindexParams,
        ResyncJob,
        RotatedKey,
        RouteStats,
        RuntimeMetrics,
        SlowQueryConfig,
        SlowQueryEntry,
        TokenIntrospection,
        TokenScope,
        UpdateApiKeyPermissionsRequest,
        VectorCountSample,
        WalSnapshot,
    )

__all__ = [
    "Transport",
    "RestTransport",
    "TransportRouter",
    "AuthState",
    "VectorizerClient",
    "CollectionsClient",
    "VectorsClient",
    "SearchClient",
    "GraphClient",
    "AdminClient",
    "AuthClient",
    "ReplicationClient",
    "HubClient",
    "DiscoveryClient",
    # phase13 tier-control dataclasses
    "BulkUpdateReport",
    "CopyReport",
    "DeleteByFilterReport",
    "ReencodeJob",
    # phase14 schema-evolution + observability dataclasses
    "ExplainResponse",
    "ExplainTrace",
    "NativeSnapshotInfo",
    "ReindexJob",
    "ReindexParams",
    "SlowQueryConfig",
    "SlowQueryEntry",
    # phase15 cluster + auth admin dataclasses
    "AddPeerRequest",
    "AuditEntry",
    "AuditQuery",
    "CreateScopedApiKeyRequest",
    "FailoverReport",
    "PeerInfo",
    "RebalanceJob",
    "ResyncJob",
    "RotatedKey",
    "TokenIntrospection",
    "TokenScope",
    # API key usage metrics + permission update
    "ApiKeyScope",
    "ApiKeyUsageBucket",
    "ApiKeyUsageReport",
    "ApiKeyView",
    "UpdateApiKeyPermissionsRequest",
    # phase25 dashboard metrics
    "RuntimeMetrics",
    "RouteStats",
    "WalSnapshot",
    "VectorCountSample",
]
