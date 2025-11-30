# Qdrant Compatibility Examples

Code examples and tutorials for using Qdrant-compatible APIs with Vectorizer.

## Table of Contents

1. [Basic Operations](#basic-operations)
2. [Collection Management](#collection-management)
3. [Vector Operations](#vector-operations)
4. [Search Operations](#search-operations)
5. [Query API](#query-api)
6. [Search Groups & Matrix](#search-groups--matrix)
7. [gRPC Examples](#grpc-examples)
8. [Filter Examples](#filter-examples)
9. [Batch Operations](#batch-operations)
10. [Error Handling](#error-handling)
11. [Best Practices](#best-practices)
12. [Testing Scripts](#testing-scripts)

## Basic Operations

### Base URL

```bash
BASE_URL="http://localhost:15002/qdrant"
```

### Authentication (if enabled)

```bash
# With API key
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://localhost:15002/qdrant/collections
```

## Collection Management

### Create Collection

```bash
curl -X PUT http://localhost:15002/qdrant/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    },
    "hnsw_config": {
      "m": 16,
      "ef_construct": 100
    }
  }'
```

**Python Example**:
```python
import requests

response = requests.put(
    "http://localhost:15002/qdrant/collections/my_collection",
    json={
        "vectors": {
            "size": 384,
            "distance": "Cosine"
        },
        "hnsw_config": {
            "m": 16,
            "ef_construct": 100
        }
    }
)
print(response.json())
```

**JavaScript Example**:
```javascript
const response = await fetch('http://localhost:15002/qdrant/collections/my_collection', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    vectors: {
      size: 384,
      distance: 'Cosine'
    },
    hnsw_config: {
      m: 16,
      ef_construct: 100
    }
  })
});
const data = await response.json();
console.log(data);
```

### List Collections

```bash
curl http://localhost:15002/qdrant/collections
```

### Get Collection Info

```bash
curl http://localhost:15002/qdrant/collections/my_collection
```

### Update Collection

```bash
curl -X PATCH http://localhost:15002/qdrant/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "optimizer_config": {
      "indexing_threshold": 20000
    }
  }'
```

### Delete Collection

```bash
curl -X DELETE http://localhost:15002/qdrant/collections/my_collection
```

## Vector Operations

### Upsert Points

```bash
curl -X PUT http://localhost:15002/qdrant/collections/my_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "1",
        "vector": [0.1, 0.2, 0.3, ...],
        "payload": {
          "text": "Example document",
          "category": "electronics",
          "price": 99.99
        }
      }
    ]
  }'
```

**Python Example**:
```python
import requests
import numpy as np

# Generate random vector
vector = np.random.rand(384).tolist()

response = requests.put(
    "http://localhost:15002/qdrant/collections/my_collection/points",
    json={
        "points": [
            {
                "id": "1",
                "vector": vector,
                "payload": {
                    "text": "Example document",
                    "category": "electronics",
                    "price": 99.99
                }
            }
        ]
    }
)
print(response.json())
```

### Batch Upsert

```bash
curl -X PUT http://localhost:15002/qdrant/collections/my_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {"id": "1", "vector": [...], "payload": {"text": "doc1"}},
      {"id": "2", "vector": [...], "payload": {"text": "doc2"}},
      {"id": "3", "vector": [...], "payload": {"text": "doc3"}}
    ]
  }'
```

### Retrieve Points

```bash
curl -X GET "http://localhost:15002/qdrant/collections/my_collection/points?ids=1,2,3" \
  -H "Content-Type: application/json"
```

**With Payload Filtering**:
```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "ids": ["1", "2", "3"],
    "with_payload": true,
    "with_vector": false
  }'
```

### Delete Points

**By ID**:
```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/delete \
  -H "Content-Type: application/json" \
  -d '{
    "points": ["1", "2", "3"]
  }'
```

**By Filter**:
```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/delete \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "must": [
        {"type": "match", "key": "category", "match_value": "old"}
      ]
    }
  }'
```

### Count Points

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/count \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "must": [
        {"type": "match", "key": "status", "match_value": "active"}
      ]
    }
  }'
```

### Scroll Points

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/scroll \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 100,
    "offset": null,
    "with_payload": true,
    "with_vector": false
  }'
```

## Search Operations

### Basic Search

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "limit": 10,
    "with_payload": true
  }'
```

**Python Example**:
```python
import requests
import numpy as np

query_vector = np.random.rand(384).tolist()

response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/search",
    json={
        "vector": query_vector,
        "limit": 10,
        "with_payload": True
    }
)

results = response.json()["result"]
for result in results:
    print(f"ID: {result['id']}, Score: {result['score']}")
    print(f"Payload: {result['payload']}")
```

### Filtered Search

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "filter": {
      "must": [
        {"type": "match", "key": "category", "match_value": "electronics"},
        {"type": "range", "key": "price", "range": {"gte": 50, "lte": 200}}
      ]
    },
    "limit": 10
  }'
```

### Batch Search

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search/batch \
  -H "Content-Type: application/json" \
  -d '{
    "searches": [
      {"vector": [...], "limit": 5},
      {"vector": [...], "limit": 5},
      {"vector": [...], "limit": 5}
    ]
  }'
```

### Recommend Points

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/recommend \
  -H "Content-Type: application/json" \
  -d '{
    "positive": ["1", "2", "3"],
    "negative": ["4", "5"],
    "limit": 10
  }'
```

## Query API

The Query API provides a unified interface for search operations with support for prefetch, filtering, and complex queries.

### Basic Query

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": [0.1, 0.2, 0.3, ...],
    "limit": 10,
    "with_payload": true
  }'
```

### Query with Prefetch

Multi-stage retrieval with prefetch for better results:

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query \
  -H "Content-Type: application/json" \
  -d '{
    "prefetch": [
      {
        "query": [0.1, 0.2, 0.3, ...],
        "limit": 100
      }
    ],
    "query": {
      "fusion": "rrf"
    },
    "limit": 10
  }'
```

### Query with Nested Prefetch

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query \
  -H "Content-Type: application/json" \
  -d '{
    "prefetch": [
      {
        "prefetch": [
          {"query": [0.1, 0.2, ...], "limit": 200}
        ],
        "query": [0.3, 0.4, ...],
        "limit": 50
      }
    ],
    "query": [0.5, 0.6, ...],
    "limit": 10,
    "with_payload": true
  }'
```

### Batch Query

Execute multiple queries in a single request:

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query/batch \
  -H "Content-Type: application/json" \
  -d '{
    "searches": [
      {"query": [0.1, 0.2, ...], "limit": 5},
      {"query": [0.3, 0.4, ...], "limit": 5},
      {"query": [0.5, 0.6, ...], "limit": 5}
    ]
  }'
```

**Python Example**:
```python
import requests

queries = [
    {"query": vector1, "limit": 10, "with_payload": True},
    {"query": vector2, "limit": 10, "with_payload": True},
    {"query": vector3, "limit": 10, "with_payload": True}
]

response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/query/batch",
    json={"searches": queries}
)

results = response.json()["result"]
for i, result in enumerate(results):
    print(f"Query {i}: {len(result['result'])} results")
```

### Query Groups

Group results by a payload field:

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query/groups \
  -H "Content-Type: application/json" \
  -d '{
    "query": [0.1, 0.2, 0.3, ...],
    "group_by": "category",
    "group_size": 3,
    "limit": 5,
    "with_payload": true
  }'
```

**Python Example**:
```python
import requests

response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/query/groups",
    json={
        "query": query_vector,
        "group_by": "category",
        "group_size": 3,
        "limit": 5,
        "with_payload": True
    }
)

groups = response.json()["result"]["groups"]
for group in groups:
    print(f"Category: {group['id']}")
    for hit in group["hits"]:
        print(f"  - {hit['id']}: {hit['score']:.4f}")
```

### Recommend Query

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": {
      "recommend": {
        "positive": ["point-id-1", "point-id-2"],
        "negative": ["point-id-3"]
      }
    },
    "limit": 10,
    "with_payload": true
  }'
```

### Discover Query

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": {
      "discover": {
        "target": [0.1, 0.2, ...],
        "context": [
          {"positive": [0.3, 0.4, ...], "negative": [0.5, 0.6, ...]}
        ]
      }
    },
    "limit": 10
  }'
```

### Fusion Query

Combine results from multiple prefetch stages using RRF or DBSF:

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/query \
  -H "Content-Type: application/json" \
  -d '{
    "prefetch": [
      {"query": [0.1, 0.2, ...], "limit": 50},
      {"query": [0.3, 0.4, ...], "limit": 50}
    ],
    "query": {
      "fusion": "rrf"
    },
    "limit": 10
  }'
```

## Search Groups & Matrix

### Search Groups

Group search results by a payload field:

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search/groups \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "group_by": "category",
    "group_size": 3,
    "limit": 5,
    "with_payload": true
  }'
