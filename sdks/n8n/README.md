# @vectorizer/n8n-nodes-vectorizer

This is an n8n community node that integrates Vectorizer vector database into your n8n workflows.

[Vectorizer](https://github.com/hivellm/vectorizer) is a high-performance vector database designed for RAG (Retrieval-Augmented Generation) and similarity search applications.

[n8n](https://n8n.io/) is a fair-code licensed workflow automation platform.

## Installation

Follow the [installation guide](https://docs.n8n.io/integrations/community-nodes/installation/) in the n8n community nodes documentation.

### Community Nodes (n8n Cloud)

1. Go to **Settings > Community Nodes**
2. Select **Install**
3. Enter `@vectorizer/n8n-nodes-vectorizer` in **Enter npm package name**
4. Agree to the risks and select **Install**

### Self-Hosted

For self-hosted n8n instances, install via npm:

```bash
npm install @vectorizer/n8n-nodes-vectorizer
```

## Operations

### Collection Resource

- **Create**: Create a new vector collection
- **Delete**: Delete an existing collection
- **Get**: Get collection information
- **List**: List all collections

### Vector Resource

- **Insert**: Insert a single vector
- **Batch Insert**: Insert multiple vectors at once
- **Delete**: Delete a vector by ID
- **Get**: Retrieve a vector by ID

### Search Resource

- **Vector Search**: Search using a pre-computed vector
- **Semantic Search**: Search using text query (auto-embedded)
- **Hybrid Search**: Combine vector and keyword search

## Credentials

The Vectorizer node requires the following credentials:

- **Host**: URL of your Vectorizer instance (e.g., `http://localhost:15002`)
- **API Key**: (Optional) API key if authentication is enabled

## Compatibility

Tested against:
- n8n v1.0.0+
- Vectorizer v1.6.0+

## Usage Examples

### Example 1: Create Collection and Insert Vectors

```json
{
  "nodes": [
    {
      "name": "Create Collection",
      "type": "@vectorizer/n8n-nodes-vectorizer",
      "parameters": {
        "resource": "collection",
        "operation": "create",
        "collectionName": "my-documents",
        "dimension": 384,
        "metric": "cosine"
      }
    },
    {
      "name": "Insert Vector",
      "type": "@vectorizer/n8n-nodes-vectorizer",
      "parameters": {
        "resource": "vector",
        "operation": "insert",
        "collectionName": "my-documents",
        "vectorId": "doc-1",
        "vectorData": "[0.1, 0.2, 0.3, ...]",
        "payload": "{\"title\": \"My Document\", \"content\": \"...\"}"
      }
    }
  ]
}
```

### Example 2: Semantic Search Workflow

```json
{
  "nodes": [
    {
      "name": "Webhook",
      "type": "n8n-nodes-base.webhook",
      "parameters": {
        "path": "search"
      }
    },
    {
      "name": "Search Vectorizer",
      "type": "@vectorizer/n8n-nodes-vectorizer",
      "parameters": {
        "resource": "search",
        "operation": "semanticSearch",
        "collectionName": "my-documents",
        "query": "={{$json[\"query\"]}}",
        "limit": 10,
        "scoreThreshold": 0.7
      }
    },
    {
      "name": "Respond",
      "type": "n8n-nodes-base.respondToWebhook",
      "parameters": {
        "respondWith": "json",
        "responseBody": "={{$json}}"
      }
    }
  ]
}
```

### Example 3: RAG Pipeline

```json
{
  "nodes": [
    {
      "name": "Question Input",
      "type": "n8n-nodes-base.webhook",
      "parameters": {
        "path": "ask"
      }
    },
    {
      "name": "Retrieve Context",
      "type": "@vectorizer/n8n-nodes-vectorizer",
      "parameters": {
        "resource": "search",
        "operation": "semanticSearch",
        "collectionName": "knowledge-base",
        "query": "={{$json[\"question\"]}}",
        "limit": 3
      }
    },
    {
      "name": "Format Context",
      "type": "n8n-nodes-base.code",
      "parameters": {
        "code": "const context = items.map(item => item.json.payload.content).join('\\n\\n');\nreturn [{ json: { context, question: items[0].json.question } }];"
      }
    },
    {
      "name": "OpenAI",
      "type": "n8n-nodes-base.openAi",
      "parameters": {
        "operation": "message",
        "text": "Based on the following context:\\n{{$json[\"context\"]}}\\n\\nAnswer this question: {{$json[\"question\"]}}"
      }
    }
  ]
}
```

### Example 4: Batch Document Ingestion

```json
{
  "nodes": [
    {
      "name": "Read CSV",
      "type": "n8n-nodes-base.readFile"
    },
    {
      "name": "Parse CSV",
      "type": "n8n-nodes-base.splitInBatches",
      "parameters": {
        "batchSize": 100
      }
    },
    {
      "name": "Embed with OpenAI",
      "type": "n8n-nodes-base.openAi",
      "parameters": {
        "operation": "embedding",
        "text": "={{$json[\"text\"]}}"
      }
    },
    {
      "name": "Batch Insert",
      "type": "@vectorizer/n8n-nodes-vectorizer",
      "parameters": {
        "resource": "vector",
        "operation": "batchInsert",
        "collectionName": "documents",
        "vectors": "={{JSON.stringify($items.map((item, i) => ({ id: `doc-${i}`, vector: item.json.embedding, payload: { text: item.json.text } })))}}"
      }
    }
  ]
}
```

## Resources

- [Vectorizer Documentation](https://github.com/hivellm/vectorizer)
- [n8n Community Nodes Documentation](https://docs.n8n.io/integrations/community-nodes/)
- [Report Issues](https://github.com/hivellm/vectorizer/issues)

## License

[Apache-2.0](LICENSE)

## Version History

See [CHANGELOG.md](../../CHANGELOG.md)
