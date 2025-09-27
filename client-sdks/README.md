# Hive Vectorizer Client SDKs

High-performance client SDKs for the Hive Vectorizer vector database, available in multiple languages.

## Available SDKs

### 🟦 TypeScript SDK
- **Package**: `@hivellm/vectorizer-client`
- **Features**: Full TypeScript support, async/await, comprehensive type safety
- **Installation**: `npm install @hivellm/vectorizer-client`
- **Documentation**: [TypeScript SDK README](./typescript/README.md)

### 🟨 JavaScript SDK
- **Package**: `@hivellm/vectorizer-client-js`
- **Features**: Modern JavaScript, multiple build formats (CJS, ESM, UMD)
- **Installation**: `npm install @hivellm/vectorizer-client-js`
- **Documentation**: [JavaScript SDK README](./javascript/README.md)

### 🐍 Python SDK
- **Package**: `hivellm-vectorizer-client`
- **Features**: Async/await support, comprehensive testing, CLI interface
- **Installation**: `pip install hivellm-vectorizer-client`
- **Documentation**: [Python SDK README](./python/README.md)

## Quick Start

### TypeScript/JavaScript

```typescript
import { VectorizerClient } from '@hivellm/vectorizer-client';

const client = new VectorizerClient({
  baseURL: 'http://localhost:15001',
  apiKey: 'your-api-key'
});

// Create collection
await client.createCollection({
  name: 'documents',
  dimension: 768,
  similarity_metric: 'cosine'
});

// Insert vectors
await client.insertVectors('documents', [{
  data: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  metadata: { source: 'document.pdf' }
}]);

// Search
const results = await client.searchVectors('documents', {
  query_vector: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  limit: 5
});
```

### Python

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    base_url="http://localhost:15001",
    api_key="your-api-key"
)

# Create collection
await client.create_collection(
    name="documents",
    dimension=768,
    metric="cosine"
)

# Insert vectors
vectors = [{
    "data": [0.1, 0.2, 0.3, ...],  # 768-dimensional vector
    "metadata": {"source": "document.pdf"}
}]
await client.insert_vectors("documents", vectors)

# Search
results = await client.search_vectors(
    collection="documents",
    query_vector=[0.1, 0.2, 0.3, ...],
    limit=5
)
```

## Features

All SDKs provide:

- ✅ **Collection Management**: Create, read, update, delete collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Embedding Generation**: Text embedding support
- ✅ **WebSocket Support**: Real-time communication
- ✅ **Authentication**: API key-based authentication
- ✅ **Error Handling**: Comprehensive exception handling
- ✅ **Logging**: Configurable logging system
- ✅ **Validation**: Input validation and type checking

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   TypeScript    │    │   JavaScript    │    │     Python      │
│      SDK        │    │      SDK        │    │      SDK        │
│                 │    │                 │    │                 │
│ • Type Safety   │    │ • Multi-format  │    │ • Async/Await   │
│ • IntelliSense  │    │ • Browser Ready │    │ • CLI Interface │
│ • ES2020+       │    │ • Node.js       │    │ • 73+ Tests     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │  Vectorizer     │
                    │     Server      │
                    │                 │
                    │ • REST API      │
                    │ • WebSocket     │
                    │ • GRPC          │
                    │ • MCP Protocol  │
                    └─────────────────┘
```

## Development

### Building SDKs

```bash
# TypeScript SDK
cd client-sdks/typescript
npm install
npm run build

# JavaScript SDK
cd client-sdks/javascript
npm install
npm run build

# Python SDK
cd client-sdks/python
pip install -r requirements.txt
python setup.py build
```

### Testing

```bash
# TypeScript SDK
cd client-sdks/typescript
npm test

# JavaScript SDK
cd client-sdks/javascript
npm test

# Python SDK
cd client-sdks/python
python run_tests.py
```

### Linting

```bash
# TypeScript SDK
cd client-sdks/typescript
npm run lint

# JavaScript SDK
cd client-sdks/javascript
npm run lint

# Python SDK
cd client-sdks/python
flake8 src/
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Support

- **Documentation**: [Vectorizer Documentation](../docs/)
- **Issues**: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hivellm/vectorizer/discussions)
