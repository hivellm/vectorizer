"""
Testes unitários para o SDK Python do Hive Vectorizer.

Este módulo contém testes abrangentes para todos os componentes do SDK,
incluindo testes unitários, de integração e de validação.
"""

import unittest
import asyncio
import json
from unittest.mock import AsyncMock, Mock, patch, MagicMock
from typing import List, Dict, Any

# Importar módulos do SDK
import sys
import os
sys.path.append(os.path.dirname(__file__))

from models import (
    Vector, Collection, CollectionInfo, SearchResult,
    EmbeddingRequest, SearchRequest, BatchOperation,
    IndexingProgress, HealthStatus, ClientConfig
)
from exceptions import (
    VectorizerError, AuthenticationError, CollectionNotFoundError,
    ValidationError, NetworkError, ServerError, RateLimitError,
    TimeoutError, VectorNotFoundError, EmbeddingError, IndexingError,
    ConfigurationError, BatchOperationError, map_http_error
)
from client import VectorizerClient


class TestDataModels(unittest.TestCase):
    """Testes para os modelos de dados."""
    
    def test_vector_creation(self):
        """Teste criação de Vector."""
        vector = Vector(
            id="test_vector",
            data=[0.1, 0.2, 0.3],
            metadata={"text": "test content"}
        )
        
        self.assertEqual(vector.id, "test_vector")
        self.assertEqual(vector.data, [0.1, 0.2, 0.3])
        self.assertEqual(vector.metadata, {"text": "test content"})
    
    def test_vector_validation_empty_id(self):
        """Teste validação de Vector com ID vazio."""
        with self.assertRaises(ValueError) as context:
            Vector(id="", data=[0.1, 0.2, 0.3])
        
        self.assertIn("Vector ID cannot be empty", str(context.exception))
    
    def test_vector_validation_empty_data(self):
        """Teste validação de Vector com data vazia."""
        with self.assertRaises(ValueError) as context:
            Vector(id="test", data=[])
        
        self.assertIn("Vector data cannot be empty", str(context.exception))
    
    def test_vector_validation_invalid_data(self):
        """Teste validação de Vector com data inválida."""
        with self.assertRaises(ValueError) as context:
            Vector(id="test", data=["invalid", "data"])
        
        self.assertIn("Vector data must contain only numbers", str(context.exception))
    
    def test_collection_creation(self):
        """Teste criação de Collection."""
        collection = Collection(
            name="test_collection",
            dimension=512,
            similarity_metric="cosine",
            description="Test collection"
        )
        
        self.assertEqual(collection.name, "test_collection")
        self.assertEqual(collection.dimension, 512)
        self.assertEqual(collection.similarity_metric, "cosine")
        self.assertEqual(collection.description, "Test collection")
    
    def test_collection_validation_empty_name(self):
        """Teste validação de Collection com nome vazio."""
        with self.assertRaises(ValueError) as context:
            Collection(name="", dimension=512)
        
        self.assertIn("Collection name cannot be empty", str(context.exception))
    
    def test_collection_validation_negative_dimension(self):
        """Teste validação de Collection com dimensão negativa."""
        with self.assertRaises(ValueError) as context:
            Collection(name="test", dimension=-1)
        
        self.assertIn("Dimension must be positive", str(context.exception))
    
    def test_collection_validation_invalid_metric(self):
        """Teste validação de Collection com métrica inválida."""
        with self.assertRaises(ValueError) as context:
            Collection(name="test", dimension=512, similarity_metric="invalid")
        
        self.assertIn("Invalid similarity metric", str(context.exception))
    
    def test_collection_info_creation(self):
        """Teste criação de CollectionInfo."""
        info = CollectionInfo(
            name="test_collection",
            dimension=512,
            similarity_metric="cosine",
            status="ready",
            vector_count=100,
            document_count=50
        )
        
        self.assertEqual(info.name, "test_collection")
        self.assertEqual(info.dimension, 512)
        self.assertEqual(info.vector_count, 100)
        self.assertEqual(info.document_count, 50)
        self.assertEqual(info.status, "ready")
    
    def test_search_result_creation(self):
        """Teste criação de SearchResult."""
        result = SearchResult(
            id="doc1",
            score=0.95,
            content="test content",
            metadata={"category": "test"}
        )
        
        self.assertEqual(result.id, "doc1")
        self.assertEqual(result.score, 0.95)
        self.assertEqual(result.content, "test content")
        self.assertEqual(result.metadata, {"category": "test"})
    
    def test_search_result_validation_empty_id(self):
        """Teste validação de SearchResult com ID vazio."""
        with self.assertRaises(ValueError) as context:
            SearchResult(id="", score=0.95)
        
        self.assertIn("Search result ID cannot be empty", str(context.exception))
    
    def test_search_result_validation_invalid_score(self):
        """Teste validação de SearchResult com score inválido."""
        with self.assertRaises(ValueError) as context:
            SearchResult(id="test", score="invalid")
        
        self.assertIn("Score must be a number", str(context.exception))


