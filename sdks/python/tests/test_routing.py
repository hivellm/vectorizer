"""
Tests for Master/Replica routing functionality
"""

import pytest
from unittest.mock import Mock, patch, MagicMock
from vectorizer_client import VectorizerClient, ReadPreference


@pytest.fixture
def mock_requests():
    """Mock requests library"""
    with patch('vectorizer_client.client.requests') as mock:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {'success': True}
        mock.post.return_value = mock_response
        mock.get.return_value = mock_response
        mock.delete.return_value = mock_response
        mock.put.return_value = mock_response
        yield mock


class TestOperationClassification:
    """Test that operations are classified correctly as read or write"""

    def test_write_operations_route_to_master(self, mock_requests):
        """All write operations should go to master regardless of read preference"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001', 'http://replica2:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        # Test insert (write operation)
        client.insert_texts('test-collection', [
            {'id': '1', 'text': 'test', 'metadata': {}}
        ])

        # Verify the call was made to master
        call_url = mock_requests.post.call_args[0][0]
        assert 'master:15001' in call_url

    def test_read_operations_route_based_on_preference(self, mock_requests):
        """Read operations should route based on readPreference"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001', 'http://replica2:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        mock_requests.post.return_value.json.return_value = {'results': []}

        # Test search (read operation)
        client.search_vectors('test-collection', [0.1, 0.2, 0.3])

        # Verify the call was made to a replica
        call_url = mock_requests.post.call_args[0][0]
        assert 'replica' in call_url

    def test_all_write_operations_classified_correctly(self, mock_requests):
        """Verify all write operations are classified as writes"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        write_operations = [
            lambda: client.insert_texts('test', [{'id': '1', 'text': 'test', 'metadata': {}}]),
            lambda: client.insert_vectors('test', [{'id': '1', 'vector': [0.1], 'metadata': {}}]),
            lambda: client.update_vector('test', '1', vector=[0.2]),
            lambda: client.delete_vector('test', '1'),
            lambda: client.create_collection('new-collection', dimension=512),
            lambda: client.delete_collection('test'),
        ]

        for op in write_operations:
            mock_requests.reset_mock()
            op()
            
            # Check that the operation went to master
            calls = (
                mock_requests.post.call_args_list +
                mock_requests.put.call_args_list +
                mock_requests.delete.call_args_list
            )
            
            assert any('master:15001' in str(call) for call in calls), \
                f"Operation {op} did not route to master"


class TestRoundRobinLoadBalancing:
    """Test round-robin load balancing across replicas"""

    def test_distributes_reads_evenly(self, mock_requests):
        """Reads should be distributed evenly across replicas"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': [
                    'http://replica1:15001',
                    'http://replica2:15001',
                    'http://replica3:15001'
                ]
            },
            read_preference=ReadPreference.REPLICA
        )

        mock_requests.post.return_value.json.return_value = {'results': []}

        calls = []
        for _ in range(6):
            client.search_vectors('test', [0.1])
            call_url = mock_requests.post.call_args[0][0]
            calls.append(call_url)

        # Verify each replica was called exactly twice
        assert sum(1 for url in calls if 'replica1' in url) == 2
        assert sum(1 for url in calls if 'replica2' in url) == 2
        assert sum(1 for url in calls if 'replica3' in url) == 2

        # Verify sequential distribution
        assert 'replica1' in calls[0]
        assert 'replica2' in calls[1]
        assert 'replica3' in calls[2]
        assert 'replica1' in calls[3]
        assert 'replica2' in calls[4]
        assert 'replica3' in calls[5]


class TestReadPreference:
    """Test read preference routing"""

    def test_read_preference_master(self, mock_requests):
        """readPreference: master should route reads to master"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001']
            },
            read_preference=ReadPreference.MASTER
        )

        mock_requests.post.return_value.json.return_value = {'results': []}

        client.search_vectors('test', [0.1])

        call_url = mock_requests.post.call_args[0][0]
        assert 'master:15001' in call_url

    def test_read_preference_replica(self, mock_requests):
        """readPreference: replica should route reads to replicas"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        mock_requests.post.return_value.json.return_value = {'results': []}

        client.search_vectors('test', [0.1])

        call_url = mock_requests.post.call_args[0][0]
        assert 'replica1:15001' in call_url


class TestReadPreferenceOverride:
    """Test per-operation read preference override"""

    def test_per_operation_override(self, mock_requests):
        """Should allow overriding readPreference for single operation"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        mock_requests.post.return_value.json.return_value = {'results': []}

        # First call without override - should go to replica
        client.search_vectors('test', [0.1])
        assert 'replica1' in mock_requests.post.call_args[0][0]

        # Second call with override - should go to master
        client.search_vectors('test', [0.1], read_preference=ReadPreference.MASTER)
        assert 'master' in mock_requests.post.call_args[0][0]

        # Third call without override - should go back to replica
        client.search_vectors('test', [0.1])
        assert 'replica1' in mock_requests.post.call_args[0][0]

    def test_with_master_context(self, mock_requests):
        """Should support with_master() context manager"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        mock_requests.post.return_value.json.return_value = {'results': [], 'success': True}

        with client.with_master() as master_client:
            # Both operations should go to master
            master_client.insert_texts('test', [{'id': '1', 'text': 'test', 'metadata': {}}])
            master_client.search_vectors('test', [0.1])

            calls = [call[0][0] for call in mock_requests.post.call_args_list]
            assert all('master' in url for url in calls)

        # Operation outside context should go to replica
        mock_requests.reset_mock()
        client.search_vectors('test', [0.1])
        assert 'replica1' in mock_requests.post.call_args[0][0]


class TestBackwardCompatibility:
    """Test backward compatibility with single-node configuration"""

    def test_single_base_url_configuration(self, mock_requests):
        """Should work with single base_url configuration"""
        client = VectorizerClient(
            base_url='http://localhost:15001'
        )

        # All operations should go to the single URL
        client.insert_texts('test', [{'id': '1', 'text': 'test', 'metadata': {}}])
        assert 'localhost:15001' in mock_requests.post.call_args[0][0]

        mock_requests.post.return_value.json.return_value = {'results': []}
        client.search_vectors('test', [0.1])
        assert 'localhost:15001' in mock_requests.post.call_args[0][0]

    def test_existing_code_unchanged(self, mock_requests):
        """Existing single-node code should work without changes"""
        # Old style client creation
        client = VectorizerClient(
            base_url='http://localhost:15001',
            api_key='test-key'
        )

        # Should work exactly as before
        client.create_collection('test', dimension=512)
        assert mock_requests.post.called


class TestErrorHandling:
    """Test error handling and failover"""

    def test_fallback_to_next_replica_on_failure(self, mock_requests):
        """Should try next replica if first one fails"""
        client = VectorizerClient(
            hosts={
                'master': 'http://master:15001',
                'replicas': ['http://replica1:15001', 'http://replica2:15001']
            },
            read_preference=ReadPreference.REPLICA
        )

        # First replica fails, second succeeds
        mock_requests.post.side_effect = [
            ConnectionError('Connection refused'),
            Mock(status_code=200, json=lambda: {'results': []})
        ]

        result = client.search_vectors('test', [0.1])
        assert result is not None

        # Should have tried twice
        assert mock_requests.post.call_count == 2

