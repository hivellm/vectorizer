"""
Validation utilities for the Hive Vectorizer SDK.

This module provides validation functions for various data types and constraints,
mirroring the validation functionality of the JavaScript/TypeScript SDKs.
"""

import math
from typing import Any, List, Union


def validate_non_empty_string(value: Any) -> str:
    """
    Validate that a value is a non-empty string.

    Args:
        value: The value to validate

    Returns:
        The validated string

    Raises:
        ValueError: If the value is not a string or is empty/whitespace-only
    """
    if not isinstance(value, str):
        raise ValueError("Value must be a string")

    if not value.strip():
        raise ValueError("String cannot be empty or whitespace-only")

    return value


def validate_positive_number(value: Any) -> Union[int, float]:
    """
    Validate that a value is a positive number.

    Args:
        value: The value to validate

    Returns:
        The validated number

    Raises:
        ValueError: If the value is not a number or is not positive
    """
    if not isinstance(value, (int, float)):
        raise ValueError("Value must be a number")

    if math.isnan(value) or math.isinf(value):
        raise ValueError("Value cannot be NaN or Infinity")

    if value <= 0:
        raise ValueError("Value must be positive")

    return value


def validate_non_negative_number(value: Any) -> Union[int, float]:
    """
    Validate that a value is a non-negative number.

    Args:
        value: The value to validate

    Returns:
        The validated number

    Raises:
        ValueError: If the value is not a number or is negative
    """
    if not isinstance(value, (int, float)):
        raise ValueError("Value must be a number")

    if math.isnan(value) or math.isinf(value):
        raise ValueError("Value cannot be NaN or Infinity")

    if value < 0:
        raise ValueError("Value must be non-negative")

    return value


def validate_number_range(value: Any, min_val: Union[int, float] = None, max_val: Union[int, float] = None) -> Union[int, float]:
    """
    Validate that a value is a number within a specified range.

    Args:
        value: The value to validate
        min_val: Minimum allowed value (optional)
        max_val: Maximum allowed value (optional)

    Returns:
        The validated number

    Raises:
        ValueError: If the value is not a number or outside the allowed range
    """
    if not isinstance(value, (int, float)):
        raise ValueError("Value must be a number")

    if math.isnan(value) or math.isinf(value):
        raise ValueError("Value cannot be NaN or Infinity")

    if min_val is not None and value < min_val:
        raise ValueError(f"Value must be at least {min_val}")

    if max_val is not None and value > max_val:
        raise ValueError(f"Value must be at most {max_val}")

    return value


def validate_number_array(value: Any) -> List[Union[int, float]]:
    """
    Validate that a value is an array of finite numbers.

    Args:
        value: The value to validate

    Returns:
        The validated array of numbers

    Raises:
        ValueError: If the value is not an array or contains invalid numbers
    """
    if not isinstance(value, list):
        raise ValueError("Value must be an array")

    if not value:
        raise ValueError("Array cannot be empty")

    for i, item in enumerate(value):
        if not isinstance(item, (int, float)):
            raise ValueError(f"Array item at index {i} must be a number")

        if math.isnan(item):
            raise ValueError(f"Array item at index {i} cannot be NaN")

        if math.isinf(item):
            raise ValueError(f"Array item at index {i} cannot be Infinity")

    return value


def validate_boolean(value: Any) -> bool:
    """
    Validate that a value is a boolean.

    Args:
        value: The value to validate

    Returns:
        The validated boolean

    Raises:
        ValueError: If the value is not a boolean
    """
    if not isinstance(value, bool):
        raise ValueError("Value must be a boolean")

    return value

