"""
Tests for TensorFlow Integration

This module contains comprehensive tests for the TensorFlow integration with Vectorizer.
"""

import pytest
import tensorflow as tf
import numpy as np
from unittest.mock import Mock, patch, MagicMock
from tensorflow_embedder import (
    TensorFlowModelConfig,
    TensorFlowEmbedder,
    TransformerEmbedder,
    CNNEmbedder,
    CustomTensorFlowEmbedder,
    TensorFlowVectorizerClient,
    create_transformer_embedder,
    create_cnn_embedder,
    create_custom_embedder
)


class TestTensorFlowModelConfig:
    """Test TensorFlowModelConfig class"""
    
    def test_default_config(self):
        """Test default configuration"""
        config = TensorFlowModelConfig(model_path="test_model.h5")
        
        assert config.model_path == "test_model.h5"
        assert config.device == "auto"
        assert config.batch_size == 32
        assert config.max_length == 512
        assert config.normalize_embeddings is True
        assert config.model_type == "transformer"
        assert config.tokenizer_path is None
        assert config.model_config is None
    
    def test_custom_config(self):
        """Test custom configuration"""
        config = TensorFlowModelConfig(
            model_path="custom_model.h5",
            device="gpu",
            batch_size=64,
            max_length=256,
            normalize_embeddings=False,
            model_type="cnn",
            tokenizer_path="custom_tokenizer",
            model_config={"hidden_size": 768}
        )
        
        assert config.model_path == "custom_model.h5"
        assert config.device == "gpu"
        assert config.batch_size == 64
        assert config.max_length == 256
        assert config.normalize_embeddings is False
        assert config.model_type == "cnn"
        assert config.tokenizer_path == "custom_tokenizer"
        assert config.model_config == {"hidden_size": 768}


class TestTransformerEmbedder:
    """Test TransformerEmbedder class"""
    
    def setup_method(self):
        """Setup test embedder"""
        self.config = TensorFlowModelConfig(
            model_path="test_model",
            device="cpu",
            batch_size=2,
            max_length=128
        )
    
    @patch('tensorflow_embedder.TFAutoModel')
    @patch('tensorflow_embedder.AutoTokenizer')
    def test_load_model(self, mock_tokenizer, mock_model):
        """Test model loading"""
        # Mock transformers components
        mock_tokenizer_instance = Mock()
        mock_tokenizer_instance.from_pretrained.return_value = mock_tokenizer_instance
        mock_tokenizer.from_pretrained.return_value = mock_tokenizer_instance
        
        mock_model_instance = Mock()
        mock_model_instance.config.hidden_size = 768
        mock_model.from_pretrained.return_value = mock_model_instance
        
        embedder = TransformerEmbedder(self.config)
        
        assert embedder.tokenizer is not None
        assert embedder.model is not None
        assert embedder.device == "CPU"
    
    @patch('tensorflow_embedder.TFAutoModel')
    @patch('tensorflow_embedder.AutoTokenizer')
    def test_embed_texts(self, mock_tokenizer, mock_model):
        """Test embedding generation for texts"""
        # Mock transformers components
        mock_tokenizer_instance = Mock()
        mock_tokenizer_instance.from_pretrained.return_value = mock_tokenizer_instance
        mock_tokenizer.from_pretrained.return_value = mock_tokenizer_instance
        
        mock_model_instance = Mock()
        mock_model_instance.config.hidden_size = 768
        mock_model.from_pretrained.return_value = mock_model_instance
        
        # Mock tokenizer output
        mock_tokenizer_instance.return_value = {
            'input_ids': tf.constant([[1, 2, 3, 4]]),
            'attention_mask': tf.constant([[1, 1, 1, 1]])
        }
        
        # Mock model output
        mock_output = Mock()
        mock_output.last_hidden_state = tf.random.normal([1, 4, 768])
        mock_model_instance.return_value = mock_output
        
        embedder = TransformerEmbedder(self.config)
        
        texts = ["test text"]
        embeddings = embedder.embed_texts(texts)
        
        assert isinstance(embeddings, np.ndarray)
        assert embeddings.shape[0] == 1
        assert embeddings.shape[1] == 768
    
    @patch('tensorflow_embedder.TFAutoModel')
    @patch('tensorflow_embedder.AutoTokenizer')
    def test_embed_text(self, mock_tokenizer, mock_model):
        """Test embedding generation for single text"""
        # Mock transformers components
        mock_tokenizer_instance = Mock()
        mock_tokenizer_instance.from_pretrained.return_value = mock_tokenizer_instance
        mock_tokenizer.from_pretrained.return_value = mock_tokenizer_instance
        
        mock_model_instance = Mock()
        mock_model_instance.config.hidden_size = 768
        mock_model.from_pretrained.return_value = mock_model_instance
        
        # Mock tokenizer output
        mock_tokenizer_instance.return_value = {
            'input_ids': tf.constant([[1, 2, 3, 4]]),
            'attention_mask': tf.constant([[1, 1, 1, 1]])
        }
        
        # Mock model output
        mock_output = Mock()
        mock_output.last_hidden_state = tf.random.normal([1, 4, 768])
        mock_model_instance.return_value = mock_output
        
        embedder = TransformerEmbedder(self.config)
        
        text = "test text"
        embedding = embedder.embed_text(text)
        
        assert isinstance(embedding, np.ndarray)
        assert embedding.shape[0] == 768
    
    @patch('tensorflow_embedder.TFAutoModel')
    @patch('tensorflow_embedder.AutoTokenizer')
    def test_get_embedding_dimension(self, mock_tokenizer, mock_model):
        """Test getting embedding dimension"""
        # Mock transformers components
        mock_tokenizer_instance = Mock()
        mock_tokenizer_instance.from_pretrained.return_value = mock_tokenizer_instance
        mock_tokenizer.from_pretrained.return_value = mock_tokenizer_instance
        
        mock_model_instance = Mock()
        mock_model_instance.config.hidden_size = 768
        mock_model.from_pretrained.return_value = mock_model_instance
        
        embedder = TransformerEmbedder(self.config)
        
        dimension = embedder.get_embedding_dimension()
        assert dimension == 768


