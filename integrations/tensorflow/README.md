# TensorFlow Integration for Vectorizer

This integration provides seamless integration with TensorFlow models for custom embeddings and vector operations using Vectorizer as the backend.

## 🚀 Features

- ✅ **Multiple Model Types**: Support for Transformer, CNN, and Custom TensorFlow models
- ✅ **Device Flexibility**: Automatic device detection (CPU, GPU)
- ✅ **Batch Processing**: Efficient batch processing for large datasets
- ✅ **Model Management**: Easy loading and configuration of TensorFlow models
- ✅ **Vectorizer Integration**: Direct integration with Vectorizer API
- ✅ **Comprehensive Testing**: Complete test suite with mocks and integration tests
- ✅ **Performance Optimization**: Configurable batch sizes and normalization

## 📦 Installation

```bash
# Install dependencies
pip install -r requirements.txt

# Or install directly
pip install tensorflow transformers requests numpy
```

## 🔧 Configuration

### Basic Configuration

```python
from tensorflow_embedder import TensorFlowModelConfig, TransformerEmbedder

# Default configuration
config = TensorFlowModelConfig(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    device="auto",
    batch_size=32,
    max_length=512,
    normalize_embeddings=True
)

# Create embedder
embedder = TransformerEmbedder(config)
```

### Advanced Configuration

```python
config = TensorFlowModelConfig(
    model_path="path/to/your/model.h5",
    device="gpu",
    batch_size=64,
    max_length=256,
    normalize_embeddings=True,
    model_type="transformer",
    tokenizer_path="path/to/tokenizer",
    model_config={"hidden_size": 768}
)
```

## 📚 Usage

### Transformer Models

```python
from tensorflow_embedder import create_transformer_embedder, TensorFlowVectorizerClient

# Create transformer embedder
embedder = create_transformer_embedder(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    device="auto",
    batch_size=16,
    max_length=256,
    normalize_embeddings=True
)

# Create Vectorizer client
client = TensorFlowVectorizerClient()
client.set_embedder(embedder)

# Create collection
client.create_collection("tensorflow_transformer_documents")

# Add documents
texts = [
    "TensorFlow is transforming machine learning",
    "Deep learning uses neural networks",
    "Natural language processing enables text understanding"
]

metadatas = [
    {"source": "tf_ml_doc.txt", "category": "ai"},
    {"source": "tf_dl_doc.txt", "category": "ai"},
    {"source": "tf_nlp_doc.txt", "category": "ai"}
]

vector_ids = client.add_texts(texts, metadatas)
print(f"Added {len(vector_ids)} documents")

# Search for similar documents
results = client.search_similar("artificial intelligence", k=3)
for result in results:
    print(f"Score: {result['score']:.3f}")
    print(f"Content: {result['payload']['text']}")
```

### CNN Models

```python
from tensorflow_embedder import create_cnn_embedder

# Create CNN embedder
embedder = create_cnn_embedder(
    model_path="path/to/cnn_model.h5",
    device="cpu",
    batch_size=8,
    max_length=128,
    normalize_embeddings=True
)

# Use with Vectorizer client
client = TensorFlowVectorizerClient()
client.set_embedder(embedder)
client.create_collection("tensorflow_cnn_documents")

# Add and search documents
texts = ["Sample text for CNN processing"]
vector_ids = client.add_texts(texts)
results = client.search_similar("sample text", k=5)
```

### Custom Models

```python
from tensorflow_embedder import create_custom_embedder

# Create custom embedder
embedder = create_custom_embedder(
    model_path="path/to/custom_model.h5",
    device="auto",
    batch_size=4,
    max_length=100,
    normalize_embeddings=True
)

# Use with Vectorizer client
client = TensorFlowVectorizerClient()
client.set_embedder(embedder)
client.create_collection("tensorflow_custom_documents")

# Add and search documents
texts = ["Custom model processing"]
vector_ids = client.add_texts(texts)
results = client.search_similar("custom processing", k=5)
```

### Batch Processing

```python
# Process large batches efficiently
embedder = create_transformer_embedder(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    batch_size=64,  # Large batch size for efficiency
    device="gpu"    # Use GPU for faster processing
)

client = TensorFlowVectorizerClient()
client.set_embedder(embedder)
client.create_collection("tensorflow_batch_documents")

# Add many documents
large_texts = [f"Document {i}" for i in range(1000)]
large_metadatas = [{"doc_id": i} for i in range(1000)]

vector_ids = client.add_texts(large_texts, large_metadatas)
print(f"Added {len(vector_ids)} documents in batch")
```

### Device Management

