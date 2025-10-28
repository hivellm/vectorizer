"""Peer connection management."""

from typing import Optional
from datetime import datetime
from umicp.types import ConnectionState


class PeerConnection:
    """Represents a peer connection."""

    def __init__(self, peer_id: str, url: str) -> None:
        """Initialize connection."""
        self.peer_id = peer_id
        self.url = url
        self.state = ConnectionState.DISCONNECTED
        self.connected_at: Optional[datetime] = None
        self.last_activity: Optional[datetime] = None

    def is_connected(self) -> bool:
        """Check if connected."""
        return self.state == ConnectionState.CONNECTED

