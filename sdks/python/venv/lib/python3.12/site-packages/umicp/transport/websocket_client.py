"""WebSocket client implementation."""

import asyncio
from typing import Optional
import websockets
from websockets.client import WebSocketClientProtocol

from umicp.envelope import Envelope
from umicp.types import ConnectionState, TransportStats
from umicp.error import TransportError
from umicp.events import EventEmitter


class WebSocketClient:
    """Async WebSocket client."""

    def __init__(self, url: str, reconnect: bool = True) -> None:
        """Initialize WebSocket client.

        Args:
            url: WebSocket URL
            reconnect: Enable auto-reconnect
        """
        self.url = url
        self.reconnect_enabled = reconnect
        self.state = ConnectionState.DISCONNECTED
        self.stats = TransportStats()
        self.events = EventEmitter()
        self._ws: Optional[WebSocketClientProtocol] = None
        self._reconnect_task: Optional[asyncio.Task] = None

    async def connect(self) -> None:
        """Connect to WebSocket server."""
        try:
            self.state = ConnectionState.CONNECTING
            self._ws = await websockets.connect(self.url)
            self.state = ConnectionState.CONNECTED
        except Exception as e:
            self.state = ConnectionState.ERROR
            raise TransportError(f"Connection failed: {e}")

    async def disconnect(self) -> None:
        """Disconnect from server."""
        if self._ws:
            await self._ws.close()
            self._ws = None
        self.state = ConnectionState.DISCONNECTED

    async def send(self, envelope: Envelope) -> None:
        """Send envelope.

        Args:
            envelope: Envelope to send
        """
        if not self._ws:
            raise TransportError("Not connected")

        try:
            message = envelope.to_json()
            await self._ws.send(message)
            self.stats.messages_sent += 1
            self.stats.bytes_sent += len(message)
        except Exception as e:
            self.stats.errors += 1
            raise TransportError(f"Send failed: {e}")

    async def receive(self) -> Optional[Envelope]:
        """Receive envelope.

        Returns:
            Received envelope or None
        """
        if not self._ws:
            return None

        try:
            message = await self._ws.recv()
            self.stats.messages_received += 1
            self.stats.bytes_received += len(message)
            return Envelope.from_json(message)
        except Exception as e:
            self.stats.errors += 1
            return None

    def get_stats(self) -> TransportStats:
        """Get transport statistics."""
        return self.stats

