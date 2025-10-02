"""
PyTorch Integration for Vectorizer

This module provides integration with PyTorch models for custom embeddings
and vector operations using Vectorizer as the backend.
"""

import torch
import torch.nn as nn
import numpy as np
from typing import List, Dict, Any, Optional, Union, Tuple
from pathlib import Path
import json
import logging
from dataclasses import dataclass
from abc import ABC, abstractmethod

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class PyTorchModelConfig:
    """Configuration for PyTorch model integration"""
    model_path: str
    device: str = "auto"
    batch_size: int = 32
    max_length: int = 512
    normalize_embeddings: bool = True
    model_type: str = "transformer"  # transformer, cnn, rnn, custom
    tokenizer_path: Optional[str] = None
    model_config: Optional[Dict[str, Any]] = None


class PyTorchEmbedder(ABC):
    """Abstract base class for PyTorch embedders"""
    
    def __init__(self, config: PyTorchModelConfig):
        self.config = config
        self.device = self._setup_device()
        self.model = None
        self.tokenizer = None
        self._load_model()
    
    def _setup_device(self) -> torch.device:
        """Setup device for model execution"""
        if self.config.device == "auto":
            if torch.cuda.is_available():
                device = torch.device("cuda")
                logger.info(f"Using CUDA device: {torch.cuda.get_device_name()}")
            elif torch.backends.mps.is_available():
                device = torch.device("mps")
                logger.info("Using MPS device")
            else:
                device = torch.device("cpu")
                logger.info("Using CPU device")
        else:
            device = torch.device(self.config.device)
            logger.info(f"Using specified device: {device}")
        
        return device
    
    @abstractmethod
    def _load_model(self):
        """Load the PyTorch model"""
        pass
    
    @abstractmethod
    def embed_texts(self, texts: List[str]) -> np.ndarray:
        """Generate embeddings for a list of texts"""
        pass
    
    @abstractmethod
    def embed_text(self, text: str) -> np.ndarray:
        """Generate embedding for a single text"""
        pass
    
    def get_embedding_dimension(self) -> int:
        """Get the dimension of the embeddings"""
        if self.model is None:
            raise ValueError("Model not loaded")
        return self._get_model_dimension()
    
    @abstractmethod
    def _get_model_dimension(self) -> int:
        """Get the dimension of the model's output"""
        pass


class TransformerEmbedder(PyTorchEmbedder):
    """PyTorch embedder for transformer models"""
    
    def _load_model(self):
        """Load transformer model and tokenizer"""
        try:
            from transformers import AutoModel, AutoTokenizer
            
            # Load tokenizer
            if self.config.tokenizer_path:
                self.tokenizer = AutoTokenizer.from_pretrained(self.config.tokenizer_path)
            else:
                self.tokenizer = AutoTokenizer.from_pretrained(self.config.model_path)
            
            # Load model
            self.model = AutoModel.from_pretrained(self.config.model_path)
            self.model.to(self.device)
            self.model.eval()
            
            logger.info(f"Loaded transformer model: {self.config.model_path}")
            
        except ImportError:
            raise ImportError("transformers library is required for transformer models. Install with: pip install transformers")
        except Exception as e:
            raise ValueError(f"Failed to load transformer model: {e}")
    
    def embed_texts(self, texts: List[str]) -> np.ndarray:
        """Generate embeddings for a list of texts"""
        embeddings = []
        
        # Process in batches
        for i in range(0, len(texts), self.config.batch_size):
            batch_texts = texts[i:i + self.config.batch_size]
            batch_embeddings = self._embed_batch(batch_texts)
            embeddings.append(batch_embeddings)
        
        return np.vstack(embeddings)
    
    def embed_text(self, text: str) -> np.ndarray:
        """Generate embedding for a single text"""
        return self.embed_texts([text])[0]
    
    def _embed_batch(self, texts: List[str]) -> np.ndarray:
        """Generate embeddings for a batch of texts"""
        with torch.no_grad():
            # Tokenize texts
            inputs = self.tokenizer(
                texts,
                padding=True,
                truncation=True,
                max_length=self.config.max_length,
                return_tensors="pt"
            )
            
            # Move to device
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Get model outputs
            outputs = self.model(**inputs)
            
            # Extract embeddings (use [CLS] token or mean pooling)
            if hasattr(outputs, 'last_hidden_state'):
                # Mean pooling
                attention_mask = inputs['attention_mask']
                embeddings = outputs.last_hidden_state
                mask_expanded = attention_mask.unsqueeze(-1).expand(embeddings.size()).float()
                sum_embeddings = torch.sum(embeddings * mask_expanded, 1)
                sum_mask = torch.clamp(mask_expanded.sum(1), min=1e-9)
                embeddings = sum_embeddings / sum_mask
            else:
                # Use pooler output if available
                embeddings = outputs.pooler_output
            
            # Normalize if configured
            if self.config.normalize_embeddings:
                embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)
            
            return embeddings.cpu().numpy()
    
    def _get_model_dimension(self) -> int:
        """Get the dimension of the model's output"""
        return self.model.config.hidden_size