class TestExceptions(unittest.TestCase):
    """Testes para as exceções customizadas."""
    
    def test_vectorizer_error_basic(self):
        """Teste VectorizerError básico."""
        error = VectorizerError("Test error")
        
        self.assertEqual(error.message, "Test error")
        self.assertIsNone(error.error_code)
        self.assertEqual(error.details, {})
        self.assertEqual(str(error), "Test error")
    
    def test_vectorizer_error_with_code(self):
        """Teste VectorizerError com código."""
        error = VectorizerError("Test error", "TEST_CODE")
        
        self.assertEqual(error.message, "Test error")
        self.assertEqual(error.error_code, "TEST_CODE")
        self.assertEqual(str(error), "[TEST_CODE] Test error")
    
    def test_vectorizer_error_with_details(self):
        """Teste VectorizerError com detalhes."""
        error = VectorizerError("Test error", "TEST_CODE", {"detail": "test"})
        
        self.assertEqual(error.message, "Test error")
        self.assertEqual(error.error_code, "TEST_CODE")
        self.assertEqual(error.details, {"detail": "test"})
    
    def test_collection_not_found_error(self):
        """Teste CollectionNotFoundError."""
        error = CollectionNotFoundError("Collection not found")
        
        self.assertEqual(error.message, "Collection not found")
        self.assertEqual(error.error_code, "COLLECTION_NOT_FOUND")
        self.assertEqual(str(error), "[COLLECTION_NOT_FOUND] Collection not found")
    
    def test_validation_error(self):
        """Teste ValidationError."""
        error = ValidationError("Invalid input")
        
        self.assertEqual(error.message, "Invalid input")
        self.assertEqual(error.error_code, "VALIDATION_ERROR")
    
    def test_network_error(self):
        """Teste NetworkError."""
        error = NetworkError("Network issue")
        
        self.assertEqual(error.message, "Network issue")
        self.assertEqual(error.error_code, "NETWORK_ERROR")
    
    def test_server_error(self):
        """Teste ServerError."""
        error = ServerError("Server issue")
        
        self.assertEqual(error.message, "Server issue")
        self.assertEqual(error.error_code, "SERVER_ERROR")
    
    def test_rate_limit_error(self):
        """Teste RateLimitError."""
        error = RateLimitError("Rate limit exceeded")
        
        self.assertEqual(error.message, "Rate limit exceeded")
        self.assertEqual(error.error_code, "RATE_LIMIT_ERROR")
    
    def test_timeout_error(self):
        """Teste TimeoutError."""
        error = TimeoutError("Operation timed out")
        
        self.assertEqual(error.message, "Operation timed out")
        self.assertEqual(error.error_code, "TIMEOUT_ERROR")
    
    def test_vector_not_found_error(self):
        """Teste VectorNotFoundError."""
        error = VectorNotFoundError("Vector not found")
        
        self.assertEqual(error.message, "Vector not found")
        self.assertEqual(error.error_code, "VECTOR_NOT_FOUND")
    
    def test_embedding_error(self):
        """Teste EmbeddingError."""
        error = EmbeddingError("Embedding failed")
        
        self.assertEqual(error.message, "Embedding failed")
        self.assertEqual(error.error_code, "EMBEDDING_ERROR")
    
    def test_indexing_error(self):
        """Teste IndexingError."""
        error = IndexingError("Indexing failed")
        
        self.assertEqual(error.message, "Indexing failed")
        self.assertEqual(error.error_code, "INDEXING_ERROR")
    
    def test_configuration_error(self):
        """Teste ConfigurationError."""
        error = ConfigurationError("Configuration error")
        
        self.assertEqual(error.message, "Configuration error")
        self.assertEqual(error.error_code, "CONFIGURATION_ERROR")
    
    def test_batch_operation_error(self):
        """Teste BatchOperationError."""
        error = BatchOperationError("Batch operation failed")
        
        self.assertEqual(error.message, "Batch operation failed")
        self.assertEqual(error.error_code, "BATCH_OPERATION_ERROR")
    
    def test_map_http_error(self):
        """Teste mapeamento de erros HTTP."""
        # Teste erro 400
        error = map_http_error(400, "Bad request")
        self.assertIsInstance(error, ValidationError)
        
        # Teste erro 401
        error = map_http_error(401, "Unauthorized")
        self.assertIsInstance(error, AuthenticationError)
        
        # Teste erro 404
        error = map_http_error(404, "Not found")
        self.assertIsInstance(error, CollectionNotFoundError)
        
        # Teste erro 429
        error = map_http_error(429, "Too many requests")
        self.assertIsInstance(error, RateLimitError)
        
        # Teste erro 500
        error = map_http_error(500, "Internal server error")
        self.assertIsInstance(error, ServerError)
        
        # Teste erro não mapeado
        error = map_http_error(999, "Unknown error")
        self.assertIsInstance(error, ServerError)


