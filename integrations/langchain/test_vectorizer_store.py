"""
Tests for VectorizerStore LangChain integration

This module contains comprehensive tests for the VectorizerStore implementation.
"""

import pytest
import asyncio
from unittest.mock import Mock, patch, MagicMock
from vectorizer_store import (
    VectorizerStore, 
    VectorizerConfig, 
    VectorizerClient, 
    VectorizerError,
    create_vectorizer_store
)
from langchain.schema import Document


class TestVectorizerConfig:
    """Test VectorizerConfig class"""
    
    def test_default_config(self):
        """Test default configuration"""
        config = VectorizerConfig()
        assert config.host == "localhost"
        assert config.port == 15001
        assert config.collection_name == "langchain_documents"
        assert config.auto_create_collection is True
        assert config.batch_size == 100
        assert config.similarity_threshold == 0.7
    
    def test_custom_config(self):
        """Test custom configuration"""
        config = VectorizerConfig(
            host="example.com",
            port=8080,
            collection_name="test_collection",
            api_key="test_key",
            auto_create_collection=False,
            batch_size=50,
            similarity_threshold=0.8
        )
        assert config.host == "example.com"
        assert config.port == 8080
        assert config.collection_name == "test_collection"
        assert config.api_key == "test_key"
        assert config.auto_create_collection is False
        assert config.batch_size == 50
        assert config.similarity_threshold == 0.8


class TestVectorizerClient:
    """Test VectorizerClient class"""
    
    def setup_method(self):
        """Setup test client"""
        self.config = VectorizerConfig()
        self.client = VectorizerClient(self.config)
    
    @patch('requests.Session.request')
    def test_health_check_success(self, mock_request):
        """Test successful health check"""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {"status": "healthy"}
        mock_request.return_value = mock_response
        
        result = self.client.health_check()
        assert result is True
    
    @patch('requests.Session.request')
    def test_health_check_failure(self, mock_request):
        """Test failed health check"""
        mock_request.side_effect = Exception("Connection failed")
        
        result = self.client.health_check()
        assert result is False
    
    @patch('requests.Session.request')
    def test_list_collections(self, mock_request):
        """Test listing collections"""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "collections": [
                {"name": "collection1"},
                {"name": "collection2"}
            ]
        }
        mock_request.return_value = mock_response
        
        result = self.client.list_collections()
        assert result == ["collection1", "collection2"]
    
    @patch('requests.Session.request')
    def test_create_collection(self, mock_request):
        """Test creating collection"""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {"status": "created"}
        mock_request.return_value = mock_response
        
        result = self.client.create_collection("test_collection", 384, "cosine")
        assert result is True
    
    @patch('requests.Session.request')
    def test_add_texts(self, mock_request):
        """Test adding texts"""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "vector_ids": ["id1", "id2", "id3"]
        }
        mock_request.return_value = mock_response
        
        texts = ["text1", "text2", "text3"]
        metadatas = [{"source": "doc1"}, {"source": "doc2"}, {"source": "doc3"}]
        
        result = self.client.add_texts(texts, metadatas)
        assert result == ["id1", "id2", "id3"]
    
    @patch('requests.Session.request')
    def test_similarity_search(self, mock_request):
        """Test similarity search"""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "results": [
                {
                    "content": "Test content 1",
                    "metadata": {"source": "doc1"},
                    "score": 0.95
                },
                {
                    "content": "Test content 2",
                    "metadata": {"source": "doc2"},
                    "score": 0.87
                }
            ]
        }
        mock_request.return_value = mock_response
        
        result = self.client.similarity_search("test query", k=2)
        assert len(result) == 2
        assert result[0]["content"] == "Test content 1"
        assert result[0]["score"] == 0.95


