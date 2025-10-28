"""UMICP type definitions."""

from enum import Enum
from typing import Dict, Any, Optional
from dataclasses import dataclass, field
from datetime import datetime


class OperationType(str, Enum):
    """Message operation types."""

    CONTROL = "control"
    DATA = "data"
    ACK = "ack"
    ERROR = "error"
    REQUEST = "request"
    RESPONSE = "response"


class PayloadType(str, Enum):
    """Payload data types."""

    VECTOR = "vector"
    TEXT = "text"
    METADATA = "metadata"
    BINARY = "binary"
    JSON = "json"
    MATRIX = "matrix"


class EncodingType(str, Enum):
    """Data encoding types."""

    FLOAT32 = "float32"
    FLOAT64 = "float64"
    INT32 = "int32"
    INT64 = "int64"
    UTF8 = "utf8"
    BASE64 = "base64"
    HEX = "hex"


class ConnectionState(str, Enum):
    """Connection state."""

    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    CONNECTED = "connected"
    RECONNECTING = "reconnecting"
    DISCONNECTING = "disconnecting"
    ERROR = "error"


@dataclass
class PayloadHint:
    """Payload metadata hints."""

    type: PayloadType
    size: Optional[int] = None
    encoding: Optional[EncodingType] = None
    count: Optional[int] = None
    compression: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        result = {"type": self.type.value}
        if self.size is not None:
            result["size"] = self.size
        if self.encoding is not None:
            result["encoding"] = self.encoding.value
        if self.count is not None:
            result["count"] = self.count
        if self.compression is not None:
            result["compression"] = self.compression
        return result

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "PayloadHint":
        """Create from dictionary."""
        return cls(
            type=PayloadType(data["type"]),
            size=data.get("size"),
            encoding=EncodingType(data["encoding"]) if "encoding" in data else None,
            count=data.get("count"),
            compression=data.get("compression"),
        )


@dataclass
class TransportStats:
    """Transport statistics."""

    messages_sent: int = 0
    messages_received: int = 0
    bytes_sent: int = 0
    bytes_received: int = 0
    errors: int = 0
    reconnections: int = 0
    connected_at: Optional[datetime] = None
    last_activity: Optional[datetime] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "messages_sent": self.messages_sent,
            "messages_received": self.messages_received,
            "bytes_sent": self.bytes_sent,
            "bytes_received": self.bytes_received,
            "errors": self.errors,
            "reconnections": self.reconnections,
            "connected_at": self.connected_at.isoformat() if self.connected_at else None,
            "last_activity": self.last_activity.isoformat() if self.last_activity else None,
        }