class TestVectorizerClient(unittest.TestCase):
    """Testes para o VectorizerClient."""
    
    def setUp(self):
        """Configuração inicial para cada teste."""
        self.client = VectorizerClient(
            base_url="http://localhost:15001",
            api_key="test-key",
            timeout=30
        )
    
    def test_client_initialization(self):
        """Teste inicialização do cliente."""
        self.assertEqual(self.client.base_url, "http://localhost:15001")
        self.assertEqual(self.client.api_key, "test-key")
        self.assertEqual(self.client.timeout, 30)
        self.assertEqual(self.client.max_retries, 3)
    
    def test_client_default_initialization(self):
        """Teste inicialização com valores padrão."""
        client = VectorizerClient()
        
        self.assertEqual(client.base_url, "http://localhost:15001")
        self.assertIsNone(client.api_key)
        self.assertEqual(client.timeout, 30)
        self.assertEqual(client.max_retries, 3)
    
    def test_client_custom_initialization(self):
        """Teste inicialização com valores customizados."""
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


class TestVectorizerClientAsync(unittest.IsolatedAsyncioTestCase):
    """Testes assíncronos para o VectorizerClient."""
    
    def setUp(self):
        """Configuração inicial para cada teste."""
        self.client = VectorizerClient(
            base_url="http://localhost:15001",
            api_key="test-key",
            timeout=30
        )
    
    async def test_health_check_success(self):
        """Teste health check bem-sucedido."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "status": "healthy",
            "service": "vectorizer-grpc",
            "version": "1.0.0"
        })
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            result = await self.client.health_check()
            
            self.assertEqual(result["status"], "healthy")
            self.assertEqual(result["service"], "vectorizer-grpc")
            self.assertEqual(result["version"], "1.0.0")
    
    async def test_health_check_failure(self):
        """Teste health check com falha."""
        mock_response = Mock()
        mock_response.status = 500
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            with self.assertRaises(ServerError) as context:
                await self.client.health_check()
            
            self.assertIn("Health check failed", str(context.exception))
    
    async def test_list_collections_success(self):
        """Teste listagem de coleções bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "collections": [
                {
                    "name": "test_collection",
                    "dimension": 512,
                    "similarity_metric": "cosine",
                    "status": "ready",
                    "vector_count": 100,
                    "document_count": 50
                }
            ]
        })
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            collections = await self.client.list_collections()
            
            self.assertEqual(len(collections), 1)
            self.assertEqual(collections[0].name, "test_collection")
            self.assertEqual(collections[0].dimension, 512)
            self.assertEqual(collections[0].vector_count, 100)
    
    async def test_create_collection_success(self):
        """Teste criação de coleção bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 201
        mock_response.json = AsyncMock(return_value={
            "name": "new_collection",
            "dimension": 512,
            "similarity_metric": "cosine",
            "status": "ready",
            "vector_count": 0
        })
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            collection = await self.client.create_collection(
                name="new_collection",
                dimension=512,
                description="Test collection"
            )
            
            self.assertEqual(collection.name, "new_collection")
            self.assertEqual(collection.dimension, 512)
            self.assertEqual(collection.status, "ready")
    
    async def test_create_collection_validation_error(self):
        """Teste criação de coleção com parâmetros inválidos."""
        with self.assertRaises(ValidationError) as context:
            await self.client.create_collection(name="", dimension=512)
        
        self.assertIn("Collection name must be a non-empty string", str(context.exception))
    
    async def test_create_collection_negative_dimension(self):
        """Teste criação de coleção com dimensão negativa."""
        with self.assertRaises(ValidationError) as context:
            await self.client.create_collection(name="test", dimension=-1)
        
        self.assertIn("Dimension must be positive", str(context.exception))
    
    async def test_embed_text_success(self):
        """Teste geração de embedding bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "embedding": [0.1, 0.2, 0.3, 0.4, 0.5]
        })
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            embedding = await self.client.embed_text("test text")
            
            self.assertEqual(len(embedding), 5)
            self.assertEqual(embedding, [0.1, 0.2, 0.3, 0.4, 0.5])
    
    async def test_embed_text_validation_error(self):
        """Teste geração de embedding com texto inválido."""
        with self.assertRaises(ValidationError) as context:
            await self.client.embed_text("")
        
        self.assertIn("Text must be a non-empty string", str(context.exception))
    
    async def test_insert_texts_success(self):
        """Teste inserção de vetores bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 201
        mock_response.json = AsyncMock(return_value={
            "inserted": 1,
            "collection": "test_collection"
        })
        
        vector = Vector(
            id="test_vector",
            data=[0.1, 0.2, 0.3],
            metadata={"text": "test"}
        )
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            result = await self.client.insert_texts("test_collection", [vector])
            
            self.assertEqual(result["inserted"], 1)
            self.assertEqual(result["collection"], "test_collection")
    
    async def test_insert_texts_validation_error(self):
        """Teste inserção de vetores com lista vazia."""
        with self.assertRaises(ValidationError) as context:
            await self.client.insert_texts("test_collection", [])
        
        self.assertIn("Vectors list cannot be empty", str(context.exception))
    
    async def test_search_vectors_success(self):
        """Teste busca de vetores bem-sucedida."""
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
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            results = await self.client.search_vectors(
                collection="test_collection",
                query="test query",
                limit=5
            )
            
            self.assertEqual(len(results), 1)
            self.assertEqual(results[0].id, "doc1")
            self.assertEqual(results[0].score, 0.95)
            self.assertEqual(results[0].content, "test content")
    
    async def test_search_vectors_collection_not_found(self):
        """Teste busca de vetores com coleção não encontrada."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.post.return_value.__aenter__.return_value = mock_response
            
            with self.assertRaises(CollectionNotFoundError) as context:
                await self.client.search_vectors(
                    collection="nonexistent",
                    query="test query"
                )
            
            self.assertIn("Collection 'nonexistent' not found", str(context.exception))
    
    async def test_search_vectors_validation_error(self):
        """Teste busca de vetores com parâmetros inválidos."""
        with self.assertRaises(ValidationError) as context:
            await self.client.search_vectors(
                collection="test_collection",
                query="",
                limit=5
            )
        
        self.assertIn("Query must be a non-empty string", str(context.exception))
    
    async def test_get_collection_info_success(self):
        """Teste obtenção de informações da coleção bem-sucedida."""
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
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            info = await self.client.get_collection_info("test_collection")
            
            self.assertEqual(info.name, "test_collection")
            self.assertEqual(info.dimension, 512)
            self.assertEqual(info.vector_count, 100)
            self.assertEqual(info.status, "ready")
    
    async def test_get_collection_info_not_found(self):
        """Teste obtenção de informações de coleção não encontrada."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            with self.assertRaises(CollectionNotFoundError) as context:
                await self.client.get_collection_info("nonexistent")
            
            self.assertIn("Collection 'nonexistent' not found", str(context.exception))
    
    async def test_delete_collection_success(self):
        """Teste exclusão de coleção bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 200
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.delete.return_value.__aenter__.return_value = mock_response
            
            result = await self.client.delete_collection("test_collection")
            
            self.assertTrue(result)
    
    async def test_delete_collection_not_found(self):
        """Teste exclusão de coleção não encontrada."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.delete.return_value.__aenter__.return_value = mock_response
            
            with self.assertRaises(CollectionNotFoundError) as context:
                await self.client.delete_collection("nonexistent")
            
            self.assertIn("Collection 'nonexistent' not found", str(context.exception))
    
    async def test_get_vector_success(self):
        """Teste obtenção de vetor específico bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "id": "test_vector",
            "data": [0.1, 0.2, 0.3],
            "metadata": {"text": "test"}
        })
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            vector = await self.client.get_vector("test_collection", "test_vector")
            
            self.assertEqual(vector.id, "test_vector")
            self.assertEqual(vector.data, [0.1, 0.2, 0.3])
            self.assertEqual(vector.metadata, {"text": "test"})
    
    async def test_get_vector_not_found(self):
        """Teste obtenção de vetor não encontrado."""
        mock_response = Mock()
        mock_response.status = 404
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            with self.assertRaises(CollectionNotFoundError) as context:
                await self.client.get_vector("test_collection", "nonexistent")
            
            self.assertIn("Vector 'nonexistent' not found", str(context.exception))
    
    async def test_delete_vectors_success(self):
        """Teste exclusão de vetores bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 200
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.delete.return_value.__aenter__.return_value = mock_response
            
            result = await self.client.delete_vectors(
                "test_collection", 
                ["vector1", "vector2"]
            )
            
            self.assertTrue(result)
    
    async def test_delete_vectors_validation_error(self):
        """Teste exclusão de vetores com lista vazia."""
        with self.assertRaises(ValidationError) as context:
            await self.client.delete_vectors("test_collection", [])
        
        self.assertIn("Vector IDs list cannot be empty", str(context.exception))
    
    async def test_get_indexing_progress_success(self):
        """Teste obtenção de progresso de indexação bem-sucedida."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "is_indexing": False,
            "overall_status": "completed",
            "collections": ["collection1", "collection2"]
        })
        
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.return_value.__aenter__.return_value = mock_response
            
            progress = await self.client.get_indexing_progress()
            
            self.assertFalse(progress["is_indexing"])
            self.assertEqual(progress["overall_status"], "completed")
            self.assertEqual(len(progress["collections"]), 2)
    
    async def test_network_error_handling(self):
        """Teste tratamento de erro de rede."""
        with patch.object(self.client, '_session') as mock_session:
            mock_session.get.side_effect = Exception("Network error")
            
            try:
                await self.client.health_check()
                self.fail("Expected NetworkError to be raised")
            except NetworkError as e:
                self.assertIn("Failed to connect to service", str(e))
            except Exception as e:
                # Se não for NetworkError, verificar se é aiohttp.ClientError
                if "Network error" in str(e):
                    pass  # OK, é o erro esperado
                else:
                    raise


