---
title: n8n Integration
module: guides
id: n8n-integration
order: 20
description: No-code workflow automation with n8n
tags: [n8n, integration, workflow, automation, no-code]
---

# n8n Integration

Vectorizer provides an official n8n community node for **no-code workflow automation**.

## Overview

The `@vectorizer/n8n-nodes-vectorizer` package provides native n8n nodes for:

- **Collection Management** - Create, delete, list collections
- **Vector Operations** - Insert, batch insert, get, delete vectors
- **Search Operations** - Vector search, semantic search, hybrid search

## Installation

### n8n Cloud

1. Go to **Settings** → **Community nodes**
2. Click **Install a community node**
3. Enter: `@vectorizer/n8n-nodes-vectorizer`
4. Click **Install**

### Self-Hosted n8n

```bash
# Navigate to your n8n installation
cd ~/.n8n

# Install the node
npm install @vectorizer/n8n-nodes-vectorizer

# Restart n8n
```

### Docker

Add to your `docker-compose.yml`:

```yaml
services:
  n8n:
    image: n8nio/n8n
    environment:
      - N8N_COMMUNITY_PACKAGES=@vectorizer/n8n-nodes-vectorizer
```

## Configuration

### Create Credentials

1. In n8n, go to **Credentials** → **Add credential**
2. Search for **Vectorizer API**
3. Enter:
   - **Host URL**: `http://your-vectorizer-host:15002`
   - **API Key**: Your API key (optional)

## Available Nodes

### Vectorizer Node

The main node with three resources:

#### Collection Resource

| Operation | Description |
|-----------|-------------|
| Create | Create a new collection |
| Delete | Delete a collection |
| Get | Get collection info |
| List | List all collections |

#### Vector Resource

| Operation | Description |
|-----------|-------------|
| Insert | Insert a single vector |
| Batch Insert | Insert multiple vectors |
| Delete | Delete a vector |
| Get | Get a vector by ID |

#### Search Resource

| Operation | Description |
|-----------|-------------|
| Vector Search | Search by vector similarity |
| Semantic Search | Search by text (auto-embedding) |
| Hybrid Search | Combined vector + keyword search |

## Example Workflows

### RAG Pipeline

Build a Retrieval-Augmented Generation pipeline:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Webhook    │───▶│ Vectorizer  │───▶│   OpenAI    │───▶│  Respond    │
│  Trigger    │    │   Search    │    │   Chat      │    │  Webhook    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

**Workflow JSON:**

```json
{
  "nodes": [
    {
      "name": "Webhook",
      "type": "n8n-nodes-base.webhook",
      "parameters": {
        "path": "rag-query",
        "httpMethod": "POST"
      }
    },
    {
      "name": "Vectorizer Search",
      "type": "@vectorizer/n8n-nodes-vectorizer.vectorizer",
      "parameters": {
        "resource": "search",
        "operation": "semanticSearch",
        "collection": "documents",
        "query": "={{ $json.query }}",
        "limit": 5
      }
    },
    {
      "name": "OpenAI",
      "type": "n8n-nodes-base.openAi",
      "parameters": {
        "operation": "chat",
        "messages": [
          {
            "role": "system",
            "content": "Answer based on context: {{ $json.results }}"
          },
          {
            "role": "user",
            "content": "={{ $('Webhook').json.query }}"
          }
        ]
      }
    }
  ]
}
```

### Document Ingestion

Automatically index documents from various sources:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Google    │───▶│    Text     │───▶│ Vectorizer  │───▶│   Slack     │
│   Drive     │    │  Extractor  │    │   Insert    │    │   Notify    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### Scheduled Search

Run periodic semantic searches:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Cron      │───▶│ Vectorizer  │───▶│   Email     │
│  Trigger    │    │   Search    │    │   Send      │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Node Parameters

### Collection: Create

| Parameter | Type | Description |
|-----------|------|-------------|
| Collection Name | string | Unique collection identifier |
| Dimension | number | Vector dimension (e.g., 384, 768, 1536) |
| Distance Metric | select | cosine, euclidean, dot |

### Vector: Insert

| Parameter | Type | Description |
|-----------|------|-------------|
| Collection | string | Target collection |
| ID | string | Vector identifier |
| Vector | array | Float array |
| Payload | json | Metadata (optional) |

### Vector: Batch Insert

| Parameter | Type | Description |
|-----------|------|-------------|
| Collection | string | Target collection |
| Vectors | array | Array of {id, vector, payload} |

### Search: Semantic Search

| Parameter | Type | Description |
|-----------|------|-------------|
| Collection | string | Collection to search |
| Query | string | Natural language query |
| Limit | number | Max results (default: 10) |
| Score Threshold | number | Min similarity score |

### Search: Hybrid Search

| Parameter | Type | Description |
|-----------|------|-------------|
| Collection | string | Collection to search |
| Query | string | Search query |
| Alpha | number | Vector vs keyword weight (0-1) |
| Limit | number | Max results |

## Integration with Other Nodes

Vectorizer integrates seamlessly with 400+ n8n nodes:

| Integration | Use Case |
|-------------|----------|
| **OpenAI** | RAG pipelines, chat with context |
| **Google Drive** | Index documents automatically |
| **Slack** | Semantic search in channels |
| **GitHub** | Code search, issue similarity |
| **Notion** | Knowledge base search |
| **Airtable** | Database-backed vector search |
| **Webhook** | Real-time search API |

## Best Practices

### Error Handling

Use n8n's error handling for robust workflows:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Vectorizer  │───▶│   IF Node   │───▶│   Error     │
│   Search    │    │  Has Error? │    │   Handler   │
└─────────────┘    └─────────────┘    └─────────────┘
```

### Rate Limiting

For batch operations, use the **Split In Batches** node:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Data      │───▶│   Split In  │───▶│ Vectorizer  │
│   Source    │    │   Batches   │    │ Batch Insert│
└─────────────┘    └─────────────┘    └─────────────┘
```

### Credentials Security

- Never hardcode API keys in workflows
- Use n8n credentials for secure storage
- Rotate API keys periodically

## Troubleshooting

### Connection Failed

- Verify Vectorizer host URL is accessible from n8n
- Check firewall rules allow port 15002
- Ensure API key is correct (if authentication enabled)

### Search Returns Empty

- Verify collection exists
- Check vectors have been indexed
- Adjust score threshold

### Batch Insert Timeout

- Reduce batch size
- Increase n8n timeout settings
- Use Split In Batches node

## Related Topics

- [Langflow Integration](./LANGFLOW_INTEGRATION.md) - Visual LLM app building
- [API Reference](../api/API_REFERENCE.md) - REST API documentation
- [SDKs](../sdks/README.md) - Programmatic access

