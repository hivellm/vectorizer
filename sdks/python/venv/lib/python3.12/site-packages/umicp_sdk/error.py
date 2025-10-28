"""UMICP error types."""

from typing import Optional, Any


class UmicpError(Exception):
    """Base exception for UMICP errors."""

    def __init__(self, message: str, details: Optional[Any] = None) -> None:
        """Initialize error.

        Args:
            message: Error message
            details: Optional error details
        """
        super().__init__(message)
        self.message = message
        self.details = details

    def __str__(self) -> str:
        """String representation."""
        if self.details:
            return f"{self.message}: {self.details}"
        return self.message


class ValidationError(UmicpError):
    """Validation error."""
    pass


class SerializationError(UmicpError):
    """Serialization/deserialization error."""
    pass


class TransportError(UmicpError):
    """Transport layer error."""
    pass


class MatrixOperationError(UmicpError):
    """Matrix operation error."""
    pass


class ConnectionError(UmicpError):
    """Connection error."""
    pass


class TimeoutError(UmicpError):
    """Timeout error."""
    pass

