"""Flat :class:`VectorizerClient` facade — composes all sub-clients.

The facade preserves the exact constructor and method surface of the
legacy 2,907-line ``sdks/python/client.py``. Internally it builds the
transport(s), creates one :class:`_base.TransportRouter`, and
instantiates one sub-client per API surface (``self.collections``,
``self.vectors``, ``self.search``, ``self.graph``, ``self.admin``,
``self.auth``). Flat calls like ``client.list_collections()`` resolve
through ``__getattr__`` to the owning sub-client — no duplicate
delegator method bodies.
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

import aiohttp

try:
    from ..exceptions import (
        AuthenticationError,
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
        VectorizerError,
    )
    from ..models import HostConfig, ReadOptions, ReadPreference
    from ..utils.http_client import HTTPClient
    from ..utils.transport import (
        TransportFactory,
        TransportProtocol,
        parse_connection_string,
    )
except ImportError:  # pragma: no cover
    from exceptions import (
        AuthenticationError,
        CollectionNotFoundError,
        NetworkError,
        ServerError,
        ValidationError,
        VectorizerError,
    )
    from models import HostConfig, ReadOptions, ReadPreference
    from utils.http_client import HTTPClient
    from utils.transport import (
        TransportFactory,
        TransportProtocol,
        parse_connection_string,
    )

from ._base import (
    AuthState,
    RestTransport,
    Transport,
    TransportRouter,
    build_router_from_hosts,
)
from .admin import AdminClient
from .auth import AuthClient
from .collections import CollectionsClient
from .graph import GraphClient
from .search import SearchClient
from .vectors import VectorsClient

logger = logging.getLogger(__name__)


class VectorizerClient:
    """Main Vectorizer SDK client.

    Supports HTTP(S), UMICP, and master/replica topologies. Identical
    constructor signature to the legacy flat client — keeps
    ``from vectorizer import VectorizerClient`` drop-in compatible.
    """

    def __init__(
        self,
        base_url: str = "http://localhost:15002",
        ws_url: str = "ws://localhost:15002/ws",
        api_key: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3,
        connection_string: Optional[str] = None,
        protocol: Optional[str] = None,
        umicp: Optional[Dict[str, Any]] = None,
        hosts: Optional[HostConfig] = None,
        read_preference: ReadPreference = ReadPreference.REPLICA,
    ):
        self.base_url = base_url.rstrip('/')
        self.ws_url = ws_url
        self.api_key = api_key
        self.timeout = timeout
        self.max_retries = max_retries
        self._session: Optional[aiohttp.ClientSession] = None
        self._ws_connection = None

        self._hosts = hosts
        self._read_preference = read_preference
        self._is_replica_mode = hosts is not None
        self._master_transport: Optional[Any] = None
        self._replica_transports: List[Any] = []
        self._replica_index = 0

        if hosts:
            self._initialize_replica_mode()
        else:
            self._initialize_single_mode(
                connection_string=connection_string,
                protocol=protocol,
                umicp=umicp,
            )

        auth = AuthState(api_key=api_key)

        # Sub-clients share a single router + auth state
        common_kwargs = dict(base_url=self.base_url, router=self._router, auth=auth)
        self.collections = CollectionsClient(self._transport, **common_kwargs)
        self.vectors = VectorsClient(self._transport, **common_kwargs)
        self.search = SearchClient(self._transport, **common_kwargs)
        self.graph = GraphClient(self._transport, **common_kwargs)
        self.admin = AdminClient(self._transport, **common_kwargs)
        self.auth = AuthClient(self._transport, api_key=api_key, **common_kwargs)

        # Ordered list used by __getattr__ to delegate unknown calls
        self._subclients: List[Any] = [
            self.collections,
            self.vectors,
            self.search,
            self.graph,
            self.admin,
            self.auth,
        ]

    # ------------------------------------------------------------------
    # Construction helpers
    # ------------------------------------------------------------------

    def _initialize_single_mode(
        self,
        *,
        connection_string: Optional[str],
        protocol: Optional[str],
        umicp: Optional[Dict[str, Any]],
    ) -> None:
        if connection_string:
            proto, config = parse_connection_string(connection_string, self.api_key)
            config['timeout'] = self.timeout
            config['max_retries'] = self.max_retries
            self._transport = TransportFactory.create(proto, config)
            self._protocol = proto
            logger.info(
                f"VectorizerClient initialized from connection string (protocol: {proto})"
            )
        elif protocol:
            proto_enum = TransportProtocol(protocol.lower())
            if proto_enum == TransportProtocol.HTTP:
                self._transport = HTTPClient(
                    base_url=self.base_url,
                    api_key=self.api_key,
                    timeout=self.timeout,
                    max_retries=self.max_retries,
                )
                self._protocol = proto_enum
            elif proto_enum == TransportProtocol.UMICP:
                if not umicp:
                    raise ValueError(
                        "UMICP configuration is required when using UMICP protocol"
                    )
                config = {
                    "host": umicp.get("host", "localhost"),
                    "port": umicp.get("port", 15003),
                    "api_key": self.api_key,
                    "timeout": self.timeout,
                }
                self._transport = TransportFactory.create(proto_enum, config)
                self._protocol = proto_enum
                logger.info(
                    f"VectorizerClient initialized with UMICP "
                    f"(host: {config['host']}, port: {config['port']})"
                )
            else:  # pragma: no cover — enum exhaustion
                raise ValueError(f"Unsupported protocol: {protocol}")
        else:
            self._transport = HTTPClient(
                base_url=self.base_url,
                api_key=self.api_key,
                timeout=self.timeout,
                max_retries=self.max_retries,
            )
            self._protocol = TransportProtocol.HTTP
            logger.info(
                f"VectorizerClient initialized with HTTP (base_url: {self.base_url})"
            )

        self._router = TransportRouter(primary=self._transport)

    def _initialize_replica_mode(self) -> None:
        assert self._hosts is not None
        self._is_replica_mode = True
        self._protocol = TransportProtocol.HTTP

        master_config = {
            "base_url": self._hosts.master,
            "api_key": self.api_key,
            "timeout": self.timeout,
            "max_retries": self.max_retries,
        }
        self._master_transport = HTTPClient(**master_config)

        self._replica_transports = []
        for replica_url in self._hosts.replicas:
            replica_config = {
                "base_url": replica_url,
                "api_key": self.api_key,
                "timeout": self.timeout,
                "max_retries": self.max_retries,
            }
            self._replica_transports.append(HTTPClient(**replica_config))

        self._transport = self._master_transport
        self._router = TransportRouter(
            primary=self._master_transport,
            master_transport=self._master_transport,
            replica_transports=list(self._replica_transports),
            read_preference=self._read_preference,
        )

        logger.info(
            f"VectorizerClient initialized with master/replica topology "
            f"(master: {self._hosts.master}, replicas: {self._hosts.replicas}, "
            f"read_preference: {self._read_preference.value})"
        )

    # ------------------------------------------------------------------
    # Public helpers (preserved from legacy client)
    # ------------------------------------------------------------------

    def get_protocol(self) -> str:
        """Return the active transport protocol identifier."""
        return self._protocol.value if hasattr(self._protocol, 'value') else str(self._protocol)

    def _get_write_transport(self) -> Any:
        """Transport for write operations (always master in replica mode)."""
        return self._router.write_transport()

    def _get_read_transport(self, options: Optional[ReadOptions] = None) -> Any:
        """Transport for read operations based on read preference."""
        return self._router.read_transport(options)

    async def with_master(self, callback):
        """Run ``callback`` with a MASTER-preference client for read-your-writes."""
        master_client = VectorizerClient(
            base_url=self.base_url,
            api_key=self.api_key,
            timeout=self.timeout,
            max_retries=self.max_retries,
            hosts=self._hosts,
            read_preference=ReadPreference.MASTER,
        )
        return await callback(master_client)

    async def __aenter__(self):
        await self.connect()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()

    async def connect(self):
        """Initialize transport session. Matches legacy behavior exactly."""
        if self._protocol == TransportProtocol.HTTP:
            if self._session is None or self._session.closed:
                headers = {}
                if self.api_key:
                    headers["Authorization"] = f"Bearer {self.api_key}"

                timeout = aiohttp.ClientTimeout(total=self.timeout)
                self._session = aiohttp.ClientSession(
                    headers=headers,
                    timeout=timeout,
                )
        elif self._protocol == TransportProtocol.UMICP:
            if hasattr(self._transport, 'connect'):
                await self._transport.connect()

    async def close(self):
        """Close transport session and any websocket connection."""
        if self._session and not self._session.closed:
            await self._session.close()

        if hasattr(self._transport, 'close'):
            await self._transport.close()
        elif hasattr(self._transport, 'disconnect'):
            await self._transport.disconnect()

        if self._ws_connection:
            await self._ws_connection.close()

    # ------------------------------------------------------------------
    # Flat-API compatibility: delegate unknown attribute access to
    # the first sub-client that exposes it. This is what keeps
    # ``await client.list_collections()`` etc. working identically.
    # ------------------------------------------------------------------

    def __getattr__(self, name: str) -> Any:
        # __getattr__ is only called when normal attribute resolution fails;
        # avoid recursion when __init__ hasn't populated _subclients yet.
        if name.startswith("_") or name in {
            "collections",
            "vectors",
            "search",
            "graph",
            "admin",
            "auth",
        }:
            raise AttributeError(name)
        try:
            subs = object.__getattribute__(self, "_subclients")
        except AttributeError:
            raise AttributeError(name)
        for sub in subs:
            if hasattr(sub, name):
                return getattr(sub, name)
        raise AttributeError(
            f"'VectorizerClient' object has no attribute {name!r}"
        )
