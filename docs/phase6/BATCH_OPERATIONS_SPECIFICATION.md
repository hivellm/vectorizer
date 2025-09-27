# Batch Operations Specification - Phase 6

## Overview

This document specifies the implementation of batch operations for the Vectorizer system, enabling multiple vector operations to be performed in a single API call. This feature significantly improves performance and reduces latency when models need to perform multiple operations.

## 1. GRPC Batch Operations

### 1.1 Proto Definitions

```protobuf
// Batch Insert Request
message BatchInsertVectorsRequest {
  string collection = 1;
  repeated Vector vectors = 2;
  bool atomic = 3; // If true, all operations succeed or all fail
  int32 batch_size_limit = 4; // Maximum vectors per batch (default: 1000)
}

message BatchInsertVectorsResponse {
  int32 inserted_count = 1;
  int32 failed_count = 2;
  repeated BatchError errors = 3;
  double processing_time_ms = 4;
  string status = 5; // "success", "partial", "failed"
}

// Batch Update Request
message BatchUpdateVectorsRequest {
  string collection = 1;
  repeated VectorUpdate updates = 2;
  bool atomic = 3;
  int32 batch_size_limit = 4;
}

message VectorUpdate {
  string id = 1;
  repeated float data = 2;
  map<string, string> metadata = 3;
}

message BatchUpdateVectorsResponse {
  int32 updated_count = 1;
  int32 failed_count = 2;
  repeated BatchError errors = 3;
  double processing_time_ms = 4;
  string status = 5;
}

// Batch Delete Request
message BatchDeleteVectorsRequest {
  string collection = 1;
  repeated string vector_ids = 2;
  bool atomic = 3;
  int32 batch_size_limit = 4;
}

message BatchDeleteVectorsResponse {
  int32 deleted_count = 1;
  int32 failed_count = 2;
  repeated BatchError errors = 3;
  double processing_time_ms = 4;
  string status = 5;
}

// Batch Search Request
message BatchSearchVectorsRequest {
  string collection = 1;
  repeated SearchQuery queries = 2;
  int32 limit = 3;
  double threshold = 4;
  bool atomic = 5;
  int32 batch_size_limit = 6;
}

message SearchQuery {
  repeated float query_vector = 1;
  string query_text = 2; // Alternative to vector
  int32 limit = 3;
  double threshold = 4;
  map<string, string> filters = 5;
}

message BatchSearchVectorsResponse {
  repeated SearchResult batch_results = 1;
  int32 total_queries = 2;
  int32 successful_queries = 3;
  int32 failed_queries = 4;
  repeated BatchError errors = 5;
  double processing_time_ms = 6;
  string status = 7;
}

message SearchResult {
  string query_id = 1;
  repeated VectorMatch results = 2;
  double query_time_ms = 3;
  string status = 4; // "success", "failed"
}

message BatchError {
  string operation_id = 1;
  string error_code = 2;
  string error_message = 3;
  string vector_id = 4; // For vector-specific errors
}

// Service Extensions
service VectorizerService {
  // Existing methods...
  
  // Batch Operations
  rpc BatchInsertVectors(BatchInsertVectorsRequest) returns (BatchInsertVectorsResponse);
  rpc BatchUpdateVectors(BatchUpdateVectorsRequest) returns (BatchUpdateVectorsResponse);
  rpc BatchDeleteVectors(BatchDeleteVectorsRequest) returns (BatchDeleteVectorsResponse);
  rpc BatchSearchVectors(BatchSearchVectorsRequest) returns (BatchSearchVectorsResponse);
}
```

### 1.2 Implementation Requirements

#### Performance Targets
- **Batch Insert**: 10,000 vectors/second minimum
- **Batch Update**: 5,000 vectors/second minimum  
- **Batch Delete**: 15,000 vectors/second minimum
- **Batch Search**: 1,000 queries/second minimum
- **Latency**: < 100ms for batches up to 1,000 operations

#### Error Handling
- **Atomic Mode**: All operations succeed or all fail
- **Non-Atomic Mode**: Individual operation failures don't affect others
- **Detailed Error Reporting**: Specific error codes and messages per failed operation
- **Partial Success Handling**: Clear indication of which operations succeeded/failed

#### Memory Management
- **Streaming Processing**: For large batches, process in chunks
- **Memory Limits**: Configurable maximum memory usage per batch
- **Progress Tracking**: Real-time progress updates for long-running operations

## 2. REST API Batch Endpoints

### 2.1 Batch Insert

