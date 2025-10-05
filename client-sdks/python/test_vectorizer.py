"""
Tests for the Hive Vectorizer Python SDK.

This module contains comprehensive tests for all SDK functionality
including unit tests, integration tests, and error handling tests.
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, Mock, patch
from typing import List, Dict, Any

from vectorizer import VectorizerClient, Vector, CollectionInfo, SearchResult
from vectorizer.exceptions import (
    VectorizerError,
    CollectionNotFoundError,
    ValidationError,
    NetworkError,
    ServerError
)


class TestVectorizerClient:
    """Test cases for VectorizerClient."""
    
    @pytest.fixture
    def client(self):
        """Create a test client."""
        return VectorizerClient(
            base_url="http://localhost:15002",
            api_key="test-key"
        )
    
    @pytest.fixture
    def sample_vector(self):
        """Create a sample vector for testing."""
        return Vector(
            id="test_vector",
            data=[0.1, 0.2, 0.3, 0.4, 0.5],
            metadata={"text": "test content"}
        )
    
    @pytest.fixture
    def sample_collection_info(self):
        """Create sample collection info for testing."""
        return CollectionInfo(
            name="test_collection",
            dimension=512,
            similarity_metric="cosine",
            status="ready",
            vector_count=100,
            document_count=50
        )
    
    @pytest.mark.asyncio
    async def test_client_initialization(self, client):
        """Test client initialization."""
        assert client.base_url == "http://localhost:15002"
        assert client.api_key == "test-key"
        assert client.timeout == 30
        assert client.max_retries == 3
    
    @pytest.mark.asyncio
    async def test_health_check_success(self, client):
        """Test successful health check."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "status": "healthy",
            "service": "vectorizer-grpc",
            "version": "1.0.0"
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            result = await client.health_check()
            
            assert result["status"] == "healthy"
            assert result["service"] == "vectorizer-grpc"
    
    @pytest.mark.asyncio
    async def test_health_check_failure(self, client):
        """Test health check failure."""
        mock_response = Mock()
        mock_response.status = 500
        
        with patch.object(client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            with pytest.raises(ServerError):
                await client.health_check()
    
    @pytest.mark.asyncio
    async def test_list_collections_success(self, client, sample_collection_info):
        """Test successful collection listing."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "collections": [{
                "name": "test_collection",
                "dimension": 512,
                "similarity_metric": "cosine",
                "status": "ready",
                "vector_count": 100,
                "document_count": 50
            }]
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            collections = await client.list_collections()
            
            assert len(collections) == 1
            assert collections[0].name == "test_collection"
            assert collections[0].dimension == 512
    
    @pytest.mark.asyncio
    async def test_create_collection_success(self, client):
        """Test successful collection creation."""
        mock_response = Mock()
        mock_response.status = 201
        mock_response.json = AsyncMock(return_value={
            "name": "new_collection",
            "dimension": 512,
            "similarity_metric": "cosine",
            "status": "ready",
            "vector_count": 0
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            collection = await client.create_collection(
                name="new_collection",
                dimension=512,
                description="Test collection"
            )
            
            assert collection.name == "new_collection"
            assert collection.dimension == 512
    
    @pytest.mark.asyncio
    async def test_create_collection_validation_error(self, client):
        """Test collection creation with invalid parameters."""
        with pytest.raises(ValidationError):
            await client.create_collection(name="", dimension=512)
        
        with pytest.raises(ValidationError):
            await client.create_collection(name="test", dimension=-1)
    
    @pytest.mark.asyncio
    async def test_embed_text_success(self, client):
        """Test successful text embedding."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "embedding": [0.1, 0.2, 0.3, 0.4, 0.5]
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            embedding = await client.embed_text("test text")
            
            assert len(embedding) == 5
            assert embedding == [0.1, 0.2, 0.3, 0.4, 0.5]
    
    @pytest.mark.asyncio
    async def test_embed_text_validation_error(self, client):
        """Test text embedding with invalid input."""
        with pytest.raises(ValidationError):
            await client.embed_text("")
        
        with pytest.raises(ValidationError):
            await client.embed_text(None)
    
    @pytest.mark.asyncio
    async def test_insert_texts_success(self, client, sample_vector):
        """Test successful vector insertion."""
        mock_response = Mock()
        mock_response.status = 201
        mock_response.json = AsyncMock(return_value={
            "inserted": 1,
            "collection": "test_collection"
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            result = await client.insert_texts("test_collection", [sample_vector])
            
            assert result["inserted"] == 1
            assert result["collection"] == "test_collection"
    
    @pytest.mark.asyncio
    async def test_insert_texts_validation_error(self, client):
        """Test vector insertion with invalid input."""
        with pytest.raises(ValidationError):
            await client.insert_texts("test_collection", [])
    
    @pytest.mark.asyncio
    async def test_search_vectors_success(self, client):
        """Test successful vector search."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "results": [
                {
                    "id": "doc1",
                    "score": 0.95,
                    "content": "test content",
                    "metadata": {"text": "test"}
                }
            ]
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            results = await client.search_vectors(
                collection="test_collection",
                query="test query",
                limit=5
            )
            
            assert len(results) == 1
            assert results[0].id == "doc1"
            assert results[0].score == 0.95
    
    @pytest.mark.asyncio
    async def test_search_vectors_collection_not_found(self, client):
        """Test vector search with non-existent collection."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            with pytest.raises(CollectionNotFoundError):
                await client.search_vectors(
                    collection="nonexistent",
                    query="test query"
                )
    
    @pytest.mark.asyncio
    async def test_get_collection_info_success(self, client, sample_collection_info):
        """Test successful collection info retrieval."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "name": "test_collection",
            "dimension": 512,
            "similarity_metric": "cosine",
            "status": "ready",
            "vector_count": 100,
            "document_count": 50
        })
        
        with patch.object(client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            info = await client.get_collection_info("test_collection")
            
            assert info.name == "test_collection"
            assert info.dimension == 512
            assert info.vector_count == 100
    
    @pytest.mark.asyncio
    async def test_get_collection_info_not_found(self, client):
        """Test collection info retrieval for non-existent collection."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            with pytest.raises(CollectionNotFoundError):
                await client.get_collection_info("nonexistent")
    
    @pytest.mark.asyncio
    async def test_delete_collection_success(self, client):
        """Test successful collection deletion."""
        mock_response = Mock()
        mock_response.status = 200
        
        with patch.object(client, '_session') as mock_session:
            mock_session.delete.return_value.__aenter__.return_value = mock_response
            
            result = await client.delete_collection("test_collection")
            
            assert result is True
    
    @pytest.mark.asyncio
    async def test_delete_collection_not_found(self, client):
        """Test collection deletion for non-existent collection."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(client, '_session') as mock_session:
            mock_session.delete.return_value.__aenter__.return_value = mock_response
            
            with pytest.raises(CollectionNotFoundError):
                await client.delete_collection("nonexistent")
    
    @pytest.mark.asyncio
    async def test_network_error_handling(self, client):
        """Test network error handling."""
        with patch.object(client, '_session') as mock_session:
            mock_session.get.side_effect = Exception("Network error")
            
            with pytest.raises(NetworkError):
                await client.health_check()
    
    @pytest.mark.asyncio
    async def test_context_manager(self):
        """Test client context manager."""
        with patch('aiohttp.ClientSession') as mock_session_class:
            mock_session = AsyncMock()
            mock_session_class.return_value = mock_session
            
            async with VectorizerClient() as client:
                assert client._session is not None
            
            mock_session.close.assert_called_once()


class TestDataModels:
    """Test cases for data models."""
    
    def test_vector_creation(self):
        """Test Vector model creation."""
        vector = Vector(
            id="test",
            data=[0.1, 0.2, 0.3],
            metadata={"text": "test"}
        )
        
        assert vector.id == "test"
        assert vector.data == [0.1, 0.2, 0.3]
        assert vector.metadata == {"text": "test"}
    
    def test_vector_validation(self):
        """Test Vector model validation."""
        with pytest.raises(ValueError):
            Vector(id="", data=[0.1, 0.2, 0.3])
        
        with pytest.raises(ValueError):
            Vector(id="test", data=[])
        
        with pytest.raises(ValueError):
            Vector(id="test", data=["invalid", "data"])
    
    def test_collection_info_creation(self):
        """Test CollectionInfo model creation."""
        info = CollectionInfo(
            name="test",
            dimension=512,
            similarity_metric="cosine",
            status="ready",
            vector_count=100
        )
        
        assert info.name == "test"
        assert info.dimension == 512
        assert info.vector_count == 100
    
    def test_collection_info_validation(self):
        """Test CollectionInfo model validation."""
        with pytest.raises(ValueError):
            CollectionInfo(
                name="",
                dimension=512,
                similarity_metric="cosine",
                status="ready",
                vector_count=100
            )
        
        with pytest.raises(ValueError):
            CollectionInfo(
                name="test",
                dimension=-1,
                similarity_metric="cosine",
                status="ready",
                vector_count=100
            )


class TestExceptions:
    """Test cases for custom exceptions."""
    
    def test_vectorizer_error(self):
        """Test base VectorizerError."""
        error = VectorizerError("Test error", "TEST_CODE", {"detail": "test"})
        
        assert str(error) == "[TEST_CODE] Test error"
        assert error.error_code == "TEST_CODE"
        assert error.details == {"detail": "test"}
    
    def test_collection_not_found_error(self):
        """Test CollectionNotFoundError."""
        error = CollectionNotFoundError("Collection not found")
        
        assert str(error) == "[COLLECTION_NOT_FOUND] Collection not found"
        assert error.error_code == "COLLECTION_NOT_FOUND"
    
    def test_validation_error(self):
        """Test ValidationError."""
        error = ValidationError("Invalid input")
        
        assert str(error) == "[VALIDATION_ERROR] Invalid input"
        assert error.error_code == "VALIDATION_ERROR"


# Integration tests (require running service)
@pytest.mark.integration
class TestIntegration:
    """Integration tests requiring a running Vectorizer service."""
    
    @pytest.fixture
    def integration_client(self):
        """Create client for integration tests."""
        return VectorizerClient(
            base_url="http://localhost:15002",
            api_key="test-key"
        )
    
    @pytest.mark.asyncio
    async def test_full_workflow(self, integration_client):
        """Test complete workflow from collection creation to search."""
        collection_name = "integration_test_collection"
        
        try:
            # Create collection
            collection = await integration_client.create_collection(
                name=collection_name,
                dimension=512,
                description="Integration test collection"
            )
            assert collection.name == collection_name
            
            # Generate embedding
            embedding = await integration_client.embed_text("test document")
            assert len(embedding) == 512
            
            # Create vector
            vector = Vector(
                id="test_doc",
                data=embedding,
                metadata={"text": "test document"}
            )
            
            # Insert vector
            result = await integration_client.insert_texts(collection_name, [vector])
            assert result is not None
            
            # Search vectors
            results = await integration_client.search_vectors(
                collection=collection_name,
                query="test document",
                limit=5
            )
            assert len(results) > 0
            assert results[0].id == "test_doc"
            
        finally:
            # Clean up
            try:
                await integration_client.delete_collection(collection_name)
            except CollectionNotFoundError:
                pass  # Collection already deleted


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
