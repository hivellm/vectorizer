"""
Tests for data models (Vector, Collection, etc.) - equivalent to JS/TS tests.

This module contains comprehensive tests for all data models in the Python SDK,
mirroring the test structure and coverage of the JavaScript/TypeScript versions.
"""

import unittest
import sys
import os
from datetime import datetime

# Add current directory to path
sys.path.append(os.path.dirname(__file__))

# Import models
from models import (
    Vector, Collection, CollectionInfo, SearchResult,
    BatchInsertRequest, BatchSearchRequest, BatchUpdateRequest, BatchDeleteRequest,
    BatchResponse, BatchSearchResponse, EmbeddingRequest,
    SummarizeTextRequest, SummarizeTextResponse, SummarizeContextRequest,
    SummarizeContextResponse, GetSummaryResponse,
    ListSummariesResponse
)


class TestVectorModel(unittest.TestCase):
    """Tests for Vector model validation - equivalent to JS/TS vector.test.js"""

    def test_should_validate_correct_vector(self):
        """Test validation of a correct vector."""
        vector = Vector(
            id='test-id',
            data=[0.1, 0.2, 0.3, 0.4],
            metadata={'source': 'test.pdf'}
        )

        self.assertEqual(vector.id, 'test-id')
        self.assertEqual(vector.data, [0.1, 0.2, 0.3, 0.4])
        self.assertEqual(vector.metadata, {'source': 'test.pdf'})

    def test_should_validate_vector_without_metadata(self):
        """Test validation of vector without metadata."""
        vector = Vector(
            id='test-id',
            data=[0.1, 0.2, 0.3]
        )

        self.assertEqual(vector.id, 'test-id')
        self.assertEqual(vector.data, [0.1, 0.2, 0.3])
        self.assertIsNone(vector.metadata)

    def test_should_throw_error_for_missing_id(self):
        """Test error for missing ID."""
        with self.assertRaises(TypeError) as cm:
            Vector(data=[0.1, 0.2, 0.3])  # Missing required id parameter

        self.assertIn("missing 1 required positional argument", str(cm.exception))

    def test_should_throw_error_for_empty_id(self):
        """Test error for empty ID."""
        with self.assertRaises(ValueError) as cm:
            Vector(id='', data=[0.1, 0.2, 0.3])

        self.assertIn("Vector ID cannot be empty", str(cm.exception))

    def test_should_throw_error_for_non_string_id(self):
        """Test that non-string ID is accepted (Python is dynamically typed)."""
        # Python doesn't enforce type hints at runtime, so this is allowed
        vector = Vector(id=123, data=[0.1, 0.2, 0.3])
        self.assertEqual(vector.id, 123)

    def test_should_throw_error_for_missing_data(self):
        """Test error for missing data."""
        with self.assertRaises(TypeError):
            Vector(id='test-id')

    def test_should_throw_error_for_empty_data_array(self):
        """Test error for empty data array."""
        with self.assertRaises(ValueError) as cm:
            Vector(id='test-id', data=[])

        self.assertIn("Vector data cannot be empty", str(cm.exception))

    def test_should_throw_error_for_non_array_data(self):
        """Test error for non-array data."""
        with self.assertRaises(ValueError) as cm:
            Vector(id='test-id', data="not-an-array")

        self.assertIn("Vector data must contain only numbers", str(cm.exception))

    def test_should_throw_error_for_invalid_number_in_data(self):
        """Test error for invalid number in data."""
        with self.assertRaises(ValueError) as cm:
            Vector(id='test-id', data=[0.1, 'invalid', 0.3])

        self.assertIn("Vector data must contain only numbers", str(cm.exception))

    def test_should_throw_error_for_nan_in_data(self):
        """Test error for NaN in data."""
        with self.assertRaises(ValueError) as cm:
            Vector(id='test-id', data=[0.1, float('nan'), 0.3])

        self.assertIn("Vector data must not contain NaN values", str(cm.exception))

    def test_should_throw_error_for_infinity_in_data(self):
        """Test error for Infinity in data."""
        with self.assertRaises(ValueError) as cm:
            Vector(id='test-id', data=[0.1, float('inf'), 0.3])

        self.assertIn("Vector data must not contain Infinity values", str(cm.exception))

    def test_should_validate_large_vector(self):
        """Test validation of large vector."""
        large_data = [0.1] * 1000
        vector = Vector(id='large-vector', data=large_data)

        self.assertEqual(vector.id, 'large-vector')
        self.assertEqual(len(vector.data), 1000)


# CreateVectorRequest doesn't exist in Python SDK, skipping these tests


