# LangChain.js Integration for Vectorizer v0.3.4

This integration provides a complete implementation of LangChain.js's VectorStore interface using Vectorizer as the backend for vector storage and similarity search.

## 🚀 Features

- ✅ **Full Compatibility**: Implements LangChain.js's VectorStore interface
- ✅ **Intelligent Search**: Advanced multi-query search with domain expansion
- ✅ **Semantic Search**: Advanced reranking and similarity thresholds
- ✅ **Contextual Search**: Context-aware search with metadata filtering
- ✅ **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- ✅ **File Operations** (v0.3.4+): Complete file-level operations for indexed documents
  - `getFileContent` - Retrieve complete files with metadata
  - `listFilesInCollection` - Advanced file listing and filtering
  - `getFileSummary` - Extractive and structural summaries
  - `getProjectOutline` - Hierarchical project visualization
  - `getRelatedFiles` - Semantic file similarity search
  - `searchByFileType` - File type-specific search
- ✅ **Discovery System** (v0.3.4+): Advanced discovery pipeline
  - `discover` - Complete 9-stage discovery pipeline
  - `filterCollections` - Pre-filter collections by patterns
  - `scoreCollections` - Rank collections by relevance
  - `expandQueries` - Generate query variations
  - `broadDiscovery` - Multi-query broad search with MMR
  - `semanticFocus` - Deep semantic search with reranking
  - `compressEvidence` - Extract key sentences with citations
  - `buildAnswerPlan` - Organize into structured sections
  - `renderLlmPrompt` - Generate LLM-ready prompts
- ✅ **TypeScript**: Complete TypeScript support with defined types
- ✅ **Batch Operations**: Support for efficient batch operations
- ✅ **Metadata Filtering**: Search with metadata filters
- ✅ **Flexible Configuration**: Customizable configuration for different environments
- ✅ **Error Handling**: Robust error handling and exceptions
- ✅ **Comprehensive Tests**: Complete test suite with Jest
- ✅ **Async/Await**: Full support for async operations

## 📦 Installation

```bash
# Install dependencies
npm install

# Or install directly
npm install @langchain/core node-fetch
```

## 🔧 Configuration

### Basic Configuration

```typescript
import { VectorizerConfig, VectorizerStore } from './vectorizer-store';

// Default configuration
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
```

### Advanced Configuration

```typescript
const config: VectorizerConfig = {
  host: 'vectorizer.example.com',
  port: 15002,
  collectionName: 'production_documents',
  apiKey: 'prod_api_key',
  timeout: 60000,
  autoCreateCollection: true,
  batchSize: 200,
  similarityThreshold: 0.8
};
```

## 📚 Usage

### Basic Usage

```typescript
import { VectorizerStore, VectorizerConfig } from './vectorizer-store';
import { Document } from '@langchain/core/documents';

// Create configuration
const config: VectorizerConfig = {
  host: 'localhost',
  port: 15002,
  collectionName: 'my_docs',
  autoCreateCollection: true,
  batchSize: 100,
  similarityThreshold: 0.7
};

// Create store
const store = new VectorizerStore(config);

// Add documents
const texts = [
  'This is the first document',
  'This is the second document',
  'This is the third document'
];

const metadatas = [
  { source: 'doc1.txt', page: 1 },
  { source: 'doc2.txt', page: 1 },
  { source: 'doc3.txt', page: 1 }
];

try {
  const vectorIds = await store.addTexts(texts, metadatas);
  console.log(`Added ${vectorIds.length} documents`);

  // Search for similar documents
  const results = await store.similaritySearch('first document', 2);
  results.forEach(doc => {
    console.log(`Content: ${doc.pageContent}`);
    console.log(`Metadata: ${JSON.stringify(doc.metadata)}`);
  });
} catch (error) {
  console.error('Error:', error);
}
```

### Usage with LangChain.js

