"""Asynchronous ``AsyncRpcClient`` over a single TCP connection.

The asyncio twin of :class:`rpc.sync_client.RpcClient`. Same wire
behaviour: HELLO + sticky auth + multiplexed call/response by
``Request.id``. Uses ``asyncio.open_connection`` for the socket and a
single background task as the reader; per-call ``asyncio.Future``
mailboxes carry responses back to awaiters.

Use this client from inside an event loop. The synchronous client is
the right choice for blocking scripts and notebooks.
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Dict, List, Optional, Sequence

from rpc._codec import encode_frame, read_frame_async
from rpc.endpoint import Endpoint, parse_endpoint
from rpc.sync_client import (
    HelloPayload,
    HelloResponse,
    RpcClientError,
    RpcConnectionClosed,
    RpcNotAuthenticated,
    RpcServerError,
    _AUTH_EXEMPT,
    _split_host_port,
)
from rpc.types import Request, Response, VectorizerValue


@dataclass
class _PendingCall:
    """Container for a future + the request that produced it. Kept in
    a dict keyed by request id; the reader task fulfills the future
    when the matching response arrives."""

    future: "asyncio.Future[Response]"


class AsyncRpcClient:
    """One asyncio connection to a Vectorizer RPC server.

    Construct via :meth:`connect` or :meth:`connect_url`. Always issue
    :meth:`hello` before any data-plane call.

    Coroutine-safe: multiple ``await client.X()`` calls from the same
    or different tasks may run concurrently; the writer is serialised
    by an ``asyncio.Lock`` and responses are demultiplexed by ``id``
    into per-call futures.
    """

    def __init__(
        self,
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter,
    ) -> None:
        self._reader = reader
        self._writer = writer
        self._writer_lock = asyncio.Lock()
        self._pending: Dict[int, _PendingCall] = {}
        self._next_id = 1
        self._authenticated = False
        self._closed = False
        self._reader_task: asyncio.Task[None] = asyncio.ensure_future(self._read_loop())

    # ── construction ─────────────────────────────────────────────────
    @classmethod
    async def connect(
        cls, address: str, *, timeout: Optional[float] = None
    ) -> "AsyncRpcClient":
        """Open a TCP connection to ``address`` (``host:port``).

        Does NOT send HELLO — callers MUST ``await client.hello(...)``
        before any data-plane command, or the server will reject it.
        """
        host, port = _split_host_port(address)
        coro = asyncio.open_connection(host=host, port=port)
        if timeout is not None:
            reader, writer = await asyncio.wait_for(coro, timeout=timeout)
        else:
            reader, writer = await coro
        # Disable Nagle on the underlying socket if accessible.
        sock = writer.get_extra_info("socket")
        if sock is not None:
            try:
                import socket as _socket  # local import keeps stdlib socket out of the hot path

                sock.setsockopt(_socket.IPPROTO_TCP, _socket.TCP_NODELAY, 1)
            except (OSError, AttributeError):
                pass
        return cls(reader, writer)

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

    # ── handshake + health ───────────────────────────────────────────
    async def hello(self, payload: HelloPayload) -> HelloResponse:
        result = await self._raw_call("HELLO", [payload.to_value()])
        parsed = HelloResponse.parse(result)
        if parsed.authenticated:
            self._authenticated = True
        return parsed

    async def ping(self) -> str:
        result = await self._raw_call("PING", [])
        s = result.as_str()
        if s is None:
            raise RpcServerError("PING returned non-string payload")
        return s

    # ── generic dispatch ─────────────────────────────────────────────
    async def call(
        self, command: str, args: Optional[Sequence[VectorizerValue]] = None
    ) -> VectorizerValue:
        if command not in _AUTH_EXEMPT and not self._authenticated:
            raise RpcNotAuthenticated(
                "HELLO must succeed before any data-plane command can be issued"
            )
        return await self._raw_call(command, list(args or []))

    def is_authenticated(self) -> bool:
        return self._authenticated

    # ── shutdown ─────────────────────────────────────────────────────
    async def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        try:
            self._writer.close()
            await self._writer.wait_closed()
        except (ConnectionError, OSError):
            pass
        self._reader_task.cancel()
        try:
            await self._reader_task
        except (asyncio.CancelledError, Exception):
            pass
        self._fail_all_pending()

    async def __aenter__(self) -> "AsyncRpcClient":
        return self

    async def __aexit__(self, *_exc: object) -> None:
        await self.close()

    # ── internals ────────────────────────────────────────────────────
    def _alloc_id(self) -> int:
        rid = self._next_id
        self._next_id = (self._next_id + 1) & 0xFFFFFFFF
        if self._next_id == 0:
            self._next_id = 1
        return rid

    async def _raw_call(self, command: str, args: List[VectorizerValue]) -> VectorizerValue:
        rid = self._alloc_id()
        loop = asyncio.get_running_loop()
        fut: "asyncio.Future[Response]" = loop.create_future()
        self._pending[rid] = _PendingCall(future=fut)

        req = Request(id=rid, command=command, args=args)
        frame = encode_frame(req.to_msgpack())

        try:
            async with self._writer_lock:
                self._writer.write(frame)
                await self._writer.drain()
        except (ConnectionError, OSError) as e:
            self._pending.pop(rid, None)
            raise RpcConnectionClosed(f"send failed: {e}") from e

        try:
            resp = await fut
        except asyncio.CancelledError:
            # Caller cancelled — drop the pending entry so the reader
            # doesn't try to fulfill a future that nobody is awaiting.
            self._pending.pop(rid, None)
            raise

        tag, payload = resp.result
        if tag == "Ok":
            assert isinstance(payload, VectorizerValue)
            return payload
        raise RpcServerError(str(payload))

    async def _read_loop(self) -> None:
        try:
            while True:
                raw = await read_frame_async(self._reader)
                resp = Response.from_msgpack(raw)
                pending = self._pending.pop(resp.id, None)
                if pending is not None and not pending.future.done():
                    pending.future.set_result(resp)
                # else: response with no pending caller — drop silently.
        except (
            asyncio.IncompleteReadError,
            ConnectionError,
            OSError,
            ValueError,
            asyncio.CancelledError,
        ):
            pass
        finally:
            self._fail_all_pending()

    def _fail_all_pending(self) -> None:
        pending = list(self._pending.items())
        self._pending.clear()
        for _rid, p in pending:
            if not p.future.done():
                p.future.set_exception(
                    RpcConnectionClosed("connection closed before response")
                )


__all__ = [
    "AsyncRpcClient",
    "HelloPayload",
    "HelloResponse",
    "RpcClientError",
    "RpcConnectionClosed",
    "RpcNotAuthenticated",
    "RpcServerError",
]
