# Qdrant Compatibility Testing Guide

Guide for testing Qdrant API compatibility and validating migration.

## Table of Contents

1. [Testing Approach](#testing-approach)
2. [Test Scenarios](#test-scenarios)
3. [Test Tools](#test-tools)
4. [Test Examples](#test-examples)
5. [Validation Checklist](#validation-checklist)

## Testing Approach

### Testing Strategy

1. **Unit Tests**: Test individual endpoints
2. **Integration Tests**: Test complete workflows
3. **Compatibility Tests**: Compare with Qdrant behavior
4. **Performance Tests**: Benchmark against native API

### Test Environment Setup

```bash
# Start Vectorizer server
./target/release/vectorizer

# Verify server is running
curl http://localhost:15002/api/status

# Verify Qdrant endpoints
curl http://localhost:15002/qdrant/collections
```

## Test Scenarios

### Collection Management Tests

#### Test: Create Collection

```bash
# Test collection creation
curl -X PUT http://localhost:15002/qdrant/collections/test_collection \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    }
  }'

# Verify collection exists
curl http://localhost:15002/qdrant/collections/test_collection
```

**Expected Result**: Collection created successfully, status 200

#### Test: Collection Already Exists

```bash
# Try to create duplicate collection
curl -X PUT http://localhost:15002/qdrant/collections/test_collection \
  -H "Content-Type: application/json" \
  -d '{"vectors": {"size": 384, "distance": "Cosine"}}'
```

**Expected Result**: HTTP 409, error message about existing collection

#### Test: Collection Not Found

```bash
# Try to access non-existent collection
curl http://localhost:15002/qdrant/collections/nonexistent
```

**Expected Result**: HTTP 404, error message about collection not found

### Vector Operations Tests

#### Test: Upsert Points

```bash
curl -X PUT http://localhost:15002/qdrant/collections/test_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "1",
        "vector": [0.1] * 384,
        "payload": {"text": "test"}
      }
    ]
  }'
```

**Expected Result**: Points inserted, operation status OK

#### Test: Retrieve Points

```bash
curl -X GET "http://localhost:15002/qdrant/collections/test_collection/points?ids=1"
```

**Expected Result**: Point retrieved with payload

#### Test: Invalid Vector Dimension

```bash
curl -X PUT http://localhost:15002/qdrant/collections/test_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "2",
        "vector": [0.1] * 100,  # Wrong dimension
        "payload": {}
      }
    ]
  }'
```

**Expected Result**: HTTP 400, dimension mismatch error

### Search Operations Tests

#### Test: Basic Search

```bash
curl -X POST http://localhost:15002/qdrant/collections/test_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1] * 384,
    "limit": 10
  }'
```

**Expected Result**: Search results with scores

#### Test: Filtered Search

```bash
curl -X POST http://localhost:15002/qdrant/collections/test_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1] * 384,
    "filter": {
      "must": [
        {"type": "match", "key": "text", "match_value": "test"}
      ]
    },
    "limit": 10
  }'
```

**Expected Result**: Filtered results matching criteria

#### Test: Invalid Filter

```bash
curl -X POST http://localhost:15002/qdrant/collections/test_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1] * 384,
    "filter": {
      "must": [
        {"type": "invalid_filter", "key": "text"}
      ]
    }
  }'
```

**Expected Result**: HTTP 400, invalid filter error

### Filter Tests

#### Test: Match Filter

```bash
# Insert test data
curl -X PUT http://localhost:15002/qdrant/collections/test_collection/points \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {"id": "1", "vector": [0.1]*384, "payload": {"status": "active"}},
      {"id": "2", "vector": [0.2]*384, "payload": {"status": "inactive"}}
    ]
  }'

# Test match filter
curl -X POST http://localhost:15002/qdrant/collections/test_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1]*384,
    "filter": {
      "must": [{"type": "match", "key": "status", "match_value": "active"}]
    }
  }'
```

**Expected Result**: Only point with status="active" returned

#### Test: Range Filter

```bash
curl -X POST http://localhost:15002/qdrant/collections/test_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1]*384,
    "filter": {
      "must": [
        {"type": "range", "key": "price", "range": {"gte": 50, "lte": 100}}
      ]
    }
  }'
```

**Expected Result**: Only points with price between 50-100

#### Test: Geo Filter

```bash
curl -X POST http://localhost:15002/qdrant/collections/test_collection/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1]*384,
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
  }'
```

**Expected Result**: Only points within radius

## Test Tools

### curl Scripts

**test_collections.sh**:
```bash
#!/bin/bash
BASE_URL="http://localhost:15002/qdrant"

# Test collection creation
echo "Testing collection creation..."
curl -X PUT "${BASE_URL}/collections/test" \
  -H "Content-Type: application/json" \
  -d '{"vectors": {"size": 384, "distance": "Cosine"}}'

# Test collection listing
echo "Testing collection listing..."
curl "${BASE_URL}/collections"

# Test collection info
echo "Testing collection info..."
curl "${BASE_URL}/collections/test"

# Cleanup
echo "Cleaning up..."
curl -X DELETE "${BASE_URL}/collections/test"
```

### Python Test Suite

**test_qdrant_compatibility.py**:
```python
import requests
import pytest

BASE_URL = "http://localhost:15002/qdrant"
COLLECTION_NAME = "test_collection"

@pytest.fixture
def collection():
    # Create collection
    requests.put(
        f"{BASE_URL}/collections/{COLLECTION_NAME}",
        json={"vectors": {"size": 384, "distance": "Cosine"}}
    )
    yield COLLECTION_NAME
    # Cleanup
    requests.delete(f"{BASE_URL}/collections/{COLLECTION_NAME}")

def test_create_collection(collection):
    response = requests.get(f"{BASE_URL}/collections/{collection}")
    assert response.status_code == 200
    assert response.json()["result"]["status"] == "green"

def test_upsert_points(collection):
    response = requests.put(
        f"{BASE_URL}/collections/{collection}/points",
        json={
            "points": [
                {
                    "id": "1",
                    "vector": [0.1] * 384,
                    "payload": {"text": "test"}
                }
            ]
        }
    )
    assert response.status_code == 200
    assert response.json()["status"]["operation_id"] is not None

def test_search_points(collection):
    # Insert test data
    requests.put(
        f"{BASE_URL}/collections/{collection}/points",
        json={
            "points": [
                {"id": "1", "vector": [0.1] * 384, "payload": {"text": "test"}}
            ]
        }
    )
    
    # Search
    response = requests.post(
        f"{BASE_URL}/collections/{collection}/points/search",
        json={"vector": [0.1] * 384, "limit": 10}
    )
    assert response.status_code == 200
    assert len(response.json()["result"]) > 0

def test_filtered_search(collection):
    # Insert test data
    requests.put(
        f"{BASE_URL}/collections/{collection}/points",
        json={
            "points": [
                {"id": "1", "vector": [0.1]*384, "payload": {"status": "active"}},
                {"id": "2", "vector": [0.2]*384, "payload": {"status": "inactive"}}
            ]
        }
    )
    
    # Filtered search
    response = requests.post(
        f"{BASE_URL}/collections/{collection}/points/search",
        json={
            "vector": [0.1] * 384,
            "filter": {
                "must": [{"type": "match", "key": "status", "match_value": "active"}]
            },
            "limit": 10
        }
    )
    assert response.status_code == 200
    results = response.json()["result"]
    assert all(r["payload"]["status"] == "active" for r in results)
```

### JavaScript Test Suite

**test_qdrant.js**:
```javascript
const BASE_URL = 'http://localhost:15002/qdrant';
const COLLECTION_NAME = 'test_collection';

async function testCollectionOperations() {
  // Create collection
  const createResponse = await fetch(
    `${BASE_URL}/collections/${COLLECTION_NAME}`,
    {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        vectors: { size: 384, distance: 'Cosine' }
      })
    }
  );
  console.assert(createResponse.ok, 'Collection creation failed');

  // List collections
  const listResponse = await fetch(`${BASE_URL}/collections`);
  const collections = await listResponse.json();
  console.assert(
    collections.result.collections.some(c => c.name === COLLECTION_NAME),
    'Collection not found in list'
  );

  // Cleanup
  await fetch(`${BASE_URL}/collections/${COLLECTION_NAME}`, {
    method: 'DELETE'
  });
}

testCollectionOperations();
```

## Test Examples

### Unit Test Examples

#### Test: Endpoint Availability

```python
def test_endpoints_available():
    endpoints = [
        "/qdrant/collections",
        "/qdrant/collections/test",
        "/qdrant/collections/test/points",
        "/qdrant/collections/test/points/search"
    ]
    
    for endpoint in endpoints:
        response = requests.get(f"http://localhost:15002{endpoint}")
        # Should not return 404 (may return 400/409 for invalid requests)
        assert response.status_code != 404
```

#### Test: Response Format

```python
def test_response_format():
    response = requests.get("http://localhost:15002/qdrant/collections")
    data = response.json()
    
    # Check Qdrant-compatible format
    assert "result" in data
    assert "status" in data
    assert data["status"] == "ok"
    assert "time" in data
```

### Integration Test Examples

#### Test: Complete Workflow

```python
def test_complete_workflow():
    collection = "test_workflow"
    
    # 1. Create collection
    requests.put(
        f"{BASE_URL}/collections/{collection}",
        json={"vectors": {"size": 384, "distance": "Cosine"}}
    )
    
    # 2. Insert points
    requests.put(
        f"{BASE_URL}/collections/{collection}/points",
        json={
            "points": [
                {"id": "1", "vector": [0.1]*384, "payload": {"text": "doc1"}},
                {"id": "2", "vector": [0.2]*384, "payload": {"text": "doc2"}}
            ]
        }
    )
    
    # 3. Search
    response = requests.post(
        f"{BASE_URL}/collections/{collection}/points/search",
        json={"vector": [0.1]*384, "limit": 10}
    )
    assert len(response.json()["result"]) == 2
    
    # 4. Count
    count_response = requests.post(
        f"{BASE_URL}/collections/{collection}/points/count",
        json={}
    )
    assert count_response.json()["result"]["count"] == 2
    
    # 5. Cleanup
    requests.delete(f"{BASE_URL}/collections/{collection}")
```

### Compatibility Test Examples

#### Test: Qdrant Client Compatibility

```python
from qdrant_client import QdrantClient

def test_qdrant_client():
    # Use Qdrant Python client with Vectorizer
    client = QdrantClient("localhost", port=15002, path="/qdrant")
    
    # Test collection operations
    client.create_collection("test", vectors_config={
        "size": 384,
        "distance": "Cosine"
    })
    
    # Test search
    results = client.search(
        collection_name="test",
        query_vector=[0.1] * 384,
        limit=10
    )
    assert len(results) >= 0
    
    # Cleanup
    client.delete_collection("test")
```

### Performance Test Examples

#### Test: Search Performance

```python
import time

def test_search_performance():
    collection = "perf_test"
    
    # Setup: Insert test data
    points = [
        {"id": str(i), "vector": [0.1]*384, "payload": {"index": i}}
        for i in range(1000)
    ]
    requests.put(
        f"{BASE_URL}/collections/{collection}/points",
        json={"points": points}
    )
    
    # Test search performance
    query_vector = [0.1] * 384
    times = []
    
    for _ in range(10):
        start = time.time()
        requests.post(
            f"{BASE_URL}/collections/{collection}/points/search",
            json={"vector": query_vector, "limit": 10}
        )
        times.append(time.time() - start)
    
    avg_time = sum(times) / len(times)
    print(f"Average search time: {avg_time:.3f}s")
    assert avg_time < 1.0  # Should be fast
    
    # Cleanup
    requests.delete(f"{BASE_URL}/collections/{collection}")
```

## Validation Checklist

### Collection Management

- [ ] Create collection succeeds
- [ ] Create duplicate collection fails (409)
- [ ] Get collection info works
- [ ] List collections includes created collection
- [ ] Update collection works
- [ ] Delete collection works
- [ ] Get deleted collection fails (404)

### Vector Operations

- [ ] Upsert single point works
- [ ] Upsert batch points works
- [ ] Retrieve points by ID works
- [ ] Retrieve with payload filtering works
- [ ] Delete points by ID works
- [ ] Delete points by filter works
- [ ] Count points works
- [ ] Count with filter works
- [ ] Scroll points works
- [ ] Invalid vector dimension fails (400)

### Search Operations

- [ ] Basic search works
- [ ] Search with limit works
- [ ] Search with filter works
- [ ] Batch search works
- [ ] Recommend works
- [ ] Batch recommend works
- [ ] Search with score_threshold works
- [ ] Search with offset works

### Filter Types

- [ ] Match filter (string) works
- [ ] Match filter (integer) works
- [ ] Match filter (boolean) works
- [ ] Range filter works
- [ ] Geo bounding box filter works
- [ ] Geo radius filter works
- [ ] Values count filter works
- [ ] Nested filters work
- [ ] Complex filter logic works

### Error Handling

- [ ] 400 errors return proper format
- [ ] 404 errors return proper format
- [ ] 409 errors return proper format
- [ ] 500 errors return proper format
- [ ] Error messages are clear

### Response Format

- [ ] Responses match Qdrant format
- [ ] Status field present
- [ ] Time field present
- [ ] Result field structure correct

## Running Tests

### Run All Tests

```bash
# Automated bash script (recommended)
bash scripts/test-qdrant-compatibility.sh

# Interactive menu-driven script
bash scripts/test-qdrant-interactive.sh

# Python (if available)
pytest tests/test_qdrant_compatibility.py -v

# JavaScript (if available)
npm test
```

### Run Specific Test Suite

```bash
# Collection tests only
pytest tests/test_qdrant_compatibility.py::test_collections -v

# Search tests only
pytest tests/test_qdrant_compatibility.py::test_search -v
```

## Test Data

### Sample Test Data

```python
TEST_POINTS = [
    {
        "id": "1",
        "vector": [0.1] * 384,
        "payload": {
            "text": "Example document",
            "category": "electronics",
            "price": 99.99,
            "status": "active",
            "tags": ["new", "popular", "featured"]
        }
    },
    # ... more test points
]
```

## Additional Resources

- [API Compatibility Matrix](./API_COMPATIBILITY.md)
- [Feature Parity](./FEATURE_PARITY.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [Examples](./EXAMPLES.md)