class CNNEmbedder(PyTorchEmbedder):
    """PyTorch embedder for CNN models"""
    
    def _load_model(self):
        """Load CNN model"""
        try:
            # Load model from file
            model_path = Path(self.config.model_path)
            if model_path.suffix == '.pt' or model_path.suffix == '.pth':
                self.model = torch.load(model_path, map_location=self.device)
            else:
                raise ValueError("CNN model must be saved as .pt or .pth file")
            
            self.model.to(self.device)
            self.model.eval()
            
            logger.info(f"Loaded CNN model: {self.config.model_path}")
            
        except Exception as e:
            raise ValueError(f"Failed to load CNN model: {e}")
    
    def embed_texts(self, texts: List[str]) -> np.ndarray:
        """Generate embeddings for a list of texts"""
        # For CNN models, we need to convert text to numerical representation
        # This is a simplified example - in practice, you'd need proper text preprocessing
        embeddings = []
        
        for text in texts:
            # Simple character-level encoding (replace with proper preprocessing)
            text_vector = self._text_to_vector(text)
            embedding = self._embed_vector(text_vector)
            embeddings.append(embedding)
        
        return np.array(embeddings)
    
    def embed_text(self, text: str) -> np.ndarray:
        """Generate embedding for a single text"""
        return self.embed_texts([text])[0]
    
    def _text_to_vector(self, text: str) -> torch.Tensor:
        """Convert text to numerical vector for CNN input"""
        # Simple character-level encoding
        chars = list(text[:self.config.max_length])
        char_to_idx = {chr(i): i for i in range(128)}  # ASCII characters
        
        vector = [char_to_idx.get(char, 0) for char in chars]
        # Pad to max_length
        while len(vector) < self.config.max_length:
            vector.append(0)
        
        return torch.tensor(vector, dtype=torch.long).unsqueeze(0).to(self.device)
    
    def _embed_vector(self, vector: torch.Tensor) -> np.ndarray:
        """Generate embedding from vector"""
        with torch.no_grad():
            output = self.model(vector)
            if isinstance(output, tuple):
                output = output[0]
            
            # Flatten and normalize
            embedding = output.view(-1).cpu().numpy()
            if self.config.normalize_embeddings:
                embedding = embedding / np.linalg.norm(embedding)
            
            return embedding
    
    def _get_model_dimension(self) -> int:
        """Get the dimension of the model's output"""
        # This would need to be determined from the model architecture
        # For now, return a default value
        return 512


