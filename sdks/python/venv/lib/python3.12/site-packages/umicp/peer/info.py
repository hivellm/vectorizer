"""Peer information."""

from typing import Dict, Any
from dataclasses import dataclass


@dataclass
class PeerInfo:
    """Information about a peer."""

    id: str
    url: str
    metadata: Dict[str, Any]
    capabilities: Dict[str, Any]

