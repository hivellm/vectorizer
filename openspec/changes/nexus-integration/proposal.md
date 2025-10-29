# Nexus Integration - Proposal

## Overview

Integration of Vectorizer with Nexus graph database to enable automatic graph construction from vector data, creating semantic relationships and enabling hybrid search (semantic + graph traversal).

## Motivation

Current Vectorizer implementation focuses on semantic search but lacks:
- **Structured Relationships**: No explicit modeling of document relationships
- **Cross-Domain Analysis**: Limited ability to find connections across different domains
- **Graph Context**: Missing graph-based context enrichment
- **Entity Relationships**: No tracking of entities and their relationships
- **Impact Analysis**: No ability to analyze document impact across the organization

## Goals

### Primary Goals
1. **Bidirectional Sync**: Automatic synchronization between Vectorizer and Nexus
2. **Relationship Extraction**: Automatic creation of document relationships in graph
3. **Domain Classification**: Organize documents into domain hierarchies (legal, financial, HR, etc.)
4. **Semantic Similarity Graph**: Create graph edges based on vector similarity
5. **Context Enrichment**: Enrich vector metadata with graph context

### Secondary Goals
1. **Entity Extraction**: Extract and link entities (people, companies, concepts)
2. **Cross-Domain Discovery**: Enable queries that span multiple domains
3. **Impact Analysis**: Track document impact and dependencies
4. **Audit Trail**: Complete graph-based audit trail for compliance

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                      VECTORIZER                              │
│                                                              │
│  ┌────────────────┐         ┌──────────────────┐           │
│  │  VectorStore   │         │ NexusSyncEngine  │           │
│  │  (Existing)    │◄───────►│     (New)        │           │
│  └────────────────┘         └────────┬─────────┘           │
│         │                             │                      │
│         │                    ┌────────▼─────────┐           │
│         │                    │  WebhookHandler  │           │
│         │                    │     (New)        │           │
│         │                    └──────────────────┘           │
└─────────┼──────────────────────────┼──────────────────────┘
          │                           │ HTTP/Webhooks
          │ REST API                  │
┌─────────▼───────────────────────────▼──────────────────────┐
│                         NEXUS                                │
│                                                              │
│  ┌────────────────┐         ┌──────────────────┐           │
│  │  Graph Engine  │         │ VectorizerSync   │           │
│  │  (Existing)    │◄───────►│     (New)        │           │
│  └────────────────┘         └──────────────────┘           │
│         │                                                    │
│  ┌──────▼──────────────────────────────────────┐           │
│  │  Graph Correlation Analysis (Existing)      │           │
│  └─────────────────────────────────────────────┘           │
└──────────────────────────────────────────────────────────────┘
```

### Data Flow

#### Vectorizer → Nexus (Document Insertion)

```
1. Document inserted into Vectorizer collection
   ↓
2. WebhookHandler triggers NexusSyncEngine
   ↓
3. NexusSyncEngine extracts metadata and classifies domain
   ↓
4. Create Document node in Nexus with embedding
   ↓
5. Query Vectorizer for similar documents (KNN)
   ↓
6. Create SIMILAR_TO relationships in graph
   ↓
7. Extract entities and create Entity nodes
   ↓
8. Create MENTIONS relationships
   ↓
9. Return sync result with node_id
```

#### Nexus → Vectorizer (Graph Enrichment)

```
1. Relationship created in Nexus
   ↓
2. WebhookHandler triggers VectorizerEnricher
   ↓
3. Query Nexus for graph context (related docs, entities)
   ↓
4. Build enriched metadata JSON
   ↓
5. Update vector payload in Vectorizer
   ↓
6. Invalidate query cache
   ↓
7. Return enrichment result
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)

**Vectorizer Changes:**
- Create `nexus_client` module for Nexus REST API communication
- Implement `NexusSyncEngine` for document synchronization
- Add webhook system for event notification
- Create `DomainClassifier` for automatic domain detection

**Deliverables:**
- Nexus client with connection pooling
- Basic sync engine with retry logic
- Webhook infrastructure
- Domain classification logic

### Phase 2: Bidirectional Sync (Week 3-4)

**Vectorizer Changes:**
- Implement sync hooks in `VectorStore::insert()`
- Add background sync worker
- Create similarity calculation service
- Implement batch sync for existing collections

**Deliverables:**
- Automatic sync on document insert/update
- Background worker for async processing
- Similarity-based relationship creation
- Collection migration tool

### Phase 3: Enrichment & Context (Week 5-6)

**Vectorizer Changes:**
- Implement context enrichment from graph data
- Add entity extraction pipeline
- Create metadata enhancement system
- Build cross-domain query support

**Deliverables:**
- Graph context in vector metadata
- Entity extraction and linking
- Enhanced search with graph context
- Cross-domain search API

### Phase 4: Production Features (Week 7-8)

**Vectorizer Changes:**
- Add sync monitoring and metrics
- Implement error handling and recovery
- Create admin API for sync management
- Add configuration system

**Deliverables:**
- Prometheus metrics for sync
- Error recovery mechanisms
- Admin endpoints
- Configuration validation

## API Changes

### New REST Endpoints

