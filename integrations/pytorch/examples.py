"""
Examples for PyTorch Integration with Vectorizer

This file demonstrates various usage patterns for the PyTorch integration
with Vectorizer for custom embeddings and vector operations.
"""

import torch
import numpy as np
from pytorch_embedder import (
    PyTorchModelConfig,
    TransformerEmbedder,
    CNNEmbedder,
    CustomPyTorchEmbedder,
    PyTorchVectorizerClient,
    create_transformer_embedder,
    create_cnn_embedder,
    create_custom_embedder
)


def transformer_example():
    """Example using transformer models"""
    print("=== Transformer Model Example ===")
    
    try:
        # Create transformer embedder
        embedder = create_transformer_embedder(
            model_path="sentence-transformers/all-MiniLM-L6-v2",
            device="auto",
            batch_size=16,
            max_length=256,
            normalize_embeddings=True
        )
        
        print(f"Embedding dimension: {embedder.get_embedding_dimension()}")
        
        # Create Vectorizer client
        client = PyTorchVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("transformer_documents"):
            print("Created collection: transformer_documents")
            
            # Add documents
            texts = [
                "Machine learning is a subset of artificial intelligence",
                "Deep learning uses neural networks with multiple layers",
                "Natural language processing enables computers to understand text",
                "Computer vision allows machines to interpret visual information"
            ]
            
            metadatas = [
                {"source": "ml_intro.txt", "category": "ai", "year": 2023},
                {"source": "dl_basics.txt", "category": "ai", "year": 2023},
                {"source": "nlp_guide.txt", "category": "ai", "year": 2023},
                {"source": "cv_overview.txt", "category": "ai", "year": 2023}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "neural networks and artificial intelligence"
            results = client.search_similar(query, k=3)
            
            print(f"\nSearch results for '{query}':")
            for i, result in enumerate(results, 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')}")
                print(f"   Metadata: {result.get('payload', {})}")
                print()
        
    except Exception as e:
        print(f"Error in transformer example: {e}")
        print("Make sure you have transformers library installed: pip install transformers")


def cnn_example():
    """Example using CNN models"""
    print("\n=== CNN Model Example ===")
    
    try:
        # Create a simple CNN model for demonstration
        class SimpleCNN(torch.nn.Module):
            def __init__(self, vocab_size=128, embed_dim=64, hidden_dim=256, output_dim=512):
                super().__init__()
                self.embedding = torch.nn.Embedding(vocab_size, embed_dim)
                self.conv1 = torch.nn.Conv1d(embed_dim, hidden_dim, kernel_size=3, padding=1)
                self.conv2 = torch.nn.Conv1d(hidden_dim, hidden_dim, kernel_size=3, padding=1)
                self.pool = torch.nn.AdaptiveAvgPool1d(1)
                self.fc = torch.nn.Linear(hidden_dim, output_dim)
                self.dropout = torch.nn.Dropout(0.1)
            
            def forward(self, x):
                # x: (batch_size, seq_len)
                x = self.embedding(x)  # (batch_size, seq_len, embed_dim)
                x = x.transpose(1, 2)  # (batch_size, embed_dim, seq_len)
                x = torch.relu(self.conv1(x))
                x = torch.relu(self.conv2(x))
                x = self.pool(x)  # (batch_size, hidden_dim, 1)
                x = x.squeeze(-1)  # (batch_size, hidden_dim)
                x = self.dropout(x)
                x = self.fc(x)  # (batch_size, output_dim)
                return x
        
        # Create and save a dummy CNN model
        model = SimpleCNN()
        torch.save(model, "dummy_cnn_model.pt")
        
        # Create CNN embedder
        embedder = create_cnn_embedder(
            model_path="dummy_cnn_model.pt",
            device="cpu",
            batch_size=8,
            max_length=128,
            normalize_embeddings=True
        )
        
        print(f"Embedding dimension: {embedder.get_embedding_dimension()}")
        
        # Create Vectorizer client
        client = PyTorchVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("cnn_documents"):
            print("Created collection: cnn_documents")
            
            # Add documents
            texts = [
                "This is a sample text for CNN processing",
                "CNN models are good for text classification",
                "Convolutional neural networks process local patterns",
                "Text CNN can capture n-gram features effectively"
            ]
            
            metadatas = [
                {"source": "cnn_doc1.txt", "model_type": "cnn"},
                {"source": "cnn_doc2.txt", "model_type": "cnn"},
                {"source": "cnn_doc3.txt", "model_type": "cnn"},
                {"source": "cnn_doc4.txt", "model_type": "cnn"}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "neural networks for text"
            results = client.search_similar(query, k=2)
            
            print(f"\nSearch results for '{query}':")
            for i, result in enumerate(results, 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')}")
                print()
        
        # Clean up
        import os
        if os.path.exists("dummy_cnn_model.pt"):
            os.remove("dummy_cnn_model.pt")
        
    except Exception as e:
        print(f"Error in CNN example: {e}")


def custom_model_example():
    """Example using custom models"""
    print("\n=== Custom Model Example ===")
    
    try:
        # Create a simple custom model
        class CustomEmbedder(torch.nn.Module):
            def __init__(self, input_dim=100, hidden_dim=256, output_dim=512):
                super().__init__()
                self.fc1 = torch.nn.Linear(input_dim, hidden_dim)
                self.fc2 = torch.nn.Linear(hidden_dim, hidden_dim)
                self.fc3 = torch.nn.Linear(hidden_dim, output_dim)
                self.dropout = torch.nn.Dropout(0.1)
                self.relu = torch.nn.ReLU()
            
            def forward(self, x):
                x = self.relu(self.fc1(x))
                x = self.dropout(x)
                x = self.relu(self.fc2(x))
                x = self.dropout(x)
                x = self.fc3(x)
                return x
        
        # Create and save a dummy custom model
        model = CustomEmbedder()
        torch.save(model, "dummy_custom_model.pt")
        
        # Create custom embedder
        embedder = create_custom_embedder(
            model_path="dummy_custom_model.pt",
            device="cpu",
            batch_size=4,
            max_length=100,
            normalize_embeddings=True
        )
        
        print(f"Embedding dimension: {embedder.get_embedding_dimension()}")
        
        # Create Vectorizer client
        client = PyTorchVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("custom_documents"):
            print("Created collection: custom_documents")
            
            # Add documents
            texts = [
                "Custom models provide flexibility for specific tasks",
                "PyTorch allows easy model customization",
                "Vector databases work well with custom embeddings",
                "Machine learning models can be tailored to specific domains"
            ]
            
            metadatas = [
                {"source": "custom_doc1.txt", "model_type": "custom"},
                {"source": "custom_doc2.txt", "model_type": "custom"},
                {"source": "custom_doc3.txt", "model_type": "custom"},
                {"source": "custom_doc4.txt", "model_type": "custom"}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "custom machine learning models"
            results = client.search_similar(query, k=2)
            
            print(f"\nSearch results for '{query}':")
            for i, result in enumerate(results, 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')}")
                print()
        
        # Clean up
        import os
        if os.path.exists("dummy_custom_model.pt"):
            os.remove("dummy_custom_model.pt")
        
    except Exception as e:
        print(f"Error in custom model example: {e}")


def batch_processing_example():
    """Example of batch processing with PyTorch"""
    print("\n=== Batch Processing Example ===")
    
    try:
        # Create transformer embedder with larger batch size
        embedder = create_transformer_embedder(
            model_path="sentence-transformers/all-MiniLM-L6-v2",
            device="auto",
            batch_size=32,  # Larger batch size for efficiency
            max_length=256,
            normalize_embeddings=True
        )
        
        # Create Vectorizer client
        client = PyTorchVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("batch_documents"):
            print("Created collection: batch_documents")
            
            # Generate large batch of documents
            texts = []
            metadatas = []
            
            for i in range(100):
                texts.append(f"Document {i}: This is sample content for document number {i} about machine learning and artificial intelligence.")
                metadatas.append({
                    "doc_id": i,
                    "batch": "example",
                    "category": "ml" if i % 2 == 0 else "ai"
                })
            
            print(f"Processing {len(texts)} documents in batch...")
            
            # Add documents in batch
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search with different queries
            queries = [
                "machine learning algorithms",
                "artificial intelligence applications",
                "neural networks and deep learning"
            ]
            
            for query in queries:
                results = client.search_similar(query, k=5)
                print(f"\nSearch results for '{query}':")
                for i, result in enumerate(results[:3], 1):  # Show top 3
                    print(f"{i}. Score: {result.get('score', 0):.3f}")
                    print(f"   Content: {result.get('payload', {}).get('text', 'N/A')[:80]}...")
            
            # Search with metadata filter
            print(f"\nSearch with filter (category=ml):")
            filtered_results = client.search_similar("machine learning", k=5, filter={"category": "ml"})
            for i, result in enumerate(filtered_results[:3], 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')[:80]}...")
        
    except Exception as e:
        print(f"Error in batch processing example: {e}")


def device_comparison_example():
    """Example comparing different devices"""
    print("\n=== Device Comparison Example ===")
    
    try:
        devices = ["cpu"]
        if torch.cuda.is_available():
            devices.append("gpu")
        if torch.backends.mps.is_available():
            devices.append("mps")
        
        texts = [
            "This is a test document for device comparison",
            "PyTorch supports multiple devices for computation",
            "GPU acceleration can significantly speed up model inference"
        ]
        
        for device in devices:
            print(f"\nTesting with device: {device}")
            
            try:
                # Create embedder for specific device
                embedder = create_transformer_embedder(
                    model_path="sentence-transformers/all-MiniLM-L6-v2",
                    device=device,
                    batch_size=8,
                    max_length=128
                )
                
                # Time the embedding generation
                import time
                start_time = time.time()
                
                embeddings = embedder.embed_texts(texts)
                
                end_time = time.time()
                duration = end_time - start_time
                
                print(f"  Device: {device}")
                print(f"  Embedding shape: {embeddings.shape}")
                print(f"  Time taken: {duration:.3f} seconds")
                print(f"  Embedding dimension: {embedder.get_embedding_dimension()}")
                
            except Exception as e:
                print(f"  Error with device {device}: {e}")
        
    except Exception as e:
        print(f"Error in device comparison example: {e}")


def model_performance_example():
    """Example demonstrating model performance metrics"""
    print("\n=== Model Performance Example ===")
    
    try:
        # Create embedder
        embedder = create_transformer_embedder(
            model_path="sentence-transformers/all-MiniLM-L6-v2",
            device="auto",
            batch_size=16,
            max_length=256
        )
        
        # Test with different batch sizes
        texts = [f"Test document {i} for performance evaluation" for i in range(50)]
        
        batch_sizes = [1, 8, 16, 32]
        
        for batch_size in batch_sizes:
            print(f"\nTesting batch size: {batch_size}")
            
            # Update embedder batch size
            embedder.config.batch_size = batch_size
            
            import time
            start_time = time.time()
            
            embeddings = embedder.embed_texts(texts)
            
            end_time = time.time()
            duration = end_time - start_time
            
            print(f"  Batch size: {batch_size}")
            print(f"  Total documents: {len(texts)}")
            print(f"  Time taken: {duration:.3f} seconds")
            print(f"  Documents per second: {len(texts) / duration:.1f}")
            print(f"  Embedding shape: {embeddings.shape}")
        
    except Exception as e:
        print(f"Error in model performance example: {e}")


def run_all_examples():
    """Run all examples"""
    print("PyTorch Integration with Vectorizer Examples")
    print("=" * 50)
    
    try:
        transformer_example()
        cnn_example()
        custom_model_example()
        batch_processing_example()
        device_comparison_example()
        model_performance_example()
        
        print("\n" + "=" * 50)
        print("All examples completed successfully!")
        
    except Exception as e:
        print(f"Error running examples: {e}")
        print("Make sure Vectorizer is running on localhost:15002")
        print("Make sure you have required dependencies installed:")
        print("  pip install torch transformers requests")


# Export functions for individual testing
export_functions = [
    transformer_example,
    cnn_example,
    custom_model_example,
    batch_processing_example,
    device_comparison_example,
    model_performance_example,
    run_all_examples
]

if __name__ == "__main__":
    run_all_examples()