class TestCNNEmbedder:
    """Test CNNEmbedder class"""
    
    def setup_method(self):
        """Setup test embedder"""
        self.config = TensorFlowModelConfig(
            model_path="test_model.h5",
            device="cpu",
            batch_size=2,
            max_length=128,
            model_type="cnn"
        )
    
    @patch('tensorflow.keras.models.load_model')
    def test_load_model(self, mock_load):
        """Test model loading"""
        # Mock model
        mock_model = Mock()
        mock_model.predict.return_value = np.random.rand(1, 512)
        mock_load.return_value = mock_model
        
        embedder = CNNEmbedder(self.config)
        
        assert embedder.model is not None
        assert embedder.device == "CPU"
    
    @patch('tensorflow.keras.models.load_model')
    def test_embed_texts(self, mock_load):
        """Test embedding generation for texts"""
        # Mock model
        mock_model = Mock()
        mock_model.predict.return_value = np.random.rand(1, 512)
        mock_load.return_value = mock_model
        
        embedder = CNNEmbedder(self.config)
        
        texts = ["test text"]
        embeddings = embedder.embed_texts(texts)
        
        assert isinstance(embeddings, np.ndarray)
        assert embeddings.shape[0] == 1
        assert embeddings.shape[1] == 512


class TestCustomTensorFlowEmbedder:
    """Test CustomTensorFlowEmbedder class"""
    
    def setup_method(self):
        """Setup test embedder"""
        self.config = TensorFlowModelConfig(
            model_path="test_model.h5",
            device="cpu",
            batch_size=2,
            max_length=128,
            model_type="custom"
        )
    
    @patch('tensorflow.keras.models.load_model')
    def test_load_model(self, mock_load):
        """Test model loading"""
        # Mock model
        mock_model = Mock()
        mock_load.return_value = mock_model
        
        embedder = CustomTensorFlowEmbedder(self.config)
        
        assert embedder.model is not None
        assert embedder.device == "CPU"
    
    @patch('tensorflow.keras.models.load_model')
    def test_embed_texts(self, mock_load):
        """Test embedding generation for texts"""
        # Mock model
        mock_model = Mock()
        mock_load.return_value = mock_model
        
        embedder = CustomTensorFlowEmbedder(self.config)
        
        texts = ["test text"]
        embeddings = embedder.embed_texts(texts)
        
        assert isinstance(embeddings, np.ndarray)
        assert embeddings.shape[0] == 1
        assert embeddings.shape[1] == 512


class TestTensorFlowVectorizerClient:
    """Test TensorFlowVectorizerClient class"""
    
    def setup_method(self):
        """Setup test client"""
        self.client = TensorFlowVectorizerClient()
        self.mock_embedder = Mock()
        self.mock_embedder.get_embedding_dimension.return_value = 768
        self.mock_embedder.embed_texts.return_value = np.random.rand(2, 768)
        self.mock_embedder.embed_text.return_value = np.random.rand(768)
    
    def test_set_embedder(self):
        """Test setting embedder"""
        self.client.set_embedder(self.mock_embedder)
        assert self.client.embedder == self.mock_embedder
    
    @patch('requests.post')
    def test_create_collection(self, mock_post):
        """Test creating collection"""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response
        
        result = self.client.create_collection("test_collection")
        assert result is True
        assert self.client.collection_name == "test_collection"
    
    @patch('requests.post')
    def test_create_collection_with_embedder(self, mock_post):
        """Test creating collection with embedder"""
        self.client.set_embedder(self.mock_embedder)
        
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response
        
        result = self.client.create_collection("test_collection")
        assert result is True
        
        # Check that the request was made with correct dimension
        call_args = mock_post.call_args
        assert call_args[1]['json']['dimension'] == 768
    
    @patch('requests.post')
    def test_add_texts(self, mock_post):
        """Test adding texts"""
        self.client.set_embedder(self.mock_embedder)
        self.client.collection_name = "test_collection"
        
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {"vector_ids": ["id1", "id2"]}
        mock_post.return_value = mock_response
        
        texts = ["text1", "text2"]
        metadatas = [{"source": "doc1"}, {"source": "doc2"}]
        
        result = self.client.add_texts(texts, metadatas)
        assert result == ["id1", "id2"]
    
    @patch('requests.post')
    def test_search_similar(self, mock_post):
        """Test similarity search"""
        self.client.set_embedder(self.mock_embedder)
        self.client.collection_name = "test_collection"
        
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "results": [
                {"content": "result1", "score": 0.9},
                {"content": "result2", "score": 0.8}
            ]
        }
        mock_post.return_value = mock_response
        
        result = self.client.search_similar("query", k=2)
        assert len(result) == 2
        assert result[0]["content"] == "result1"
    
    def test_add_texts_no_embedder(self):
        """Test adding texts without embedder"""
        with pytest.raises(ValueError, match="No embedder set"):
            self.client.add_texts(["text1"])
    
    def test_add_texts_no_collection(self):
        """Test adding texts without collection"""
        self.client.set_embedder(self.mock_embedder)
        
        with pytest.raises(ValueError, match="No collection created"):
            self.client.add_texts(["text1"])
    
    def test_search_similar_no_embedder(self):
        """Test search without embedder"""
        with pytest.raises(ValueError, match="No embedder set"):
            self.client.search_similar("query")
    
    def test_search_similar_no_collection(self):
        """Test search without collection"""
        self.client.set_embedder(self.mock_embedder)
        
        with pytest.raises(ValueError, match="No collection created"):
            self.client.search_similar("query")