class CustomPyTorchEmbedder(PyTorchEmbedder):
    """PyTorch embedder for custom models"""
    
    def _load_model(self):
        """Load custom model"""
        try:
            # Load model from file
            model_path = Path(self.config.model_path)
            if model_path.suffix == '.pt' or model_path.suffix == '.pth':
                self.model = torch.load(model_path, map_location=self.device)
            else:
                raise ValueError("Custom model must be saved as .pt or .pth file")
            
            self.model.to(self.device)
            self.model.eval()
            
            logger.info(f"Loaded custom model: {self.config.model_path}")
            
        except Exception as e:
            raise ValueError(f"Failed to load custom model: {e}")
    
    def embed_texts(self, texts: List[str]) -> np.ndarray:
        """Generate embeddings for a list of texts"""
        embeddings = []
        
        for text in texts:
            # Preprocess text according to model requirements
            processed_text = self._preprocess_text(text)
            embedding = self._embed_processed_text(processed_text)
            embeddings.append(embedding)
        
        return np.array(embeddings)
    
    def embed_text(self, text: str) -> np.ndarray:
        """Generate embedding for a single text"""
        return self.embed_texts([text])[0]
    
    def _preprocess_text(self, text: str) -> Any:
        """Preprocess text for custom model"""
        # This should be implemented based on your model's requirements
        # For now, return the text as-is
        return text
    
    def _embed_processed_text(self, processed_text: Any) -> np.ndarray:
        """Generate embedding from processed text"""
        with torch.no_grad():
            # This should be implemented based on your model's forward pass
            # For now, return a dummy embedding
            dummy_embedding = torch.randn(512).to(self.device)
            if self.config.normalize_embeddings:
                dummy_embedding = torch.nn.functional.normalize(dummy_embedding, p=2, dim=0)
            return dummy_embedding.cpu().numpy()
    
    def _get_model_dimension(self) -> int:
        """Get the dimension of the model's output"""
        # This should be determined from your model architecture
        return 512


