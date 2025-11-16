# Qdrant Compatibility Troubleshooting Examples

Practical examples for diagnosing and resolving common Qdrant compatibility issues.

## Table of Contents

1. [Collection Issues](#collection-issues)
2. [Vector Dimension Issues](#vector-dimension-issues)
3. [Filter Issues](#filter-issues)
4. [Performance Issues](#performance-issues)
5. [Error Handling Examples](#error-handling-examples)

## Collection Issues

### Example 1: Collection Not Found

**Problem**: Getting 404 when accessing collection

**Diagnosis Script**:

```bash
#!/bin/bash
COLLECTION_NAME="my_collection"
BASE_URL="http://localhost:15002/qdrant"

# Check if collection exists
echo "Checking collections..."
curl -s "$BASE_URL/collections" | jq '.result.collections[] | .name'

# Try to access collection
echo "Accessing collection..."
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/collections/$COLLECTION_NAME")
http_code=$(echo "$response" | tail -n1)

if [ "$http_code" -eq 404 ]; then
    echo "Collection not found. Creating..."
    curl -X PUT "$BASE_URL/collections/$COLLECTION_NAME" \
        -H "Content-Type: application/json" \
        -d '{"vectors": {"size": 384, "distance": "Cosine"}}'
else
    echo "Collection exists:"
    echo "$response" | sed '$d' | jq '.'
fi
```

**Solution**:

```python
import requests

BASE_URL = "http://localhost:15002/qdrant"
collection_name = "my_collection"

# Check if collection exists
response = requests.get(f"{BASE_URL}/collections/{collection_name}")
if response.status_code == 404:
    # Create collection
    requests.put(
        f"{BASE_URL}/collections/{collection_name}",
        json={"vectors": {"size": 384, "distance": "Cosine"}}
    )
    print("Collection created")
else:
    print("Collection exists:", response.json())
```

### Example 2: Collection Already Exists

**Problem**: Getting 409 when creating collection

**Solution Script**:

```bash
#!/bin/bash
COLLECTION_NAME="my_collection"
BASE_URL="http://localhost:15002/qdrant"

# Check if collection exists first
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/collections/$COLLECTION_NAME")
http_code=$(echo "$response" | tail -n1)

if [ "$http_code" -eq 200 ]; then
    echo "Collection exists. Updating instead..."
    curl -X PATCH "$BASE_URL/collections/$COLLECTION_NAME" \
        -H "Content-Type: application/json" \
        -d '{"optimizer_config": {"indexing_threshold": 20000}}'
elif [ "$http_code" -eq 404 ]; then
    echo "Creating new collection..."
    curl -X PUT "$BASE_URL/collections/$COLLECTION_NAME" \
        -H "Content-Type: application/json" \
        -d '{"vectors": {"size": 384, "distance": "Cosine"}}'
fi
```

## Vector Dimension Issues

### Example 3: Invalid Vector Dimension

**Problem**: Getting 400 error with "Invalid vector dimension"

**Diagnosis Script**:

```python
import requests
import json

BASE_URL = "http://localhost:15002/qdrant"
collection_name = "my_collection"

# Get collection info to check dimension
response = requests.get(f"{BASE_URL}/collections/{collection_name}")
collection_info = response.json()

# Extract dimension from config
dimension = collection_info["result"]["config"]["params"]["vectors"]["size"]
print(f"Collection dimension: {dimension}")

# Validate vector before insertion
def validate_and_insert(vector_data, payload=None):
    if len(vector_data) != dimension:
        raise ValueError(
            f"Vector dimension mismatch: expected {dimension}, got {len(vector_data)}"
        )

    point = {"id": "1", "vector": vector_data}
    if payload:
        point["payload"] = payload

    response = requests.put(
        f"{BASE_URL}/collections/{collection_name}/points",
        json={"points": [point]}
    )
    return response.json()

# Example usage
try:
    # This will fail - wrong dimension
    validate_and_insert([0.1] * 100)
except ValueError as e:
    print(f"Validation error: {e}")

# This will succeed - correct dimension
result = validate_and_insert([0.1] * dimension, {"text": "example"})
print("Inserted successfully:", result)
```

**Prevention Function**:

```python
def safe_upsert(collection_name, points, base_url="http://localhost:15002/qdrant"):
    """Safely upsert points with dimension validation"""
    # Get collection dimension
    response = requests.get(f"{base_url}/collections/{collection_name}")
    if response.status_code == 404:
        raise ValueError(f"Collection {collection_name} not found")

    dimension = response.json()["result"]["config"]["params"]["vectors"]["size"]

    # Validate all points
    validated_points = []
    for point in points:
        if len(point["vector"]) != dimension:
            raise ValueError(
                f"Point {point.get('id', 'unknown')} has wrong dimension: "
                f"expected {dimension}, got {len(point['vector'])}"
            )
        validated_points.append(point)

    # Insert validated points
    response = requests.put(
        f"{base_url}/collections/{collection_name}/points",
        json={"points": validated_points}
    )
    response.raise_for_status()
    return response.json()
```

## Filter Issues

### Example 4: Filter Not Working

**Problem**: Filters return no results or unexpected results

**Diagnosis Script**:

```python
import requests

BASE_URL = "http://localhost:15002/qdrant"
collection_name = "my_collection"

# First, check what payload fields exist
def inspect_collection_payloads(collection_name, limit=10):
    """Inspect actual payload structure in collection"""
    response = requests.post(
        f"{BASE_URL}/collections/{collection_name}/points/scroll",
        json={"limit": limit, "with_payload": True}
    )

    points = response.json()["result"]["points"]

    # Analyze payload structure
    payload_fields = set()
    for point in points:
        if "payload" in point:
            payload_fields.update(point["payload"].keys())

    print(f"Found payload fields: {payload_fields}")
    return payload_fields

# Test filter step by step
def test_filter_incrementally(collection_name, query_vector):
    """Test filter incrementally to find the issue"""

    # Step 1: Search without filter
    print("Step 1: Search without filter")
    response = requests.post(
        f"{BASE_URL}/collections/{collection_name}/points/search",
        json={"vector": query_vector, "limit": 10}
    )
    results_no_filter = response.json()["result"]
    print(f"Results without filter: {len(results_no_filter)}")

    # Step 2: Search with simple match filter
    print("\nStep 2: Search with simple match filter")
    response = requests.post(
        f"{BASE_URL}/collections/{collection_name}/points/search",
        json={
            "vector": query_vector,
            "filter": {
                "must": [{"type": "match", "key": "status", "match_value": "active"}]
            },
            "limit": 10
        }
    )
    results_with_filter = response.json()["result"]
    print(f"Results with filter: {len(results_with_filter)}")

    # Step 3: Check if filter field exists
    print("\nStep 3: Inspecting payload structure")
    payload_fields = inspect_collection_payloads(collection_name)

    if "status" not in payload_fields:
        print("WARNING: 'status' field not found in payloads!")
        print(f"Available fields: {payload_fields}")

    return results_with_filter

# Example usage
query_vector = [0.1] * 384
results = test_filter_incrementally(collection_name, query_vector)
```

**Common Filter Fixes**:

```python
# Fix 1: Check field name case sensitivity
# Wrong:
filter = {"must": [{"type": "match", "key": "Status", "match_value": "active"}]}

# Right:
filter = {"must": [{"type": "match", "key": "status", "match_value": "active"}]}

# Fix 2: Use correct filter type
# Wrong:
filter = {"must": [{"type": "match", "key": "price", "match_value": 50}]}  # For numeric

# Right:
filter = {"must": [{"type": "range", "key": "price", "range": {"gte": 50}}]}

# Fix 3: Nested field access
# Wrong:
filter = {"must": [{"type": "match", "key": "user.name", "match_value": "John"}]}

# Right: Use dot notation (if supported) or flatten payload structure
```

## Performance Issues

### Example 5: Slow Search Queries

**Problem**: Search queries are slow

**Performance Diagnosis Script**:

```python
import requests
import time
import statistics

BASE_URL = "http://localhost:15002/qdrant"
collection_name = "my_collection"

def benchmark_search(query_vector, limit=10, iterations=10):
    """Benchmark search performance"""
    times = []

    for i in range(iterations):
        start = time.time()
        response = requests.post(
            f"{BASE_URL}/collections/{collection_name}/points/search",
            json={"vector": query_vector, "limit": limit}
        )
        elapsed = time.time() - start

        result = response.json()
        server_time = result.get("time", 0)
        times.append({
            "total": elapsed,
            "server": server_time,
            "results": len(result.get("result", []))
        })

    avg_total = statistics.mean([t["total"] for t in times])
    avg_server = statistics.mean([t["server"] for t in times])

    print(f"Average total time: {avg_total:.3f}s")
    print(f"Average server time: {avg_server:.3f}s")
    print(f"Overhead: {avg_total - avg_server:.3f}s")

    return times

def optimize_search_performance(collection_name, query_vector):
    """Try different optimizations"""

    optimizations = [
        {"limit": 5, "description": "Lower limit"},
        {"limit": 10, "score_threshold": 0.5, "description": "With score threshold"},
        {"limit": 10, "description": "Default"},
    ]

    for opt in optimizations:
        print(f"\nTesting: {opt['description']}")
        times = benchmark_search(query_vector, **{k: v for k, v in opt.items() if k != "description"})
        avg_time = statistics.mean([t["total"] for t in times])
        print(f"Average: {avg_time:.3f}s")

# Example usage
query_vector = [0.1] * 384
optimize_search_performance(collection_name, query_vector)
```

**Performance Optimization Examples**:

```python
# Optimization 1: Reduce limit
# Before:
search_params = {"vector": query_vector, "limit": 100}

# After:
search_params = {"vector": query_vector, "limit": 10}  # Only request what you need

# Optimization 2: Use score threshold
# Before:
search_params = {"vector": query_vector, "limit": 10}

# After:
search_params = {
    "vector": query_vector,
    "limit": 10,
    "score_threshold": 0.5  # Filter low-quality results early
}

# Optimization 3: Enable payload indexing
# Check if payload indexing is enabled for filtered fields
# Use indexed fields in filters for better performance

# Optimization 4: Use batch operations
# Before: Multiple individual searches
for query in queries:
    results = search(query)

# After: Single batch search
results = batch_search(queries)
```

## Error Handling Examples

### Example 6: Comprehensive Error Handling

**Robust Error Handling**:

```python
import requests
from requests.exceptions import RequestException, Timeout, ConnectionError

BASE_URL = "http://localhost:15002/qdrant"

class QdrantAPIError(Exception):
    """Base exception for Qdrant API errors"""
    pass

class CollectionNotFoundError(QdrantAPIError):
    """Collection not found"""
    pass

class InvalidDimensionError(QdrantAPIError):
    """Invalid vector dimension"""
    pass

class CollectionExistsError(QdrantAPIError):
    """Collection already exists"""
    pass

def handle_qdrant_response(response):
    """Handle Qdrant API response and raise appropriate errors"""
    if response.status_code == 200:
        return response.json()

    error_data = response.json() if response.text else {}
    error_msg = error_data.get("status", {}).get("error", "Unknown error")

    if response.status_code == 404:
        if "collection" in error_msg.lower():
            raise CollectionNotFoundError(error_msg)
        raise QdrantAPIError(f"Not found: {error_msg}")

    elif response.status_code == 409:
        if "already exists" in error_msg.lower():
            raise CollectionExistsError(error_msg)
        raise QdrantAPIError(f"Conflict: {error_msg}")

    elif response.status_code == 400:
        if "dimension" in error_msg.lower():
            raise InvalidDimensionError(error_msg)
        raise QdrantAPIError(f"Bad request: {error_msg}")

    else:
        raise QdrantAPIError(f"HTTP {response.status_code}: {error_msg}")

def safe_qdrant_request(method, endpoint, **kwargs):
    """Make a safe Qdrant API request with error handling"""
    try:
        response = requests.request(
            method,
            f"{BASE_URL}{endpoint}",
            timeout=5.0,
            **kwargs
        )
        return handle_qdrant_response(response)

    except Timeout:
        raise QdrantAPIError("Request timeout - server may be overloaded")

    except ConnectionError:
        raise QdrantAPIError("Connection error - check if server is running")

    except RequestException as e:
        raise QdrantAPIError(f"Request failed: {e}")

# Example usage with error handling
try:
    # Create collection
    result = safe_qdrant_request(
        "PUT",
        "/collections/my_collection",
        json={"vectors": {"size": 384, "distance": "Cosine"}}
    )
    print("Collection created:", result)

except CollectionExistsError:
    print("Collection already exists, updating instead...")
    result = safe_qdrant_request(
        "PATCH",
        "/collections/my_collection",
        json={"optimizer_config": {"indexing_threshold": 20000}}
    )

except CollectionNotFoundError as e:
    print(f"Collection not found: {e}")

except InvalidDimensionError as e:
    print(f"Dimension error: {e}")

except QdrantAPIError as e:
    print(f"API error: {e}")
```

### Example 7: Retry Logic

**Retry on Transient Errors**:

```python
import requests
import time
from functools import wraps

def retry_on_error(max_retries=3, backoff=1):
    """Decorator to retry on transient errors"""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            for attempt in range(max_retries):
                try:
                    return func(*args, **kwargs)
                except requests.exceptions.RequestException as e:
                    if attempt == max_retries - 1:
                        raise

                    wait_time = backoff * (2 ** attempt)
                    print(f"Retry {attempt + 1}/{max_retries} after {wait_time}s...")
                    time.sleep(wait_time)
            return None
        return wrapper
    return decorator

@retry_on_error(max_retries=3, backoff=1)
def create_collection_with_retry(name, config):
    """Create collection with retry logic"""
    response = requests.put(
        f"http://localhost:15002/qdrant/collections/{name}",
        json=config,
        timeout=5.0
    )
    response.raise_for_status()
    return response.json()

# Example usage
try:
    result = create_collection_with_retry(
        "my_collection",
        {"vectors": {"size": 384, "distance": "Cosine"}}
    )
    print("Success:", result)
except requests.exceptions.RequestException as e:
    print(f"Failed after retries: {e}")
```

## Complete Troubleshooting Workflow

**End-to-End Troubleshooting Script**:

```python
#!/usr/bin/env python3
"""
Complete Qdrant compatibility troubleshooting script
"""

import requests
import json
import sys

BASE_URL = "http://localhost:15002/qdrant"

def check_server_health():
    """Check if server is running"""
    try:
        response = requests.get(f"{BASE_URL.replace('/qdrant', '')}/api/status", timeout=2)
        if response.status_code == 200:
            print("✓ Server is running")
            return True
    except:
        pass

    print("✗ Server is not running or not accessible")
    return False

def check_collection_exists(name):
    """Check if collection exists"""
    try:
        response = requests.get(f"{BASE_URL}/collections/{name}")
        if response.status_code == 200:
            info = response.json()
            print(f"✓ Collection '{name}' exists")
            print(f"  Dimension: {info['result']['config']['params']['vectors']['size']}")
            print(f"  Points: {info['result']['points_count']}")
            return True, info
    except:
        pass

    print(f"✗ Collection '{name}' not found")
    return False, None

def test_endpoint(method, endpoint, data=None, expected_status=200):
    """Test an endpoint"""
    try:
        if data:
            response = requests.request(method, f"{BASE_URL}{endpoint}", json=data)
        else:
            response = requests.request(method, f"{BASE_URL}{endpoint}")

        if response.status_code == expected_status:
            print(f"✓ {method} {endpoint}")
            return True, response.json()
        else:
            print(f"✗ {method} {endpoint} (got {response.status_code}, expected {expected_status})")
            return False, response.json() if response.text else None
    except Exception as e:
        print(f"✗ {method} {endpoint} (error: {e})")
        return False, None

def main():
    """Main troubleshooting workflow"""
    print("Qdrant Compatibility Troubleshooting\n")

    # Step 1: Check server
    if not check_server_health():
        print("\nPlease start the Vectorizer server first")
        sys.exit(1)

    # Step 2: List collections
    print("\nListing collections...")
    test_endpoint("GET", "/collections")

    # Step 3: Test collection operations
    test_collection = "test_troubleshooting"
    print(f"\nTesting collection: {test_collection}")

    exists, info = check_collection_exists(test_collection)

    if not exists:
        print("Creating test collection...")
        test_endpoint("PUT", f"/collections/{test_collection}", {
            "vectors": {"size": 384, "distance": "Cosine"}
        })

    # Step 4: Test vector operations
    print("\nTesting vector operations...")
    test_endpoint("PUT", f"/collections/{test_collection}/points", {
        "points": [{
            "id": "1",
            "vector": [0.1] * 384,
            "payload": {"text": "test", "status": "active"}
        }]
    })

    # Step 5: Test search
    print("\nTesting search operations...")
    test_endpoint("POST", f"/collections/{test_collection}/points/search", {
        "vector": [0.1] * 384,
        "limit": 10
    })

    # Step 6: Test filtered search
    print("\nTesting filtered search...")
    test_endpoint("POST", f"/collections/{test_collection}/points/search", {
        "vector": [0.1] * 384,
        "filter": {
            "must": [{"type": "match", "key": "status", "match_value": "active"}]
        },
        "limit": 10
    })

    print("\n✓ All tests completed")

if __name__ == "__main__":
    main()

```

## Additional Resources

- [Troubleshooting Guide](./TROUBLESHOOTING.md) - Complete troubleshooting reference
- [API Compatibility](./API_COMPATIBILITY.md) - API compatibility details
- [Examples](./EXAMPLES.md) - More code examples
- [Testing Guide](./TESTING.md) - Testing procedures
- [Automated Test Script](../scripts/test-qdrant-compatibility.sh) - Run automated compatibility tests
- [Interactive Test Script](../scripts/test-qdrant-interactive.sh) - Interactive testing menu
