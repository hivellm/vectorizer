# Text Summarization

Vectorizer includes built-in text summarization capabilities for generating concise summaries of documents and search results.

## Overview

The summarization system supports multiple methods:
- **Extractive**: Selects key sentences from the original text
- **Abstractive**: Generates new text that captures the meaning
- **Hybrid**: Combines both approaches for optimal results

## API Endpoints

### Summarize Text

```http
POST /api/v1/summarize
Content-Type: application/json

{
  "text": "Your long text to summarize...",
  "method": "extractive",
  "max_length": 150,
  "min_length": 50
}
```

Response:

```json
{
  "summary": "Concise summary of the original text...",
  "method": "extractive",
  "original_length": 2500,
  "summary_length": 145,
  "compression_ratio": 0.058
}
```

### Summarize Document

```http
POST /api/v1/summarize/document
Content-Type: application/json

{
  "collection": "my_docs",
  "document_id": "doc-123",
  "method": "hybrid",
  "max_length": 200
}
```

### Summarize Search Results

```http
POST /api/v1/summarize/results
Content-Type: application/json

{
  "collection": "my_docs",
  "query": "machine learning basics",
  "limit": 5,
  "summarize": true,
  "summary_method": "abstractive"
}
```

Response:

```json
{
  "results": [...],
  "summary": "Combined summary of top search results...",
  "sources": ["doc-1", "doc-2", "doc-3"]
}
```

## Summarization Methods

### Extractive Summarization

Extracts the most important sentences from the original text.

```json
{
  "method": "extractive",
  "options": {
    "sentence_count": 5,
    "use_mmr": true,
    "diversity": 0.7
  }
}
```

**Advantages:**
- Fast processing
- Preserves original wording
- No hallucination risk

**Use Cases:**
- News articles
- Research papers
- Technical documentation

### Abstractive Summarization

Generates new text that captures the essence of the original using OpenAI's GPT models.

**Requirements:**
- OpenAI API key (configure via `api_key` in method config or `OPENAI_API_KEY` environment variable)
- OpenAI API access (internet connection required)

**Configuration:**
```yaml
summarization:
  methods:
    abstractive:
      enabled: true
      api_key: "sk-..."  # Or use OPENAI_API_KEY env var
      model: "gpt-4o-mini"  # Default: gpt-4o-mini (latest GPT model)
      max_tokens: 150
      temperature: 0.7
```

**Usage:**
```json
{
  "method": "abstractive",
  "text": "Long document text...",
  "options": {
    "model": "gpt-4o-mini",
    "temperature": 0.7,
    "max_tokens": 150
  }
}
```

**Advantages:**
- More natural language
- Better compression
- Can rephrase complex concepts
- Produces fluent, coherent summaries

**Limitations:**
- Requires OpenAI API key (costs apply)
- Requires internet connection
- Slower than extractive methods (API call overhead)
- Disabled by default

**Use Cases:**
- User-facing summaries
- Executive briefings
- Content previews
- Marketing materials

**Note:** If no API key is configured, abstractive summarization will return an error. Use extractive, keyword, or sentence methods for local-only summarization.

### Hybrid Summarization

Combines extractive and abstractive approaches.

```json
{
  "method": "hybrid",
  "options": {
    "extractive_ratio": 0.6,
    "abstractive_ratio": 0.4,
    "blend_strategy": "sequential"
  }
}
```

**Advantages:**
- Balanced accuracy and fluency
- Configurable blend
- Best of both methods

## Configuration

### Summarization Config

```yaml
# vectorizer.yaml
summarization:
  enabled: true
  default_method: "extractive"
  max_input_length: 50000
  default_max_length: 200
  default_min_length: 50
  cache_enabled: true
  cache_ttl: 3600
```

### Method-Specific Settings

```yaml
summarization:
  extractive:
    algorithm: "textrank"
    sentence_weight: "tfidf"
    mmr_lambda: 0.7
  
  abstractive:
    model: "default"
    max_new_tokens: 200
    temperature: 0.7
    top_p: 0.9
  
  hybrid:
    extractive_weight: 0.6
    blend_mode: "sequential"
```

## SDK Usage

### Python

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Simple summarization
summary = await client.summarize(
    text="Your long text here...",
    method="extractive",
    max_length=150
)
print(summary.text)

# Summarize document
doc_summary = await client.summarize_document(
    collection="my_docs",
    document_id="doc-123",
    method="hybrid"
)

# Summarize search results
results = await client.search_with_summary(
    collection="my_docs",
    query="machine learning",
    limit=5,
    summarize=True
)
print(results.summary)
```

### TypeScript

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

const client = new VectorizerClient("http://localhost:15002");

// Simple summarization
const summary = await client.summarize({
  text: "Your long text here...",
  method: "extractive",
  maxLength: 150
});
console.log(summary.text);

// Summarize search results
const results = await client.searchWithSummary({
  collection: "my_docs",
  query: "machine learning",
  limit: 5,
  summarize: true
});
console.log(results.summary);
```

## Advanced Features

### Multi-Document Summarization

Summarize multiple documents into a single coherent summary:

```http
POST /api/v1/summarize/multi
Content-Type: application/json

{
  "documents": [
    {"collection": "docs", "id": "doc-1"},
    {"collection": "docs", "id": "doc-2"},
    {"collection": "docs", "id": "doc-3"}
  ],
  "method": "hybrid",
  "max_length": 300,
  "focus": "common_themes"
}
```

### Query-Focused Summarization

Generate summaries focused on specific queries:

```http
POST /api/v1/summarize/focused
Content-Type: application/json

{
  "text": "Long document text...",
  "query": "What are the main benefits?",
  "max_length": 150
}
```

### Summarization with Keywords

Extract key terms along with the summary:

```http
POST /api/v1/summarize
Content-Type: application/json

{
  "text": "Your text...",
  "method": "extractive",
  "extract_keywords": true,
  "keyword_count": 10
}
```

Response:

```json
{
  "summary": "...",
  "keywords": [
    {"term": "machine learning", "score": 0.95},
    {"term": "neural networks", "score": 0.87},
    ...
  ]
}
```

## Performance Considerations

### Caching

Summaries are cached to improve performance:

```yaml
summarization:
  cache:
    enabled: true
    ttl: 3600  # 1 hour
    max_size: 1000
```

### Batch Processing

For multiple documents:

```http
POST /api/v1/summarize/batch
Content-Type: application/json

{
  "texts": [
    "First document...",
    "Second document...",
    "Third document..."
  ],
  "method": "extractive",
  "max_length": 100
}
```

## Related Topics

- [Search Guide](../search/SEARCH.md) - Search operations
- [Discovery API](./DISCOVERY.md) - Content discovery
- [Intelligent Search](../search/ADVANCED.md) - Advanced search features