```

**Python Example**:
```python
import requests

response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/search/groups",
    json={
        "vector": query_vector,
        "group_by": "category",
        "group_size": 3,
        "limit": 5,
        "with_payload": True,
        "score_threshold": 0.5
    }
)

groups = response.json()["result"]["groups"]
for group in groups:
    print(f"Group: {group['id']}")
    for hit in group["hits"]:
        print(f"  ID: {hit['id']}, Score: {hit['score']:.4f}")
```

### Search Matrix Pairs

Compute pairwise similarity between sampled points:

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search/matrix/pairs \
  -H "Content-Type: application/json" \
  -d '{
    "sample": 50,
    "limit": 100
  }'
```

**Response**:
```json
{
  "result": {
    "pairs": [
      {"a": "point-1", "b": "point-2", "score": 0.95},
      {"a": "point-1", "b": "point-3", "score": 0.87},
      {"a": "point-2", "b": "point-3", "score": 0.82}
    ]
  }
}
```

**Python Example**:
```python
import requests

response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/search/matrix/pairs",
    json={
        "sample": 100,
        "limit": 500,
        "filter": {
            "must": [
                {"type": "match", "key": "status", "match_value": "active"}
            ]
        }
    }
)

pairs = response.json()["result"]["pairs"]
print(f"Found {len(pairs)} similar pairs")
for pair in pairs[:10]:
    print(f"  {pair['a']} <-> {pair['b']}: {pair['score']:.4f}")
```