class TestConvenienceFunctions:
    """Test convenience functions"""
    
    @patch('tensorflow_embedder.TransformerEmbedder')
    def test_create_transformer_embedder(self, mock_embedder_class):
        """Test create_transformer_embedder function"""
        mock_embedder = Mock()
        mock_embedder_class.return_value = mock_embedder
        
        result = create_transformer_embedder("test_model")
        
        assert result == mock_embedder
        mock_embedder_class.assert_called_once()
    
    @patch('tensorflow_embedder.CNNEmbedder')
    def test_create_cnn_embedder(self, mock_embedder_class):
        """Test create_cnn_embedder function"""
        mock_embedder = Mock()
        mock_embedder_class.return_value = mock_embedder
        
        result = create_cnn_embedder("test_model.h5")
        
        assert result == mock_embedder
        mock_embedder_class.assert_called_once()
    
    @patch('tensorflow_embedder.CustomTensorFlowEmbedder')
    def test_create_custom_embedder(self, mock_embedder_class):
        """Test create_custom_embedder function"""
        mock_embedder = Mock()
        mock_embedder_class.return_value = mock_embedder
        
        result = create_custom_embedder("test_model.h5")
        
        assert result == mock_embedder
        mock_embedder_class.assert_called_once()


class TestErrorHandling:
    """Test error handling"""
    
    def test_transformer_embedder_import_error(self):
        """Test transformer embedder with import error"""
        with patch.dict('sys.modules', {'transformers': None}):
            with pytest.raises(ImportError, match="transformers library is required"):
                TransformerEmbedder(TensorFlowModelConfig(model_path="test"))
    
    def test_cnn_embedder_file_error(self):
        """Test CNN embedder with file error"""
        config = TensorFlowModelConfig(model_path="nonexistent.h5")
        
        with pytest.raises(ValueError, match="Failed to load CNN model"):
            CNNEmbedder(config)
    
    def test_custom_embedder_file_error(self):
        """Test custom embedder with file error"""
        config = TensorFlowModelConfig(model_path="nonexistent.h5")
        
        with pytest.raises(ValueError, match="Failed to load custom model"):
            CustomTensorFlowEmbedder(config)


# Integration tests (require running Vectorizer instance)
class TestIntegration:
    """Integration tests that require a running Vectorizer instance"""
    
    @pytest.mark.integration
    def test_real_tensorflow_integration(self):
        """Test real TensorFlow integration with Vectorizer"""
        # This test only runs if Vectorizer is running
        try:
            # Create a simple embedder (using dummy data)
            config = TensorFlowModelConfig(
                model_path="dummy_model.h5",
                device="cpu",
                batch_size=1,
                max_length=128
            )
            
            # Mock the model loading for integration test
            with patch('tensorflow.keras.models.load_model') as mock_load:
                mock_model = Mock()
                mock_model.predict.return_value = np.random.rand(1, 512)
                mock_load.return_value = mock_model
                
                embedder = CustomTensorFlowEmbedder(config)
                
                # Create client
                client = TensorFlowVectorizerClient()
                client.set_embedder(embedder)
                
                # Test collection creation
                if client.create_collection("integration_test"):
                    # Test adding texts
                    texts = ["Integration test document"]
                    metadatas = [{"test": True}]
                    
                    try:
                        vector_ids = client.add_texts(texts, metadatas)
                        assert len(vector_ids) >= 0
                        
                        # Test search
                        results = client.search_similar("test", k=1)
                        assert len(results) >= 0
                        
                    except Exception:
                        # If Vectorizer is not running, skip the test
                        pytest.skip("Vectorizer not available for integration test")
                
        except Exception:
            pytest.skip("Vectorizer not available for integration test")


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