```http
POST /api/v1/collections/{collection}/vectors/batch
Content-Type: application/json

{
  "vectors": [
    {
      "id": "vec1",
      "data": [0.1, 0.2, 0.3, ...],
      "metadata": {"text": "Sample text 1"}
    },
    {
      "id": "vec2", 
      "data": [0.4, 0.5, 0.6, ...],
      "metadata": {"text": "Sample text 2"}
    }
  ],
  "atomic": true,
  "batch_size_limit": 1000
}
```

**Response:**
```json
{
  "inserted_count": 2,
  "failed_count": 0,
  "errors": [],
  "processing_time_ms": 15.2,
  "status": "success"
}
```

### 2.2 Batch Update

```http
PUT /api/v1/collections/{collection}/vectors/batch
Content-Type: application/json

{
  "updates": [
    {
      "id": "vec1",
      "data": [0.1, 0.2, 0.3, ...],
      "metadata": {"text": "Updated text 1"}
    },
    {
      "id": "vec2",
      "data": [0.4, 0.5, 0.6, ...],
      "metadata": {"text": "Updated text 2"}
    }
  ],
  "atomic": false,
  "batch_size_limit": 1000
}
```

### 2.3 Batch Delete

```http
DELETE /api/v1/collections/{collection}/vectors/batch
Content-Type: application/json

{
  "vector_ids": ["vec1", "vec2", "vec3"],
  "atomic": true,
  "batch_size_limit": 1000
}
```

### 2.4 Batch Search

```http
POST /api/v1/collections/{collection}/search/batch
Content-Type: application/json

{
  "queries": [
    {
      "query_vector": [0.1, 0.2, 0.3, ...],
      "limit": 10,
      "threshold": 0.7
    },
    {
      "query_text": "search text",
      "limit": 5,
      "threshold": 0.8
    }
  ],
  "atomic": false,
  "batch_size_limit": 100
}
```

**Response:**
```json
{
  "batch_results": [
    {
      "query_id": "query_1",
      "results": [
        {
          "id": "vec1",
          "score": 0.95,
          "data": [0.1, 0.2, 0.3, ...],
          "metadata": {"text": "Match 1"}
        }
      ],
      "query_time_ms": 2.1,
      "status": "success"
    }
  ],
  "total_queries": 2,
  "successful_queries": 2,
  "failed_queries": 0,
  "errors": [],
  "processing_time_ms": 12.5,
  "status": "success"
}
```

## 3. MCP Batch Tools

### 3.1 Tool Definitions

```json
{
  "name": "batch_insert_vectors",
  "description": "Insert multiple vectors into a collection in a single operation",
  "inputSchema": {
    "type": "object",
    "properties": {
      "collection": {
        "type": "string",
        "description": "Collection name"
      },
      "vectors": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "id": {"type": "string"},
            "data": {
              "type": "array",
              "items": {"type": "number"}
            },
            "metadata": {"type": "object"}
          }
        }
      },
      "atomic": {
        "type": "boolean",
        "description": "If true, all operations succeed or all fail",
        "default": true
      },
      "batch_size_limit": {
        "type": "integer",
        "description": "Maximum vectors per batch",
        "default": 1000
      }
    },
    "required": ["collection", "vectors"]
  }
}

{
  "name": "batch_update_vectors", 
  "description": "Update multiple vectors in a collection atomically",
  "inputSchema": {
    "type": "object",
    "properties": {
      "collection": {"type": "string"},
      "updates": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "id": {"type": "string"},
            "data": {
              "type": "array",
              "items": {"type": "number"}
            },
            "metadata": {"type": "object"}
          }
        }
      },
      "atomic": {"type": "boolean", "default": true},
      "batch_size_limit": {"type": "integer", "default": 1000}
    },
    "required": ["collection", "updates"]
  }
}

{
  "name": "batch_delete_vectors",
  "description": "Delete multiple vectors from a collection efficiently", 
  "inputSchema": {
    "type": "object",
    "properties": {
      "collection": {"type": "string"},
      "vector_ids": {
        "type": "array",
        "items": {"type": "string"}
      },
      "atomic": {"type": "boolean", "default": true},
      "batch_size_limit": {"type": "integer", "default": 1000}
    },
    "required": ["collection", "vector_ids"]
  }
}

{
  "name": "batch_search_vectors",
  "description": "Perform multiple vector searches in parallel",
  "inputSchema": {
    "type": "object", 
    "properties": {
      "collection": {"type": "string"},
      "queries": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "query_vector": {
              "type": "array",
              "items": {"type": "number"}
            },
            "query_text": {"type": "string"},
            "limit": {"type": "integer", "default": 10},
            "threshold": {"type": "number", "default": 0.0},
            "filters": {"type": "object"}
          }
        }
      },
      "atomic": {"type": "boolean", "default": false},
      "batch_size_limit": {"type": "integer", "default": 100}
    },
    "required": ["collection", "queries"]
  }
}
```