### Search Matrix Offsets

Get similarity matrix in compact offset format (for larger computations):

```bash
curl -X POST http://localhost:15002/qdrant/collections/my_collection/points/search/matrix/offsets \
  -H "Content-Type: application/json" \
  -d '{
    "sample": 100,
    "limit": 1000
  }'
```

**Response**:
```json
{
  "result": {
    "ids": ["point-1", "point-2", "point-3", ...],
    "offsets": [0, 3, 6, ...],
    "scores": [0.95, 0.87, 0.82, ...]
  }
}
```

**Python Example - Building a Similarity Graph**:
```python
import requests
import networkx as nx

# Get similarity matrix
response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/search/matrix/pairs",
    json={"sample": 100, "limit": 500}
)

pairs = response.json()["result"]["pairs"]

# Build graph from similarity pairs
G = nx.Graph()
for pair in pairs:
    if pair["score"] > 0.8:  # Threshold
        G.add_edge(pair["a"], pair["b"], weight=pair["score"])

print(f"Graph has {G.number_of_nodes()} nodes and {G.number_of_edges()} edges")

# Find clusters
from networkx.algorithms import community
communities = community.louvain_communities(G)
print(f"Found {len(communities)} communities")
```

## gRPC Examples

Vectorizer supports the Qdrant gRPC protocol on port 15003 (configurable).

### Python gRPC Client

First, install the Qdrant gRPC client:

```bash
pip install grpcio grpcio-tools qdrant-client
```

**Using qdrant-client with Vectorizer gRPC**:
```python
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct

# Connect to Vectorizer gRPC endpoint
client = QdrantClient(
    host="localhost",
    port=15003,  # Vectorizer gRPC port
    prefer_grpc=True
)

# Create collection
client.create_collection(
    collection_name="my_collection",
    vectors_config=VectorParams(size=384, distance=Distance.COSINE)
)

# Upsert points
client.upsert(
    collection_name="my_collection",
    points=[
        PointStruct(
            id=1,
            vector=[0.1] * 384,
            payload={"text": "First document", "category": "A"}
        ),
        PointStruct(
            id=2,
            vector=[0.2] * 384,
            payload={"text": "Second document", "category": "B"}
        )
    ]
)

# Search
results = client.search(
    collection_name="my_collection",
    query_vector=[0.15] * 384,
    limit=10
)

for result in results:
    print(f"ID: {result.id}, Score: {result.score:.4f}")
    print(f"  Payload: {result.payload}")
```

### Raw gRPC with Python

