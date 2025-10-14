# Vectorizer Integrations v0.3.1

This directory contains integrations with external frameworks and libraries for Vectorizer v0.3.1.

## 📁 Structure

```
integrations/
├── langchain/          # LangChain integration (Python)
├── langchain-js/       # LangChain.js integration (JavaScript/TypeScript)
├── pytorch/            # PyTorch integration for custom embeddings
└── tensorflow/         # TensorFlow integration for custom embeddings
```

## 🚀 Available Integrations

### LangChain Integration (Python)
- **Location**: `integrations/langchain/`
- **Language**: Python
- **Description**: Complete VectorStore implementation for LangChain
- **Status**: ✅ Complete (v0.3.1)
- **Features**: 
  - Complete VectorStore interface
  - Intelligent search with multi-query expansion
  - Semantic search with advanced reranking
  - Contextual search with metadata filtering
  - Multi-collection search with cross-collection reranking
  - Batch operations
  - Metadata filtering
  - Robust error handling
  - Comprehensive tests
  - Updated for v0.3.1 API with intelligent search features

### LangChain.js Integration (JavaScript/TypeScript)
- **Location**: `integrations/langchain-js/`
- **Language**: JavaScript/TypeScript
- **Description**: Complete VectorStore implementation for LangChain.js
- **Status**: ✅ Complete (v0.3.1)
- **Features**:
  - Full TypeScript support
  - Complete VectorStore interface
  - Intelligent search with multi-query expansion
  - Semantic search with advanced reranking
  - Contextual search with metadata filtering
  - Multi-collection search with cross-collection reranking
  - Async operations
  - Flexible configuration
  - Jest tests
  - Updated for v0.3.1 API with intelligent search features

### PyTorch Integration
- **Location**: `integrations/pytorch/`
- **Language**: Python
- **Description**: Support for custom PyTorch models
- **Status**: ✅ Complete (v0.3.0)
- **Features**:
  - Multiple model types (Transformer, CNN, Custom)
  - Device flexibility (CPU, MPS)
  - Batch processing
  - Model management
  - Comprehensive testing
  - Updated for v0.3.0 API (port 15002, new routes)

### TensorFlow Integration
- **Location**: `integrations/tensorflow/`
- **Language**: Python
- **Description**: Support for custom TensorFlow models
- **Status**: ✅ Complete (v0.3.0)
- **Features**:
  - Multiple model types (Transformer, CNN, Custom)
  - Device flexibility (CPU, GPU)
  - Batch processing
  - Model management
  - Comprehensive testing
  - Updated for v0.3.0 API (port 15002, new routes)


## 📋 Integration Roadmap

### Phase 9: Advanced Integrations & Enterprise Features (v0.3.0)
- [x] **LangChain VectorStore**: Complete LangChain integration (Python) - Updated for v0.3.0
- [x] **LangChain.js VectorStore**: Complete LangChain.js integration (JavaScript/TypeScript) - Updated for v0.3.0
- [x] **PyTorch Support**: Support for custom PyTorch models - Updated for v0.3.0
- [x] **TensorFlow Support**: Support for custom TensorFlow models - Updated for v0.3.0
- [ ] **Model Management**: Model management system
- [ ] **Custom Embeddings**: Support for custom embeddings

## 🛠️ How to Use

### Installation
```bash
# Install LangChain dependencies (Python)
pip install -r integrations/langchain/requirements.txt

# Install LangChain.js dependencies (JavaScript/TypeScript)
cd integrations/langchain-js && npm install

# Install PyTorch dependencies
pip install -r integrations/pytorch/requirements.txt

# Install TensorFlow dependencies
pip install -r integrations/tensorflow/requirements.txt
```

