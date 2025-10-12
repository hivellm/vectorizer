"""
Utils package for the Hive Vectorizer SDK.

This package contains utility functions for validation, HTTP client, and other common operations.
"""

from .validation import (
    validate_non_empty_string,
    validate_positive_number,
    validate_non_negative_number,
    validate_number_range,
    validate_number_array,
    validate_boolean
)
from .http_client import HTTPClient
from .transport import TransportFactory, TransportProtocol, parse_connection_string

try:
    from .umicp_client import UMICPClient
    UMICP_AVAILABLE = True
except ImportError:
    UMICPClient = None
    UMICP_AVAILABLE = False

__all__ = [
    'validate_non_empty_string',
    'validate_positive_number',
    'validate_non_negative_number',
    'validate_number_range',
    'validate_number_array',
    'validate_boolean',
    'HTTPClient',
    'TransportFactory',
    'TransportProtocol',
    'parse_connection_string',
]

if UMICP_AVAILABLE:
    __all__.append('UMICPClient')
