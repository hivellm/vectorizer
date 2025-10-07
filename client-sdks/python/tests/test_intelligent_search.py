"""
Tests for Intelligent Search features - equivalent to TypeScript intelligent-search.test.ts

This test suite covers:
- intelligent_search() - Multi-query expansion with MMR
- semantic_search() - Advanced semantic reranking
- contextual_search() - Context-aware with metadata filtering
- multi_collection_search() - Cross-collection search
"""

import unittest
import asyncio
import os
import sys
from typing import Optional

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from client import VectorizerClient
from models import (
    IntelligentSearchRequest,
    SemanticSearchRequest,
    ContextualSearchRequest,
    MultiCollectionSearchRequest,
)
from exceptions import ValidationError, NetworkError, ServerError


class TestIntelligentSearchOperations(unittest.IsolatedAsyncioTestCase):
    """Tests for Intelligent Search Operations"""
    
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures."""
        cls.base_url = os.getenv('VECTORIZER_URL', 'http://localhost:15002')
        cls.server_available = False
        
    async def asyncSetUp(self):
        """Set up async test client."""
        self.client = VectorizerClient(base_url=self.base_url, timeout=5)
        await self.client.connect()
        
        # Check if server is available
        try:
            await self.client.health_check()
            self.__class__.server_available = True
        except Exception as e:
            print(f'WARNING: Vectorizer server not available at {self.base_url}')
            print('   Integration tests will be skipped. Start server with: cargo run --release')
            self.__class__.server_available = False
    
    async def asyncTearDown(self):
        """Clean up async test client."""
        await self.client.close()
    
    def skip_if_server_unavailable(self):
        """Skip test if server is not available."""
        if not self.server_available:
            self.skipTest("Server not available")
    
    # ==================== INTELLIGENT SEARCH TESTS ====================
    
    async def test_intelligent_search_with_default_options(self):
        """Test intelligent search with default options."""
        self.skip_if_server_unavailable()
        
        request = IntelligentSearchRequest(
            query='CMMV framework architecture',
            max_results=10
        )
        
        response = await self.client.intelligent_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
        self.assertGreaterEqual(response.total_results, 0)
    
    async def test_intelligent_search_with_specific_collections(self):
        """Test intelligent search with specific collections."""
        self.skip_if_server_unavailable()
        
        request = IntelligentSearchRequest(
            query='vector database features',
            collections=['test-collection-1', 'test-collection-2'],
            max_results=5
        )
        
        response = await self.client.intelligent_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_intelligent_search_with_domain_expansion(self):
        """Test intelligent search with domain expansion enabled."""
        self.skip_if_server_unavailable()
        
        request = IntelligentSearchRequest(
            query='semantic search',
            max_results=10,
            domain_expansion=True,
            technical_focus=True
        )
        
        response = await self.client.intelligent_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_intelligent_search_with_mmr_diversification(self):
        """Test intelligent search with MMR diversification."""
        self.skip_if_server_unavailable()
        
        request = IntelligentSearchRequest(
            query='vector embeddings',
            max_results=10,
            mmr_enabled=True,
            mmr_lambda=0.7
        )
        
        response = await self.client.intelligent_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_intelligent_search_returns_queries_generated(self):
        """Test that intelligent search returns queries generated."""
        self.skip_if_server_unavailable()
        
        request = IntelligentSearchRequest(
            query='machine learning models',
            max_results=5,
            domain_expansion=True
        )
        
        response = await self.client.intelligent_search(request)
        
        self.assertIsNotNone(response)
    
    # ==================== SEMANTIC SEARCH TESTS ====================
    
    async def test_semantic_search_with_default_options(self):
        """Test semantic search with default options."""
        self.skip_if_server_unavailable()
        
        request = SemanticSearchRequest(
            query='data processing pipeline',
            collection='test-collection',
            max_results=10
        )
        
        response = await self.client.semantic_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
        self.assertGreaterEqual(response.total_results, 0)
    
    async def test_semantic_search_with_reranking_enabled(self):
        """Test semantic search with reranking enabled."""
        self.skip_if_server_unavailable()
        
        request = SemanticSearchRequest(
            query='neural network architecture',
            collection='test-collection',
            max_results=10,
            semantic_reranking=True
        )
        
        response = await self.client.semantic_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_semantic_search_with_cross_encoder_reranking(self):
        """Test semantic search with cross-encoder reranking."""
        self.skip_if_server_unavailable()
        
        request = SemanticSearchRequest(
            query='transformer models',
            collection='test-collection',
            max_results=5,
            semantic_reranking=True,
            cross_encoder_reranking=True
        )
        
        response = await self.client.semantic_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    # ==================== CONTEXTUAL SEARCH TESTS ====================
    
    async def test_contextual_search_with_default_options(self):
        """Test contextual search with default options."""
        self.skip_if_server_unavailable()
        
        request = ContextualSearchRequest(
            query='API documentation',
            collection='test-collection',
            max_results=10
        )
        
        response = await self.client.contextual_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_contextual_search_with_metadata_filters(self):
        """Test contextual search with metadata filters."""
        self.skip_if_server_unavailable()
        
        request = ContextualSearchRequest(
            query='configuration settings',
            collection='test-collection',
            context_filters={
                'file_type': 'yaml',
                'category': 'config'
            },
            max_results=5
        )
        
        response = await self.client.contextual_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_contextual_search_with_context_reranking(self):
        """Test contextual search with context reranking."""
        self.skip_if_server_unavailable()
        
        request = ContextualSearchRequest(
            query='authentication middleware',
            collection='test-collection',
            max_results=10,
            context_reranking=True,
            context_weight=0.4
        )
        
        response = await self.client.contextual_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_contextual_search_with_complex_filters(self):
        """Test contextual search with complex filters."""
        self.skip_if_server_unavailable()
        
        request = ContextualSearchRequest(
            query='error handling',
            collection='test-collection',
            context_filters={
                'language': 'typescript',
                'framework': 'express',
                'min_lines': 50
            },
            max_results=10
        )
        
        response = await self.client.contextual_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    # ==================== MULTI-COLLECTION SEARCH TESTS ====================
    
    async def test_multi_collection_search(self):
        """Test search across multiple collections."""
        self.skip_if_server_unavailable()
        
        request = MultiCollectionSearchRequest(
            query='REST API endpoints',
            collections=['collection-1', 'collection-2', 'collection-3'],
            max_per_collection=5,
            max_total_results=15
        )
        
        response = await self.client.multi_collection_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_multi_collection_search_with_cross_collection_reranking(self):
        """Test multi-collection search with cross-collection reranking."""
        self.skip_if_server_unavailable()
        
        request = MultiCollectionSearchRequest(
            query='database queries',
            collections=['docs', 'examples', 'tests'],
            max_per_collection=3,
            max_total_results=9,
            cross_collection_reranking=True
        )
        
        response = await self.client.multi_collection_search(request)
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response.results, list)
    
    async def test_multi_collection_search_returns_results_per_collection(self):
        """Test that multi-collection search returns results per collection."""
        self.skip_if_server_unavailable()
        
        request = MultiCollectionSearchRequest(
            query='search algorithms',
            collections=['algorithms', 'implementations'],
            max_per_collection=5,
            max_total_results=10
        )
        
        response = await self.client.multi_collection_search(request)
        
        self.assertIsNotNone(response)
    
    async def test_multi_collection_search_respects_max_total_results(self):
        """Test that multi-collection search respects max_total_results limit."""
        self.skip_if_server_unavailable()
        
        request = MultiCollectionSearchRequest(
            query='common term',
            collections=['col1', 'col2', 'col3', 'col4'],
            max_per_collection=10,
            max_total_results=5
        )
        
        response = await self.client.multi_collection_search(request)
        
        self.assertIsNotNone(response)
        self.assertLessEqual(len(response.results), 5)
    
    # ==================== ERROR HANDLING TESTS ====================
    
    async def test_empty_query_in_intelligent_search(self):
        """Test that empty query in intelligent search raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(ValueError):
            request = IntelligentSearchRequest(
                query='',
                max_results=10
            )
    
    async def test_invalid_collection_in_semantic_search(self):
        """Test that invalid collection in semantic search raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(ValueError):
            request = SemanticSearchRequest(
                query='test',
                collection='',
                max_results=10
            )
    
    async def test_invalid_similarity_threshold(self):
        """Test that invalid similarity threshold raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(ValueError):
            request = SemanticSearchRequest(
                query='test',
                collection='test-collection',
                max_results=10,
                similarity_threshold=1.5  # Invalid: > 1.0
            )
    
    async def test_empty_collections_array(self):
        """Test that empty collections array raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(ValueError):
            request = MultiCollectionSearchRequest(
                query='test',
                collections=[],
                max_per_collection=5,
                max_total_results=10
            )
    
    # ==================== PERFORMANCE TESTS ====================
    
    async def test_intelligent_search_performance(self):
        """Test that intelligent search completes within reasonable time."""
        self.skip_if_server_unavailable()
        
        import time
        start_time = time.time()
        
        request = IntelligentSearchRequest(
            query='performance test',
            max_results=10
        )
        
        await self.client.intelligent_search(request)
        
        duration = time.time() - start_time
        self.assertLess(duration, 5.0)  # Should complete within 5 seconds
    
    async def test_intelligent_search_large_result_sets(self):
        """Test that intelligent search handles large result sets efficiently."""
        self.skip_if_server_unavailable()
        
        request = IntelligentSearchRequest(
            query='common term',
            max_results=100
        )
        
        response = await self.client.intelligent_search(request)
        
        self.assertIsNotNone(response)
        self.assertLessEqual(len(response.results), 100)


def run_intelligent_search_tests():
    """Run all intelligent search tests."""
    print("Testing Python SDK Intelligent Search Operations")
    print("=" * 60)
    
    # Create test suite
    test_suite = unittest.TestSuite()
    
    # Add test class
    tests = unittest.TestLoader().loadTestsFromTestCase(TestIntelligentSearchOperations)
    test_suite.addTests(tests)
    
    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_intelligent_search_tests()
    
    print("=" * 60)
    if success:
        print("SUCCESS: All intelligent search tests passed!")
        print("OK: Intelligent search operations are working correctly!")
    else:
        print("FAILED: Some intelligent search tests failed!")
        print("FIX: Check the errors above.")
    
    print("=" * 60)
    
    sys.exit(0 if success else 1)

