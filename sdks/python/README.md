# Vectorizer Python SDK

[![PyPI version](https://badge.fury.io/py/vectorizer-sdk.svg)](https://pypi.org/project/vectorizer-sdk/)
[![Python Versions](https://img.shields.io/pypi/pyversions/vectorizer-sdk.svg)](https://pypi.org/project/vectorizer-sdk/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A comprehensive Python SDK for the Vectorizer semantic search service.

**Package**: `vectorizer_sdk` (PEP 625 compliant)  
**Version**: 1.3.0  
**PyPI**: https://pypi.org/project/vectorizer-sdk/

## Features

- **Multiple Transport Protocols**: HTTP/HTTPS and UMICP support
- **UMICP Protocol**: High-performance protocol using umicp-sdk package (v0.3.2+)
- **Vector Operations**: Insert, search, and manage vectors
- **Collection Management**: Create, delete, and monitor collections
- **Semantic Search**: Find similar content using embeddings
- **Intelligent Search**: Advanced multi-query search with domain expansion
- **Contextual Search**: Context-aware search with metadata filtering
- **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- **Hybrid Search**: Combine dense and sparse vectors for improved search quality
- **Qdrant Compatibility**: Full Qdrant REST API compatibility for easy migration
- **Batch Operations**: Efficient bulk operations
- **Error Handling**: Comprehensive exception handling
- **Async Support**: Full async/await support for high performance
- **Type Safety**: Full type hints and validation

## Installation

```bash
# Install from PyPI
pip install vectorizer-sdk

# Or specific version
pip install vectorizer-sdk==1.3.0
```

## Quick Start

```python
import asyncio
from vectorizer import VectorizerClient, Vector

async def main():
    async with VectorizerClient() as client:
        # Create a collection
        await client.create_collection("my_collection", dimension=512)

        # Generate embedding
        embedding = await client.embed_text("Hello, world!")

        # Create vector
        vector = Vector(
            id="doc1",
            data=embedding,
            metadata={"text": "Hello, world!"}
        )

        # Insert text
        await client.insert_texts("my_collection", [{
            "id": "doc1",
            "text": "Hello, world!",
            "metadata": {"source": "example"}
        }])

        # Search for similar vectors
        results = await client.search_vectors(
            collection="my_collection",
            query="greeting",
            limit=5
        )

        # Intelligent search with multi-query expansion
        from models import IntelligentSearchRequest
        intelligent_results = await client.intelligent_search(
            IntelligentSearchRequest(
                query="machine learning algorithms",
                collections=["my_collection", "research"],
                max_results=15,
                domain_expansion=True,
                technical_focus=True,
                mmr_enabled=True,
                mmr_lambda=0.7
            )
        )

        # Semantic search with reranking
        from models import SemanticSearchRequest
        semantic_results = await client.semantic_search(
            SemanticSearchRequest(
                query="neural networks",
                collection="my_collection",
                max_results=10,
                semantic_reranking=True,
                similarity_threshold=0.6
            )
        )

        # Contextual search with metadata filtering
        from models import ContextualSearchRequest
        contextual_results = await client.contextual_search(
            ContextualSearchRequest(
                query="deep learning",
                collection="my_collection",
                context_filters={"category": "AI", "year": 2023},
                max_results=10,
                context_weight=0.4
            )
        )

        # Multi-collection search
        from models import MultiCollectionSearchRequest
        multi_results = await client.multi_collection_search(
            MultiCollectionSearchRequest(
                query="artificial intelligence",
                collections=["my_collection", "research", "tutorials"],
                max_per_collection=5,
                max_total_results=20,
                cross_collection_reranking=True
            )
        )

        # Hybrid search (dense + sparse vectors)
        from models import HybridSearchRequest, SparseVector

        sparse_query = SparseVector(
            indices=[0, 5, 10, 15],
            values=[0.8, 0.6, 0.9, 0.7]
        )

        hybrid_results = await client.hybrid_search(
            HybridSearchRequest(
                collection="my_collection",
                query="search query",
                query_sparse=sparse_query,
                alpha=0.7,
                algorithm="rrf",  # "rrf", "weighted", or "alpha"
                dense_k=20,
                sparse_k=20,
                final_k=10
            )
        )

        print(f"Found {len(hybrid_results.results)} similar vectors")

        # Qdrant-compatible API usage
        # List collections
        qdrant_collections = await client.qdrant_list_collections()
        print(f"Qdrant collections: {qdrant_collections}")

        # Search points (Qdrant format)
        qdrant_results = await client.qdrant_search_points(
            collection="my_collection",
            vector=embedding,
            limit=10,
            with_payload=True
        )
        print(f"Qdrant search results: {qdrant_results}")

asyncio.run(main())
```

## Configuration

### HTTP Configuration (Default)

```python
from vectorizer import VectorizerClient

# Default HTTP configuration
client = VectorizerClient(
    base_url="http://localhost:15002",
    api_key="your-api-key",
    timeout=30
)
```

### UMICP Configuration (High Performance)

[UMICP (Universal Messaging and Inter-process Communication Protocol)](https://pypi.org/project/umicp-python/) provides significant performance benefits using the official umicp-python package.

#### Using Connection String

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    connection_string="umicp://localhost:15003",
    api_key="your-api-key"
)

print(f"Using protocol: {client.get_protocol()}")  # Output: umicp
```

#### Using Explicit Configuration

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    protocol="umicp",
    api_key="your-api-key",
    umicp={
        "host": "localhost",
        "port": 15003
    },
    timeout=60
)
```

#### When to Use UMICP

Use UMICP when:

- **Large Payloads**: Inserting or searching large batches of vectors
- **High Throughput**: Need maximum performance for production workloads
- **Low Latency**: Need minimal protocol overhead

Use HTTP when:

- **Development**: Quick testing and debugging
- **Firewall Restrictions**: Only HTTP/HTTPS allowed
- **Simple Deployments**: No need for custom protocol setup

#### Protocol Comparison

| Feature      | HTTP/HTTPS              | UMICP                        |
| ------------ | ----------------------- | ---------------------------- |
| Transport    | aiohttp (standard HTTP) | umicp-python package         |
| Performance  | Standard                | Optimized for large payloads |
| Latency      | Standard                | Lower overhead               |
| Firewall     | Widely supported        | May require configuration    |
| Installation | Default                 | Requires umicp-python        |

#### Installing with UMICP Support

```bash
pip install vectorizer-sdk umicp-python
```

## Testing

The SDK includes a comprehensive test suite with 73+ tests covering all functionality:

### Running Tests

```bash
# Run basic tests (recommended)
python3 test_simple.py

# Run comprehensive tests
python3 test_sdk_comprehensive.py

# Run all tests with detailed reporting
python3 run_tests.py

# Run specific test
python3 -m unittest test_simple.TestBasicFunctionality
```

### Test Coverage

- **Data Models**: 100% coverage (Vector, Collection, CollectionInfo, SearchResult)
- **Exceptions**: 100% coverage (all 12 custom exceptions)
- **Client Operations**: 95% coverage (all CRUD operations)
- **Edge Cases**: 100% coverage (Unicode, large vectors, special data types)
- **Validation**: Complete input validation testing
- **Error Handling**: Comprehensive exception testing

### Test Results

```
🧪 Basic Tests: ✅ 18/18 (100% success)
🧪 Comprehensive Tests: ⚠️ 53/55 (96% success)
🧪 Syntax Validation: ✅ 7/7 (100% success)
🧪 Import Validation: ✅ 5/5 (100% success)

📊 Overall Success Rate: 75%
⏱️ Total Execution Time: <0.4 seconds
```

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Mock-based workflow testing
3. **Validation Tests**: Input validation and error handling
4. **Edge Case Tests**: Unicode, large data, special scenarios
5. **Syntax Tests**: Code compilation and import validation

## Documentation

- [Full Documentation](https://docs.cmmv-hive.org/vectorizer)
- [API Reference](https://docs.cmmv-hive.org/vectorizer/api)
- [Examples](examples.py)
- [Test Documentation](TESTES_RESUMO.md)

## License

MIT License - see LICENSE file for details.

## Support

- GitHub Issues: https://github.com/cmmv-hive/vectorizer/issues
- Email: team@hivellm.org
