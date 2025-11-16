---
title: Integration Guide
module: integration
id: integration-guide
order: 1
description: Integrating Vectorizer with other systems and tools
tags: [integration, api, sdk, tools, systems]
---

# Integration Guide

Complete guide to integrating Vectorizer with other systems, frameworks, and tools.

## Integration Methods

### REST API

Vectorizer provides a comprehensive REST API that can be integrated with any HTTP client.

**Base URL:**

```
http://localhost:15002
```

**Example:**

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "my_collection", "dimension": 384}'
```

**See:** [REST API Reference](../api/API_REFERENCE.md) for complete endpoint documentation.

### SDKs

Vectorizer provides official SDKs for multiple languages:

- **Python**: `pip install vectorizer-sdk`
- **TypeScript/JavaScript**: `npm install @hivellm/vectorizer-sdk`
- **Rust**: `cargo add vectorizer-sdk`

**See:** [SDKs Guide](../sdks/README.md) for SDK documentation.

### MCP (Model Context Protocol)

Vectorizer supports MCP for AI assistant integration.

**Endpoint:** `ws://localhost:15002/mcp`

**See:** MCP documentation for details.

## Web Framework Integration

### Python (FastAPI)

**Example:**

```python
from fastapi import FastAPI
from vectorizer_sdk import VectorizerClient

app = FastAPI()
vectorizer = VectorizerClient("http://localhost:15002")

@app.post("/search")
async def search(query: str, collection: str = "default"):
    results = await vectorizer.search(collection, query, limit=10)
    return {"results": results}
```

### Node.js (Express)

**Example:**

```javascript
const express = require('express');
const { VectorizerClient } = require('@hivellm/vectorizer-sdk');

const app = express();
const vectorizer = new VectorizerClient('http://localhost:15002');

app.post('/search', async (req, res) => {
  const { query, collection = 'default' } = req.body;
  const results = await vectorizer.search(collection, query, { limit: 10 });
  res.json({ results });
});
```

### Rust (Axum)

**Example:**

```rust
use axum::{extract::Query, Json};
use vectorizer_sdk::VectorizerClient;

async fn search(
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let client = VectorizerClient::new("http://localhost:15002")?;
    let query = params.get("query").unwrap();
    let results = client.search("default", query, 10).await?;
    Json(json!({ "results": results }))
}
```

## Database Integration

### PostgreSQL (pgvector alternative)

Use Vectorizer alongside PostgreSQL for hybrid storage:

```python
import psycopg2
from vectorizer_sdk import VectorizerClient

# PostgreSQL for relational data
pg_conn = psycopg2.connect("dbname=mydb")

# Vectorizer for vector search
vectorizer = VectorizerClient("http://localhost:15002")

# Store metadata in PostgreSQL, vectors in Vectorizer
async def store_document(doc_id, content, metadata):
    # Store in PostgreSQL
    cursor = pg_conn.cursor()
    cursor.execute(
        "INSERT INTO documents (id, content, metadata) VALUES (%s, %s, %s)",
        (doc_id, content, json.dumps(metadata))
    )
    
    # Store vector in Vectorizer
    await vectorizer.insert_text("documents", content, id=doc_id, metadata=metadata)
```

### MongoDB Integration

```python
from pymongo import MongoClient
from vectorizer_sdk import VectorizerClient

mongo = MongoClient("mongodb://localhost:27017")
db = mongo.mydb
vectorizer = VectorizerClient("http://localhost:15002")

async def store_document(doc):
    # Store in MongoDB
    db.documents.insert_one(doc)
    
    # Store vector in Vectorizer
    await vectorizer.insert_text(
        "documents",
        doc["content"],
        id=str(doc["_id"]),
        metadata=doc.get("metadata", {})
    )
```

## LLM Integration

### OpenAI Integration

**Example with RAG:**

```python
import openai
from vectorizer_sdk import VectorizerClient

openai_client = openai.OpenAI()
vectorizer = VectorizerClient("http://localhost:15002")

async def answer_question(question: str):
    # Retrieve relevant context
    results = await vectorizer.search("knowledge_base", question, limit=5)
    context = "\n".join([r["payload"]["content"] for r in results])
    
    # Generate answer with LLM
    response = openai_client.chat.completions.create(
        model="gpt-4",
        messages=[
            {"role": "system", "content": "Answer based on the following context:"},
            {"role": "user", "content": f"Context: {context}\n\nQuestion: {question}"}
        ]
    )
    
    return response.choices[0].message.content
```

### LangChain Integration

**Example:**

```python
from langchain.vectorstores import VectorStore
from langchain.embeddings import OpenAIEmbeddings
from vectorizer_sdk import VectorizerClient

class VectorizerStore(VectorStore):
    def __init__(self, collection_name: str):
        self.client = VectorizerClient("http://localhost:15002")
        self.collection = collection_name
    
    def add_texts(self, texts, metadatas=None):
        for i, text in enumerate(texts):
            metadata = metadatas[i] if metadatas else {}
            self.client.insert_text(self.collection, text, metadata=metadata)
    
    def similarity_search(self, query, k=4):
        results = self.client.search(self.collection, query, limit=k)
        return [{"page_content": r["payload"].get("content", ""), 
                 "metadata": r["payload"]} for r in results]
```

