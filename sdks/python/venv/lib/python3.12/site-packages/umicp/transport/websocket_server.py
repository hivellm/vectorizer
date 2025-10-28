"""WebSocket server implementation."""

import asyncio
import json
import uuid
from typing import Dict, Set, Optional, Callable, Awaitable
import websockets
from websockets.server import WebSocketServerProtocol
from datetime import datetime

from umicp.envelope import Envelope
from umicp.types import TransportStats
from umicp.error import TransportError
from umicp.events import EventEmitter


class ClientConnection:
    """Client connection info."""

    def __init__(self, websocket: WebSocketServerProtocol, client_id: str):
        """Initialize client connection."""
        self.websocket = websocket
        self.client_id = client_id
        self.connected_at = datetime.now()
        self.metadata: Dict[str, str] = {}


class WebSocketServer:
    """Async WebSocket server with feature parity to client."""

    def __init__(
        self,
        host: str = "0.0.0.0",
        port: int = 8080,
        ping_interval: float = 30.0,
        ping_timeout: float = 10.0,
        max_payload: int = 100 * 1024 * 1024,  # 100MB
        compression: Optional[str] = None,
    ) -> None:
        """Initialize WebSocket server.

        Args:
            host: Host to bind
            port: Port to bind
            ping_interval: Interval for ping messages (seconds)
            ping_timeout: Timeout for pong response (seconds)
            max_payload: Maximum message size in bytes
            compression: Compression method ('deflate' or None)
        """
        self.host = host
        self.port = port
        self.ping_interval = ping_interval
        self.ping_timeout = ping_timeout
        self.max_payload = max_payload
        self.compression = compression
        self.stats = TransportStats()
        self.events = EventEmitter()

        self._clients: Dict[str, ClientConnection] = {}
        self._server: Optional[asyncio.AbstractServer] = None
        self._message_handler: Optional[Callable[[Envelope, str], Awaitable[None]]] = None
        self._connection_handler: Optional[Callable[[str], Awaitable[None]]] = None
        self._disconnection_handler: Optional[Callable[[str], Awaitable[None]]] = None

    async def start(self) -> None:
        """Start server."""
        compression_kwargs = {}
        if self.compression == "deflate":
            compression_kwargs["compression"] = "deflate"

        self._server = await websockets.serve(
            self._handle_client,
            self.host,
            self.port,
            ping_interval=self.ping_interval,
            ping_timeout=self.ping_timeout,
            max_size=self.max_payload,
            **compression_kwargs
        )
        self.events.emit("server_started", {"host": self.host, "port": self.port})

    async def stop(self) -> None:
        """Stop server."""
        if self._server:
            self._server.close()
            await self._server.wait_closed()

            # Close all client connections
            for client in list(self._clients.values()):
                await client.websocket.close()

            self.events.emit("server_stopped")

    def set_message_handler(
        self, handler: Callable[[Envelope, str], Awaitable[None]]
    ) -> None:
        """Set message handler callback.

        Args:
            handler: Async function(envelope, client_id) to handle messages
        """
        self._message_handler = handler

    def set_connection_handler(
        self, handler: Callable[[str], Awaitable[None]]
    ) -> None:
        """Set connection handler callback.

        Args:
            handler: Async function(client_id) called on new connection
        """
        self._connection_handler = handler

    def set_disconnection_handler(
        self, handler: Callable[[str], Awaitable[None]]
    ) -> None:
        """Set disconnection handler callback.

        Args:
            handler: Async function(client_id) called on disconnection
        """
        self._disconnection_handler = handler

    async def _handle_client(self, websocket: WebSocketServerProtocol) -> None:
        """Handle client connection."""
        client_id = str(uuid.uuid4())
        client = ClientConnection(websocket, client_id)
        self._clients[client_id] = client
        self.stats.active_connections += 1
        self.stats.total_connections += 1

        self.events.emit("client_connected", {"client_id": client_id})

        if self._connection_handler:
            try:
                await self._connection_handler(client_id)
            except Exception as e:
                self.events.emit("error", {"error": str(e), "client_id": client_id})

        try:
            async for message in websocket:
                self.stats.messages_received += 1
                self.stats.bytes_received += len(message) if isinstance(message, (str, bytes)) else 0

                try:
                    # Parse envelope
                    if isinstance(message, bytes):
                        message = message.decode('utf-8')

                    data = json.loads(message)
                    envelope = Envelope.from_dict(data)

                    # Call message handler
                    if self._message_handler:
                        await self._message_handler(envelope, client_id)

                    self.events.emit("message_received", {
                        "client_id": client_id,
                        "envelope": envelope
                    })

                except json.JSONDecodeError as e:
                    self.events.emit("error", {
                        "error": f"Invalid JSON: {e}",
                        "client_id": client_id
                    })
                except Exception as e:
                    self.events.emit("error", {
                        "error": str(e),
                        "client_id": client_id
                    })

        except websockets.exceptions.ConnectionClosed:
            pass
        except Exception as e:
            self.events.emit("error", {"error": str(e), "client_id": client_id})
        finally:
            # Cleanup
            self._clients.pop(client_id, None)
            self.stats.active_connections -= 1

            self.events.emit("client_disconnected", {"client_id": client_id})

            if self._disconnection_handler:
                try:
                    await self._disconnection_handler(client_id)
                except Exception as e:
                    self.events.emit("error", {"error": str(e), "client_id": client_id})

    async def send_to(self, client_id: str, envelope: Envelope) -> None:
        """Send envelope to specific client.

        Args:
            client_id: Target client ID
            envelope: Envelope to send

        Raises:
            TransportError: If client not found or send fails
        """
        client = self._clients.get(client_id)
        if not client:
            raise TransportError(f"Client {client_id} not found")

        try:
            message = envelope.to_json()
            await client.websocket.send(message)
            self.stats.messages_sent += 1
            self.stats.bytes_sent += len(message)
        except Exception as e:
            raise TransportError(f"Failed to send to client {client_id}: {e}")

    async def broadcast(self, envelope: Envelope) -> int:
        """Broadcast envelope to all clients.

        Args:
            envelope: Envelope to broadcast

        Returns:
            Number of clients message was sent to
        """
        message = envelope.to_json()
        results = await asyncio.gather(
            *[client.websocket.send(message) for client in self._clients.values()],
            return_exceptions=True
        )

        success_count = sum(1 for r in results if not isinstance(r, Exception))
        self.stats.messages_sent += success_count
        self.stats.bytes_sent += len(message) * success_count

        return success_count

    def get_client_ids(self) -> list[str]:
        """Get list of connected client IDs."""
        return list(self._clients.keys())

    def get_client_count(self) -> int:
        """Get number of connected clients."""
        return len(self._clients)

    def get_stats(self) -> TransportStats:
        """Get server statistics."""
        return self.stats