```typescript
import { VectorizerStore, VectorizerConfig } from './vectorizer-store';
import { Document } from '@langchain/core/documents';

// Load documents
const documents = [
  new Document({
    pageContent: 'Artificial intelligence is revolutionizing many industries.',
    metadata: { source: 'ai_doc.txt', category: 'technology' }
  }),
  new Document({
    pageContent: 'Natural language processing enables computers to understand human language.',
    metadata: { source: 'nlp_doc.txt', category: 'technology' }
  })
];

// Create store and add documents
const config: VectorizerConfig = {
  host: 'localhost',
  port: 15002,
  collectionName: 'document_chunks',
  autoCreateCollection: true,
  batchSize: 100,
  similarityThreshold: 0.7
};

const store = new VectorizerStore(config);

// Add documents
const texts = documents.map(doc => doc.pageContent);
const metadatas = documents.map(doc => doc.metadata);
await store.addTexts(texts, metadatas);

// Search for specific information
const results = await store.similaritySearch('artificial intelligence', 5);
results.forEach(doc => {
  console.log(`Relevant chunk: ${doc.pageContent.substring(0, 100)}...`);
});

// Intelligent search with multi-query expansion
const intelligentResults = await store.intelligentSearch(
  'machine learning algorithms',
  ['document_chunks'],
  10,
  true, // domainExpansion
  true, // technicalFocus
  true, // mmrEnabled
  0.7   // mmrLambda
);
console.log(`Intelligent search found ${intelligentResults.length} results`);

// Semantic search with reranking
const semanticResults = await store.semanticSearch(
  'neural networks',
  'document_chunks',
  5,
  true,  // semanticReranking
  false, // crossEncoderReranking
  0.6    // similarityThreshold
);
console.log(`Semantic search found ${semanticResults.length} results`);

// Contextual search with metadata filtering
const contextualResults = await store.contextualSearch(
  'deep learning',
  'document_chunks',
  { source: 'document.txt' }, // contextFilters
  5,
  true, // contextReranking
  0.4   // contextWeight
);
console.log(`Contextual search found ${contextualResults.length} results`);
```

### Search with Filters

```typescript
// Search without filter
const results = await store.similaritySearch('programming', 5);

// Search with metadata filter
const filterDict = { category: 'technology' };
const filteredResults = await store.similaritySearch('programming', 5, filterDict);

// Search with multiple filters
const multiFilter = {
  category: 'technology',
  year: 2023,
  language: 'python'
};
const multiFilteredResults = await store.similaritySearch('programming', 5, multiFilter);
```

### Batch Operations

```typescript
// Add many documents
const largeTexts = Array.from({ length: 1000 }, (_, i) => `Document ${i}`);
const largeMetadatas = Array.from({ length: 1000 }, (_, i) => ({ docId: i }));

const vectorIds = await store.addTexts(largeTexts, largeMetadatas);
console.log(`Added ${vectorIds.length} documents in batch`);

// Delete documents in batch
const idsToDelete = vectorIds.slice(0, 100); // Delete first 100
const success = await store.delete(idsToDelete);
console.log(`Deleted ${idsToDelete.length} documents: ${success}`);
```

### Search with Scores

```typescript
// Search with similarity scores
const resultsWithScores = await store.similaritySearchWithScore('query', 5);

resultsWithScores.forEach(([doc, score]) => {
  console.log(`Score: ${score.toFixed(3)}`);
  console.log(`Content: ${doc.pageContent}`);
  console.log(`Metadata: ${JSON.stringify(doc.metadata)}`);
  console.log('---');
});
```

## 🔧 Available Methods

### VectorizerStore