```python
import grpc
from qdrant_client.grpc import qdrant_pb2, qdrant_pb2_grpc

# Create channel
channel = grpc.insecure_channel('localhost:15003')

# Collections service
collections_stub = qdrant_pb2_grpc.CollectionsStub(channel)

# List collections
response = collections_stub.List(qdrant_pb2.ListCollectionsRequest())
for collection in response.collections:
    print(f"Collection: {collection.name}")

# Points service
points_stub = qdrant_pb2_grpc.PointsStub(channel)

# Search
search_request = qdrant_pb2.SearchPoints(
    collection_name="my_collection",
    vector=qdrant_pb2.Vector(data=[0.1] * 384),
    limit=10,
    with_payload=qdrant_pb2.WithPayloadSelector(enable=True)
)
search_response = points_stub.Search(search_request)

for result in search_response.result:
    print(f"ID: {result.id.num}, Score: {result.score}")
```

### Rust gRPC Client

```rust
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    CreateCollection, Distance, SearchPoints, VectorParams, VectorsConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Vectorizer gRPC
    let client = QdrantClient::from_url("http://localhost:15003").build()?;

    // Create collection
    client
        .create_collection(&CreateCollection {
            collection_name: "my_collection".to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(vectors_config::Config::Params(VectorParams {
                    size: 384,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        })
        .await?;

    // Upsert points
    client
        .upsert_points_batch_blocking(
            "my_collection",
            None,
            vec![
                PointStruct::new(
                    1,
                    vec![0.1f32; 384],
                    Payload::try_from(serde_json::json!({
                        "text": "Document 1",
                        "category": "A"
                    }))?,
                ),
            ],
            None,
            100,
        )
        .await?;

    // Search
    let results = client
        .search_points(&SearchPoints {
            collection_name: "my_collection".to_string(),
            vector: vec![0.15f32; 384],
            limit: 10,
            with_payload: Some(true.into()),
            ..Default::default()
        })
        .await?;

    for result in results.result {
        println!("ID: {:?}, Score: {}", result.id, result.score);
    }

    Ok(())
}
```

### Go gRPC Client

```go
package main

import (
    "context"
    "fmt"
    "log"

    "github.com/qdrant/go-client/qdrant"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
)

func main() {
    // Connect to Vectorizer gRPC
    conn, err := grpc.Dial(
        "localhost:15003",
        grpc.WithTransportCredentials(insecure.NewCredentials()),
    )
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()

    // Create client
    client := qdrant.NewQdrantClient(conn)

    // List collections
    collections, err := client.ListCollections(context.Background())
    if err != nil {
        log.Fatal(err)
    }

    for _, c := range collections.GetCollections() {
        fmt.Printf("Collection: %s\n", c.GetName())
    }

    // Search
    results, err := client.Search(context.Background(), &qdrant.SearchPoints{
        CollectionName: "my_collection",
        Vector:         make([]float32, 384), // Your query vector
        Limit:          10,
        WithPayload:    &qdrant.WithPayloadSelector{Enable: true},
    })
    if err != nil {
        log.Fatal(err)
    }

    for _, result := range results.GetResult() {
        fmt.Printf("ID: %v, Score: %f\n", result.GetId(), result.GetScore())
    }
}
```

### gRPC Services Available

| Service | Methods | Description |
|---------|---------|-------------|
| `Collections` | Get, List, Create, Update, Delete | Collection management |
| `Points` | Upsert, Get, Delete, Search, SearchBatch, Scroll, Recommend, Count | Vector operations |
| `Snapshots` | Create, List, Delete | Snapshot management |

### gRPC vs REST Performance

| Metric | gRPC | REST |
|--------|------|------|
| Latency | Lower (~20-30% faster) | Baseline |
| Throughput | Higher (~40% more) | Baseline |
| Binary efficiency | High (protobuf) | Medium (JSON) |
| Streaming | Supported | Not supported |

## Filter Examples

### Match Filter

```json
{
  "filter": {
    "must": [
      {"type": "match", "key": "status", "match_value": "active"}
    ]
  }
}
```

### Range Filter

```json
{
  "filter": {
    "must": [
      {
        "type": "range",
        "key": "price",
        "range": {"gte": 50, "lte": 200}
      }
    ]
  }
}
```

### Geo Bounding Box Filter

```json
{
  "filter": {
    "must": [
      {
        "type": "geo_bounding_box",
        "key": "location",
        "geo_bounding_box": {
          "top_right": {"lat": 40.8, "lon": -73.9},
          "bottom_left": {"lat": 40.7, "lon": -74.0}
        }
      }
    ]
  }
}
```

### Geo Radius Filter

```json
{
  "filter": {
    "must": [
      {
        "type": "geo_radius",
        "key": "location",
        "geo_radius": {
          "center": {"lat": 40.7589, "lon": -73.9851},
          "radius": 5.0
        }
      }
    ]
  }
}
```

### Values Count Filter

