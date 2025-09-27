"""
Testes simples e diretos para o SDK Python do Hive Vectorizer.

Este módulo contém testes básicos que podem ser executados sem dependências
externas, usando apenas o módulo unittest padrão do Python.
"""

import unittest
import sys
import os

# Adicionar o diretório atual ao path
sys.path.append(os.path.dirname(__file__))

# Importar módulos do SDK
from models import Vector, Collection, CollectionInfo, SearchResult
from exceptions import (
    VectorizerError, ValidationError, CollectionNotFoundError,
    NetworkError, ServerError, AuthenticationError
)
from client import VectorizerClient


class TestBasicFunctionality(unittest.TestCase):
    """Testes básicos de funcionalidade."""
    
    def test_vector_creation_and_validation(self):
        """Teste criação e validação de Vector."""
        # Teste criação válida
        vector = Vector(
            id="test_vector",
            data=[0.1, 0.2, 0.3, 0.4, 0.5],
            metadata={"text": "test content", "category": "test"}
        )
        
        self.assertEqual(vector.id, "test_vector")
        self.assertEqual(len(vector.data), 5)
        self.assertEqual(vector.metadata["text"], "test content")
        
        # Teste validação de ID vazio
        with self.assertRaises(ValueError):
            Vector(id="", data=[0.1, 0.2, 0.3])
        
        # Teste validação de data vazia
        with self.assertRaises(ValueError):
            Vector(id="test", data=[])
        
        # Teste validação de data inválida
        with self.assertRaises(ValueError):
            Vector(id="test", data=["invalid", "data"])
    
    def test_collection_creation_and_validation(self):
        """Teste criação e validação de Collection."""
        # Teste criação válida
        collection = Collection(
            name="test_collection",
            dimension=512,
            similarity_metric="cosine",
            description="Test collection"
        )
        
        self.assertEqual(collection.name, "test_collection")
        self.assertEqual(collection.dimension, 512)
        self.assertEqual(collection.similarity_metric, "cosine")
        
        # Teste validação de nome vazio
        with self.assertRaises(ValueError):
            Collection(name="", dimension=512)
        
        # Teste validação de dimensão negativa
        with self.assertRaises(ValueError):
            Collection(name="test", dimension=-1)
        
        # Teste validação de métrica inválida
        with self.assertRaises(ValueError):
            Collection(name="test", dimension=512, similarity_metric="invalid")
    
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
        
        # Teste validação de ID vazio
        with self.assertRaises(ValueError):
            SearchResult(id="", score=0.95)
        
        # Teste validação de score inválido
        with self.assertRaises(ValueError):
            SearchResult(id="test", score="invalid")


class TestExceptions(unittest.TestCase):
    """Testes para exceções customizadas."""
    
    def test_vectorizer_error_basic(self):
        """Teste VectorizerError básico."""
        error = VectorizerError("Test error message")
        
        self.assertEqual(error.message, "Test error message")
        self.assertIsNone(error.error_code)
        self.assertEqual(error.details, {})
        self.assertEqual(str(error), "Test error message")
    
    def test_vectorizer_error_with_code(self):
        """Teste VectorizerError com código de erro."""
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
    
    def test_specific_exceptions(self):
        """Teste exceções específicas."""
        # Teste ValidationError
        error = ValidationError("Invalid input")
        self.assertEqual(error.error_code, "VALIDATION_ERROR")
        self.assertEqual(str(error), "[VALIDATION_ERROR] Invalid input")
        
        # Teste CollectionNotFoundError
        error = CollectionNotFoundError("Collection not found")
        self.assertEqual(error.error_code, "COLLECTION_NOT_FOUND")
        self.assertEqual(str(error), "[COLLECTION_NOT_FOUND] Collection not found")
        
        # Teste NetworkError
        error = NetworkError("Network issue")
        self.assertEqual(error.error_code, "NETWORK_ERROR")
        self.assertEqual(str(error), "[NETWORK_ERROR] Network issue")
        
        # Teste ServerError
        error = ServerError("Server issue")
        self.assertEqual(error.error_code, "SERVER_ERROR")
        self.assertEqual(str(error), "[SERVER_ERROR] Server issue")
        
        # Teste AuthenticationError
        error = AuthenticationError("Auth failed")
        self.assertEqual(error.error_code, "AUTH_ERROR")
        self.assertEqual(str(error), "[AUTH_ERROR] Auth failed")


