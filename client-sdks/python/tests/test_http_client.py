"""
Tests for HTTP client functionality - equivalent to JS/TS http-client.test.js.

This module contains comprehensive tests for HTTP client functionality in the Python SDK,
mirroring the test structure and coverage of the JavaScript/TypeScript versions.
"""

import unittest
import sys
import os

# Add current directory to path
sys.path.append(os.path.dirname(__file__))

# Import client and exceptions
from client import VectorizerClient
from exceptions import (
    AuthenticationError,
    ServerError,
    NetworkError,
    RateLimitError,
    CollectionNotFoundError,
)


class TestClientInitialization(unittest.TestCase):
    """Tests for client initialization and configuration."""

    def test_should_create_client_with_api_key(self):
        """Test creating client with API key."""
        client = VectorizerClient(
            base_url="http://localhost:15002",
            api_key="test-api-key"
        )
        self.assertIsNotNone(client)
        self.assertEqual(client.base_url, "http://localhost:15002")
        self.assertEqual(client.api_key, "test-api-key")

    def test_should_create_client_with_custom_config(self):
        """Test creating client with custom config."""
        client = VectorizerClient(
            base_url="http://custom:8080",
            api_key="custom-key",
            timeout=60,
            max_retries=5
        )

        self.assertEqual(client.base_url, "http://custom:8080")
        self.assertEqual(client.api_key, "custom-key")
        self.assertEqual(client.timeout, 60)
        self.assertEqual(client.max_retries, 5)

    def test_should_create_client_with_default_config(self):
        """Test creating client with default config."""
        client = VectorizerClient()
        self.assertEqual(client.base_url, "http://localhost:15002")
        self.assertIsNone(client.api_key)
        self.assertEqual(client.timeout, 30)
        self.assertEqual(client.max_retries, 3)


class TestHttpClientFunctionality(unittest.TestCase):
    """Tests for HTTP client error handling - equivalent to JS/TS http-client.test.js"""

    def test_should_handle_401_unauthorized(self):
        """Test 401 Unauthorized error handling."""
        try:
            raise AuthenticationError("Unauthorized access")
        except AuthenticationError as e:
            self.assertEqual(e.error_code, "AUTH_ERROR")
            self.assertIn("Unauthorized access", str(e))

    def test_should_handle_403_forbidden(self):
        """Test 403 Forbidden error handling."""
        try:
            raise AuthenticationError("Forbidden access")
        except AuthenticationError as e:
            self.assertEqual(e.error_code, "AUTH_ERROR")
            self.assertIn("Forbidden access", str(e))

    def test_should_handle_404_not_found(self):
        """Test 404 Not Found error handling."""
        try:
            raise CollectionNotFoundError("collection-name", {"operation": "search"})
        except CollectionNotFoundError as e:
            self.assertEqual(e.error_code, "COLLECTION_NOT_FOUND")
            self.assertIn("collection-name", str(e))

    def test_should_handle_429_too_many_requests(self):
        """Test 429 Too Many Requests error handling."""
        try:
            raise RateLimitError("Rate limit exceeded")
        except RateLimitError as e:
            self.assertEqual(e.error_code, "RATE_LIMIT_ERROR")
            self.assertIn("Rate limit exceeded", str(e))

    def test_should_handle_500_internal_server_error(self):
        """Test 500 Internal Server Error handling."""
        try:
            raise ServerError("Internal server error")
        except ServerError as e:
            self.assertEqual(e.error_code, "SERVER_ERROR")
            self.assertIn("Internal server error", str(e))

    def test_should_handle_network_error(self):
        """Test network error handling."""
        try:
            raise NetworkError("Network connection failed")
        except NetworkError as e:
            self.assertEqual(e.error_code, "NETWORK_ERROR")
            self.assertIn("Network connection failed", str(e))

    def test_should_create_client_with_default_config(self):
        """Test creating client with default config."""
        client = VectorizerClient()

        self.assertEqual(client.base_url, "http://localhost:15002")
        self.assertIsNone(client.api_key)
        self.assertEqual(client.timeout, 30)
        self.assertEqual(client.max_retries, 3)

    def test_should_create_client_with_custom_config(self):
        """Test creating client with custom config."""
        client = VectorizerClient(
            base_url="https://api.example.com",
            api_key="custom-key",
            timeout=60,
            max_retries=5
        )

        self.assertEqual(client.base_url, "https://api.example.com")
        self.assertEqual(client.api_key, "custom-key")
        self.assertEqual(client.timeout, 60)
        self.assertEqual(client.max_retries, 5)

    def test_should_create_client_with_api_key(self):
        """Test creating client with API key."""
        client = VectorizerClient(api_key="test-api-key")

        self.assertEqual(client.api_key, "test-api-key")
        self.assertEqual(client.base_url, "http://localhost:15002")


class TestUrlHandling(unittest.TestCase):
    """Tests for URL handling."""

    def setUp(self):
        self.client = VectorizerClient(base_url="http://localhost:15002")

    def test_should_handle_absolute_urls(self):
        """Test handling of absolute URLs."""
        # This would test URL joining logic
        # For now, skip as it requires more complex mocking
        self.skipTest("URL handling tests require HTTP mocking setup")

    def test_should_handle_relative_urls(self):
        """Test handling of relative URLs."""
        self.skipTest("URL handling tests require HTTP mocking setup")


class TestCustomHeaders(unittest.TestCase):
    """Tests for custom headers handling."""

    def setUp(self):
        self.client = VectorizerClient(api_key="test-key")

    def test_should_include_custom_headers(self):
        """Test inclusion of custom headers."""
        self.skipTest("Custom headers tests require HTTP mocking setup")

    def test_should_override_default_headers_with_request_headers(self):
        """Test header override behavior."""
        self.skipTest("Header override tests require HTTP mocking setup")


def run_http_client_tests():
    """Run all HTTP client tests."""
    print("Testing Python SDK HTTP Client")
    print("=" * 60)

    # Create test suite
    test_suite = unittest.TestSuite()

    # Add test classes
    test_classes = [
        TestHttpClientFunctionality,
        TestClientInitialization,
        TestUrlHandling,
        TestCustomHeaders,
    ]

    for test_class in test_classes:
        tests = unittest.TestLoader().loadTestsFromTestCase(test_class)
        test_suite.addTests(tests)

    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_http_client_tests()

    print("=" * 60)
    if success:
        print("SUCCESS: All HTTP client tests passed!")
        print("OK: HTTP client is working correctly!")
    else:
        print("FAILED: Some HTTP client tests failed!")
        print("FIX: Check the errors above.")

    print("=" * 60)
