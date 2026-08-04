"""Asynchronous ``AsyncRpcClient`` over a single TCP connection.

The asyncio twin of :class:`rpc.sync_client.RpcClient`, backed by
:class:`thunder_rpc.AsyncClient`. Same wire behaviour: credentials in the
connection handshake, HELLO for capabilities, and concurrent calls
multiplexed over one connection by frame id.

Use this client from inside an event loop. The synchronous client is
the right choice for blocking scripts and notebooks.
"""

from __future__ import annotations

from typing import Optional, Sequence

from thunder_rpc import AsyncClient, ClientConfig, ThunderError

from rpc.endpoint import Endpoint, parse_endpoint
from rpc.sync_client import (
    HelloPayload,
    HelloResponse,
    RpcClientError,
    RpcConnectionClosed,
    RpcNotAuthenticated,
    RpcProtocolError,
    RpcServerError,
    RpcTimeout,
    _to_rpc_error,
    protocol_config,
)
from rpc.types import VectorizerValue


class AsyncRpcClient:
    """One asyncio connection to a Vectorizer RPC server.

    Construct via :meth:`connect` or :meth:`connect_url`. Issue
    :meth:`hello` with credentials when the server enforces auth.

    Coroutine-safe: multiple ``await client.X()`` calls from the same or
    different tasks may run concurrently; Thunder multiplexes them over the
    one connection and demultiplexes the replies by frame id.
    """

    def __init__(
        self, client: AsyncClient, endpoint: str, client_config: ClientConfig
    ) -> None:
        self._client = client
        self._endpoint = endpoint
        self._client_config = client_config
        self._closed = False

    # ── construction ─────────────────────────────────────────────────
    @classmethod
    async def connect(
        cls, address: str, *, timeout: Optional[float] = None
    ) -> "AsyncRpcClient":
        """Dial ``address`` — ``host:port``, or any form
        :func:`thunder_rpc.parse_endpoint` accepts.

        Does NOT authenticate: pass credentials to :meth:`hello`, which
        re-dials with them in the handshake.
        """
        client_config = ClientConfig(client_name="vectorizer-python-sdk")
        if timeout is not None:
            client_config = ClientConfig(
                connect_timeout=timeout,
                call_timeout=timeout,
                client_name="vectorizer-python-sdk",
            )
        client = await cls._dial(address, client_config)
        return cls(client, address, client_config)

    @classmethod
    async def connect_url(
        cls, url: str, *, timeout: Optional[float] = None
    ) -> "AsyncRpcClient":
        """Parse a ``vectorizer://host[:port]`` URL and dial it.

        REST URLs (``http(s)://``) are rejected with a clear error
        pointing the caller at the HTTP client.
        """
        ep = parse_endpoint(url)
        if isinstance(ep, Endpoint.Rpc):
            return await cls.connect(f"{ep.host}:{ep.port}", timeout=timeout)
        if isinstance(ep, Endpoint.Rest):
            raise RpcServerError(
                f"AsyncRpcClient cannot dial REST URL '{ep.url}'; "
                f"use the HTTP client (VectorizerClient) instead, "
                f"or pass a 'vectorizer://' URL"
            )
        raise RpcServerError(f"unrecognised endpoint shape: {ep!r}")

    @staticmethod
    async def _dial(endpoint: str, client_config: ClientConfig) -> AsyncClient:
        try:
            return await AsyncClient.connect(endpoint, protocol_config(), client_config)
        except ThunderError as exc:
            raise _to_rpc_error(exc) from exc

    # ── handshake + health ───────────────────────────────────────────
    async def hello(self, payload: HelloPayload) -> HelloResponse:
        """Issue the HELLO handshake and return the server's capability
        list and auth flags.

        When ``payload`` carries a token or an API key, the connection is
        re-dialed so those credentials travel in Thunder's ``AUTH``
        handshake — that is what authenticates the session every later
        command runs under.
        """
        credentials = payload.credentials()
        if credentials is not None:
            client_config = ClientConfig(
                connect_timeout=self._client_config.connect_timeout,
                call_timeout=self._client_config.call_timeout,
                credentials=credentials,
                client_name=payload.client_name or self._client_config.client_name,
            )
            fresh = await self._dial(self._endpoint, client_config)
            previous = self._client
            self._client = fresh
            self._client_config = client_config
            await previous.close()
        return HelloResponse.parse(await self.call("HELLO", [payload.to_value()]))

    async def ping(self) -> str:
        """Health check. Auth-exempt per wire spec § 4 — works pre-HELLO."""
        s = (await self.call("PING", [])).as_str()
        if s is None:
            raise RpcServerError("PING returned non-string payload")
        return s

    # ── generic dispatch ─────────────────────────────────────────────
    async def call(
        self, command: str, args: Optional[Sequence[VectorizerValue]] = None
    ) -> VectorizerValue:
        """Dispatch a generic command. Most callers should reach for a
        typed wrapper from :mod:`rpc.commands` instead.

        The server gates un-authenticated sessions, so a data-plane command
        on a session that never authenticated raises
        :class:`RpcNotAuthenticated`.
        """
        try:
            return await self._client.call(command, list(args or []))
        except ThunderError as exc:
            raise _to_rpc_error(exc) from exc

    def is_authenticated(self) -> bool:
        """``True`` once the connection's handshake authenticated. Always
        ``False`` against an open (single-user) server, which authenticates
        nobody because it gates nothing."""
        return self._client.is_authenticated()

    # ── shutdown ─────────────────────────────────────────────────────
    async def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        await self._client.close()

    async def __aenter__(self) -> "AsyncRpcClient":
        return self

    async def __aexit__(self, *_exc: object) -> None:
        await self.close()


__all__ = [
    "AsyncRpcClient",
    "HelloPayload",
    "HelloResponse",
    "RpcClientError",
    "RpcConnectionClosed",
    "RpcNotAuthenticated",
    "RpcProtocolError",
    "RpcServerError",
    "RpcTimeout",
]