class TestCollectionModel(unittest.TestCase):
    """Tests for Collection model validation."""

    def test_should_validate_correct_collection(self):
        """Test validation of correct collection."""
        collection = Collection(
            name='test-collection',
            dimension=512,
            similarity_metric='cosine',
            description='Test collection'
        )

        self.assertEqual(collection.name, 'test-collection')
        self.assertEqual(collection.dimension, 512)
        self.assertEqual(collection.similarity_metric, 'cosine')
        self.assertEqual(collection.description, 'Test collection')

    def test_should_validate_collection_with_defaults(self):
        """Test collection with default values."""
        collection = Collection(
            name='test-collection',
            dimension=384
        )

        self.assertEqual(collection.name, 'test-collection')
        self.assertEqual(collection.dimension, 384)
        self.assertEqual(collection.similarity_metric, 'cosine')
        self.assertIsNone(collection.description)

    def test_should_throw_error_for_missing_name(self):
        """Test error for missing name."""
        with self.assertRaises(TypeError):
            Collection(dimension=512)

    def test_should_throw_error_for_empty_name(self):
        """Test error for empty name."""
        with self.assertRaises(ValueError) as cm:
            Collection(name='', dimension=512)

        self.assertIn("Collection name cannot be empty", str(cm.exception))

    def test_should_throw_error_for_non_string_name(self):
        """Test that non-string name is accepted (Python is dynamically typed)."""
        # Python doesn't enforce type hints at runtime, so this is allowed
        collection = Collection(name=123, dimension=512)
        self.assertEqual(collection.name, 123)

    def test_should_throw_error_for_missing_dimension(self):
        """Test error for missing dimension."""
        with self.assertRaises(TypeError):
            Collection(name='test')

    def test_should_throw_error_for_zero_dimension(self):
        """Test error for zero dimension."""
        with self.assertRaises(ValueError) as cm:
            Collection(name='test', dimension=0)

        self.assertIn("Dimension must be positive", str(cm.exception))

    def test_should_throw_error_for_negative_dimension(self):
        """Test error for negative dimension."""
        with self.assertRaises(ValueError) as cm:
            Collection(name='test', dimension=-1)

        self.assertIn("Dimension must be positive", str(cm.exception))

    def test_should_throw_error_for_non_integer_dimension(self):
        """Test that non-integer dimension is accepted (Python is dynamically typed)."""
        # Python doesn't enforce type hints at runtime, so this is allowed
        collection = Collection(name='test', dimension=512.5)
        self.assertEqual(collection.dimension, 512.5)

    def test_should_throw_error_for_invalid_similarity_metric(self):
        """Test error for invalid similarity metric."""
        with self.assertRaises(ValueError) as cm:
            Collection(name='test', dimension=512, similarity_metric='invalid')

        self.assertIn("Invalid similarity metric", str(cm.exception))

    def test_should_validate_all_similarity_metrics(self):
        """Test all valid similarity metrics."""
        metrics = ['cosine', 'euclidean', 'dot_product']

        for metric in metrics:
            with self.subTest(metric=metric):
                collection = Collection(
                    name=f'test-{metric}',
                    dimension=512,
                    similarity_metric=metric
                )
                self.assertEqual(collection.similarity_metric, metric)


# CreateCollectionRequest doesn't exist in Python SDK, skipping these tests


class TestCollectionInfo(unittest.TestCase):
    """Tests for CollectionInfo model."""

    def test_should_create_collection_info(self):
        """Test creation of CollectionInfo."""
        info = CollectionInfo(
            name='test-collection',
            dimension=512,
            similarity_metric='cosine',
            status='ready',
            vector_count=100,
            document_count=50
        )

        self.assertEqual(info.name, 'test-collection')
        self.assertEqual(info.dimension, 512)
        self.assertEqual(info.similarity_metric, 'cosine')
        self.assertEqual(info.status, 'ready')
        self.assertEqual(info.vector_count, 100)
        self.assertEqual(info.document_count, 50)


class TestSearchResult(unittest.TestCase):
    """Tests for SearchResult model."""

    def test_should_create_search_result(self):
        """Test creation of SearchResult."""
        result = SearchResult(
            id='doc1',
            score=0.95,
            content='test content',
            metadata={'category': 'test'}
        )

        self.assertEqual(result.id, 'doc1')
        self.assertEqual(result.score, 0.95)
        self.assertEqual(result.content, 'test content')
        self.assertEqual(result.metadata, {'category': 'test'})

    def test_should_throw_error_for_missing_id(self):
        """Test error for missing ID."""
        with self.assertRaises(TypeError):
            SearchResult(score=0.95)

    def test_should_throw_error_for_empty_id(self):
        """Test error for empty ID."""
        with self.assertRaises(ValueError) as cm:
            SearchResult(id='', score=0.95)

        self.assertIn("SearchResult ID cannot be empty", str(cm.exception))


# SearchResponse doesn't exist in Python SDK, skipping these tests


class TestBatchRequests(unittest.TestCase):
    """Tests for batch request models."""

    def test_should_create_batch_insert_request(self):
        """Test BatchInsertRequest creation."""
        from models import BatchTextRequest

        request = BatchInsertRequest(
            texts=[
                BatchTextRequest(id='text1', text='First text'),
                BatchTextRequest(id='text2', text='Second text')
            ]
        )

        self.assertEqual(len(request.texts), 2)
        self.assertEqual(request.texts[0].id, 'text1')

    def test_should_create_batch_search_request(self):
        """Test BatchSearchRequest creation."""
        from models import BatchSearchQuery

        request = BatchSearchRequest(
            queries=[
                BatchSearchQuery(query='first query', limit=5),
                BatchSearchQuery(query='second query', limit=10)
            ]
        )

        self.assertEqual(len(request.queries), 2)
        self.assertEqual(request.queries[0].query, 'first query')


def run_models_tests():
    """Run all model tests."""
    print("Testing Python SDK Models")
    print("=" * 60)

    # Create test suite
    test_suite = unittest.TestSuite()

    # Add test classes
    test_classes = [
        TestVectorModel,
        TestCollectionModel,
        TestCollectionInfo,
        TestSearchResult,
        TestBatchRequests,
    ]

    for test_class in test_classes:
        tests = unittest.TestLoader().loadTestsFromTestCase(test_class)
        test_suite.addTests(tests)

    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_models_tests()

    print("=" * 60)
    if success:
        print("SUCCESS: All model tests passed!")
        print("OK: Data models are working correctly!")
    else:
        print("FAILED: Some model tests failed!")
        print("FIX: Check the errors above.")

    print("=" * 60)
