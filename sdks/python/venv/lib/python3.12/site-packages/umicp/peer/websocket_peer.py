"""Multiplexed WebSocket peer implementation."""

from typing import Dict, List, Optional
from umicp.transport.websocket_server import WebSocketServer
from umicp.transport.websocket_client import WebSocketClient
from umicp.envelope import Envelope
from umicp.events import EventEmitter
from umicp.peer.info import PeerInfo


class WebSocketPeer:
    """Multiplexed peer (server + multiple clients)."""

    def __init__(self, peer_id: str, port: int = 0, host: str = "0.0.0.0") -> None:
        """Initialize peer.

        Args:
            peer_id: Unique peer identifier
            port: Server port (0 for random)
            host: Server host
        """
        self.peer_id = peer_id
        self.server = WebSocketServer(host, port) if port else None
        self.clients: Dict[str, WebSocketClient] = {}
        self.peers: Dict[str, PeerInfo] = {}
        self.events = EventEmitter()

    async def start(self) -> None:
        """Start peer server."""
        if self.server:
            await self.server.start()

    async def connect_to_peer(self, url: str, peer_id: Optional[str] = None) -> None:
        """Connect to remote peer."""
        client = WebSocketClient(url)
        await client.connect()
        if peer_id:
            self.clients[peer_id] = client

    async def send_to_peer(self, peer_id: str, envelope: Envelope) -> None:
        """Send message to specific peer."""
        if peer_id in self.clients:
            await self.clients[peer_id].send(envelope)

    async def broadcast(self, envelope: Envelope) -> None:
        """Broadcast to all connected peers."""
        for client in self.clients.values():
            await client.send(envelope)

    async def disconnect(self) -> None:
        """Disconnect from all peers."""
        for client in self.clients.values():
            await client.disconnect()
        if self.server:
            await self.server.stop()