- `addTexts(texts: string[], metadatas?: Record<string, any>[])` - Add texts
- `similaritySearch(query: string, k?: number, filter?: Record<string, any>)` - Similarity search
- `similaritySearchWithScore(query: string, k?: number, filter?: Record<string, any>)` - Search with scores
- `intelligentSearch(query: string, collections?: string[], maxResults?: number, domainExpansion?: boolean, technicalFocus?: boolean, mmrEnabled?: boolean, mmrLambda?: number)` - Intelligent search with multi-query expansion
- `semanticSearch(query: string, collection: string, maxResults?: number, semanticReranking?: boolean, crossEncoderReranking?: boolean, similarityThreshold?: number)` - Semantic search with advanced reranking
- `contextualSearch(query: string, collection: string, contextFilters?: Record<string, any>, maxResults?: number, contextReranking?: boolean, contextWeight?: number)` - Context-aware search with metadata filtering
- `multiCollectionSearch(query: string, collections: string[], maxPerCollection?: number, maxTotalResults?: number, crossCollectionReranking?: boolean)` - Multi-collection search with cross-collection reranking
- `delete(ids: string[])` - Delete vectors by IDs
- `fromTexts(texts: string[], metadatas?: Record<string, any>[], embeddings?: Embeddings, config?: VectorizerConfig)` - Create store from texts
- `fromDocuments(documents: Document[], embeddings?: Embeddings, config?: VectorizerConfig)` - Create store from documents

### VectorizerClient

- `healthCheck()` - Check API health
- `listCollections()` - List collections
- `createCollection(name: string, dimension?: number, metric?: string)` - Create collection
- `deleteCollection(name: string)` - Delete collection
- `addTexts(texts: string[], metadatas?: Record<string, any>[])` - Add texts
- `similaritySearch(query: string, k?: number, filter?: Record<string, any>)` - Similarity search
- `similaritySearchWithScore(query: string, k?: number, filter?: Record<string, any>)` - Search with scores
- `deleteVectors(ids: string[])` - Delete vectors

### VectorizerUtils

- `validateConfig(config: VectorizerConfig)` - Validate configuration
- `createDefaultConfig(overrides?: Partial<VectorizerConfig>)` - Create default configuration
- `checkAvailability(config: VectorizerConfig)` - Check Vectorizer availability

## 🧪 Tests

### Run Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm test -- --coverage
```

### Available Tests

- **VectorizerConfig**: Configuration tests
- **VectorizerClient**: HTTP client tests
- **VectorizerStore**: LangChain.js store tests
- **VectorizerUtils**: Utility tests
- **Error Handling**: Error handling tests
- **Integration Tests**: Integration tests with real Vectorizer

## 📋 Examples

See the `examples.ts` file for complete usage examples:

- Basic example
- Document loading
- Text splitting
- Metadata filtering
- Batch operations
- Configuration and validation
- Error handling

```bash
# Run examples
npm run build
node dist/examples.js
```

## 🔧 Vectorizer Configuration

Make sure Vectorizer is running:

```bash
# Start Vectorizer
cargo run --bin vectorizer

# Or use Docker
docker run -p 15002:15002 vectorizer:latest
```

## 🚨 Error Handling

```typescript
import { VectorizerError } from './vectorizer-store';

try {
  await store.addTexts(texts, metadatas);
} catch (error) {
  if (error instanceof VectorizerError) {
    console.error(`Vectorizer error: ${error.message}`);
  } else {
    console.error(`General error: ${error}`);
  }
}
```

## 📈 Performance

### Recommended Optimizations

1. **Batch Size**: Use appropriate `batchSize` (100-200)
2. **Filters**: Use metadata filters to reduce results
3. **Connections**: Reuse store instances
4. **Timeout**: Configure appropriate timeout for your network

### Performance Metrics

- **Addition**: ~1000 documents/second
- **Search**: ~100 queries/second
- **Latency**: <50ms for simple searches

## 🔧 Development

### Available Scripts

```bash
# Build
npm run build

# Development with watch
npm run dev

# Lint
npm run lint

# Lint with fix
npm run lint:fix

# Format
npm run format
```

### Project Structure

```
langchain-js/
├── vectorizer-store.ts      # Main implementation
├── examples.ts              # Usage examples
├── vectorizer-store.test.ts # Tests
├── package.json             # Package configuration
├── tsconfig.json           # TypeScript configuration
├── jest.config.js          # Jest configuration
├── jest.setup.ts           # Jest setup
└── README.md               # Documentation
```

## 🤝 Contributing

1. Fork the repository
2. Create a branch for your feature
3. Implement tests
4. Submit a Pull Request

## 📄 License

This integration follows the same license as the Vectorizer project.
