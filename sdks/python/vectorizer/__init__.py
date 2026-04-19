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
from .client import VectorizerClient

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
]
