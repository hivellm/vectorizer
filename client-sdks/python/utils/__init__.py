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

__all__ = [
    'validate_non_empty_string',
    'validate_positive_number',
    'validate_non_negative_number',
    'validate_number_range',
    'validate_number_array',
    'validate_boolean'
]