```json
{
  "filter": {
    "must": [
      {
        "type": "values_count",
        "key": "tags",
        "values_count": {"gte": 3}
      }
    ]
  }
}
```

### Complex Nested Filter

```json
{
  "filter": {
    "must": [
      {"type": "match", "key": "category", "match_value": "electronics"}
    ],
    "should": [
      {"type": "range", "key": "price", "range": {"lte": 100}},
      {"type": "match", "key": "on_sale", "match_value": true}
    ],
    "must_not": [
      {"type": "match", "key": "discontinued", "match_value": true}
    ]
  }
}
```

## Batch Operations

### Batch Upsert

```python
import requests

points = []
for i in range(1000):
    points.append({
        "id": str(i),
        "vector": [0.1] * 384,  # Your vector here
        "payload": {"index": i, "text": f"Document {i}"}
    })

# Batch in chunks of 500
for i in range(0, len(points), 500):
    chunk = points[i:i+500]
    response = requests.put(
        "http://localhost:15002/qdrant/collections/my_collection/points",
        json={"points": chunk}
    )
    print(f"Inserted {len(chunk)} points")
```

### Batch Search

```python
import requests

queries = [
    {"vector": vector1, "limit": 10},
    {"vector": vector2, "limit": 10},
    {"vector": vector3, "limit": 10}
]

response = requests.post(
    "http://localhost:15002/qdrant/collections/my_collection/points/search/batch",
    json={"searches": queries}
)

results = response.json()["result"]
for i, result_set in enumerate(results):
    print(f"Query {i}: {len(result_set)} results")
```

## Error Handling

### Python Error Handling

```python
import requests
from requests.exceptions import RequestException

try:
    response = requests.put(
        "http://localhost:15002/qdrant/collections/my_collection",
        json={"vectors": {"size": 384, "distance": "Cosine"}}
    )
    response.raise_for_status()
    print("Success:", response.json())
except requests.exceptions.HTTPError as e:
    if e.response.status_code == 409:
        print("Collection already exists")
    elif e.response.status_code == 404:
        print("Collection not found")
    else:
        print(f"Error: {e.response.status_code} - {e.response.text}")
except RequestException as e:
    print(f"Request failed: {e}")
```

### JavaScript Error Handling

```javascript
async function createCollection(name) {
  try {
    const response = await fetch(
      `http://localhost:15002/qdrant/collections/${name}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          vectors: { size: 384, distance: 'Cosine' }
        })
      }
    );
    
    if (!response.ok) {
      const error = await response.json();
      if (response.status === 409) {
        console.error('Collection already exists');
      } else {
        console.error('Error:', error);
      }
      return;
    }
    
    const data = await response.json();
    console.log('Success:', data);
  } catch (error) {
    console.error('Request failed:', error);
  }
}
```

## Best Practices

### 1. Use Batch Operations

**Bad**:
```python
for point in points:
    requests.put(url, json={"points": [point]})  # One request per point
```

**Good**:
```python
requests.put(url, json={"points": points})  # Batch all points
```

### 2. Enable Payload Indexing

```python
# Index frequently filtered fields
# This happens automatically for common fields like file_path, chunk_index
```

### 3. Optimize Search Parameters

```python
# Use appropriate limit
search_params = {
    "vector": query_vector,
    "limit": 10,  # Don't request more than needed
    "score_threshold": 0.5,  # Filter low-quality results early
    "with_payload": True,  # Only if needed
    "with_vector": False  # Usually not needed
}
```

### 4. Handle Errors Gracefully

```python
def safe_search(collection, vector, limit=10):
    try:
        response = requests.post(
            f"http://localhost:15002/qdrant/collections/{collection}/points/search",
            json={"vector": vector, "limit": limit},
            timeout=5.0
        )
        response.raise_for_status()
        return response.json()["result"]
    except requests.exceptions.Timeout:
        print("Request timeout")
        return []
    except requests.exceptions.HTTPError as e:
        print(f"HTTP error: {e}")
        return []
```

### 5. Use Connection Pooling

```python
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

session = requests.Session()
retry_strategy = Retry(
    total=3,
    backoff_factor=1,
    status_forcelist=[500, 502, 503, 504]
)
adapter = HTTPAdapter(max_retries=retry_strategy)
session.mount("http://", adapter)
session.mount("https://", adapter)

# Use session for all requests
response = session.post(url, json=data)
```

### 6. Monitor Performance

```python
import time

start = time.time()
response = requests.post(url, json=data)
elapsed = time.time() - start

