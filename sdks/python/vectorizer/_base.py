"""Transport abstraction and shared routing helpers for the Vectorizer SDK.

This module hosts:

* :class:`Transport` — the abstract base class every concrete transport
  inherits from. The REST transport ships here; the binary RPC transport
  from ``phase6_sdk-python-rpc`` plugs in by subclassing the same ABC.
* :class:`RestTransport` — the concrete REST implementation backed by the
  existing ``utils.http_client.HTTPClient`` (aiohttp). It accepts the
  canonical ``http://host:port`` URL form.
* :class:`TransportRouter` — routes read/write operations across a master
  and one or more replicas. The routing rules were previously buried
  inside :class:`client.VectorizerClient`; they are now reusable by any
  surface client.
* :class:`AuthState` — a light container for the bearer token so
  per-surface modules don't reach into the facade to read it.

Per-surface client classes take a :class:`TransportRouter` in their
constructor and *must not* call ``aiohttp`` directly — that guarantees
the same surface works against an RPC transport once
``phase6_sdk-python-rpc`` lands.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, List, Optional

try:  # package-relative imports for `import vectorizer` consumers
    from ..models import HostConfig, ReadOptions, ReadPreference
except ImportError:  # pragma: no cover - flat import path used by legacy tests
    from models import HostConfig, ReadOptions, ReadPreference

try:
    from ..utils.http_client import HTTPClient
except ImportError:  # pragma: no cover
    from utils.http_client import HTTPClient


class Transport(ABC):
    """Abstract transport base class.

    Concrete transports (REST, RPC, UMICP, ...) subclass this and
    implement the four HTTP-style primitives. Surface clients only call
    methods declared here — never ``aiohttp`` / ``httpx`` directly.
    """

    base_url: str = ""

    @abstractmethod
    async def get(self, path: str) -> Any:
        """Issue a GET to ``path`` and return the decoded response body."""

    @abstractmethod
    async def post(self, path: str, data: Optional[Any] = None) -> Any:
        """Issue a POST to ``path`` with ``data`` and return the body."""

    @abstractmethod
    async def put(self, path: str, data: Optional[Any] = None) -> Any:
        """Issue a PUT to ``path`` with ``data`` and return the body."""

    @abstractmethod
    async def delete(self, path: str) -> Any:
        """Issue a DELETE to ``path`` and return the body."""

    async def connect(self) -> None:
        """Open any resources the transport needs. Default: no-op."""

    async def close(self) -> None:
        """Release any held resources. Default: no-op."""


class RestTransport(Transport):
    """REST transport backed by ``utils.http_client.HTTPClient`` (aiohttp).

    This matches the legacy ``VectorizerClient`` behavior byte-for-byte:
    URLs like ``http://localhost:15002`` are accepted; paths are joined
    against ``base_url``; errors are mapped to the SDK exception tree by
    the underlying :class:`HTTPClient`.
    """

    def __init__(
        self,
        base_url: str,
        *,
        api_key: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self._http = HTTPClient(
            base_url=self.base_url,
            api_key=api_key,
            timeout=timeout,
            max_retries=max_retries,
        )

    async def get(self, path: str) -> Any:
        return await self._http.get(path)

    async def post(self, path: str, data: Optional[Any] = None) -> Any:
        return await self._http.post(path, data)

    async def put(self, path: str, data: Optional[Any] = None) -> Any:
        return await self._http.put(path, data)

    async def delete(self, path: str) -> Any:
        return await self._http.delete(path)

    async def close(self) -> None:
        await self._http.close()


@dataclass
class AuthState:
    """Bearer-token + API-key state shared across surface clients."""

    api_key: Optional[str] = None

    def headers(self) -> dict:
        if not self.api_key:
            return {}
        return {"Authorization": f"Bearer {self.api_key}"}


@dataclass
class TransportRouter:
    """Routes read/write operations across master + replica transports.

    * Writes always go to ``master_transport`` (or ``primary`` when no
      master/replica topology is configured).
    * Reads honor ``read_preference`` (MASTER / REPLICA / NEAREST). When
      no replicas are configured, reads fall back to master.

    Surface clients never inspect the routing rules — they call
    :meth:`read_transport` and :meth:`write_transport`.
    """

    primary: Transport
    master_transport: Optional[Transport] = None
    replica_transports: List[Transport] = field(default_factory=list)
    read_preference: ReadPreference = ReadPreference.REPLICA
    _replica_index: int = 0

    @property
    def is_replica_mode(self) -> bool:
        return self.master_transport is not None and bool(self.replica_transports)

    def write_transport(self) -> Transport:
        if self.is_replica_mode and self.master_transport is not None:
            return self.master_transport
        return self.primary

    def read_transport(self, options: Optional[ReadOptions] = None) -> Transport:
        if not self.is_replica_mode:
            return self.primary

        preference = (
            options.read_preference
            if options and options.read_preference
            else self.read_preference
        )

        if preference == ReadPreference.MASTER:
            assert self.master_transport is not None
            return self.master_transport

        if preference in (ReadPreference.REPLICA, ReadPreference.NEAREST):
            if not self.replica_transports:
                assert self.master_transport is not None
                return self.master_transport
            transport = self.replica_transports[self._replica_index]
            self._replica_index = (self._replica_index + 1) % len(self.replica_transports)
            return transport

        assert self.master_transport is not None
        return self.master_transport

    async def close(self) -> None:
        seen = set()
        for t in (self.primary, self.master_transport, *self.replica_transports):
            if t is None or id(t) in seen:
                continue
            seen.add(id(t))
            await t.close()


class _ApiBase:
    """Shared base for per-surface clients.

    Holds the router + auth state + ``base_url``. Provides the same
    ``_get_read_transport``/``_get_write_transport`` accessors the legacy
    :class:`client.VectorizerClient` exposed, so the extracted methods
    work without edits.
    """

    def __init__(
        self,
        transport: Transport,
        *,
        base_url: Optional[str] = None,
        router: Optional[TransportRouter] = None,
        auth: Optional[AuthState] = None,
    ) -> None:
        self._transport: Transport = transport
        self.base_url: str = base_url if base_url is not None else getattr(transport, "base_url", "")
        self._router: TransportRouter = router or TransportRouter(primary=transport)
        self._auth: AuthState = auth or AuthState()

    @property
    def api_key(self) -> Optional[str]:
        return self._auth.api_key

    def _get_read_transport(self, options: Optional[ReadOptions] = None) -> Transport:
        return self._router.read_transport(options)

    def _get_write_transport(self) -> Transport:
        return self._router.write_transport()


def build_router_from_hosts(
    hosts: HostConfig,
    *,
    api_key: Optional[str],
    timeout: int,
    max_retries: int,
    read_preference: ReadPreference,
) -> TransportRouter:
    """Build a :class:`TransportRouter` from a :class:`HostConfig`.

    Used by :class:`client.VectorizerClient` when the caller passes a
    master/replica host configuration. The master transport doubles as
    the router's ``primary`` so operations fall back sanely if all
    replicas are unhealthy.
    """

    master = RestTransport(
        hosts.master,
        api_key=api_key,
        timeout=timeout,
        max_retries=max_retries,
    )
    replicas: List[Transport] = [
        RestTransport(url, api_key=api_key, timeout=timeout, max_retries=max_retries)
        for url in hosts.replicas
    ]
    return TransportRouter(
        primary=master,
        master_transport=master,
        replica_transports=replicas,
        read_preference=read_preference,
    )