class TestClientInitialization(unittest.TestCase):
    """Testes para inicialização do cliente."""
    
    def test_client_default_initialization(self):
        """Teste inicialização padrão do cliente."""
        client = VectorizerClient()
        
        self.assertEqual(client.base_url, "http://localhost:15001")
        self.assertIsNone(client.api_key)
        self.assertEqual(client.timeout, 30)
        self.assertEqual(client.max_retries, 3)
    
    def test_client_custom_initialization(self):
        """Teste inicialização customizada do cliente."""
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
    
    def test_client_with_api_key(self):
        """Teste inicialização com API key."""
        client = VectorizerClient(api_key="test-api-key")
        
        self.assertEqual(client.api_key, "test-api-key")
        self.assertEqual(client.base_url, "http://localhost:15001")


class TestDataValidation(unittest.TestCase):
    """Testes para validação de dados."""
    
    def test_vector_data_types(self):
        """Teste tipos de dados em Vector."""
        # Teste com números inteiros
        vector_int = Vector(id="test1", data=[1, 2, 3])
        self.assertEqual(vector_int.data, [1, 2, 3])
        
        # Teste com números float
        vector_float = Vector(id="test2", data=[1.1, 2.2, 3.3])
        self.assertEqual(vector_float.data, [1.1, 2.2, 3.3])
        
        # Teste com mistura de int e float
        vector_mixed = Vector(id="test3", data=[1, 2.5, 3])
        self.assertEqual(vector_mixed.data, [1, 2.5, 3])
    
    def test_collection_dimensions(self):
        """Teste diferentes dimensões de coleção."""
        # Teste dimensão pequena
        collection_small = Collection(name="small", dimension=128)
        self.assertEqual(collection_small.dimension, 128)
        
        # Teste dimensão média
        collection_medium = Collection(name="medium", dimension=512)
        self.assertEqual(collection_medium.dimension, 512)
        
        # Teste dimensão grande
        collection_large = Collection(name="large", dimension=1024)
        self.assertEqual(collection_large.dimension, 1024)
    
    def test_similarity_metrics(self):
        """Teste diferentes métricas de similaridade."""
        metrics = ["cosine", "euclidean", "dot_product"]
        
        for metric in metrics:
            with self.subTest(metric=metric):
                collection = Collection(
                    name=f"test_{metric}",
                    dimension=512,
                    similarity_metric=metric
                )
                self.assertEqual(collection.similarity_metric, metric)
        
        # Teste métrica inválida
        with self.assertRaises(ValueError):
            Collection(name="test", dimension=512, similarity_metric="invalid")


class TestEdgeCases(unittest.TestCase):
    """Testes para casos extremos."""
    
    def test_empty_metadata(self):
        """Teste com metadata vazia."""
        vector = Vector(id="test", data=[0.1, 0.2, 0.3], metadata={})
        self.assertEqual(vector.metadata, {})
        
        vector_none = Vector(id="test2", data=[0.1, 0.2, 0.3], metadata=None)
        self.assertIsNone(vector_none.metadata)
    
    def test_large_vector(self):
        """Teste com vetor grande."""
        large_data = [0.1] * 1000  # 1000 dimensões
        vector = Vector(id="large_vector", data=large_data)
        self.assertEqual(len(vector.data), 1000)
    
    def test_unicode_strings(self):
        """Teste com strings Unicode."""
        vector = Vector(
            id="unicode_test",
            data=[0.1, 0.2, 0.3],
            metadata={"text": "Hello 世界", "emoji": "🚀"}
        )
        self.assertEqual(vector.metadata["text"], "Hello 世界")
        self.assertEqual(vector.metadata["emoji"], "🚀")
    
    def test_numeric_string_ids(self):
        """Teste com IDs numéricos como string."""
        vector = Vector(id="123", data=[0.1, 0.2, 0.3])
        self.assertEqual(vector.id, "123")
        
        vector_uuid = Vector(id="550e8400-e29b-41d4-a716-446655440000", data=[0.1, 0.2, 0.3])
        self.assertEqual(vector_uuid.id, "550e8400-e29b-41d4-a716-446655440000")


def run_simple_tests():
    """Executa os testes simples."""
    print("🧪 Executando testes simples do SDK Python")
    print("=" * 50)
    
    # Criar suite de testes
    test_suite = unittest.TestSuite()
    
    # Adicionar testes
    test_classes = [
        TestBasicFunctionality,
        TestExceptions,
        TestClientInitialization,
        TestDataValidation,
        TestEdgeCases,
    ]
    
    for test_class in test_classes:
        tests = unittest.TestLoader().loadTestsFromTestCase(test_class)
        test_suite.addTests(tests)
    
    # Executar testes
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_simple_tests()
    
    print("=" * 50)
    if success:
        print("🎉 TODOS OS TESTES SIMPLES PASSARAM!")
        print("✅ Funcionalidades básicas estão funcionando!")
    else:
        print("❌ ALGUNS TESTES SIMPLES FALHARAM!")
        print("🔧 Verifique os erros acima.")
    
    print("=" * 50)
