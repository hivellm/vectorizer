"""
Examples for TensorFlow Integration with Vectorizer

This file demonstrates various usage patterns for the TensorFlow integration
with Vectorizer for custom embeddings and vector operations.
"""

import tensorflow as tf
import numpy as np
from tensorflow_embedder import (
    TensorFlowModelConfig,
    TransformerEmbedder,
    CNNEmbedder,
    CustomTensorFlowEmbedder,
    TensorFlowVectorizerClient,
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
        client = TensorFlowVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("tensorflow_transformer_documents"):
            print("Created collection: tensorflow_transformer_documents")
            
            # Add documents
            texts = [
                "TensorFlow is a popular machine learning framework",
                "Deep learning models can be trained with TensorFlow",
                "Neural networks are the foundation of deep learning",
                "TensorFlow supports both CPU and GPU computation"
            ]
            
            metadatas = [
                {"source": "tf_intro.txt", "category": "framework", "year": 2023},
                {"source": "tf_training.txt", "category": "framework", "year": 2023},
                {"source": "neural_nets.txt", "category": "theory", "year": 2023},
                {"source": "tf_hardware.txt", "category": "framework", "year": 2023}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "machine learning frameworks and neural networks"
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
        def create_simple_cnn_model(vocab_size=128, embed_dim=64, hidden_dim=256, output_dim=512):
            model = tf.keras.Sequential([
                tf.keras.layers.Embedding(vocab_size, embed_dim, input_length=128),
                tf.keras.layers.Conv1D(hidden_dim, 3, activation='relu', padding='same'),
                tf.keras.layers.Conv1D(hidden_dim, 3, activation='relu', padding='same'),
                tf.keras.layers.GlobalAveragePooling1D(),
                tf.keras.layers.Dense(output_dim, activation='linear')
            ])
            return model
        
        # Create and save a dummy CNN model
        model = create_simple_cnn_model()
        model.save("dummy_cnn_model.h5")
        
        # Create CNN embedder
        embedder = create_cnn_embedder(
            model_path="dummy_cnn_model.h5",
            device="cpu",
            batch_size=8,
            max_length=128,
            normalize_embeddings=True
        )
        
        print(f"Embedding dimension: {embedder.get_embedding_dimension()}")
        
        # Create Vectorizer client
        client = TensorFlowVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("tensorflow_cnn_documents"):
            print("Created collection: tensorflow_cnn_documents")
            
            # Add documents
            texts = [
                "This is a sample text for CNN processing with TensorFlow",
                "CNN models are excellent for text classification tasks",
                "Convolutional neural networks process local patterns effectively",
                "Text CNN can capture n-gram features in TensorFlow"
            ]
            
            metadatas = [
                {"source": "tf_cnn_doc1.txt", "model_type": "cnn"},
                {"source": "tf_cnn_doc2.txt", "model_type": "cnn"},
                {"source": "tf_cnn_doc3.txt", "model_type": "cnn"},
                {"source": "tf_cnn_doc4.txt", "model_type": "cnn"}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "neural networks for text processing"
            results = client.search_similar(query, k=2)
            
            print(f"\nSearch results for '{query}':")
            for i, result in enumerate(results, 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')}")
                print()
        
        # Clean up
        import os
        if os.path.exists("dummy_cnn_model.h5"):
            os.remove("dummy_cnn_model.h5")
        
    except Exception as e:
        print(f"Error in CNN example: {e}")


def custom_model_example():
    """Example using custom models"""
    print("\n=== Custom Model Example ===")
    
    try:
        # Create a simple custom model
        def create_custom_model(input_dim=100, hidden_dim=256, output_dim=512):
            model = tf.keras.Sequential([
                tf.keras.layers.Dense(hidden_dim, activation='relu', input_shape=(input_dim,)),
                tf.keras.layers.Dropout(0.1),
                tf.keras.layers.Dense(hidden_dim, activation='relu'),
                tf.keras.layers.Dropout(0.1),
                tf.keras.layers.Dense(output_dim, activation='linear')
            ])
            return model
        
        # Create and save a dummy custom model
        model = create_custom_model()
        model.save("dummy_custom_model.h5")
        
        # Create custom embedder
        embedder = create_custom_embedder(
            model_path="dummy_custom_model.h5",
            device="cpu",
            batch_size=4,
            max_length=100,
            normalize_embeddings=True
        )
        
        print(f"Embedding dimension: {embedder.get_embedding_dimension()}")
        
        # Create Vectorizer client
        client = TensorFlowVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("tensorflow_custom_documents"):
            print("Created collection: tensorflow_custom_documents")
            
            # Add documents
            texts = [
                "Custom TensorFlow models provide flexibility for specific tasks",
                "TensorFlow allows easy model customization and deployment",
                "Vector databases work excellently with custom TensorFlow embeddings",
                "Machine learning models can be tailored to specific domains in TensorFlow"
            ]
            
            metadatas = [
                {"source": "tf_custom_doc1.txt", "model_type": "custom"},
                {"source": "tf_custom_doc2.txt", "model_type": "custom"},
                {"source": "tf_custom_doc3.txt", "model_type": "custom"},
                {"source": "tf_custom_doc4.txt", "model_type": "custom"}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "custom TensorFlow machine learning models"
            results = client.search_similar(query, k=2)
            
            print(f"\nSearch results for '{query}':")
            for i, result in enumerate(results, 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')}")
                print()
        
        # Clean up
        import os
        if os.path.exists("dummy_custom_model.h5"):
            os.remove("dummy_custom_model.h5")
        
    except Exception as e:
        print(f"Error in custom model example: {e}")


def batch_processing_example():
    """Example of batch processing with TensorFlow"""
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
        client = TensorFlowVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("tensorflow_batch_documents"):
            print("Created collection: tensorflow_batch_documents")
            
            # Generate large batch of documents
            texts = []
            metadatas = []
            
            for i in range(100):
                texts.append(f"Document {i}: This is sample content for document number {i} about TensorFlow machine learning and artificial intelligence.")
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
                "TensorFlow machine learning algorithms",
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
        if tf.config.list_physical_devices('GPU'):
            devices.append("gpu")
        
        texts = [
            "This is a test document for device comparison with TensorFlow",
            "TensorFlow supports multiple devices for computation",
            "GPU acceleration can significantly speed up TensorFlow model inference"
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
        texts = [f"Test document {i} for TensorFlow performance evaluation" for i in range(50)]
        
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


def tensorflow_features_example():
    """Example demonstrating TensorFlow-specific features"""
    print("\n=== TensorFlow Features Example ===")
    
    try:
        # Show TensorFlow version and available devices
        print(f"TensorFlow version: {tf.__version__}")
        print(f"Available devices: {tf.config.list_physical_devices()}")
        
        # Create embedder
        embedder = create_transformer_embedder(
            model_path="sentence-transformers/all-MiniLM-L6-v2",
            device="auto",
            batch_size=8
        )
        
        # Create Vectorizer client
        client = TensorFlowVectorizerClient()
        client.set_embedder(embedder)
        
        # Create collection
        if client.create_collection("tensorflow_features_documents"):
            print("Created collection: tensorflow_features_documents")
            
            # Add documents
            texts = [
                "TensorFlow provides comprehensive machine learning tools",
                "TensorFlow 2.x offers eager execution by default",
                "TensorFlow Hub provides pre-trained models",
                "TensorFlow Lite enables mobile and edge deployment"
            ]
            
            metadatas = [
                {"source": "tf_tools.txt", "version": "2.x"},
                {"source": "tf_execution.txt", "version": "2.x"},
                {"source": "tf_hub.txt", "version": "2.x"},
                {"source": "tf_lite.txt", "version": "2.x"}
            ]
            
            vector_ids = client.add_texts(texts, metadatas)
            print(f"Added {len(vector_ids)} documents")
            
            # Search for similar documents
            query = "TensorFlow machine learning tools and deployment"
            results = client.search_similar(query, k=3)
            
            print(f"\nSearch results for '{query}':")
            for i, result in enumerate(results, 1):
                print(f"{i}. Score: {result.get('score', 0):.3f}")
                print(f"   Content: {result.get('payload', {}).get('text', 'N/A')}")
                print(f"   Version: {result.get('payload', {}).get('version', 'N/A')}")
        
    except Exception as e:
        print(f"Error in TensorFlow features example: {e}")


def run_all_examples():
    """Run all examples"""
    print("TensorFlow Integration with Vectorizer Examples")
    print("=" * 50)
    
    try:
        transformer_example()
        cnn_example()
        custom_model_example()
        batch_processing_example()
        device_comparison_example()
        model_performance_example()
        tensorflow_features_example()
        
        print("\n" + "=" * 50)
        print("All examples completed successfully!")
        
    except Exception as e:
        print(f"Error running examples: {e}")
        print("Make sure Vectorizer is running on localhost:15002")
        print("Make sure you have required dependencies installed:")
        print("  pip install tensorflow transformers requests")


# Export functions for individual testing
export_functions = [
    transformer_example,
    cnn_example,
    custom_model_example,
    batch_processing_example,
    device_comparison_example,
    model_performance_example,
    tensorflow_features_example,
    run_all_examples
]

if __name__ == "__main__":
    run_all_examples()
