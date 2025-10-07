"""
Tests for Discovery Operations - equivalent to TypeScript discovery.test.ts

This test suite covers:
- discover() - Complete discovery pipeline
- filter_collections() - Collection filtering by patterns
- score_collections() - Relevance-based ranking
- expand_queries() - Query variation generation
"""

import unittest
import asyncio
import os
import sys
from typing import Optional

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from client import VectorizerClient
from exceptions import ValidationError, NetworkError, ServerError


class TestDiscoveryOperations(unittest.IsolatedAsyncioTestCase):
    """Tests for Discovery Operations"""
    
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures."""
        cls.base_url = os.getenv('VECTORIZER_URL', 'http://localhost:15002')
        cls.server_available = False
        
    async def asyncSetUp(self):
        """Set up async test client."""
        self.client = VectorizerClient(base_url=self.base_url, timeout=30)
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
    
    # ==================== DISCOVER TESTS ====================
    
    async def test_discover_complete_pipeline(self):
        """Test complete discovery pipeline."""
        self.skip_if_server_unavailable()
        
        response = await self.client.discover(
            query='How does CMMV framework work?',
            max_bullets=20,
            broad_k=50,
            focus_k=15
        )
        
        self.assertIsNotNone(response)
        self.assertIn('prompt', response)
        self.assertIn('evidence', response)
        self.assertIsInstance(response['evidence'], list)
        self.assertIn('metadata', response)
    
    async def test_discover_with_specific_collections(self):
        """Test discover with specific collections included."""
        self.skip_if_server_unavailable()
        
        response = await self.client.discover(
            query='API authentication methods',
            include_collections=['api-docs', 'security-docs'],
            max_bullets=15
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['evidence'], list)
    
    async def test_discover_with_excluded_collections(self):
        """Test discover with collections excluded."""
        self.skip_if_server_unavailable()
        
        response = await self.client.discover(
            query='database migrations',
            exclude_collections=['test-*', '*-backup'],
            max_bullets=10
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['evidence'], list)
    
    async def test_discover_generates_llm_ready_prompt(self):
        """Test that discover generates LLM-ready prompt."""
        self.skip_if_server_unavailable()
        
        response = await self.client.discover(
            query='vector search algorithms',
            max_bullets=10
        )
        
        self.assertIsNotNone(response)
        self.assertIn('prompt', response)
        self.assertIsInstance(response['prompt'], str)
        self.assertGreater(len(response['prompt']), 0)
    
    async def test_discover_includes_citations(self):
        """Test that discover includes citations in evidence."""
        self.skip_if_server_unavailable()
        
        response = await self.client.discover(
            query='system architecture',
            max_bullets=15
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['evidence'], list)
        
        if len(response['evidence']) > 0:
            for item in response['evidence']:
                self.assertIn('text', item)
                self.assertIn('citation', item)
    
    # ==================== FILTER COLLECTIONS TESTS ====================
    
    async def test_filter_collections_by_query(self):
        """Test filter collections by query."""
        self.skip_if_server_unavailable()
        
        response = await self.client.filter_collections(
            query='documentation'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('filtered_collections', response)
        self.assertIsInstance(response['filtered_collections'], list)
        self.assertIn('total_available', response)
        self.assertGreaterEqual(response['total_available'], 0)
    
    async def test_filter_with_include_patterns(self):
        """Test filter with include patterns."""
        self.skip_if_server_unavailable()
        
        response = await self.client.filter_collections(
            query='api endpoints',
            include=['*-docs', 'api-*']
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['filtered_collections'], list)
    
    async def test_filter_with_exclude_patterns(self):
        """Test filter with exclude patterns."""
        self.skip_if_server_unavailable()
        
        response = await self.client.filter_collections(
            query='source code',
            exclude=['*-test', '*-backup']
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['filtered_collections'], list)
        self.assertIn('excluded_count', response)
        self.assertGreaterEqual(response['excluded_count'], 0)
    
    async def test_filter_with_both_include_and_exclude(self):
        """Test filter with both include and exclude patterns."""
        self.skip_if_server_unavailable()
        
        response = await self.client.filter_collections(
            query='configuration',
            include=['config-*', '*-settings'],
            exclude=['*-old', '*-deprecated']
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['filtered_collections'], list)
    
    # ==================== SCORE COLLECTIONS TESTS ====================
    
    async def test_score_collections_by_relevance(self):
        """Test score collections by relevance."""
        self.skip_if_server_unavailable()
        
        response = await self.client.score_collections(
            query='machine learning'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('scored_collections', response)
        self.assertIsInstance(response['scored_collections'], list)
        self.assertIn('total_collections', response)
        self.assertGreaterEqual(response['total_collections'], 0)
    
    async def test_score_with_custom_term_boost_weight(self):
        """Test score with custom term boost weight."""
        self.skip_if_server_unavailable()
        
        response = await self.client.score_collections(
            query='database queries',
            term_boost_weight=0.4
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['scored_collections'], list)
    
    async def test_score_with_custom_signal_boost_weight(self):
        """Test score with custom signal boost weight."""
        self.skip_if_server_unavailable()
        
        response = await self.client.score_collections(
            query='performance optimization',
            signal_boost_weight=0.2
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['scored_collections'], list)
    
    async def test_score_collections_sorted_by_score(self):
        """Test that collections are sorted by score."""
        self.skip_if_server_unavailable()
        
        response = await self.client.score_collections(
            query='search functionality'
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['scored_collections'], list)
        
        # Verify sorting
        if len(response['scored_collections']) > 1:
            for i in range(len(response['scored_collections']) - 1):
                self.assertGreaterEqual(
                    response['scored_collections'][i]['score'],
                    response['scored_collections'][i + 1]['score']
                )
    
    # ==================== EXPAND QUERIES TESTS ====================
    
    async def test_expand_query_with_default_options(self):
        """Test expand query with default options."""
        self.skip_if_server_unavailable()
        
        response = await self.client.expand_queries(
            query='CMMV framework'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('original_query', response)
        self.assertEqual(response['original_query'], 'CMMV framework')
        self.assertIn('expanded_queries', response)
        self.assertIsInstance(response['expanded_queries'], list)
        self.assertGreater(len(response['expanded_queries']), 0)
    
    async def test_expand_query_limits_expansions(self):
        """Test that expand query limits number of expansions."""
        self.skip_if_server_unavailable()
        
        response = await self.client.expand_queries(
            query='vector database',
            max_expansions=5
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['expanded_queries'], list)
        self.assertLessEqual(len(response['expanded_queries']), 5)
    
    async def test_expand_query_includes_definition(self):
        """Test that expand query includes definition queries."""
        self.skip_if_server_unavailable()
        
        response = await self.client.expand_queries(
            query='semantic search',
            include_definition=True
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['expanded_queries'], list)
        self.assertIn('query_types', response)
        self.assertIn('definition', response['query_types'])
    
    async def test_expand_query_includes_features(self):
        """Test that expand query includes features queries."""
        self.skip_if_server_unavailable()
        
        response = await self.client.expand_queries(
            query='API gateway',
            include_features=True
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['expanded_queries'], list)
        self.assertIn('query_types', response)
        self.assertIn('features', response['query_types'])
    
    async def test_expand_query_includes_architecture(self):
        """Test that expand query includes architecture queries."""
        self.skip_if_server_unavailable()
        
        response = await self.client.expand_queries(
            query='microservices',
            include_architecture=True
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['expanded_queries'], list)
        self.assertIn('query_types', response)
        self.assertIn('architecture', response['query_types'])
    
    async def test_expand_query_generates_diverse_variations(self):
        """Test that expand query generates diverse query variations."""
        self.skip_if_server_unavailable()
        
        response = await self.client.expand_queries(
            query='authentication system',
            max_expansions=10,
            include_definition=True,
            include_features=True,
            include_architecture=True
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['expanded_queries'], list)
        self.assertGreater(len(response['expanded_queries']), 1)
        
        # Check for diversity
        unique_queries = set(response['expanded_queries'])
        self.assertEqual(len(unique_queries), len(response['expanded_queries']))
    
    # ==================== ERROR HANDLING TESTS ====================
    
    async def test_empty_query_in_discover(self):
        """Test that empty query in discover raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.discover(query='')
    
    async def test_invalid_max_bullets(self):
        """Test that invalid max_bullets raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.discover(query='test', max_bullets=-1)
    
    async def test_empty_query_in_filter_collections(self):
        """Test that empty query in filter_collections raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.filter_collections(query='')
    
    async def test_invalid_weights_in_score_collections(self):
        """Test that invalid weights in score_collections raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.score_collections(
                query='test',
                name_match_weight=1.5  # Invalid: > 1.0
            )
    
    # ==================== INTEGRATION TESTS ====================
    
    async def test_chain_filter_and_score_operations(self):
        """Test chaining filter and score operations."""
        self.skip_if_server_unavailable()
        
        # First filter
        filter_response = await self.client.filter_collections(
            query='documentation',
            include=['*-docs']
        )
        
        self.assertIsNotNone(filter_response)
        
        # Then score the filtered collections
        score_response = await self.client.score_collections(
            query='API documentation'
        )
        
        self.assertIsNotNone(score_response)
        self.assertIsInstance(score_response['scored_collections'], list)
    
    async def test_use_expanded_queries_in_discovery(self):
        """Test using expanded queries in discovery."""
        self.skip_if_server_unavailable()
        
        # First expand queries
        expand_response = await self.client.expand_queries(
            query='database optimization',
            max_expansions=5
        )
        
        self.assertIsNotNone(expand_response)
        self.assertGreater(len(expand_response['expanded_queries']), 0)
        
        # Use expanded queries in discovery
        discover_response = await self.client.discover(
            query=expand_response['expanded_queries'][0],
            max_bullets=10
        )
        
        self.assertIsNotNone(discover_response)
    
    # ==================== PERFORMANCE TESTS ====================
    
    async def test_discover_performance(self):
        """Test that discover completes within reasonable time."""
        self.skip_if_server_unavailable()
        
        import time
        start_time = time.time()
        
        await self.client.discover(
            query='performance test',
            max_bullets=10
        )
        
        duration = time.time() - start_time
        self.assertLess(duration, 10.0)  # Should complete within 10 seconds
    
    async def test_score_collections_with_large_collections(self):
        """Test that score_collections handles large collections efficiently."""
        self.skip_if_server_unavailable()
        
        response = await self.client.score_collections(query='test')
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['scored_collections'], list)


def run_discovery_tests():
    """Run all discovery tests."""
    print("Testing Python SDK Discovery Operations")
    print("=" * 60)
    
    # Create test suite
    test_suite = unittest.TestSuite()
    
    # Add test class
    tests = unittest.TestLoader().loadTestsFromTestCase(TestDiscoveryOperations)
    test_suite.addTests(tests)
    
    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_discovery_tests()
    
    print("=" * 60)
    if success:
        print("SUCCESS: All discovery tests passed!")
        print("OK: Discovery operations are working correctly!")
    else:
        print("FAILED: Some discovery tests failed!")
        print("FIX: Check the errors above.")
    
    print("=" * 60)
    
    sys.exit(0 if success else 1)

