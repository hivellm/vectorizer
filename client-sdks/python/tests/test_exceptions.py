"""
Tests for exception classes - equivalent to JS/TS vectorizer-error.test.js.

This module contains comprehensive tests for all exception classes in the Python SDK,
mirroring the test structure and coverage of the JavaScript/TypeScript versions.
"""

import unittest
import sys
import os

# Add current directory to path
sys.path.append(os.path.dirname(__file__))

# Import exception classes
from exceptions import (
    VectorizerError,
    AuthenticationError,
    CollectionNotFoundError,
    ValidationError,
    NetworkError,
    ServerError,
    TimeoutError,
    RateLimitError,
    ConfigurationError,
    EmbeddingError,
    IndexingError,
    VectorNotFoundError,
    BatchOperationError,
)


class TestVectorizerError(unittest.TestCase):
    """Tests for VectorizerError base class."""

    def test_should_create_error_with_message_only(self):
        """Test creating error with message only."""
        error = VectorizerError('Test error')

        self.assertEqual(error.message, 'Test error')
        self.assertEqual(error.name, 'VectorizerError')
        self.assertIsNone(error.error_code)
        self.assertEqual(error.details, {})

    def test_should_create_error_with_message_and_error_code(self):
        """Test creating error with message and error code."""
        error = VectorizerError('Test error', 'TEST_ERROR')

        self.assertEqual(error.message, 'Test error')
        self.assertEqual(error.name, 'VectorizerError')
        self.assertEqual(error.error_code, 'TEST_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_error_with_message_code_and_details(self):
        """Test creating error with message, code, and details."""
        details = {'field': 'test', 'value': 123}
        error = VectorizerError('Test error', 'TEST_ERROR', details)

        self.assertEqual(error.message, 'Test error')
        self.assertEqual(error.name, 'VectorizerError')
        self.assertEqual(error.error_code, 'TEST_ERROR')
        self.assertEqual(error.details, details)

    def test_should_return_correct_string_representation_with_error_code(self):
        """Test string representation with error code."""
        error = VectorizerError('Test error', 'TEST_ERROR')

        self.assertEqual(str(error), '[TEST_ERROR] Test error')

    def test_should_return_correct_string_representation_without_error_code(self):
        """Test string representation without error code."""
        error = VectorizerError('Test error')

        self.assertEqual(str(error), 'Test error')

    def test_should_be_instance_of_error(self):
        """Test that VectorizerError is instance of Exception."""
        error = VectorizerError('Test error')

        self.assertIsInstance(error, Exception)
        self.assertIsInstance(error, VectorizerError)


class TestAuthenticationError(unittest.TestCase):
    """Tests for AuthenticationError."""

    def test_should_create_authentication_error_with_default_message(self):
        """Test creating authentication error with default message."""
        error = AuthenticationError()

        self.assertEqual(error.message, 'Authentication failed')
        self.assertEqual(error.name, 'AuthenticationError')
        self.assertEqual(error.error_code, 'AUTH_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_authentication_error_with_custom_message(self):
        """Test creating authentication error with custom message."""
        error = AuthenticationError('Invalid API key')

        self.assertEqual(error.message, 'Invalid API key')
        self.assertEqual(error.name, 'AuthenticationError')
        self.assertEqual(error.error_code, 'AUTH_ERROR')

    def test_should_create_authentication_error_with_details(self):
        """Test creating authentication error with details."""
        details = {'apiKey': 'invalid-key'}
        error = AuthenticationError('Invalid API key', details)

        self.assertEqual(error.message, 'Invalid API key')
        self.assertEqual(error.details, details)

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that AuthenticationError is instance of VectorizerError."""
        error = AuthenticationError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, AuthenticationError)


class TestCollectionNotFoundError(unittest.TestCase):
    """Tests for CollectionNotFoundError."""

    def test_should_create_collection_not_found_error_with_default_message(self):
        """Test creating collection not found error with default message."""
        error = CollectionNotFoundError()

        self.assertEqual(error.message, 'Collection not found')
        self.assertEqual(error.name, 'CollectionNotFoundError')
        self.assertEqual(error.error_code, 'COLLECTION_NOT_FOUND')
        self.assertEqual(error.details, {})

    def test_should_create_collection_not_found_error_with_collection_name(self):
        """Test creating collection not found error with collection name."""
        error = CollectionNotFoundError('test-collection')

        self.assertEqual(error.message, "Collection 'test-collection' not found")
        self.assertEqual(error.name, 'CollectionNotFoundError')
        self.assertEqual(error.error_code, 'COLLECTION_NOT_FOUND')
        self.assertEqual(error.details, {'collectionName': 'test-collection'})

    def test_should_create_collection_not_found_error_with_details(self):
        """Test creating collection not found error with details."""
        details = {'operation': 'search'}
        error = CollectionNotFoundError('test-collection', details)

        expected_details = {'collectionName': 'test-collection', 'operation': 'search'}
        self.assertEqual(error.details, expected_details)

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that CollectionNotFoundError is instance of VectorizerError."""
        error = CollectionNotFoundError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, CollectionNotFoundError)


class TestValidationError(unittest.TestCase):
    """Tests for ValidationError."""

    def test_should_create_validation_error_with_default_message(self):
        """Test creating validation error with default message."""
        error = ValidationError()

        self.assertEqual(error.message, 'Validation failed')
        self.assertEqual(error.name, 'ValidationError')
        self.assertEqual(error.error_code, 'VALIDATION_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_validation_error_with_custom_message(self):
        """Test creating validation error with custom message."""
        error = ValidationError('Invalid input data')

        self.assertEqual(error.message, 'Invalid input data')
        self.assertEqual(error.name, 'ValidationError')
        self.assertEqual(error.error_code, 'VALIDATION_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that ValidationError is instance of VectorizerError."""
        error = ValidationError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, ValidationError)


class TestNetworkError(unittest.TestCase):
    """Tests for NetworkError."""

    def test_should_create_network_error_with_default_message(self):
        """Test creating network error with default message."""
        error = NetworkError()

        self.assertEqual(error.message, 'Network error occurred')
        self.assertEqual(error.name, 'NetworkError')
        self.assertEqual(error.error_code, 'NETWORK_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_network_error_with_custom_message(self):
        """Test creating network error with custom message."""
        error = NetworkError('Connection timeout')

        self.assertEqual(error.message, 'Connection timeout')
        self.assertEqual(error.name, 'NetworkError')
        self.assertEqual(error.error_code, 'NETWORK_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that NetworkError is instance of VectorizerError."""
        error = NetworkError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, NetworkError)


class TestServerError(unittest.TestCase):
    """Tests for ServerError."""

    def test_should_create_server_error_with_default_message(self):
        """Test creating server error with default message."""
        error = ServerError()

        self.assertEqual(error.message, 'Server error occurred')
        self.assertEqual(error.name, 'ServerError')
        self.assertEqual(error.error_code, 'SERVER_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_server_error_with_custom_message(self):
        """Test creating server error with custom message."""
        error = ServerError('Internal server error')

        self.assertEqual(error.message, 'Internal server error')
        self.assertEqual(error.name, 'ServerError')
        self.assertEqual(error.error_code, 'SERVER_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that ServerError is instance of VectorizerError."""
        error = ServerError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, ServerError)


class TestTimeoutError(unittest.TestCase):
    """Tests for TimeoutError."""

    def test_should_create_timeout_error_with_default_message(self):
        """Test creating timeout error with default message."""
        error = TimeoutError()

        self.assertEqual(error.message, 'Request timeout')
        self.assertEqual(error.name, 'TimeoutError')
        self.assertEqual(error.error_code, 'TIMEOUT_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_timeout_error_with_custom_message(self):
        """Test creating timeout error with custom message."""
        error = TimeoutError('Connection timed out')

        self.assertEqual(error.message, 'Connection timed out')
        self.assertEqual(error.name, 'TimeoutError')
        self.assertEqual(error.error_code, 'TIMEOUT_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that TimeoutError is instance of VectorizerError."""
        error = TimeoutError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, TimeoutError)


class TestRateLimitError(unittest.TestCase):
    """Tests for RateLimitError."""

    def test_should_create_rate_limit_error_with_default_message(self):
        """Test creating rate limit error with default message."""
        error = RateLimitError()

        self.assertEqual(error.message, 'Rate limit exceeded')
        self.assertEqual(error.name, 'RateLimitError')
        self.assertEqual(error.error_code, 'RATE_LIMIT_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_rate_limit_error_with_custom_message(self):
        """Test creating rate limit error with custom message."""
        error = RateLimitError('Too many requests')

        self.assertEqual(error.message, 'Too many requests')
        self.assertEqual(error.name, 'RateLimitError')
        self.assertEqual(error.error_code, 'RATE_LIMIT_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that RateLimitError is instance of VectorizerError."""
        error = RateLimitError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, RateLimitError)


class TestConfigurationError(unittest.TestCase):
    """Tests for ConfigurationError."""

    def test_should_create_configuration_error_with_default_message(self):
        """Test creating configuration error with default message."""
        error = ConfigurationError()

        self.assertEqual(error.message, 'Configuration error')
        self.assertEqual(error.name, 'ConfigurationError')
        self.assertEqual(error.error_code, 'CONFIGURATION_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_configuration_error_with_custom_message(self):
        """Test creating configuration error with custom message."""
        error = ConfigurationError('Invalid configuration')

        self.assertEqual(error.message, 'Invalid configuration')
        self.assertEqual(error.name, 'ConfigurationError')
        self.assertEqual(error.error_code, 'CONFIGURATION_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that ConfigurationError is instance of VectorizerError."""
        error = ConfigurationError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, ConfigurationError)


class TestEmbeddingError(unittest.TestCase):
    """Tests for EmbeddingError."""

    def test_should_create_embedding_error_with_default_message(self):
        """Test creating embedding error with default message."""
        error = EmbeddingError()

        self.assertEqual(error.message, 'Embedding generation failed')
        self.assertEqual(error.name, 'EmbeddingError')
        self.assertEqual(error.error_code, 'EMBEDDING_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_embedding_error_with_custom_message(self):
        """Test creating embedding error with custom message."""
        error = EmbeddingError('Embedding service unavailable')

        self.assertEqual(error.message, 'Embedding service unavailable')
        self.assertEqual(error.name, 'EmbeddingError')
        self.assertEqual(error.error_code, 'EMBEDDING_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that EmbeddingError is instance of VectorizerError."""
        error = EmbeddingError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, EmbeddingError)


class TestIndexingError(unittest.TestCase):
    """Tests for IndexingError."""

    def test_should_create_indexing_error_with_default_message(self):
        """Test creating indexing error with default message."""
        error = IndexingError()

        self.assertEqual(error.message, 'Indexing operation failed')
        self.assertEqual(error.name, 'IndexingError')
        self.assertEqual(error.error_code, 'INDEXING_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_indexing_error_with_custom_message(self):
        """Test creating indexing error with custom message."""
        error = IndexingError('Index creation failed')

        self.assertEqual(error.message, 'Index creation failed')
        self.assertEqual(error.name, 'IndexingError')
        self.assertEqual(error.error_code, 'INDEXING_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that IndexingError is instance of VectorizerError."""
        error = IndexingError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, IndexingError)


class TestVectorNotFoundError(unittest.TestCase):
    """Tests for VectorNotFoundError."""

    def test_should_create_vector_not_found_error_with_default_message(self):
        """Test creating vector not found error with default message."""
        error = VectorNotFoundError()

        self.assertEqual(error.message, 'Vector not found')
        self.assertEqual(error.name, 'VectorNotFoundError')
        self.assertEqual(error.error_code, 'VECTOR_NOT_FOUND')
        self.assertEqual(error.details, {})

    def test_should_create_vector_not_found_error_with_custom_message(self):
        """Test creating vector not found error with custom message."""
        error = VectorNotFoundError('Vector with ID 123 not found')

        self.assertEqual(error.message, 'Vector with ID 123 not found')
        self.assertEqual(error.name, 'VectorNotFoundError')
        self.assertEqual(error.error_code, 'VECTOR_NOT_FOUND')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that VectorNotFoundError is instance of VectorizerError."""
        error = VectorNotFoundError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, VectorNotFoundError)


class TestBatchOperationError(unittest.TestCase):
    """Tests for BatchOperationError."""

    def test_should_create_batch_operation_error_with_default_message(self):
        """Test creating batch operation error with default message."""
        error = BatchOperationError()

        self.assertEqual(error.message, 'Batch operation failed')
        self.assertEqual(error.name, 'BatchOperationError')
        self.assertEqual(error.error_code, 'BATCH_OPERATION_ERROR')
        self.assertEqual(error.details, {})

    def test_should_create_batch_operation_error_with_custom_message(self):
        """Test creating batch operation error with custom message."""
        error = BatchOperationError('Batch insert failed')

        self.assertEqual(error.message, 'Batch insert failed')
        self.assertEqual(error.name, 'BatchOperationError')
        self.assertEqual(error.error_code, 'BATCH_OPERATION_ERROR')

    def test_should_be_instance_of_vectorizer_error(self):
        """Test that BatchOperationError is instance of VectorizerError."""
        error = BatchOperationError()

        self.assertIsInstance(error, VectorizerError)
        self.assertIsInstance(error, BatchOperationError)


def run_exceptions_tests():
    """Run all exception tests."""
    print("Testing Python SDK Exceptions")
    print("=" * 60)

    # Create test suite
    test_suite = unittest.TestSuite()

    # Add test classes
    test_classes = [
        TestVectorizerError,
        TestAuthenticationError,
        TestCollectionNotFoundError,
        TestValidationError,
        TestNetworkError,
        TestServerError,
        TestTimeoutError,
        TestRateLimitError,
        TestConfigurationError,
        TestEmbeddingError,
        TestIndexingError,
        TestVectorNotFoundError,
        TestBatchOperationError,
    ]

    for test_class in test_classes:
        tests = unittest.TestLoader().loadTestsFromTestCase(test_class)
        test_suite.addTests(tests)

    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_exceptions_tests()

    print("=" * 60)
    if success:
        print("SUCCESS: All exception tests passed!")
        print("OK: Exception classes are working correctly!")
    else:
        print("FAILED: Some exception tests failed!")
        print("FIX: Check the errors above.")

    print("=" * 60)