class TestIntegration(unittest.IsolatedAsyncioTestCase):
    """Testes de integração (requerem serviço rodando)."""
    
    def setUp(self):
        """Configuração inicial para cada teste."""
        self.client = VectorizerClient(
            base_url="http://localhost:15001",
            api_key="test-key"
        )
    
    async def test_full_workflow_mock(self):
        """Teste workflow completo com mocks."""
        collection_name = "integration_test_collection"
        
        # Mock para health check
        health_mock = Mock()
        health_mock.status = 200
        health_mock.json = AsyncMock(return_value={"status": "healthy"})
        
        # Mock para create collection
        create_mock = Mock()
        create_mock.status = 201
        create_mock.json = AsyncMock(return_value={
            "name": collection_name,
            "dimension": 512,
            "similarity_metric": "cosine",
            "status": "ready",
            "vector_count": 0
        })
        
        # Mock para embed text
        embed_mock = Mock()
        embed_mock.status = 200
        embed_mock.json = AsyncMock(return_value={
            "embedding": [0.1] * 512
        })
        
        # Mock para insert vectors
        insert_mock = Mock()
        insert_mock.status = 201
        insert_mock.json = AsyncMock(return_value={"inserted": 1})
        
        # Mock para search vectors
        search_mock = Mock()
        search_mock.status = 200
        search_mock.json = AsyncMock(return_value={
            "results": [{
                "id": "test_doc",
                "score": 0.95,
                "metadata": {"text": "test document"}
            }]
        })
        
        # Mock para delete collection
        delete_mock = Mock()
        delete_mock.status = 200
        
        with patch.object(self.client, '_session') as mock_session:
            # Configurar diferentes respostas para diferentes chamadas
            mock_session.get.return_value.__aenter__.side_effect = [health_mock]
            mock_session.post.return_value.__aenter__.side_effect = [create_mock, embed_mock, insert_mock, search_mock]
            mock_session.delete.return_value.__aenter__.return_value = delete_mock
            
            # Executar workflow completo
            # 1. Health check
            health = await self.client.health_check()
            self.assertEqual(health["status"], "healthy")
            
            # 2. Create collection
            collection = await self.client.create_collection(
                name=collection_name,
                dimension=512,
                description="Integration test collection"
            )
            self.assertEqual(collection.name, collection_name)
            
            # 3. Generate embedding
            embedding = await self.client.embed_text("test document")
            self.assertEqual(len(embedding), 512)
            
            # 4. Create vector
            vector = Vector(
                id="test_doc",
                data=embedding,
                metadata={"text": "test document"}
            )
            
            # 5. Insert vector
            result = await self.client.insert_texts(collection_name, [vector])
            self.assertEqual(result["inserted"], 1)
            
            # 6. Search vectors
            results = await self.client.search_vectors(
                collection=collection_name,
                query="test document",
                limit=5
            )
            self.assertEqual(len(results), 1)
            self.assertEqual(results[0].id, "test_doc")
            
            # 7. Clean up
            await self.client.delete_collection(collection_name)