```python
# Automatic device detection
embedder = create_transformer_embedder(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    device="auto"  # Automatically uses GPU or CPU
)

# Manual device specification
embedder = create_transformer_embedder(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    device="gpu"  # Force GPU
)

# Check available devices
import tensorflow as tf
print(f"Available devices: {tf.config.list_physical_devices()}")
```

## 🔧 Available Classes

### TensorFlowModelConfig

Configuration class for TensorFlow models:

- `model_path`: Path to the model file
- `device`: Device to use ("auto", "cpu", "gpu")
- `batch_size`: Batch size for processing
- `max_length`: Maximum sequence length
- `normalize_embeddings`: Whether to normalize embeddings
- `model_type`: Type of model ("transformer", "cnn", "custom")
- `tokenizer_path`: Path to tokenizer (for transformers)
- `model_config`: Additional model configuration

### TensorFlowEmbedder (Abstract Base Class)

Base class for all TensorFlow embedders:

- `embed_texts(texts: List[str])` - Generate embeddings for multiple texts
- `embed_text(text: str)` - Generate embedding for single text
- `get_embedding_dimension()` - Get embedding dimension

### TransformerEmbedder

Embedder for transformer models:

- Supports Hugging Face transformers
- Automatic tokenization
- Mean pooling or CLS token usage
- Configurable normalization

### CNNEmbedder

Embedder for CNN models:

- Character-level text processing
- Customizable preprocessing
- Configurable normalization

### CustomTensorFlowEmbedder

Embedder for custom TensorFlow models:

- Flexible model loading
- Custom preprocessing
- Extensible architecture

### TensorFlowVectorizerClient

Client for integrating TensorFlow models with Vectorizer:

- `set_embedder(embedder)` - Set the TensorFlow embedder
- `create_collection(name, dimension)` - Create Vectorizer collection
- `add_texts(texts, metadatas)` - Add texts with embeddings
- `search_similar(query, k, filter)` - Search for similar texts

## 🧪 Tests

### Run Tests

```bash
# Run all tests
python -m pytest test_tensorflow_embedder.py -v

# Run only unit tests
python -m pytest test_tensorflow_embedder.py::TestTransformerEmbedder -v

# Run integration tests (requires running Vectorizer)
python -m pytest test_tensorflow_embedder.py::TestIntegration -v -m integration
```

### Available Tests

- **TestTensorFlowModelConfig**: Configuration tests
- **TestTransformerEmbedder**: Transformer model tests
- **TestCNNEmbedder**: CNN model tests
- **TestCustomTensorFlowEmbedder**: Custom model tests
- **TestTensorFlowVectorizerClient**: Client integration tests
- **TestConvenienceFunctions**: Convenience function tests
- **TestErrorHandling**: Error handling tests
- **TestIntegration**: Integration tests with real Vectorizer

## 📋 Examples

See the `examples.py` file for complete usage examples:

- Transformer model example
- CNN model example
- Custom model example
- Batch processing example
- Device comparison example
- Model performance example
- TensorFlow features example

```bash
# Run examples
python examples.py
```

## 🔧 Vectorizer Configuration

Make sure Vectorizer is running:

```bash
# Start Vectorizer
cargo run --bin vectorizer-server

# Or use Docker
docker run -p 15001:15001 vectorizer:latest
```

## 🚨 Error Handling

```python
from tensorflow_embedder import TensorFlowModelConfig, TransformerEmbedder

try:
    config = TensorFlowModelConfig(model_path="invalid_model")
    embedder = TransformerEmbedder(config)
except ImportError as e:
    print(f"Missing dependency: {e}")
except ValueError as e:
    print(f"Configuration error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## 📈 Performance

### Recommended Optimizations

1. **Batch Size**: Use appropriate batch size (16-64 for transformers)
2. **Device**: Use GPU for faster processing
3. **Model Size**: Choose models appropriate for your hardware
4. **Normalization**: Enable normalization for better similarity search

### Performance Metrics

- **Transformer Models**: ~100-500 documents/second (GPU)
- **CNN Models**: ~200-1000 documents/second (GPU)
- **Custom Models**: Depends on model complexity
- **Latency**: <100ms for batch processing

### Memory Usage

- **Transformer Models**: 2-8GB GPU memory (depending on model size)
- **CNN Models**: 1-4GB GPU memory
- **Custom Models**: Depends on architecture

## 🔧 Development

### Adding New Model Types

1. Inherit from `TensorFlowEmbedder`
2. Implement required abstract methods
3. Add configuration options
4. Create convenience function
5. Add tests

### Model Requirements

- Must be compatible with TensorFlow 2.x
- Should support batch processing
- Output should be consistent dimensions
- Should be serializable with `model.save()`

## 🤝 Contributing

1. Fork the repository
2. Create a branch for your feature
3. Implement tests
4. Submit a Pull Request

## 📄 License

This integration follows the same license as the Vectorizer project.
