"""
Tests for validation utilities - equivalent to JS/TS validation.test.js.

This module contains comprehensive tests for all validation functions in the Python SDK,
mirroring the test structure and coverage of the JavaScript/TypeScript versions.
"""

import unittest
import sys
import os
import math
from datetime import datetime

# Add current directory to path
sys.path.append(os.path.dirname(__file__))

# Import validation functions
from utils.validation import (
    validate_non_empty_string,
    validate_positive_number,
    validate_non_negative_number,
    validate_number_range,
    validate_number_array,
    validate_boolean
)


class TestValidationUtilities(unittest.TestCase):
    """Tests for validation utilities - equivalent to JS/TS validation.test.js"""

    def test_validate_non_empty_string_should_validate(self):
        """Test validateNonEmptyString with valid input."""
        result = validate_non_empty_string("valid string")
        self.assertEqual(result, "valid string")

        result = validate_non_empty_string("  valid with spaces  ")
        self.assertEqual(result, "  valid with spaces  ")

    def test_validate_non_empty_string_empty_string(self):
        """Test validateNonEmptyString with empty string."""
        with self.assertRaises(ValueError) as cm:
            validate_non_empty_string("")
        self.assertIn("String cannot be empty", str(cm.exception))

    def test_validate_non_empty_string_whitespace_only(self):
        """Test validateNonEmptyString with whitespace-only string."""
        with self.assertRaises(ValueError) as cm:
            validate_non_empty_string("   ")
        self.assertIn("String cannot be empty", str(cm.exception))

        with self.assertRaises(ValueError) as cm:
            validate_non_empty_string("\t\n")
        self.assertIn("String cannot be empty", str(cm.exception))

    def test_validate_non_empty_string_non_string(self):
        """Test validateNonEmptyString with non-string value."""
        with self.assertRaises(ValueError) as cm:
            validate_non_empty_string(123)
        self.assertIn("Value must be a string", str(cm.exception))

        with self.assertRaises(ValueError) as cm:
            validate_non_empty_string(None)
        self.assertIn("Value must be a string", str(cm.exception))

    def test_validate_positive_number_should_validate(self):
        """Test validatePositiveNumber with valid input."""
        result = validate_positive_number(5)
        self.assertEqual(result, 5)

        result = validate_positive_number(3.14)
        self.assertEqual(result, 3.14)

        result = validate_positive_number(0.001)
        self.assertEqual(result, 0.001)

    def test_validate_positive_number_zero(self):
        """Test validatePositiveNumber with zero."""
        with self.assertRaises(ValueError) as cm:
            validate_positive_number(0)
        self.assertIn("Value must be positive", str(cm.exception))

    def test_validate_positive_number_negative(self):
        """Test validatePositiveNumber with negative number."""
        with self.assertRaises(ValueError) as cm:
            validate_positive_number(-5)
        self.assertIn("Value must be positive", str(cm.exception))

    def test_validate_non_negative_number_should_validate(self):
        """Test validateNonNegativeNumber with valid input."""
        result = validate_non_negative_number(5)
        self.assertEqual(result, 5)

        result = validate_non_negative_number(0)
        self.assertEqual(result, 0)

        result = validate_non_negative_number(3.14)
        self.assertEqual(result, 3.14)

    def test_validate_non_negative_number_negative(self):
        """Test validateNonNegativeNumber with negative number."""
        with self.assertRaises(ValueError) as cm:
            validate_non_negative_number(-5)
        self.assertIn("Value must be non-negative", str(cm.exception))

    def test_validate_number_range_should_validate(self):
        """Test validateNumberRange with valid input."""
        result = validate_number_range(5, min_val=0, max_val=10)
        self.assertEqual(result, 5)

        result = validate_number_range(0, min_val=0)
        self.assertEqual(result, 0)

        result = validate_number_range(10, max_val=10)
        self.assertEqual(result, 10)

        result = validate_number_range(3.14)
        self.assertEqual(result, 3.14)

    def test_validate_number_range_below_min(self):
        """Test validateNumberRange below minimum."""
        with self.assertRaises(ValueError) as cm:
            validate_number_range(5, min_val=10)
        self.assertIn("Value must be at least 10", str(cm.exception))

    def test_validate_number_range_above_max(self):
        """Test validateNumberRange above maximum."""
        with self.assertRaises(ValueError) as cm:
            validate_number_range(15, max_val=10)
        self.assertIn("Value must be at most 10", str(cm.exception))

    def test_validate_number_range_invalid_number(self):
        """Test validateNumberRange with invalid number."""
        with self.assertRaises(ValueError) as cm:
            validate_number_range("not a number", min_val=0, max_val=10)
        self.assertIn("Value must be a number", str(cm.exception))

    def test_validate_number_array_should_validate(self):
        """Test validateNumberArray with valid input."""
        result = validate_number_array([1, 2, 3])
        self.assertEqual(result, [1, 2, 3])

        result = validate_number_array([1.5, 2.7, 3.14])
        self.assertEqual(result, [1.5, 2.7, 3.14])

        result = validate_number_array([0, -1, 100])
        self.assertEqual(result, [0, -1, 100])

    def test_validate_number_array_empty(self):
        """Test validateNumberArray with empty array."""
        with self.assertRaises(ValueError) as cm:
            validate_number_array([])
        self.assertIn("Array cannot be empty", str(cm.exception))

    def test_validate_number_array_non_array(self):
        """Test validateNumberArray with non-array."""
        with self.assertRaises(ValueError) as cm:
            validate_number_array("not an array")
        self.assertIn("Value must be an array", str(cm.exception))

        with self.assertRaises(ValueError) as cm:
            validate_number_array(123)
        self.assertIn("Value must be an array", str(cm.exception))

    def test_validate_number_array_invalid_numbers(self):
        """Test validateNumberArray with invalid numbers."""
        with self.assertRaises(ValueError) as cm:
            validate_number_array([1, "invalid", 3])
        self.assertIn("must be a number", str(cm.exception))

    def test_validate_number_array_nan(self):
        """Test validateNumberArray with NaN values."""
        with self.assertRaises(ValueError) as cm:
            validate_number_array([1, float('nan'), 3])
        self.assertIn("cannot be NaN", str(cm.exception))

    def test_validate_boolean_should_validate(self):
        """Test validateBoolean with valid input."""
        result = validate_boolean(True)
        self.assertEqual(result, True)

        result = validate_boolean(False)
        self.assertEqual(result, False)

    def test_validate_boolean_non_boolean(self):
        """Test validateBoolean with non-boolean."""
        with self.assertRaises(ValueError) as cm:
            validate_boolean("true")
        self.assertIn("Value must be a boolean", str(cm.exception))

        with self.assertRaises(ValueError) as cm:
            validate_boolean(1)
        self.assertIn("Value must be a boolean", str(cm.exception))

        with self.assertRaises(ValueError) as cm:
            validate_boolean(None)
        self.assertIn("Value must be a boolean", str(cm.exception))


def run_validation_tests():
    """Run all validation tests."""
    print("Testing Python SDK Validation")
    print("=" * 60)

    # Create test suite
    test_suite = unittest.TestSuite()

    # Add test classes
    test_classes = [
        TestValidationUtilities,
    ]

    for test_class in test_classes:
        tests = unittest.TestLoader().loadTestsFromTestCase(test_class)
        test_suite.addTests(tests)

    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_validation_tests()

    print("=" * 60)
    if success:
        print("SUCCESS: All validation tests passed!")
        print("OK: Validation functions are working correctly!")
    else:
        print("FAILED: Some validation tests failed!")
        print("FIX: Check the errors above.")

    print("=" * 60)
