"""
Tests for File Operations - equivalent to TypeScript file-operations.test.ts

This test suite covers:
- get_file_content() - Retrieve complete file content
- list_files_in_collection() - List indexed files
- get_file_summary() - Get file summaries
- get_file_chunks_ordered() - Get file chunks in order
- get_project_outline() - Get project structure
- get_related_files() - Find semantically related files
- search_by_file_type() - Search filtered by file type
"""

import unittest
import asyncio
import os
import sys

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from client import VectorizerClient
from exceptions import ValidationError, NetworkError, ServerError


class TestFileOperations(unittest.IsolatedAsyncioTestCase):
    """Tests for File Operations"""
    
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures."""
        cls.base_url = os.getenv('VECTORIZER_URL', 'http://localhost:15002')
        cls.test_collection = 'test-collection'
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
    
    # ==================== GET FILE CONTENT TESTS ====================
    
    async def test_retrieve_complete_file_content(self):
        """Test retrieve complete file content."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_content(
            collection=self.test_collection,
            file_path='README.md'
        )
        
        self.assertIsNotNone(response)
        self.assertEqual(response['file_path'], 'README.md')
        self.assertIn('content', response)
        self.assertIn('metadata', response)
    
    async def test_retrieve_file_content_with_size_limit(self):
        """Test retrieve file content with size limit."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_content(
            collection=self.test_collection,
            file_path='large-file.md',
            max_size_kb=100
        )
        
        self.assertIsNotNone(response)
        self.assertLessEqual(response['size_kb'], 100)
    
    async def test_file_content_includes_metadata(self):
        """Test that file content includes metadata."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_content(
            collection=self.test_collection,
            file_path='src/main.ts'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('metadata', response)
        self.assertIn('file_type', response['metadata'])
        self.assertGreater(response['metadata']['size'], 0)
    
    async def test_non_existent_file_raises_error(self):
        """Test that non-existent file raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.get_file_content(
                collection=self.test_collection,
                file_path='non-existent-file.txt'
            )
    
    # ==================== LIST FILES IN COLLECTION TESTS ====================
    
    async def test_list_all_files_in_collection(self):
        """Test list all files in collection."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection
        )
        
        self.assertIsNotNone(response)
        self.assertIn('files', response)
        self.assertIsInstance(response['files'], list)
        self.assertIn('total_count', response)
        self.assertGreaterEqual(response['total_count'], 0)
    
    async def test_filter_files_by_type(self):
        """Test filter files by type."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection,
            filter_by_type=['ts', 'js']
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['files'], list)
        
        if len(response['files']) > 0:
            for file in response['files']:
                self.assertTrue(
                    any(file['file_path'].endswith(f'.{ext}') for ext in ['ts', 'js'])
                )
    
    async def test_filter_files_by_minimum_chunks(self):
        """Test filter files by minimum chunks."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection,
            min_chunks=5
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['files'], list)
        
        if len(response['files']) > 0:
            for file in response['files']:
                self.assertGreaterEqual(file['chunk_count'], 5)
    
    async def test_limit_file_results(self):
        """Test limit file results."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection,
            max_results=10
        )
        
        self.assertIsNotNone(response)
        self.assertLessEqual(len(response['files']), 10)
    
    async def test_sort_files_by_name(self):
        """Test sort files by name."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection,
            sort_by='name'
        )
        
        self.assertIsNotNone(response)
        
        # Verify sorting
        if len(response['files']) > 1:
            for i in range(len(response['files']) - 1):
                self.assertLessEqual(
                    response['files'][i]['file_path'].lower(),
                    response['files'][i + 1]['file_path'].lower()
                )
    
    async def test_sort_files_by_size(self):
        """Test sort files by size."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection,
            sort_by='size'
        )
        
        self.assertIsNotNone(response)
        
        # Verify sorting
        if len(response['files']) > 1:
            for i in range(len(response['files']) - 1):
                self.assertGreaterEqual(
                    response['files'][i]['size'],
                    response['files'][i + 1]['size']
                )
    
    async def test_sort_files_by_chunks(self):
        """Test sort files by chunks."""
        self.skip_if_server_unavailable()
        
        response = await self.client.list_files_in_collection(
            collection=self.test_collection,
            sort_by='chunks'
        )
        
        self.assertIsNotNone(response)
        
        # Verify sorting
        if len(response['files']) > 1:
            for i in range(len(response['files']) - 1):
                self.assertGreaterEqual(
                    response['files'][i]['chunk_count'],
                    response['files'][i + 1]['chunk_count']
                )
    
    # ==================== GET FILE SUMMARY TESTS ====================
    
    async def test_get_extractive_summary(self):
        """Test get extractive summary."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_summary(
            collection=self.test_collection,
            file_path='README.md',
            summary_type='extractive',
            max_sentences=5
        )
        
        self.assertIsNotNone(response)
        self.assertIn('summary', response)
        self.assertEqual(response['summary_type'], 'extractive')
        self.assertLessEqual(len(response['sentences']), 5)
    
    async def test_get_structural_summary(self):
        """Test get structural summary."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_summary(
            collection=self.test_collection,
            file_path='src/main.ts',
            summary_type='structural'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('summary', response)
        self.assertEqual(response['summary_type'], 'structural')
        self.assertIn('structure', response)
    
    async def test_get_both_summary_types(self):
        """Test get both summary types."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_summary(
            collection=self.test_collection,
            file_path='docs/api.md',
            summary_type='both'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('extractive_summary', response)
        self.assertIn('structural_summary', response)
    
    # ==================== GET FILE CHUNKS ORDERED TESTS ====================
    
    async def test_get_file_chunks_in_order(self):
        """Test get file chunks in order."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_chunks_ordered(
            collection=self.test_collection,
            file_path='README.md'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('chunks', response)
        self.assertIsInstance(response['chunks'], list)
        self.assertIn('total_chunks', response)
        self.assertGreaterEqual(response['total_chunks'], 0)
    
    async def test_get_chunks_from_specific_position(self):
        """Test get chunks from specific position."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_chunks_ordered(
            collection=self.test_collection,
            file_path='README.md',
            start_chunk=5,
            limit=10
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['chunks'], list)
        self.assertEqual(response['start_chunk'], 5)
        self.assertLessEqual(len(response['chunks']), 10)
    
    async def test_get_chunks_with_context_hints(self):
        """Test get chunks with context hints."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_file_chunks_ordered(
            collection=self.test_collection,
            file_path='README.md',
            include_context=True
        )
        
        self.assertIsNotNone(response)
        self.assertIsInstance(response['chunks'], list)
        
        if len(response['chunks']) > 0:
            for chunk in response['chunks']:
                self.assertIn('has_prev', chunk)
                self.assertIn('has_next', chunk)
    
    async def test_paginate_through_chunks(self):
        """Test paginate through chunks."""
        self.skip_if_server_unavailable()
        
        # Get first page
        page1 = await self.client.get_file_chunks_ordered(
            collection=self.test_collection,
            file_path='README.md',
            start_chunk=0,
            limit=5
        )
        
        self.assertIsNotNone(page1)
        
        if page1['total_chunks'] > 5:
            # Get second page
            page2 = await self.client.get_file_chunks_ordered(
                collection=self.test_collection,
                file_path='README.md',
                start_chunk=5,
                limit=5
            )
            
            self.assertIsNotNone(page2)
            self.assertEqual(page2['start_chunk'], 5)
    
    # ==================== GET PROJECT OUTLINE TESTS ====================
    
    async def test_get_project_outline(self):
        """Test get project outline."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_project_outline(
            collection=self.test_collection
        )
        
        self.assertIsNotNone(response)
        self.assertIn('structure', response)
        self.assertIn('statistics', response)
    
    async def test_outline_with_depth_limit(self):
        """Test outline with depth limit."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_project_outline(
            collection=self.test_collection,
            max_depth=3
        )
        
        self.assertIsNotNone(response)
        self.assertEqual(response['max_depth'], 3)
    
    async def test_outline_with_file_summaries(self):
        """Test outline with file summaries."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_project_outline(
            collection=self.test_collection,
            include_summaries=True
        )
        
        self.assertIsNotNone(response)
        self.assertIn('structure', response)
        
        # Check if summaries are included
        import json
        has_summaries = 'summary' in json.dumps(response['structure'])
        self.assertTrue(has_summaries)
    
    async def test_outline_highlights_key_files(self):
        """Test outline highlights key files."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_project_outline(
            collection=self.test_collection,
            highlight_key_files=True
        )
        
        self.assertIsNotNone(response)
        self.assertIn('key_files', response)
        self.assertIsInstance(response['key_files'], list)
    
    # ==================== GET RELATED FILES TESTS ====================
    
    async def test_find_related_files(self):
        """Test find related files."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_related_files(
            collection=self.test_collection,
            file_path='src/main.ts'
        )
        
        self.assertIsNotNone(response)
        self.assertIn('related_files', response)
        self.assertIsInstance(response['related_files'], list)
    
    async def test_limit_related_files_results(self):
        """Test limit related files results."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_related_files(
            collection=self.test_collection,
            file_path='README.md',
            limit=5
        )
        
        self.assertIsNotNone(response)
        self.assertLessEqual(len(response['related_files']), 5)
    
    async def test_filter_related_files_by_similarity(self):
        """Test filter related files by similarity threshold."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_related_files(
            collection=self.test_collection,
            file_path='src/main.ts',
            similarity_threshold=0.7
        )
        
        self.assertIsNotNone(response)
        
        if len(response['related_files']) > 0:
            for file in response['related_files']:
                self.assertGreaterEqual(file['similarity_score'], 0.7)
    
    async def test_related_files_includes_reason(self):
        """Test that related files includes reason."""
        self.skip_if_server_unavailable()
        
        response = await self.client.get_related_files(
            collection=self.test_collection,
            file_path='src/main.ts',
            include_reason=True
        )
        
        self.assertIsNotNone(response)
        
        if len(response['related_files']) > 0:
            for file in response['related_files']:
                self.assertIn('reason', file)
    
    # ==================== SEARCH BY FILE TYPE TESTS ====================
    
    async def test_search_by_file_type_limits_results(self):
        """Test search by file type limits results."""
        self.skip_if_server_unavailable()
        
        response = await self.client.search_by_file_type(
            collection=self.test_collection,
            query='test',
            file_types=['ts', 'js'],
            limit=10
        )
        
        self.assertIsNotNone(response)
        self.assertLessEqual(len(response['results']), 10)
    
    async def test_search_by_multiple_file_types(self):
        """Test search by multiple file types."""
        self.skip_if_server_unavailable()
        
        response = await self.client.search_by_file_type(
            collection=self.test_collection,
            query='code',
            file_types=['ts', 'js', 'py', 'rs']
        )
        
        self.assertIsNotNone(response)
        self.assertIn('results', response)
        self.assertIsInstance(response['results'], list)
    
    # ==================== ERROR HANDLING TESTS ====================
    
    async def test_invalid_collection_raises_error(self):
        """Test that invalid collection raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.get_file_content(
                collection='non-existent-collection',
                file_path='README.md'
            )
    
    async def test_invalid_max_size_kb_raises_error(self):
        """Test that invalid max_size_kb raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.get_file_content(
                collection=self.test_collection,
                file_path='README.md',
                max_size_kb=-1
            )
    
    async def test_empty_file_types_array_raises_error(self):
        """Test that empty file types array raises error."""
        self.skip_if_server_unavailable()
        
        with self.assertRaises(Exception):
            await self.client.search_by_file_type(
                collection=self.test_collection,
                query='test',
                file_types=[]
            )
    
    # ==================== PERFORMANCE TESTS ====================
    
    async def test_list_files_efficiently(self):
        """Test that list files completes efficiently."""
        self.skip_if_server_unavailable()
        
        import time
        start_time = time.time()
        
        await self.client.list_files_in_collection(
            collection=self.test_collection,
            max_results=100
        )
        
        duration = time.time() - start_time
        self.assertLess(duration, 5.0)
    
    async def test_retrieve_file_content_quickly(self):
        """Test that retrieve file content completes quickly."""
        self.skip_if_server_unavailable()
        
        import time
        start_time = time.time()
        
        await self.client.get_file_content(
            collection=self.test_collection,
            file_path='README.md'
        )
        
        duration = time.time() - start_time
        self.assertLess(duration, 3.0)


def run_file_operations_tests():
    """Run all file operations tests."""
    print("Testing Python SDK File Operations")
    print("=" * 60)
    
    # Create test suite
    test_suite = unittest.TestSuite()
    
    # Add test class
    tests = unittest.TestLoader().loadTestsFromTestCase(TestFileOperations)
    test_suite.addTests(tests)
    
    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_file_operations_tests()
    
    print("=" * 60)
    if success:
        print("SUCCESS: All file operations tests passed!")
        print("OK: File operations are working correctly!")
    else:
        print("FAILED: Some file operations tests failed!")
        print("FIX: Check the errors above.")
    
    print("=" * 60)
    
    sys.exit(0 if success else 1)