```
POST   /api/v1/sync/nexus/enable          Enable Nexus sync for collection
POST   /api/v1/sync/nexus/disable         Disable Nexus sync
GET    /api/v1/sync/nexus/status          Get sync status
POST   /api/v1/sync/nexus/trigger         Manually trigger sync
GET    /api/v1/sync/nexus/stats           Get sync statistics

POST   /api/v1/webhooks/nexus              Register Nexus webhook
DELETE /api/v1/webhooks/nexus/:id          Unregister webhook
GET    /api/v1/webhooks/nexus              List webhooks
```

### Configuration

```yaml
# config.yml
nexus_integration:
  enabled: true
  nexus_url: "http://localhost:15474"
  api_key: "${NEXUS_API_KEY}"
  
  # Sync settings
  sync_mode: "async"  # "sync" or "async"
  batch_size: 100
  worker_threads: 4
  retry_attempts: 3
  retry_delay_ms: 1000
  
  # Similarity settings
  similarity_threshold: 0.75
  similarity_top_k: 20
  
  # Domain classification
  auto_classify_domains: true
  default_domain: "general"
  
  # Collections to sync
  sync_collections:
    - name: "legal_documents"
      domain: "legal"
      enabled: true
    - name: "financial_documents"
      domain: "financial"
      enabled: true
    - name: "hr_documents"
      domain: "hr"
      enabled: true
```

## Data Models

### NexusSyncMetadata

```rust
/// Metadata stored in vector payload for Nexus sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusSyncMetadata {
    /// Nexus node ID
    pub node_id: String,
    
    /// Last sync timestamp
    pub synced_at: DateTime<Utc>,
    
    /// Sync status
    pub sync_status: SyncStatus,
    
    /// Domain classification
    pub domain: String,
    
    /// Number of graph relationships
    pub relationship_count: usize,
    
    /// Entities extracted
    pub entities: Vec<ExtractedEntity>,
    
    /// Related document IDs
    pub related_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,
    Syncing,
    Synced,
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f32,
}
```

### SyncResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub node_id: Option<String>,
    pub relationships_created: usize,
    pub entities_extracted: usize,
    pub domain: String,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}
```

## Testing Strategy

### Unit Tests
- Nexus client communication
- Domain classification logic
- Entity extraction
- Webhook handling
- Error recovery

### Integration Tests
- End-to-end sync flow
- Bidirectional sync
- Batch processing
- Error scenarios
- Performance benchmarks

### Load Tests
- 10K documents sync
- Concurrent inserts
- Webhook delivery
- Graph query performance

## Metrics

```
# Sync metrics
vectorizer_nexus_sync_total{collection, status}
vectorizer_nexus_sync_duration_seconds{collection}
vectorizer_nexus_sync_errors_total{collection, error_type}

# Relationship metrics
vectorizer_nexus_relationships_created_total{relationship_type}
vectorizer_nexus_entities_extracted_total{entity_type}

# Performance metrics
vectorizer_nexus_webhook_latency_seconds
vectorizer_nexus_batch_size
vectorizer_nexus_queue_depth
```

## Security Considerations

1. **API Authentication**: Secure API key storage and rotation
2. **Webhook Security**: HMAC signature verification for webhooks
3. **Data Privacy**: Respect document permissions in sync
4. **Rate Limiting**: Prevent sync storms
5. **Audit Logging**: Log all sync operations

## Migration Path

### Existing Collections

```bash
# 1. Enable Nexus integration
curl -X POST http://localhost:15002/api/v1/sync/nexus/enable \
  -H "Content-Type: application/json" \
  -d '{"collection": "legal_documents", "domain": "legal"}'

# 2. Trigger batch sync
curl -X POST http://localhost:15002/api/v1/sync/nexus/trigger \
  -H "Content-Type: application/json" \
  -d '{"collection": "legal_documents", "batch_size": 100}'

# 3. Monitor sync progress
curl http://localhost:15002/api/v1/sync/nexus/status
```

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Sync failures | High | Retry logic, dead letter queue |
| Performance degradation | Medium | Async processing, batching |
| Data inconsistency | High | Transaction boundaries, reconciliation |
| Nexus unavailability | Medium | Circuit breaker, fallback mode |
| Memory pressure | Medium | Streaming, pagination |

## Success Criteria

1. ✅ 99% of documents synced within 5 seconds
2. ✅ Zero data loss during sync
3. ✅ < 10ms overhead on document insertion
4. ✅ Bidirectional sync with < 1 second lag
5. ✅ Support for 1M+ documents per collection
6. ✅ 95%+ test coverage

## Future Enhancements

1. **Real-time Sync**: WebSocket-based real-time updates
2. **Conflict Resolution**: Automatic conflict resolution
3. **Schema Evolution**: Handle schema changes gracefully
4. **Multi-tenant**: Separate graphs per tenant
5. **Advanced Analytics**: ML-powered relationship prediction

## References

- Nexus API Documentation: `nexus/docs/specs/api-protocols.md`
- Nexus Graph Correlation: `nexus/docs/specs/graph-correlation-analysis.md`
- Vectorizer MCP Integration: `vectorizer/docs/specs/MCP_INTEGRATION.md`

