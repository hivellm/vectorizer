# Hive Vectorizer TypeScript Client SDK

High-performance TypeScript client for the Hive Vectorizer vector database.

## Features

- ✅ **Complete TypeScript Support**: Full type safety and IntelliSense
- ✅ **Async/Await**: Modern async programming patterns
- ✅ **HTTP Client**: Native fetch-based HTTP client
- ✅ **WebSocket Support**: Real-time communication
- ✅ **Comprehensive Validation**: Input validation and error handling
- ✅ **12 Custom Exceptions**: Robust error management
- ✅ **Logging**: Configurable logging system
- ✅ **Collection Management**: CRUD operations for collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Embedding Generation**: Text embedding support

## Installation

```bash
npm install @hivellm/vectorizer-client
```

## Quick Start

```typescript
import { VectorizerClient } from '@hivellm/vectorizer-client';

// Create client
const client = new VectorizerClient({
  baseURL: 'http://localhost:15001',
  apiKey: 'your-api-key-here'
});

// Health check
const health = await client.healthCheck();
console.log('Server status:', health.status);

// Create collection
const collection = await client.createCollection({
  name: 'documents',
  dimension: 768,
  similarity_metric: 'cosine'
});

// Insert vectors
const vectors = [{
  data: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  metadata: { source: 'document1.pdf' }
}];

await client.insertVectors('documents', vectors);

// Search vectors
const results = await client.searchVectors('documents', {
  query_vector: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  limit: 5
});

// Text search
const textResults = await client.searchText('documents', {
  query: 'machine learning algorithms',
  limit: 5
});

// Generate embeddings
const embedding = await client.embedText({
  text: 'machine learning algorithms'
});
```

## Configuration

```typescript
const client = new VectorizerClient({
  baseURL: 'http://localhost:15001',     // API base URL
  wsURL: 'ws://localhost:15001/ws',      // WebSocket URL (optional)
  apiKey: 'your-api-key',                // API key for authentication
  timeout: 30000,                        // Request timeout in ms
  headers: {                             // Custom headers
    'User-Agent': 'MyApp/1.0'
  },
  logger: {                              // Logger configuration
    level: 'info',                       // debug, info, warn, error
    enabled: true
  }
});
```

## API Reference

### Collection Management

```typescript
// List collections
const collections = await client.listCollections();

// Get collection info
const info = await client.getCollection('documents');

// Create collection
const collection = await client.createCollection({
  name: 'documents',
  dimension: 768,
  similarity_metric: 'cosine',
  description: 'Document embeddings'
});

// Update collection
const updated = await client.updateCollection('documents', {
  description: 'Updated description'
});

// Delete collection
await client.deleteCollection('documents');
```

### Vector Operations

```typescript
// Insert vectors
const vectors = [{
  data: [0.1, 0.2, 0.3],
  metadata: { source: 'doc1.pdf' }
}];
await client.insertVectors('documents', vectors);

// Get vector
const vector = await client.getVector('documents', 'vector-id');

// Update vector
const updated = await client.updateVector('documents', 'vector-id', {
  metadata: { updated: true }
});

// Delete vector
await client.deleteVector('documents', 'vector-id');

// Delete multiple vectors
await client.deleteVectors('documents', ['id1', 'id2', 'id3']);
```

### Search Operations

```typescript
// Vector similarity search
const results = await client.searchVectors('documents', {
  query_vector: [0.1, 0.2, 0.3],
  limit: 10,
  threshold: 0.8,
  include_metadata: true
});

// Text semantic search
const textResults = await client.searchText('documents', {
  query: 'machine learning',
  limit: 10,
  threshold: 0.8,
  include_metadata: true,
  model: 'bert-base'
});
```

### Embedding Operations

```typescript
// Generate embeddings
const embedding = await client.embedText({
  text: 'machine learning algorithms',
  model: 'bert-base',
  parameters: {
    max_length: 512,
    normalize: true
  }
});
```

### WebSocket Operations

```typescript
// Connect to WebSocket
await client.connectWebSocket();

// Listen for events
client.onWebSocketEvent('message', (data) => {
  console.log('Received:', data);
});

// Send message
client.sendWebSocketMessage({
  type: 'ping',
  timestamp: Date.now()
});

// Check connection status
if (client.isWebSocketConnected) {
  console.log('WebSocket connected');
}

// Disconnect
client.disconnectWebSocket();
```

## Error Handling

```typescript
import {
  VectorizerError,
  AuthenticationError,
  CollectionNotFoundError,
  ValidationError,
  NetworkError,
  ServerError
} from '@hivellm/vectorizer-client';

try {
  await client.createCollection({
    name: 'documents',
    dimension: 768
  });
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Authentication failed:', error.message);
  } else if (error instanceof ValidationError) {
    console.error('Validation error:', error.message);
  } else if (error instanceof NetworkError) {
    console.error('Network error:', error.message);
  } else {
    console.error('Unknown error:', error.message);
  }
}
```

## Types

```typescript
// Vector types
interface Vector {
  id: string;
  data: number[];
  metadata?: Record<string, unknown>;
}

// Collection types
interface Collection {
  name: string;
  dimension: number;
  similarity_metric: 'cosine' | 'euclidean' | 'dot_product';
  description?: string;
  created_at?: Date;
  updated_at?: Date;
}

// Search result types
interface SearchResult {
  id: string;
  score: number;
  data: number[];
  metadata?: Record<string, unknown>;
}

// Client configuration
interface VectorizerClientConfig {
  baseURL?: string;
  wsURL?: string;
  apiKey?: string;
  timeout?: number;
  headers?: Record<string, string>;
  logger?: LoggerConfig;
}
```

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Watch mode
npm run build:watch

# Test
npm test

# Test with coverage
npm run test:coverage

# Lint
npm run lint

# Lint and fix
npm run lint:fix
```

## License

MIT License - see [LICENSE](LICENSE) for details.