class PyTorchVectorizerClient:
    """Client for integrating PyTorch models with Vectorizer"""
    
    def __init__(self, vectorizer_url: str = "http://localhost:15001", api_key: Optional[str] = None):
        self.vectorizer_url = vectorizer_url
        self.api_key = api_key
        self.embedder = None
        self.collection_name = None
    
    def set_embedder(self, embedder: PyTorchEmbedder):
        """Set the PyTorch embedder"""
        self.embedder = embedder
        logger.info(f"Set PyTorch embedder: {type(embedder).__name__}")
    
    def create_collection(self, collection_name: str, dimension: Optional[int] = None) -> bool:
        """Create a collection in Vectorizer"""
        try:
            import requests
            
            if dimension is None and self.embedder:
                dimension = self.embedder.get_embedding_dimension()
            elif dimension is None:
                dimension = 512  # Default dimension
            
            url = f"{self.vectorizer_url}/api/v1/collections"
            headers = {"Content-Type": "application/json"}
            if self.api_key:
                headers["Authorization"] = f"Bearer {self.api_key}"
            
            data = {
                "name": collection_name,
                "dimension": dimension,
                "metric": "cosine",
                "embedding_model": "custom_pytorch"
            }
            
            response = requests.post(url, json=data, headers=headers)
            response.raise_for_status()
            
            self.collection_name = collection_name
            logger.info(f"Created collection: {collection_name}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to create collection: {e}")
            return False
    
    def add_texts(self, texts: List[str], metadatas: Optional[List[Dict[str, Any]]] = None) -> List[str]:
        """Add texts to the collection using PyTorch embeddings"""
        if not self.embedder:
            raise ValueError("No embedder set. Call set_embedder() first.")
        
        if not self.collection_name:
            raise ValueError("No collection created. Call create_collection() first.")
        
        try:
            import requests
            
            # Generate embeddings
            embeddings = self.embedder.embed_texts(texts)
            
            # Prepare data for Vectorizer
            vectors_data = []
            for i, (text, embedding) in enumerate(zip(texts, embeddings)):
                metadata = metadatas[i] if metadatas else {}
                metadata["text"] = text
                metadata["embedder"] = "pytorch"
                
                vectors_data.append({
                    "vector": embedding.tolist(),
                    "payload": metadata
                })
            
            # Send to Vectorizer
            url = f"{self.vectorizer_url}/api/v1/collections/{self.collection_name}/vectors/batch"
            headers = {"Content-Type": "application/json"}
            if self.api_key:
                headers["Authorization"] = f"Bearer {self.api_key}"
            
            data = {"vectors": vectors_data}
            
            response = requests.post(url, json=data, headers=headers)
            response.raise_for_status()
            
            result = response.json()
            vector_ids = result.get("vector_ids", [])
            
            logger.info(f"Added {len(vector_ids)} texts to collection {self.collection_name}")
            return vector_ids
            
        except Exception as e:
            logger.error(f"Failed to add texts: {e}")
            raise
    
    def search_similar(self, query: str, k: int = 5, filter: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """Search for similar texts using PyTorch embeddings"""
        if not self.embedder:
            raise ValueError("No embedder set. Call set_embedder() first.")
        
        if not self.collection_name:
            raise ValueError("No collection created. Call create_collection() first.")
        
        try:
            import requests
            
            # Generate query embedding
            query_embedding = self.embedder.embed_text(query)
            
            # Prepare search request
            url = f"{self.vectorizer_url}/api/v1/collections/{self.collection_name}/search"
            headers = {"Content-Type": "application/json"}
            if self.api_key:
                headers["Authorization"] = f"Bearer {self.api_key}"
            
            data = {
                "query_vector": query_embedding.tolist(),
                "top_k": k,
                "filter": filter or {}
            }
            
            response = requests.post(url, json=data, headers=headers)
            response.raise_for_status()
            
            result = response.json()
            return result.get("results", [])
            
        except Exception as e:
            logger.error(f"Failed to search: {e}")
            raise


def create_transformer_embedder(
    model_path: str,
    device: str = "auto",
    batch_size: int = 32,
    max_length: int = 512,
    normalize_embeddings: bool = True,
    tokenizer_path: Optional[str] = None
) -> TransformerEmbedder:
    """Create a transformer embedder"""
    config = PyTorchModelConfig(
        model_path=model_path,
        device=device,
        batch_size=batch_size,
        max_length=max_length,
        normalize_embeddings=normalize_embeddings,
        tokenizer_path=tokenizer_path,
        model_type="transformer"
    )
    return TransformerEmbedder(config)


def create_cnn_embedder(
    model_path: str,
    device: str = "auto",
    batch_size: int = 32,
    max_length: int = 512,
    normalize_embeddings: bool = True
) -> CNNEmbedder:
    """Create a CNN embedder"""
    config = PyTorchModelConfig(
        model_path=model_path,
        device=device,
        batch_size=batch_size,
        max_length=max_length,
        normalize_embeddings=normalize_embeddings,
        model_type="cnn"
    )
    return CNNEmbedder(config)


def create_custom_embedder(
    model_path: str,
    device: str = "auto",
    batch_size: int = 32,
    max_length: int = 512,
    normalize_embeddings: bool = True
) -> CustomPyTorchEmbedder:
    """Create a custom embedder"""
    config = PyTorchModelConfig(
        model_path=model_path,
        device=device,
        batch_size=batch_size,
        max_length=max_length,
        normalize_embeddings=normalize_embeddings,
        model_type="custom"
    )
    return CustomPyTorchEmbedder(config)


# Example usage
if __name__ == "__main__":
    # Example: Using a transformer model
    try:
        # Create transformer embedder
        embedder = create_transformer_embedder(
            model_path="sentence-transformers/all-MiniLM-L6-v2",
            device="auto",
            batch_size=16
        )
        
        # Create Vectorizer client
        client = PyTorchVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        client.create_collection("pytorch_documents")
        
        # Add texts
        texts = [
            "This is a sample document about machine learning",
            "PyTorch is a popular deep learning framework",
            "Vector databases are essential for AI applications"
        ]
        
        metadatas = [
            {"source": "doc1.txt", "topic": "ml"},
            {"source": "doc2.txt", "topic": "pytorch"},
            {"source": "doc3.txt", "topic": "vectors"}
        ]
        
        vector_ids = client.add_texts(texts, metadatas)
        print(f"Added {len(vector_ids)} documents")
        
        # Search
        results = client.search_similar("deep learning frameworks", k=2)
        print(f"Found {len(results)} similar documents")
        
    except Exception as e:
        print(f"Error: {e}")