## ETL Pipeline Integration

### Apache Airflow

**Example DAG:**

```python
from airflow import DAG
from airflow.operators.python import PythonOperator
from vectorizer_sdk import VectorizerClient

def index_documents():
    client = VectorizerClient("http://localhost:15002")
    # Index documents from source
    documents = load_documents_from_source()
    await client.batch_insert_text("documents", documents)

dag = DAG('vectorizer_indexing', schedule_interval='@daily')
index_task = PythonOperator(
    task_id='index_documents',
    python_callable=index_documents,
    dag=dag
)
```

### Apache Kafka

**Example consumer:**

```python
from kafka import KafkaConsumer
from vectorizer_sdk import VectorizerClient
import json

consumer = KafkaConsumer('documents', bootstrap_servers='localhost:9092')
vectorizer = VectorizerClient("http://localhost:15002")

for message in consumer:
    doc = json.loads(message.value)
    await vectorizer.insert_text(
        "documents",
        doc["content"],
        id=doc["id"],
        metadata=doc.get("metadata", {})
    )
```

## Monitoring Integration

### Prometheus

Vectorizer exposes Prometheus metrics at `/prometheus/metrics`.

**Prometheus configuration:**

```yaml
scrape_configs:
  - job_name: 'vectorizer'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:15002']
    metrics_path: '/prometheus/metrics'
```

### Grafana Dashboard

Import Vectorizer metrics into Grafana:

1. Add Prometheus as data source
2. Import dashboard JSON (if available)
3. Or create custom dashboard using metrics

**Key metrics:**

- `vectorizer_search_requests_total`
- `vectorizer_search_latency_seconds`
- `vectorizer_collection_vectors_total`
- `vectorizer_memory_usage_bytes`

### Datadog Integration

**Custom metrics:**

```python
from datadog import initialize, api

options = {
    'api_key': 'your_api_key',
    'app_key': 'your_app_key'
}
initialize(**options)

# Send custom metrics
api.Metric.send(
    metric='vectorizer.search.latency',
    points=latency_ms,
    tags=['collection:my_collection']
)
```

## CI/CD Integration

### GitHub Actions

**Example workflow:**

```yaml
name: Test Vectorizer Integration

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      vectorizer:
        image: ghcr.io/hivellm/vectorizer:latest
        ports:
          - 15002:15002
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: |
          pytest tests/
```

### GitLab CI

**Example pipeline:**

```yaml
test:
  image: python:3.11
  services:
    - name: ghcr.io/hivellm/vectorizer:latest
      alias: vectorizer
  variables:
    VECTORIZER_URL: http://vectorizer:15002
  script:
    - pip install -r requirements.txt
    - pytest tests/
```

## Reverse Proxy Integration

### Nginx

**Basic configuration:**

```nginx
upstream vectorizer {
    server 127.0.0.1:15002;
}

server {
    listen 80;
    server_name vectorizer.example.com;

    location / {
        proxy_pass http://vectorizer;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Caddy

**Caddyfile:**

```
vectorizer.example.com {
    reverse_proxy localhost:15002
}
```

### Traefik

**Docker labels:**

```yaml
services:
  vectorizer:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.vectorizer.rule=Host(`vectorizer.example.com`)"
      - "traefik.http.services.vectorizer.loadbalancer.server.port=15002"
```

## Authentication Integration

### API Key Authentication (via Reverse Proxy)

**Nginx with API key:**

```nginx
map $http_x_api_key $api_key_valid {
    default 0;
    "your-secret-key" 1;
}

server {
    if ($api_key_valid = 0) {
        return 403;
    }
    
    location / {
        proxy_pass http://127.0.0.1:15002;
    }
}
```

### OAuth2 Integration

**Example with OAuth2 proxy:**

```yaml
services:
  oauth2-proxy:
    image: quay.io/oauth2-proxy/oauth2-proxy
    environment:
      - UPSTREAM=http://vectorizer:15002
  vectorizer:
    image: ghcr.io/hivellm/vectorizer:latest
```

## Load Balancing

### Multiple Instances

**Nginx load balancing:**

```nginx
upstream vectorizer {
    least_conn;
    server 127.0.0.1:15002;
    server 127.0.0.1:15012;
    server 127.0.0.1:15022;
}

server {
    location / {
        proxy_pass http://vectorizer;
    }
}
```

### Health Checks

**Nginx with health checks:**

```nginx
upstream vectorizer {
    server 127.0.0.1:15002 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:15012 backup;
}

server {
    location /health {
        proxy_pass http://vectorizer/health;
    }
}
```

## Related Topics

- [REST API Reference](../api/API_REFERENCE.md) - Complete API documentation
- [SDKs Guide](../sdks/README.md) - Client SDKs
- [Configuration Guide](../configuration/SERVER.md) - Server configuration
- [Monitoring Guide](../monitoring/MONITORING.md) - Monitoring setup