## 4. Implementation Architecture

### 4.1 Core Components

```rust
// Batch operation processor
pub struct BatchProcessor {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    config: BatchConfig,
}

pub struct BatchConfig {
    pub max_batch_size: usize,
    pub max_memory_usage_mb: usize,
    pub parallel_workers: usize,
    pub chunk_size: usize,
}

// Batch operation types
pub enum BatchOperation {
    Insert {
        vectors: Vec<Vector>,
        atomic: bool,
    },
    Update {
        updates: Vec<VectorUpdate>,
        atomic: bool,
    },
    Delete {
        vector_ids: Vec<String>,
        atomic: bool,
    },
    Search {
        queries: Vec<SearchQuery>,
        atomic: bool,
    },
}

// Batch result
pub struct BatchResult<T> {
    pub successful_operations: Vec<T>,
    pub failed_operations: Vec<BatchError>,
    pub processing_time: Duration,
    pub status: BatchStatus,
}

pub enum BatchStatus {
    Success,
    Partial,
    Failed,
}
```

### 4.2 Performance Optimizations

#### Parallel Processing
- **Worker Pool**: Configurable number of worker threads
- **Chunk Processing**: Split large batches into smaller chunks
- **Memory Streaming**: Process batches without loading all into memory
- **Lock-Free Operations**: Minimize contention for high-throughput scenarios

#### Memory Management
- **Bounded Queues**: Prevent memory exhaustion
- **Garbage Collection**: Periodic cleanup of temporary data
- **Memory Pools**: Reuse allocated memory for similar operations
- **Progress Tracking**: Real-time memory usage monitoring

#### Error Handling
- **Circuit Breaker**: Prevent cascade failures
- **Retry Logic**: Exponential backoff for transient failures
- **Dead Letter Queue**: Handle permanently failed operations
- **Detailed Logging**: Comprehensive error tracking and debugging

## 5. Testing Requirements

### 5.1 Unit Tests
- Individual batch operation components
- Error handling scenarios
- Memory management edge cases
- Atomic transaction behavior

### 5.2 Integration Tests
- End-to-end batch operation workflows
- Multi-collection batch operations
- Concurrent batch operation handling
- Performance under load

### 5.3 Performance Tests
- Latency benchmarks for different batch sizes
- Throughput measurements under various loads
- Memory usage profiling
- Scalability testing with large datasets

## 6. Configuration

### 6.1 Default Settings
```yaml
batch_operations:
  max_batch_size: 1000
  max_memory_usage_mb: 512
  parallel_workers: 4
  chunk_size: 100
  atomic_by_default: true
  progress_reporting: true
  error_retry_attempts: 3
  error_retry_delay_ms: 100
```

### 6.2 Runtime Configuration
- Environment variable overrides
- Dynamic configuration updates
- Per-collection batch limits
- Resource usage monitoring

## 7. Migration Strategy

### 7.1 Backward Compatibility
- Existing single operations continue to work
- Gradual migration path for existing clients
- Deprecation warnings for non-batch usage
- Performance comparison tools

### 7.2 Rollout Plan
1. **Phase 1**: Core batch operations (insert, delete)
2. **Phase 2**: Update and search batch operations
3. **Phase 3**: Advanced features (transactions, streaming)
4. **Phase 4**: Performance optimizations and monitoring

## 8. Success Metrics

### 8.1 Performance Targets
- **10x improvement** in throughput for bulk operations
- **50% reduction** in latency for multiple operations
- **95% reduction** in network round trips
- **Linear scalability** up to 10,000 operations per batch

### 8.2 Quality Metrics
- **99.9% success rate** for atomic operations
- **<1% data loss** for non-atomic operations
- **<100ms p95 latency** for batch operations
- **Zero memory leaks** during extended operation

---

**Implementation Priority**: High - This feature directly addresses the efficiency needs of AI models and significantly improves the system's usability for production workloads.

**Estimated Development Time**: 4-6 weeks for full implementation including testing and documentation.

**Dependencies**: Phase 5 completion (✅ Complete), GRPC infrastructure (✅ Complete), MCP framework (✅ Complete).
