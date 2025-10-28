"""
UMICO Python SDK
================

High-performance Python SDK for the Universal Matrix Inter-Communication Protocol (UMICO).

Example:
    >>> from umicp_sdk import Envelope, OperationType
    >>> envelope = Envelope(
    ...     from_id="client-001",
    ...     to_id="server-001",
    ...     operation=OperationType.DATA,
    ...     message_id="msg-12345"
    ... )
    >>> serialized = envelope.to_json()
"""

__version__ = "0.3.0"
__author__ = "HiveLLM AI Collaborative Team"
__license__ = "MIT"

# Core exports
from umicp_sdk.envelope import Envelope, EnvelopeBuilder
from umicp_sdk.matrix import Matrix, MatrixResult, DotProductResult, CosineSimilarityResult
from umicp_sdk.types import (
    OperationType,
    PayloadType,
    EncodingType,
    PayloadHint,
    ConnectionState,
    TransportStats,
)
from umicp_sdk.error import (
    UmicpError,
    ValidationError,
    SerializationError,
    TransportError,
    MatrixOperationError,
)

# Transport exports
from umicp_sdk.transport.websocket_client import WebSocketClient
from umicp_sdk.transport.websocket_server import WebSocketServer
from umicp_sdk.transport.http_client import HttpClient
from umicp_sdk.transport.http_server import HttpServer

# Peer exports
from umicp_sdk.peer.websocket_peer import WebSocketPeer
from umicp_sdk.peer.connection import PeerConnection
from umicp_sdk.peer.info import PeerInfo
from umicp_sdk.peer.handshake import HandshakeProtocol

# Advanced features
from umicp_sdk.events import EventEmitter, Event, EventType
from umicp_sdk.discovery import ServiceDiscovery, ServiceInfo
from umicp_sdk.tool_discovery import (
    DiscoverableService,
    OperationSchema,
    ServerInfo as ToolServerInfo,
    generate_operations_response,
    generate_schema_response,
    generate_server_info_response,
)
from umicp_sdk.pool import ConnectionPool, PoolConfig
from umicp_sdk.compression import Compression, CompressionType, CompressionError

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

