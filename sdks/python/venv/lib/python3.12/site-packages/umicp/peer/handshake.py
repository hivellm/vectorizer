"""Handshake protocol implementation."""

from umicp.envelope import Envelope, EnvelopeBuilder
from umicp.types import OperationType


class HandshakeProtocol:
    """Handles peer handshake (HELLO → ACK)."""

    @staticmethod
    def create_hello(from_id: str, to_id: str) -> Envelope:
        """Create HELLO envelope."""
        return EnvelopeBuilder() \
            .from_id(from_id) \
            .to_id(to_id) \
            .operation(OperationType.CONTROL) \
            .capability("type", "HELLO") \
            .build()

    @staticmethod
    def create_ack(from_id: str, to_id: str, correlation_id: str) -> Envelope:
        """Create ACK envelope."""
        return EnvelopeBuilder() \
            .from_id(from_id) \
            .to_id(to_id) \
            .operation(OperationType.ACK) \
            .correlation_id(correlation_id) \
            .capability("type", "ACK") \
            .build()

    @staticmethod
    def is_hello(envelope: Envelope) -> bool:
        """Check if envelope is HELLO."""
        return (
            envelope.operation == OperationType.CONTROL
            and envelope.capabilities.get("type") == "HELLO"
        )

    @staticmethod
    def is_ack(envelope: Envelope) -> bool:
        """Check if envelope is ACK."""
        return envelope.operation == OperationType.ACK