result = response.json()
print(f"Query time: {elapsed:.3f}s")
print(f"Server time: {result.get('time', 0)}s")
print(f"Results: {len(result.get('result', []))}")
```

## Migration Examples

### Migrating from Qdrant Python Client

**Qdrant Client**:
```python
from qdrant_client import QdrantClient

client = QdrantClient("localhost", port=6333)
client.create_collection("my_collection", vectors_config=...)
```

**Vectorizer REST API**:
```python
import requests

BASE_URL = "http://localhost:15002/qdrant"

def create_collection(name, vectors_config):
    response = requests.put(
        f"{BASE_URL}/collections/{name}",
        json={"vectors": vectors_config}
    )
    return response.json()
```

### Migrating Search Queries

**Qdrant Client**:
```python
results = client.search(
    collection_name="my_collection",
    query_vector=vector,
    limit=10
)
```

**Vectorizer REST API**:
```python
response = requests.post(
    f"{BASE_URL}/collections/my_collection/points/search",
    json={"vector": vector, "limit": 10}
)
results = response.json()["result"]
```

## Migration Examples

### Export Collection from Qdrant

```rust
use vectorizer::migration::qdrant::QdrantDataExporter;

// Export collection from Qdrant instance
let exported = QdrantDataExporter::export_collection(
    "http://localhost:6333",
    "my_collection"
).await?;

// Save to file for backup
QdrantDataExporter::export_to_file(&exported, "my_collection_backup.json")?;

println!("Exported {} points", exported.points.len());
```

### Import Collection into Vectorizer

```rust
use vectorizer::migration::qdrant::{QdrantDataImporter, MigrationValidator};

// Load exported collection
let exported = QdrantDataImporter::import_from_file("my_collection_backup.json")?;

// Validate before importing
let validation = MigrationValidator::validate_export(&exported)?;
if !validation.is_valid {
    eprintln!("Validation errors: {:?}", validation.errors);
    return Err("Invalid export data".into());
}

// Import into Vectorizer
let result = QdrantDataImporter::import_collection(&store, &exported).await?;

// Validate integrity
let integrity = MigrationValidator::validate_integrity(&exported, result.imported_count)?;
println!("Import complete: {:.2}% integrity", integrity.integrity_percentage);
```

### Convert Qdrant Configuration

```rust
use vectorizer::migration::qdrant::{QdrantConfigParser, ConfigFormat};

// Parse Qdrant config file
let qdrant_config = QdrantConfigParser::parse_file("qdrant_config.yaml")?;

// Validate
let validation = QdrantConfigParser::validate(&qdrant_config)?;
if !validation.is_valid {
    for error in &validation.errors {
        eprintln!("Error: {}", error);
    }
}

// Convert to Vectorizer format
let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&qdrant_config)?;

// Create collections
for (name, config) in vectorizer_configs {
    store.create_collection(&name, config)?;
    println!("Created collection: {}", name);
}
```

## Testing Scripts

### Automated Test Script

Run comprehensive compatibility tests:

```bash
# Run all Qdrant compatibility tests
bash scripts/test-qdrant-compatibility.sh

# With verbose output
VERBOSE=true bash scripts/test-qdrant-compatibility.sh

# Custom base URL
BASE_URL=http://localhost:15002/qdrant bash scripts/test-qdrant-compatibility.sh
```

The script tests:
- Collection management (create, list, get, delete, update)
- Vector operations (upsert, retrieve, delete, scroll, count)
- Search operations (single, batch, recommend)
- Error handling (404, 400, 409 errors)

### Interactive Test Script

Interactive menu-driven testing:

```bash
# Start interactive test menu
bash scripts/test-qdrant-interactive.sh
```

Menu options:
1. Test Collection Management
2. Test Vector Operations
3. Test Search Operations
4. Test Filter Operations
5. Test Error Handling
6. Run All Tests
7. Performance Benchmark

### Troubleshooting Examples

For detailed troubleshooting examples with scripts, see [Troubleshooting Examples](./TROUBLESHOOTING_EXAMPLES.md).

## Additional Resources

- [API Compatibility Matrix](./API_COMPATIBILITY.md)
- [Feature Parity](./FEATURE_PARITY.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [Troubleshooting Examples](./TROUBLESHOOTING_EXAMPLES.md) - Practical troubleshooting scripts
- [Testing Guide](./TESTING.md)
- [Migration Guide](../specs/QDRANT_MIGRATION.md)