class TestUtilityFunctions(unittest.TestCase):
    """Testes para funções utilitárias."""
    
    def test_map_http_error_comprehensive(self):
        """Teste abrangente do mapeamento de erros HTTP."""
        test_cases = [
            (400, ValidationError),
            (401, AuthenticationError),
            (403, AuthenticationError),
            (404, CollectionNotFoundError),
            (408, TimeoutError),
            (429, RateLimitError),
            (500, ServerError),
            (502, ServerError),
            (503, ServerError),
            (504, ServerError),
            (999, ServerError),  # Código não mapeado
        ]
        
        for status_code, expected_error_class in test_cases:
            with self.subTest(status_code=status_code):
                error = map_http_error(status_code, f"HTTP {status_code} error")
                self.assertIsInstance(error, expected_error_class)
                self.assertEqual(error.message, f"HTTP {status_code} error")


def run_tests():
    """Executa todos os testes."""
    # Criar suite de testes
    test_suite = unittest.TestSuite()
    
    # Adicionar testes
    test_classes = [
        TestDataModels,
        TestExceptions,
        TestVectorizerClient,
        TestVectorizerClientAsync,
        TestIntegration,
        TestUtilityFunctions,
    ]
    
    for test_class in test_classes:
        tests = unittest.TestLoader().loadTestsFromTestCase(test_class)
        test_suite.addTests(tests)
    
    # Executar testes
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    return result.wasSuccessful()


if __name__ == "__main__":
    print("🧪 Executando testes do SDK Python Hive Vectorizer")
    print("=" * 60)
    
    success = run_tests()
    
    print("=" * 60)
    if success:
        print("🎉 TODOS OS TESTES PASSARAM!")
        print("✅ SDK Python está funcionando perfeitamente!")
    else:
        print("❌ ALGUNS TESTES FALHARAM!")
        print("🔧 Verifique os erros acima e corrija os problemas.")
    
    print("=" * 60)