### Usage Example - LangChain
```python
from integrations.langchain.vectorizer_store import VectorizerStore

# Create VectorizerStore instance
store = VectorizerStore(
    host="localhost",
    port=15002,
    collection_name="my_documents"
)

# Add documents
documents = [
    {"page_content": "Document 1", "metadata": {"source": "file1.txt"}},
    {"page_content": "Document 2", "metadata": {"source": "file2.txt"}}
]
store.add_documents(documents)

# Search for similar documents
results = store.similarity_search("query", k=5)

# Intelligent search with multi-query expansion
intelligent_results = store.intelligent_search(
    query="machine learning algorithms",
    collections=["my_documents"],
    max_results=10,
    domain_expansion=True,
    technical_focus=True,
    mmr_enabled=True,
    mmr_lambda=0.7
)

# Semantic search with reranking
semantic_results = store.semantic_search(
    query="neural networks",
    collection="my_documents",
    max_results=5,
    semantic_reranking=True,
    similarity_threshold=0.6
)

# Contextual search with metadata filtering
contextual_results = store.contextual_search(
    query="deep learning",
    collection="my_documents",
    context_filters={"category": "AI"},
    max_results=5,
    context_weight=0.4
)
```

### Usage Example - LangChain.js
```typescript
import { VectorizerStore, VectorizerConfig } from './integrations/langchain-js/vectorizer-store';

// Create configuration
const config: VectorizerConfig = {
  host: 'localhost',
  port: 15002,
  collectionName: 'my_documents',
  autoCreateCollection: true,
  batchSize: 100,
  similarityThreshold: 0.7
};

// Create store
const store = new VectorizerStore(config);

// Add documents
const texts = ['Document 1', 'Document 2'];
const metadatas = [{ source: 'file1.txt' }, { source: 'file2.txt' }];
await store.addTexts(texts, metadatas);

// Search for similar documents
const results = await store.similaritySearch('query', 5);

// Intelligent search with multi-query expansion
const intelligentResults = await store.intelligentSearch(
  'machine learning algorithms',
  ['my_documents'],
  10,
  true, // domainExpansion
  true, // technicalFocus
  true, // mmrEnabled
  0.7   // mmrLambda
);

// Semantic search with reranking
const semanticResults = await store.semanticSearch(
  'neural networks',
  'my_documents',
  5,
  true,  // semanticReranking
  false, // crossEncoderReranking
  0.6    // similarityThreshold
);

// Contextual search with metadata filtering
const contextualResults = await store.contextualSearch(
  'deep learning',
  'my_documents',
  { category: 'AI' }, // contextFilters
  5,
  true, // contextReranking
  0.4   // contextWeight
);
```

### Usage Example - PyTorch
```python
from integrations.pytorch.pytorch_embedder import create_transformer_embedder, PyTorchVectorizerClient

# Create PyTorch embedder
embedder = create_transformer_embedder(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    device="auto",
    batch_size=16
)

# Use with Vectorizer
client = PyTorchVectorizerClient()
client.set_embedder(embedder)
client.create_collection("pytorch_documents")

# Add documents
texts = ["PyTorch is great for deep learning"]
vector_ids = client.add_texts(texts)
results = client.search_similar("deep learning", k=5)
```

### Usage Example - TensorFlow
```python
from integrations.tensorflow.tensorflow_embedder import create_transformer_embedder, TensorFlowVectorizerClient

# Create TensorFlow embedder
embedder = create_transformer_embedder(
    model_path="sentence-transformers/all-MiniLM-L6-v2",
    device="auto",
    batch_size=16
)

# Use with Vectorizer
client = TensorFlowVectorizerClient()
client.set_embedder(embedder)
client.create_collection("tensorflow_documents")

# Add documents
texts = ["TensorFlow is excellent for machine learning"]
vector_ids = client.add_texts(texts)
results = client.search_similar("machine learning", k=5)
```

## 🔧 Development

### Development Structure
1. **Common**: Shared functionality between integrations
2. **Bindings**: Communication with Vectorizer Rust backend
3. **Frameworks**: Specific integrations with ML frameworks
4. **Tests**: Tests for all integrations

### Adding New Integration
1. Create directory in `integrations/new_integration/`
2. Implement main class following established patterns
3. Add tests in `integrations/new_integration/tests/`
4. Document usage in `integrations/new_integration/README.md`
5. Update this main README

## 📚 Documentation

- [LangChain Integration](langchain/README.md)
- [LangChain.js Integration](langchain-js/README.md)
- [PyTorch Integration](pytorch/README.md)
- [TensorFlow Integration](tensorflow/README.md)

## 🤝 Contributing

To contribute to integrations:

1. Fork the repository
2. Create a branch for your integration
3. Implement following established patterns
4. Add comprehensive tests
5. Document usage
6. Submit a Pull Request

## 📄 License

The integrations follow the same license as the main project.
