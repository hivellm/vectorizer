"""UMICP connection pooling."""

from typing import Optional, Any, Dict, List
from dataclasses import dataclass
from datetime import datetime, timedelta
import asyncio


@dataclass
class PoolConfig:
    """Connection pool configuration."""

    min_size: int = 1
    max_size: int = 10
    idle_timeout_seconds: int = 300
    stale_timeout_seconds: int = 600
    acquire_timeout_seconds: int = 10


class ConnectionPool:
    """Generic connection pool."""

    def __init__(self, config: Optional[PoolConfig] = None) -> None:
        """Initialize connection pool.

        Args:
            config: Pool configuration
        """
        self.config = config or PoolConfig()
        self._connections: List[Dict[str, Any]] = []
        self._available: asyncio.Queue = asyncio.Queue()
        self._in_use: int = 0
        self._lock = asyncio.Lock()
        self._cleanup_task: Optional[asyncio.Task] = None

    async def start(self) -> None:
        """Start pool."""
        if not self._cleanup_task:
            self._cleanup_task = asyncio.create_task(self._cleanup_loop())

    async def stop(self) -> None:
        """Stop pool and close all connections."""
        if self._cleanup_task:
            self._cleanup_task.cancel()
            try:
                await self._cleanup_task
            except asyncio.CancelledError:
                pass

        async with self._lock:
            self._connections.clear()
            while not self._available.empty():
                try:
                    self._available.get_nowait()
                except asyncio.QueueEmpty:
                    break

    async def acquire(self) -> Optional[Any]:
        """Acquire connection from pool.

        Returns:
            Connection object or None if timeout
        """
        try:
            conn_info = await asyncio.wait_for(
                self._available.get(),
                timeout=self.config.acquire_timeout_seconds
            )
            async with self._lock:
                self._in_use += 1
            return conn_info["connection"]
        except asyncio.TimeoutError:
            return None

    async def release(self, connection: Any) -> None:
        """Release connection back to pool.

        Args:
            connection: Connection to release
        """
        async with self._lock:
            self._in_use = max(0, self._in_use - 1)
            conn_info = {
                "connection": connection,
                "last_used": datetime.utcnow(),
            }
            await self._available.put(conn_info)

    async def add_connection(self, connection: Any) -> None:
        """Add connection to pool.

        Args:
            connection: Connection to add
        """
        async with self._lock:
            if len(self._connections) < self.config.max_size:
                conn_info = {
                    "connection": connection,
                    "created_at": datetime.utcnow(),
                    "last_used": datetime.utcnow(),
                }
                self._connections.append(conn_info)
                await self._available.put(conn_info)

    def size(self) -> int:
        """Get total pool size."""
        return len(self._connections)

    def available_count(self) -> int:
        """Get available connection count."""
        return self._available.qsize()

    def in_use_count(self) -> int:
        """Get in-use connection count."""
        return self._in_use

    async def _cleanup_loop(self) -> None:
        """Background cleanup of idle connections."""
        while True:
            try:
                await asyncio.sleep(60)
                await self._remove_idle_connections()
            except asyncio.CancelledError:
                break

    async def _remove_idle_connections(self) -> None:
        """Remove idle connections."""
        now = datetime.utcnow()
        idle_timeout = timedelta(seconds=self.config.idle_timeout_seconds)

        async with self._lock:
            self._connections = [
                conn for conn in self._connections
                if now - conn["last_used"] < idle_timeout
            ]

