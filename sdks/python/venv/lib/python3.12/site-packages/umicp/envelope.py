"""UMICP Envelope implementation."""

import json
import hashlib
from typing import Dict, Any, Optional, List
from datetime import datetime
from uuid import uuid4

from umicp.types import OperationType, PayloadHint
from umicp.error import ValidationError, SerializationError


class Envelope:
    """UMICP message envelope."""

    def __init__(
        self,
        from_id: str,
        to_id: str,
        operation: OperationType,
        message_id: Optional[str] = None,
        correlation_id: Optional[str] = None,
        timestamp: Optional[datetime] = None,
        capabilities: Optional[Dict[str, Any]] = None,
        payload_hints: Optional[List[PayloadHint]] = None,
        payload_refs: Optional[List[str]] = None,
        hash_value: Optional[str] = None,
    ):
        """Initialize envelope.

        Args:
            from_id: Sender identifier
            to_id: Recipient identifier
            operation: Operation type
            message_id: Unique message ID (generated if not provided)
            correlation_id: Correlation ID for request/response pairs
            timestamp: Message timestamp (current time if not provided)
            capabilities: Metadata capabilities
            payload_hints: List of payload hints
            payload_refs: List of payload references
            hash_value: Envelope hash
        """
        self.from_id = from_id
        self.to_id = to_id
        self.operation = operation
        self.message_id = message_id or str(uuid4())
        self.correlation_id = correlation_id
        self.timestamp = timestamp or datetime.utcnow()
        self.capabilities = capabilities or {}
        self.payload_hints = payload_hints or []
        self.payload_refs = payload_refs or []
        self.hash_value = hash_value

        self.validate()

    def validate(self) -> None:
        """Validate envelope fields.

        Raises:
            ValidationError: If validation fails
        """
        if not self.from_id:
            raise ValidationError("from_id is required")
        if not self.to_id:
            raise ValidationError("to_id is required")
        if not self.message_id:
            raise ValidationError("message_id is required")
        if not isinstance(self.operation, OperationType):
            raise ValidationError(f"Invalid operation type: {self.operation}")

    def compute_hash(self) -> str:
        """Compute SHA-256 hash of envelope.

        Returns:
            Hexadecimal hash string
        """
        # Create canonical JSON without hash field
        data = self.to_dict()
        data.pop("hash", None)
        canonical = json.dumps(data, sort_keys=True, separators=(",", ":"))
        return hashlib.sha256(canonical.encode()).hexdigest()

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary.

        Returns:
            Dictionary representation
        """
        result = {
            "from": self.from_id,
            "to": self.to_id,
            "operation": self.operation.value,
            "message_id": self.message_id,
            "timestamp": self.timestamp.isoformat() + "Z",
        }

        if self.correlation_id:
            result["correlation_id"] = self.correlation_id

        if self.capabilities:
            result["capabilities"] = self.capabilities

        if self.payload_hints:
            result["payload_hints"] = [hint.to_dict() for hint in self.payload_hints]

        if self.payload_refs:
            result["payload_refs"] = self.payload_refs

        if self.hash_value:
            result["hash"] = self.hash_value

        return result

    def to_json(self) -> str:
        """Serialize to JSON string.

        Returns:
            JSON string

        Raises:
            SerializationError: If serialization fails
        """
        try:
            return json.dumps(self.to_dict())
        except Exception as e:
            raise SerializationError(f"Failed to serialize envelope: {e}")

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Envelope":
        """Create envelope from dictionary.

        Args:
            data: Dictionary representation

        Returns:
            Envelope instance

        Raises:
            SerializationError: If deserialization fails
        """
        try:
            # Parse timestamp
            timestamp_str = data.get("timestamp", "")
            timestamp = datetime.fromisoformat(timestamp_str.replace("Z", "+00:00"))

            # Parse payload hints
            payload_hints = None
            if "payload_hints" in data:
                payload_hints = [
                    PayloadHint.from_dict(hint) for hint in data["payload_hints"]
                ]

            return cls(
                from_id=data["from"],
                to_id=data["to"],
                operation=OperationType(data["operation"]),
                message_id=data.get("message_id"),
                correlation_id=data.get("correlation_id"),
                timestamp=timestamp,
                capabilities=data.get("capabilities"),
                payload_hints=payload_hints,
                payload_refs=data.get("payload_refs"),
                hash_value=data.get("hash"),
            )
        except Exception as e:
            raise SerializationError(f"Failed to deserialize envelope: {e}")

    @classmethod
    def from_json(cls, json_str: str) -> "Envelope":
        """Create envelope from JSON string.

        Args:
            json_str: JSON string

        Returns:
            Envelope instance

        Raises:
            SerializationError: If deserialization fails
        """
        try:
            data = json.loads(json_str)
            return cls.from_dict(data)
        except json.JSONDecodeError as e:
            raise SerializationError(f"Invalid JSON: {e}")

    def __repr__(self) -> str:
        """String representation."""
        return (
            f"Envelope(from={self.from_id}, to={self.to_id}, "
            f"operation={self.operation.value}, message_id={self.message_id})"
        )


class EnvelopeBuilder:
    """Builder for creating envelopes."""

    def __init__(self) -> None:
        """Initialize builder."""
        self._from_id: Optional[str] = None
        self._to_id: Optional[str] = None
        self._operation: Optional[OperationType] = None
        self._message_id: Optional[str] = None
        self._correlation_id: Optional[str] = None
        self._timestamp: Optional[datetime] = None
        self._capabilities: Dict[str, Any] = {}
        self._payload_hints: List[PayloadHint] = []
        self._payload_refs: List[str] = []

    def from_id(self, from_id: str) -> "EnvelopeBuilder":
        """Set sender ID."""
        self._from_id = from_id
        return self

    def to_id(self, to_id: str) -> "EnvelopeBuilder":
        """Set recipient ID."""
        self._to_id = to_id
        return self

    def operation(self, operation: OperationType) -> "EnvelopeBuilder":
        """Set operation type."""
        self._operation = operation
        return self

    def message_id(self, message_id: str) -> "EnvelopeBuilder":
        """Set message ID."""
        self._message_id = message_id
        return self

    def correlation_id(self, correlation_id: str) -> "EnvelopeBuilder":
        """Set correlation ID."""
        self._correlation_id = correlation_id
        return self

    def timestamp(self, timestamp: datetime) -> "EnvelopeBuilder":
        """Set timestamp."""
        self._timestamp = timestamp
        return self

    def capability(self, key: str, value: Any) -> "EnvelopeBuilder":
        """Add capability."""
        self._capabilities[key] = value
        return self

    def payload_hint(self, hint: PayloadHint) -> "EnvelopeBuilder":
        """Add payload hint."""
        self._payload_hints.append(hint)
        return self

    def payload_ref(self, ref: str) -> "EnvelopeBuilder":
        """Add payload reference."""
        self._payload_refs.append(ref)
        return self

    def build(self) -> Envelope:
        """Build envelope.

        Returns:
            Envelope instance

        Raises:
            ValidationError: If required fields are missing
        """
        if not self._from_id:
            raise ValidationError("from_id is required")
        if not self._to_id:
            raise ValidationError("to_id is required")
        if not self._operation:
            raise ValidationError("operation is required")

        envelope = Envelope(
            from_id=self._from_id,
            to_id=self._to_id,
            operation=self._operation,
            message_id=self._message_id,
            correlation_id=self._correlation_id,
            timestamp=self._timestamp,
            capabilities=self._capabilities if self._capabilities else None,
            payload_hints=self._payload_hints if self._payload_hints else None,
            payload_refs=self._payload_refs if self._payload_refs else None,
        )

        # Compute hash
        envelope.hash_value = envelope.compute_hash()

        return envelope

