"""
UMICP Python Bindings
=====================

High-performance Python bindings for the Universal Matrix Inter-Communication Protocol (UMICP).

Example:
    >>> from umicp import Envelope, OperationType
    >>> envelope = Envelope(
    ...     from_id="client-001",
    ...     to_id="server-001",
    ...     operation=OperationType.DATA,
    ...     message_id="msg-12345"
    ... )
    >>> serialized = envelope.to_json()
"""

__version__ = "0.2.0"
__author__ = "HiveLLM AI Collaborative Team"
__license__ = "MIT"

# Core exports
from umicp.envelope import Envelope, EnvelopeBuilder
from umicp.matrix import Matrix, MatrixResult, DotProductResult, CosineSimilarityResult
from umicp.types import (
    OperationType,
    PayloadType,
    EncodingType,
    PayloadHint,
    ConnectionState,
    TransportStats,
)
from umicp.error import (
    UmicpError,
    ValidationError,
    SerializationError,
    TransportError,
    MatrixOperationError,
)

# Transport exports
from umicp.transport.websocket_client import WebSocketClient
from umicp.transport.websocket_server import WebSocketServer
from umicp.transport.http_client import HttpClient
from umicp.transport.http_server import HttpServer

# Peer exports
from umicp.peer.websocket_peer import WebSocketPeer
from umicp.peer.connection import PeerConnection
from umicp.peer.info import PeerInfo
from umicp.peer.handshake import HandshakeProtocol

# Advanced features
from umicp.events import EventEmitter, Event, EventType
from umicp.discovery import ServiceDiscovery, ServiceInfo
from umicp.tool_discovery import (
    DiscoverableService,
    OperationSchema,
    ServerInfo as ToolServerInfo,
    generate_operations_response,
    generate_schema_response,
    generate_server_info_response,
)
from umicp.pool import ConnectionPool, PoolConfig
from umicp.compression import Compression, CompressionType, CompressionError

__all__ = [
    # Version
    "__version__",
    # Core
    "Envelope",
    "EnvelopeBuilder",
    "Matrix",
    "MatrixResult",
    "DotProductResult",
    "CosineSimilarityResult",
    # Types
    "OperationType",
    "PayloadType",
    "EncodingType",
    "PayloadHint",
    "ConnectionState",
    "TransportStats",
    # Errors
    "UmicpError",
    "ValidationError",
    "SerializationError",
    "TransportError",
    "MatrixOperationError",
    # Transport
    "WebSocketClient",
    "WebSocketServer",
    "HttpClient",
    "HttpServer",
    # Peer
    "WebSocketPeer",
    "PeerConnection",
    "PeerInfo",
    "HandshakeProtocol",
    # Advanced
    "EventEmitter",
    "Event",
    "EventType",
    "ServiceDiscovery",
    "ServiceInfo",
    # Tool Discovery (v0.2.0)
    "DiscoverableService",
    "OperationSchema",
    "ToolServerInfo",
    "generate_operations_response",
    "generate_schema_response",
    "generate_server_info_response",
    # Pool
    "ConnectionPool",
    "PoolConfig",
    # Compression
    "Compression",
    "CompressionType",
    "CompressionError",
]

