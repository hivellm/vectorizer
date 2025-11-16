---
title: Integration
module: integration
id: integration-index
order: 0
description: Integration guides for Vectorizer
tags: [integration, api, sdk, tools]
---

# Integration

Guides for integrating Vectorizer with other systems, frameworks, and tools.

## Guides

### [Integration Guide](./INTEGRATION.md)
Complete integration guide:
- Web framework integration (FastAPI, Express, Axum)
- Database integration (PostgreSQL, MongoDB)
- LLM integration (OpenAI, LangChain)
- ETL pipeline integration (Airflow, Kafka)
- Monitoring integration (Prometheus, Grafana, Datadog)
- CI/CD integration (GitHub Actions, GitLab CI)
- Reverse proxy setup (Nginx, Caddy, Traefik)
- Authentication and load balancing

## Quick Integration Examples

### Python (FastAPI)

```python
from fastapi import FastAPI
from vectorizer_sdk import VectorizerClient

app = FastAPI()
vectorizer = VectorizerClient("http://localhost:15002")

@app.post("/search")
async def search(query: str):
    results = await vectorizer.search("my_collection", query)
    return {"results": results}
```

### Node.js (Express)

```javascript
const { VectorizerClient } = require('@hivellm/vectorizer-sdk');
const vectorizer = new VectorizerClient('http://localhost:15002');

app.post('/search', async (req, res) => {
  const results = await vectorizer.search('my_collection', req.body.query);
  res.json({ results });
});
```

## Related Topics

- [REST API Reference](../api/API_REFERENCE.md) - API documentation
- [SDKs Guide](../sdks/README.md) - Client SDKs
- [Configuration Guide](../configuration/SERVER.md) - Server setup

