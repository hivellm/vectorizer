"""Minimal RPC connection pools.

A bounded pool of :class:`RpcClient` (sync) and a coroutine-safe pool
of :class:`AsyncRpcClient` (async). ``acquire()`` returns an idle
client (or builds a new one if the pool isn't at capacity); the
returned guard returns the client to the pool on exit.

Intentionally NOT ``aiohttp``-style with retries / health checks /
backoff — those bring complexity that the v1 SDK doesn't need. If a
future workload requires fancier pooling (per-connection health checks,
idle eviction), swap to a real pool implementation at that point.
"""

from __future__ import annotations

import asyncio
import threading
from dataclasses import dataclass, field
from typing import List, Optional

from rpc.async_client import AsyncRpcClient
from rpc.sync_client import HelloPayload, RpcClient


@dataclass
class RpcPoolConfig:
    """Configuration shared by sync and async pools.

    Attributes:
        address: ``host:port`` every connection in the pool dials.
        max_connections: Maximum number of concurrent open
            connections. Calls block on ``acquire()`` once this many
            are checked out.
        hello: HELLO payload sent on every newly-built connection.
        connect_timeout: Per-connection connect timeout in seconds.
            ``None`` blocks indefinitely.
    """

    address: str
    max_connections: int = 8
    hello: HelloPayload = field(default_factory=HelloPayload)
    connect_timeout: Optional[float] = None


# ── Synchronous pool ──────────────────────────────────────────────────


class PooledClient:
    """Context-manager guard returned by :meth:`RpcPool.acquire`.

    Returns the inner client to the pool on ``__exit__`` so subsequent
    acquires reuse the connection. Use :attr:`client` to access the
    underlying :class:`RpcClient`.
    """

    def __init__(self, pool: "RpcPool", client: RpcClient) -> None:
        self._pool = pool
        self._client: Optional[RpcClient] = client

    @property
    def client(self) -> RpcClient:
        if self._client is None:
            raise RuntimeError("PooledClient already released")
        return self._client

    def release(self) -> None:
        if self._client is None:
            return
        client = self._client
        self._client = None
        self._pool._return(client)

    def __enter__(self) -> RpcClient:
        return self.client

    def __exit__(self, *_exc: object) -> None:
        self.release()

    def __del__(self) -> None:  # pragma: no cover — destructor path
        try:
            self.release()
        except Exception:
            pass


class RpcPool:
    """Bounded pool of synchronous :class:`RpcClient` connections.

    Does NOT open any connections eagerly; the first :meth:`acquire`
    dials the first connection. ``max_connections`` is enforced via a
    semaphore so simultaneous acquires beyond the cap block until a
    slot frees.
    """

    def __init__(self, config: RpcPoolConfig) -> None:
        max_conns = max(1, config.max_connections)
        self._config = RpcPoolConfig(
            address=config.address,
            max_connections=max_conns,
            hello=config.hello,
            connect_timeout=config.connect_timeout,
        )
        self._semaphore = threading.Semaphore(max_conns)
        self._idle: List[RpcClient] = []
        self._idle_lock = threading.Lock()

    def acquire(self) -> PooledClient:
        """Acquire a client from the pool. Blocks when the pool is at
        capacity until a slot frees. The returned :class:`PooledClient`
        returns the client to the pool when used as a context manager
        or when :meth:`PooledClient.release` is called explicitly."""
        self._semaphore.acquire()
        try:
            client = self._take_idle() or self._build_new()
        except BaseException:
            # Building failed; release the permit so other acquires
            # don't hang waiting for a slot that's not actually held.
            self._semaphore.release()
            raise
        return PooledClient(self, client)

    def idle_count(self) -> int:
        """Number of idle clients currently in the pool. For diagnostics
        and tests only — production code should not branch on this."""
        with self._idle_lock:
            return len(self._idle)

    def close(self) -> None:
        """Close every idle connection. In-flight clients (held by
        callers) are unaffected; they close on their own ``release``
        path or when the caller drops the reference."""
        with self._idle_lock:
            idles = self._idle
            self._idle = []
        for c in idles:
            c.close()

    # ── internals ────────────────────────────────────────────────────
    def _take_idle(self) -> Optional[RpcClient]:
        with self._idle_lock:
            if self._idle:
                return self._idle.pop()
        return None

    def _build_new(self) -> RpcClient:
        client = RpcClient.connect(
            self._config.address, timeout=self._config.connect_timeout
        )
        client.hello(self._config.hello)
        return client

    def _return(self, client: RpcClient) -> None:
        with self._idle_lock:
            self._idle.append(client)
        self._semaphore.release()


# ── Asynchronous pool ─────────────────────────────────────────────────


class AsyncPooledClient:
    """``async with`` guard returned by :meth:`AsyncRpcPool.acquire`."""

    def __init__(self, pool: "AsyncRpcPool", client: AsyncRpcClient) -> None:
        self._pool = pool
        self._client: Optional[AsyncRpcClient] = client

    @property
    def client(self) -> AsyncRpcClient:
        if self._client is None:
            raise RuntimeError("AsyncPooledClient already released")
        return self._client

    async def release(self) -> None:
        if self._client is None:
            return
        client = self._client
        self._client = None
        await self._pool._return(client)

    async def __aenter__(self) -> AsyncRpcClient:
        return self.client

    async def __aexit__(self, *_exc: object) -> None:
        await self.release()


class AsyncRpcPool:
    """Bounded pool of :class:`AsyncRpcClient` connections.

    Use one pool per event loop. Capacity is enforced by an
    :class:`asyncio.Semaphore`. Like the sync pool, idle connections
    are reused; a torn connection surfaces on the next call as
    :class:`RpcConnectionClosed` rather than being re-validated up-front.
    """

    def __init__(self, config: RpcPoolConfig) -> None:
        max_conns = max(1, config.max_connections)
        self._config = RpcPoolConfig(
            address=config.address,
            max_connections=max_conns,
            hello=config.hello,
            connect_timeout=config.connect_timeout,
        )
        self._semaphore = asyncio.Semaphore(max_conns)
        self._idle: List[AsyncRpcClient] = []
        self._idle_lock = asyncio.Lock()

    async def acquire(self) -> AsyncPooledClient:
        await self._semaphore.acquire()
        try:
            client = await self._take_idle() or await self._build_new()
        except BaseException:
            self._semaphore.release()
            raise
        return AsyncPooledClient(self, client)

    async def idle_count(self) -> int:
        async with self._idle_lock:
            return len(self._idle)

    async def close(self) -> None:
        async with self._idle_lock:
            idles = self._idle
            self._idle = []
        for c in idles:
            await c.close()

    async def _take_idle(self) -> Optional[AsyncRpcClient]:
        async with self._idle_lock:
            if self._idle:
                return self._idle.pop()
        return None

    async def _build_new(self) -> AsyncRpcClient:
        client = await AsyncRpcClient.connect(
            self._config.address, timeout=self._config.connect_timeout
        )
        await client.hello(self._config.hello)
        return client

    async def _return(self, client: AsyncRpcClient) -> None:
        async with self._idle_lock:
            self._idle.append(client)
        self._semaphore.release()


__all__ = [
    "AsyncPooledClient",
    "AsyncRpcPool",
    "PooledClient",
    "RpcPool",
    "RpcPoolConfig",
]