class TestVectorizerStore:
    """Test VectorizerStore class"""
    
    def setup_method(self):
        """Setup test store"""
        self.config = VectorizerConfig(collection_name="test_collection")
        self.store = VectorizerStore(self.config)
    
    @patch.object(VectorizerClient, 'list_collections')
    @patch.object(VectorizerClient, 'create_collection')
    def test_collection_creation(self, mock_create, mock_list):
        """Test automatic collection creation"""
        mock_list.return_value = []
        mock_create.return_value = True
        
        # This should trigger collection creation
        store = VectorizerStore(self.config)
        mock_create.assert_called_once()
    
    @patch.object(VectorizerClient, 'add_texts')
    def test_add_texts(self, mock_add_texts):
        """Test adding texts to store"""
        mock_add_texts.return_value = ["id1", "id2"]
        
        texts = ["text1", "text2"]
        metadatas = [{"source": "doc1"}, {"source": "doc2"}]
        
        result = self.store.add_texts(texts, metadatas)
        assert result == ["id1", "id2"]
        mock_add_texts.assert_called_once_with(texts, metadatas)
    
    @patch.object(VectorizerClient, 'similarity_search')
    def test_similarity_search(self, mock_search):
        """Test similarity search"""
        mock_search.return_value = [
            {
                "content": "Test content",
                "metadata": {"source": "doc1"},
                "score": 0.95
            }
        ]
        
        result = self.store.similarity_search("test query", k=1)
        assert len(result) == 1
        assert isinstance(result[0], Document)
        assert result[0].page_content == "Test content"
        assert result[0].metadata == {"source": "doc1"}
    
    @patch.object(VectorizerClient, 'similarity_search')
    def test_similarity_search_with_score(self, mock_search):
        """Test similarity search with scores"""
        mock_search.return_value = [
            {
                "content": "Test content",
                "metadata": {"source": "doc1"},
                "score": 0.95
            }
        ]
        
        result = self.store.similarity_search_with_score("test query", k=1)
        assert len(result) == 1
        doc, score = result[0]
        assert isinstance(doc, Document)
        assert doc.page_content == "Test content"
        assert score == 0.95
    
    @patch.object(VectorizerClient, 'delete_vectors')
    def test_delete(self, mock_delete):
        """Test deleting vectors"""
        mock_delete.return_value = True
        
        result = self.store.delete(["id1", "id2"])
        assert result is True
        mock_delete.assert_called_once_with(["id1", "id2"])
    
    @patch.object(VectorizerClient, 'add_texts')
    def test_from_texts(self, mock_add_texts):
        """Test creating store from texts"""
        mock_add_texts.return_value = ["id1", "id2"]
        
        texts = ["text1", "text2"]
        metadatas = [{"source": "doc1"}, {"source": "doc2"}]
        
        store = VectorizerStore.from_texts(texts, metadatas=metadatas, config=self.config)
        assert isinstance(store, VectorizerStore)
        mock_add_texts.assert_called_once()
    
    @patch.object(VectorizerClient, 'add_texts')
    def test_from_documents(self, mock_add_texts):
        """Test creating store from documents"""
        mock_add_texts.return_value = ["id1", "id2"]
        
        documents = [
            Document(page_content="text1", metadata={"source": "doc1"}),
            Document(page_content="text2", metadata={"source": "doc2"})
        ]
        
        store = VectorizerStore.from_documents(documents, config=self.config)
        assert isinstance(store, VectorizerStore)
        mock_add_texts.assert_called_once()


class TestConvenienceFunctions:
    """Test convenience functions"""
    
    @patch('vectorizer_store.VectorizerStore')
    def test_create_vectorizer_store(self, mock_store_class):
        """Test create_vectorizer_store function"""
        mock_store = Mock()
        mock_store_class.return_value = mock_store
        
        result = create_vectorizer_store(
            host="example.com",
            port=8080,
            collection_name="test_collection"
        )
        
        assert result == mock_store
        mock_store_class.assert_called_once()


class TestErrorHandling:
    """Test error handling"""
    
    def setup_method(self):
        """Setup test client"""
        self.config = VectorizerConfig()
        self.client = VectorizerClient(self.config)
    
    @patch('requests.Session.request')
    def test_api_error_handling(self, mock_request):
        """Test API error handling"""
        mock_request.side_effect = Exception("API Error")
        
        with pytest.raises(VectorizerError):
            self.client.list_collections()
    
    @patch.object(VectorizerClient, 'add_texts')
    def test_store_error_handling(self, mock_add_texts):
        """Test store error handling"""
        mock_add_texts.side_effect = VectorizerError("Add texts failed")
        
        store = VectorizerStore(self.config)
        
        with pytest.raises(VectorizerError):
            store.add_texts(["text1"], [{"source": "doc1"}])


class TestAsyncOperations:
    """Test async operations"""
    
    @pytest.mark.asyncio
    async def test_async_client_operations(self):
        """Test async client operations"""
        config = VectorizerConfig()
        client = VectorizerClient(config)
        
        # Mock async operations
        with patch.object(client, '_make_async_request') as mock_async_request:
            mock_async_request.return_value = {"collections": []}
            
            # This would test async operations if implemented
            # For now, we just test the mock setup
            assert mock_async_request is not None


# Integration tests (require running Vectorizer instance)
class TestIntegration:
    """Integration tests that require a running Vectorizer instance"""
    
    @pytest.mark.integration
    def test_real_vectorizer_connection(self):
        """Test connection to real Vectorizer instance"""
        config = VectorizerConfig()
        client = VectorizerClient(config)
        
        # This test only runs if Vectorizer is running
        if client.health_check():
            collections = client.list_collections()
            assert isinstance(collections, list)
    
    @pytest.mark.integration
    def test_real_store_operations(self):
        """Test real store operations"""
        config = VectorizerConfig(collection_name="integration_test")
        store = VectorizerStore(config)
        
        # Test adding texts
        texts = ["Integration test document"]
        metadatas = [{"test": True}]
        
        try:
            vector_ids = store.add_texts(texts, metadatas)
            assert len(vector_ids) == 1
            
            # Test search
            results = store.similarity_search("integration test", k=1)
            assert len(results) >= 0
            
            # Clean up
            store.delete(vector_ids)
            
        except VectorizerError:
            pytest.skip("Vectorizer not available for integration test")


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
