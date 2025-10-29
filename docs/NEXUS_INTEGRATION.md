# Nexus Integration - Technical Implementation Guide

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [API Specifications](#api-specifications)
5. [Data Models](#data-models)
6. [Implementation Details](#implementation-details)
7. [Configuration](#configuration)
8. [Testing](#testing)
9. [Deployment](#deployment)
10. [Monitoring](#monitoring)

---

## Overview

This document provides comprehensive technical specifications for integrating Vectorizer with Nexus graph database. The integration enables:

- **Automatic Graph Construction**: Documents inserted into Vectorizer are automatically represented as nodes in Nexus
- **Semantic Relationships**: Similar documents are linked via `SIMILAR_TO` edges based on vector similarity
- **Domain Organization**: Documents are classified and organized by domain (legal, financial, HR, etc.)
- **Bidirectional Sync**: Changes in either system are reflected in the other
- **Context Enrichment**: Graph context enhances vector metadata for better search

### Key Benefits

- **Hybrid Search**: Combine semantic search with graph traversal
- **Cross-Domain Discovery**: Find connections across organizational silos
- **Impact Analysis**: Understand document relationships and dependencies
- **Audit Trail**: Complete graph-based audit trail for compliance

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      VECTORIZER CORE                             │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌─────────────────┐  │
│  │ VectorStore  │───►│ SyncEngine   │───►│ NexusClient     │  │
│  │              │    │              │    │                 │  │
│  │ - insert()   │    │ - queue      │    │ - create_node() │  │
│  │ - update()   │    │ - workers    │    │ - cypher()      │  │
│  │ - delete()   │    │ - retry      │    │ - knn_search()  │  │
│  └──────┬───────┘    └──────┬───────┘    └────────┬────────┘  │
│         │                   │                      │            │
│         │                   │                      │ HTTP/REST  │
│         │                   ▼                      │            │
│         │          ┌────────────────┐              │            │
│         │          │ WebhookHandler │              │            │
│         │          │                │              │            │
│         │          │ - on_insert    │              │            │
│         │          │ - on_update    │              │            │
│         │          └────────────────┘              │            │
│         │                                          │            │
│         ▼                                          ▼            │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │               Event Queue (Tokio Channels)               │  │
│  └─────────────────────────────────────────────────────────┘  │
└────────────────────────────────┬─────────────────────────────┘
                                 │
                                 │ HTTP/JSON
                                 │
┌────────────────────────────────▼─────────────────────────────┐
│                         NEXUS GRAPH                           │
│                                                               │
│  ┌────────────────┐         ┌──────────────────┐            │
│  │ Graph Engine   │◄────────│ REST API         │            │
│  │                │         │                  │            │
│  │ - Cypher       │         │ /cypher          │            │
│  │ - KNN Search   │         │ /knn_traverse    │            │
│  │ - Indexes      │         │ /ingest          │            │
│  └────────────────┘         └──────────────────┘            │
└───────────────────────────────────────────────────────────────┘
```

### Component Interaction Sequence

```
User/API
  │
  │ 1. Insert Document
  ├──────────────────────►VectorStore
  │                          │
  │                          │ 2. Generate Embedding
  │                          ├──────────►EmbeddingManager
  │                          │              │
  │                          │◄─────────────┘ embedding
  │                          │
  │                          │ 3. Store Vector
  │                          ├──────────►HNSW Index
  │                          │
  │                          │ 4. Trigger Sync Hook
  │                          ├──────────►SyncEngine
  │                          │              │
  │◄─────────────────────────┘              │ 5. Async Processing
  │ Return Success                          │
                                           │
                                           │ 6. Classify Domain
                                           ├──────────►DomainClassifier
                                           │              │
                                           │◄─────────────┘ domain
                                           │
                                           │ 7. Create Node
                                           ├──────────►NexusClient
                                           │              │
                                           │              │ 8. POST /cypher
                                           │              ├──────────►Nexus
                                           │              │              │
                                           │              │◄─────────────┘ node_id
                                           │◄─────────────┘
                                           │
                                           │ 9. Find Similar
                                           ├──────────►VectorStore.search()
                                           │              │
                                           │◄─────────────┘ similar_docs
                                           │
                                           │ 10. Create Edges
                                           ├──────────►NexusClient
                                           │              │
                                           │              │ POST /cypher (batch)
                                           │              ├──────────►Nexus
                                           │              │              │
                                           │              │◄─────────────┘ OK
                                           │◄─────────────┘
                                           │
                                           ▼
                                       Complete
```

---

## Core Components

### 1. NexusClient (`src/nexus_client/mod.rs`)

HTTP client for Nexus REST API with connection pooling, retry logic, and error handling.

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

pub struct NexusClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
}

impl NexusClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()?;

        Ok(Self {
            client,
            base_url,
            api_key,
            timeout: Duration::from_secs(30),
        })
    }

    /// Execute Cypher query
    pub async fn execute_cypher(
        &self,
        query: &str,
        params: HashMap<String, Value>,
    ) -> Result<CypherResponse> {
        let url = format!("{}/cypher", self.base_url);
        
        let request = CypherRequest {
            query: query.to_string(),
            params,
            timeout_ms: Some(self.timeout.as_millis() as u64),
        };

        let mut req = self.client.post(&url).json(&request);
        
        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(Error::NexusApi(error));
        }

        Ok(response.json().await?)
    }

    /// Create document node in graph
    pub async fn create_document_node(
        &self,
        doc: &DocumentNode,
    ) -> Result<String> {
        let cypher = r#"
            CREATE (d:Document {
                id: $id,
                vector_id: $vector_id,
                collection: $collection,
                title: $title,
                domain: $domain,
                doc_type: $doc_type,
                created_at: datetime(),
                embedding: $embedding
            })
            RETURN d.id as node_id
        "#;

        let params = hashmap! {
            "id".to_string() => json!(Uuid::new_v4().to_string()),
            "vector_id".to_string() => json!(doc.vector_id),
            "collection".to_string() => json!(doc.collection),
            "title".to_string() => json!(doc.title),
            "domain".to_string() => json!(doc.domain),
            "doc_type".to_string() => json!(doc.doc_type),
            "embedding".to_string() => json!(doc.embedding),
        };

        let response = self.execute_cypher(cypher, params).await?;
        
        let node_id = response.rows
            .first()
            .and_then(|row| row.first())
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidResponse("Missing node_id".to_string()))?;

        Ok(node_id.to_string())
    }

    /// Create similarity relationships in batch
    pub async fn create_similarity_edges(
        &self,
        source_id: &str,
        similar_docs: &[(String, f32)], // (node_id, score)
    ) -> Result<usize> {
        let cypher = r#"
            MATCH (source:Document {id: $source_id})
            UNWIND $similar AS sim
            MATCH (target:Document {vector_id: sim.vector_id})
            MERGE (source)-[r:SIMILAR_TO]-(target)
            ON CREATE SET r.score = sim.score,
                         r.method = 'embedding',
                         r.computed_at = datetime()
            RETURN count(r) as edges_created
        "#;

        let similar_data: Vec<Value> = similar_docs
            .iter()
            .map(|(vector_id, score)| {
                json!({
                    "vector_id": vector_id,
                    "score": score,
                })
            })
            .collect();

        let params = hashmap! {
            "source_id".to_string() => json!(source_id),
            "similar".to_string() => json!(similar_data),
        };

        let response = self.execute_cypher(cypher, params).await?;
        
        let count = response.rows
            .first()
            .and_then(|row| row.first())
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        Ok(count)
    }

    /// KNN search in Nexus graph
    pub async fn knn_search(
        &self,
        label: &str,
        embedding: &[f32],
        k: usize,
    ) -> Result<Vec<KnnResult>> {
        let url = format!("{}/knn_traverse", self.base_url);
        
        let request = KnnRequest {
            label: label.to_string(),
            vector: embedding.to_vec(),
            k,
            expand: None,
            where_clause: None,
            return_fields: None,
            order_by: None,
            limit: None,
        };

        let mut req = self.client.post(&url).json(&request);
        
        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await?;
        
        if !response.status().is_success() {
            return Err(Error::NexusApi(response.json().await?));
        }

        let knn_response: KnnResponse = response.json().await?;
        Ok(knn_response.results)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CypherRequest {
    pub query: String,
    pub params: HashMap<String, Value>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CypherResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub execution_time_ms: u64,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnnRequest {
    pub label: String,
    pub vector: Vec<f32>,
    pub k: usize,
    pub expand: Option<Vec<String>>,
    #[serde(rename = "where")]
    pub where_clause: Option<String>,
    #[serde(rename = "return")]
    pub return_fields: Option<Vec<String>>,
    pub order_by: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnnResponse {
    pub results: Vec<KnnResult>,
    pub execution_time_ms: u64,
    pub knn_time_ms: u64,
    pub result_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnnResult {
    pub node: NodeData,
    pub score: f32,
}
```

### 2. SyncEngine (`src/nexus_sync/engine.rs`)

Orchestrates synchronization between Vectorizer and Nexus.

```rust
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct NexusSyncEngine {
    nexus_client: Arc<NexusClient>,
    vector_store: Arc<VectorStore>,
    domain_classifier: Arc<DomainClassifier>,
    config: SyncConfig,
    event_tx: mpsc::UnboundedSender<SyncEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<SyncEvent>>>>,
    workers: Vec<tokio::task::JoinHandle<()>>,
    state: Arc<RwLock<SyncState>>,
}

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub enabled: bool,
    pub worker_threads: usize,
    pub batch_size: usize,
    pub retry_attempts: usize,
    pub retry_delay_ms: u64,
    pub similarity_threshold: f32,
    pub similarity_top_k: usize,
    pub auto_classify_domains: bool,
    pub sync_collections: Vec<CollectionSyncConfig>,
}

#[derive(Debug, Clone)]
pub struct CollectionSyncConfig {
    pub name: String,
    pub domain: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum SyncEvent {
    VectorInserted {
        collection: String,
        vector_id: String,
        payload: Payload,
    },
    VectorUpdated {
        collection: String,
        vector_id: String,
        payload: Payload,
    },
    VectorDeleted {
        collection: String,
        vector_id: String,
    },
    BatchSync {
        collection: String,
        vector_ids: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct SyncState {
    pub total_synced: u64,
    pub total_errors: u64,
    pub last_sync: Option<DateTime<Utc>>,
    pub in_progress: HashSet<String>,
}

impl NexusSyncEngine {
    pub fn new(
        nexus_client: Arc<NexusClient>,
        vector_store: Arc<VectorStore>,
        config: SyncConfig,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let domain_classifier = Arc::new(DomainClassifier::new());

        Ok(Self {
            nexus_client,
            vector_store,
            domain_classifier,
            config,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            workers: Vec::new(),
            state: Arc::new(RwLock::new(SyncState::default())),
        })
    }

    /// Start sync workers
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Nexus sync is disabled");
            return Ok(());
        }

        let mut rx = self.event_rx.write()
            .take()
            .ok_or_else(|| Error::AlreadyStarted)?;

        // Start worker threads
        for worker_id in 0..self.config.worker_threads {
            let nexus = self.nexus_client.clone();
            let store = self.vector_store.clone();
            let classifier = self.domain_classifier.clone();
            let config = self.config.clone();
            let state = self.state.clone();

            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    &mut rx,
                    nexus,
                    store,
                    classifier,
                    config,
                    state,
                ).await;
            });

            self.workers.push(handle);
        }

        info!("Started {} Nexus sync workers", self.config.worker_threads);
        Ok(())
    }

    /// Worker loop processing sync events
    async fn worker_loop(
        worker_id: usize,
        rx: &mut mpsc::UnboundedReceiver<SyncEvent>,
        nexus: Arc<NexusClient>,
        store: Arc<VectorStore>,
        classifier: Arc<DomainClassifier>,
        config: SyncConfig,
        state: Arc<RwLock<SyncState>>,
    ) {
        debug!("Nexus sync worker {} started", worker_id);

        while let Some(event) = rx.recv().await {
            let result = Self::process_event(
                &event,
                &nexus,
                &store,
                &classifier,
                &config,
            ).await;

            match result {
                Ok(sync_result) => {
                    let mut state = state.write();
                    state.total_synced += 1;
                    state.last_sync = Some(Utc::now());
                    
                    info!(
                        "Worker {} synced: node_id={}, relationships={}",
                        worker_id,
                        sync_result.node_id.as_deref().unwrap_or("N/A"),
                        sync_result.relationships_created
                    );
                }
                Err(e) => {
                    let mut state = state.write();
                    state.total_errors += 1;
                    
                    error!(
                        "Worker {} sync failed: {:?}",
                        worker_id,
                        e
                    );
                }
            }
        }

        debug!("Nexus sync worker {} stopped", worker_id);
    }

    /// Process sync event
    async fn process_event(
        event: &SyncEvent,
        nexus: &Arc<NexusClient>,
        store: &Arc<VectorStore>,
        classifier: &Arc<DomainClassifier>,
        config: &SyncConfig,
    ) -> Result<SyncResult> {
        match event {
            SyncEvent::VectorInserted { collection, vector_id, payload } => {
                Self::sync_insert(
                    collection,
                    vector_id,
                    payload,
                    nexus,
                    store,
                    classifier,
                    config,
                ).await
            }
            SyncEvent::VectorUpdated { collection, vector_id, payload } => {
                Self::sync_update(
                    collection,
                    vector_id,
                    payload,
                    nexus,
                    config,
                ).await
            }
            SyncEvent::VectorDeleted { collection, vector_id } => {
                Self::sync_delete(
                    collection,
                    vector_id,
                    nexus,
                ).await
            }
            SyncEvent::BatchSync { collection, vector_ids } => {
                Self::sync_batch(
                    collection,
                    vector_ids,
                    nexus,
                    store,
                    classifier,
                    config,
                ).await
            }
        }
    }

    /// Sync document insertion
    async fn sync_insert(
        collection: &str,
        vector_id: &str,
        payload: &Payload,
        nexus: &Arc<NexusClient>,
        store: &Arc<VectorStore>,
        classifier: &Arc<DomainClassifier>,
        config: &SyncConfig,
    ) -> Result<SyncResult> {
        let start = Instant::now();

        // 1. Classify domain
        let domain = classifier.classify(collection, payload)?;

        // 2. Extract metadata
        let title = payload.data.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let doc_type = payload.data.get("doc_type")
            .and_then(|v| v.as_str())
            .unwrap_or("document")
            .to_string();

        // 3. Get embedding
        let collection_obj = store.get_collection(collection)?;
        let vector = collection_obj.get_vector(vector_id)?;

        // 4. Create document node
        let doc_node = DocumentNode {
            vector_id: vector_id.to_string(),
            collection: collection.to_string(),
            title,
            domain: domain.clone(),
            doc_type,
            embedding: vector.data.clone(),
        };

        let node_id = nexus.create_document_node(&doc_node).await?;

        // 5. Find similar documents
        let similar = store.search(
            collection,
            &vector.data,
            config.similarity_top_k,
        )?;

        // 6. Filter by threshold and create relationships
        let similar_above_threshold: Vec<_> = similar
            .into_iter()
            .filter(|r| r.score >= config.similarity_threshold)
            .map(|r| (r.id, r.score))
            .collect();

        let relationships_created = if !similar_above_threshold.is_empty() {
            nexus.create_similarity_edges(&node_id, &similar_above_threshold).await?
        } else {
            0
        };

        // 7. Store sync metadata in payload
        let sync_metadata = NexusSyncMetadata {
            node_id: node_id.clone(),
            synced_at: Utc::now(),
            sync_status: SyncStatus::Synced,
            domain: domain.clone(),
            relationship_count: relationships_created,
            entities: Vec::new(),
            related_documents: similar_above_threshold.iter().map(|(id, _)| id.clone()).collect(),
        };

        // Update payload with sync metadata
        // (This would require extending VectorStore API)

        Ok(SyncResult {
            success: true,
            node_id: Some(node_id),
            relationships_created,
            entities_extracted: 0,
            domain,
            execution_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        })
    }

    /// Enqueue sync event (called from VectorStore hooks)
    pub fn enqueue(&self, event: SyncEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.event_tx.send(event)
            .map_err(|e| Error::SyncQueueFull(e.to_string()))?;

        Ok(())
    }

    /// Get current sync state
    pub fn get_state(&self) -> SyncState {
        self.state.read().clone()
    }

    /// Graceful shutdown
    pub async fn shutdown(mut self) -> Result<()> {
        drop(self.event_tx); // Close channel

        for handle in self.workers.drain(..) {
            handle.await?;
        }

        info!("Nexus sync engine shut down gracefully");
        Ok(())
    }
}
```

### 3. DomainClassifier (`src/nexus_sync/domain_classifier.rs`)

Classifies documents into domains based on collection name and metadata.

```rust
use regex::Regex;

pub struct DomainClassifier {
    rules: Vec<DomainRule>,
}

#[derive(Debug, Clone)]
pub struct DomainRule {
    pub domain: String,
    pub collection_patterns: Vec<Regex>,
    pub keywords: Vec<String>,
    pub metadata_matchers: HashMap<String, Vec<String>>,
    pub priority: u32,
}

impl DomainClassifier {
    pub fn new() -> Self {
        let rules = vec![
            DomainRule {
                domain: "legal".to_string(),
                collection_patterns: vec![
                    Regex::new(r"legal").unwrap(),
                    Regex::new(r"law").unwrap(),
                    Regex::new(r"contract").unwrap(),
                ],
                keywords: vec![
                    "contract".to_string(),
                    "agreement".to_string(),
                    "legal".to_string(),
                    "jurisdiction".to_string(),
                    "clause".to_string(),
                ],
                metadata_matchers: hashmap! {
                    "doc_type".to_string() => vec![
                        "contract".to_string(),
                        "law".to_string(),
                        "regulation".to_string(),
                    ],
                },
                priority: 100,
            },
            DomainRule {
                domain: "financial".to_string(),
                collection_patterns: vec![
                    Regex::new(r"financ").unwrap(),
                    Regex::new(r"invoice").unwrap(),
                    Regex::new(r"accounting").unwrap(),
                ],
                keywords: vec![
                    "invoice".to_string(),
                    "payment".to_string(),
                    "budget".to_string(),
                    "revenue".to_string(),
                    "expense".to_string(),
                ],
                metadata_matchers: hashmap! {
                    "doc_type".to_string() => vec![
                        "invoice".to_string(),
                        "receipt".to_string(),
                        "report".to_string(),
                    ],
                },
                priority: 100,
            },
            DomainRule {
                domain: "hr".to_string(),
                collection_patterns: vec![
                    Regex::new(r"hr").unwrap(),
                    Regex::new(r"human.?resources").unwrap(),
                    Regex::new(r"employee").unwrap(),
                ],
                keywords: vec![
                    "employee".to_string(),
                    "resume".to_string(),
                    "cv".to_string(),
                    "policy".to_string(),
                    "benefits".to_string(),
                ],
                metadata_matchers: hashmap! {
                    "doc_type".to_string() => vec![
                        "resume".to_string(),
                        "policy".to_string(),
                        "evaluation".to_string(),
                    ],
                },
                priority: 100,
            },
            DomainRule {
                domain: "engineering".to_string(),
                collection_patterns: vec![
                    Regex::new(r"code").unwrap(),
                    Regex::new(r"engineer").unwrap(),
                    Regex::new(r"tech").unwrap(),
                ],
                keywords: vec![
                    "api".to_string(),
                    "function".to_string(),
                    "class".to_string(),
                    "implementation".to_string(),
                ],
                metadata_matchers: hashmap! {
                    "doc_type".to_string() => vec![
                        "code".to_string(),
                        "spec".to_string(),
                        "design".to_string(),
                    ],
                },
                priority: 100,
            },
        ];

        Self { rules }
    }

    /// Classify document into domain
    pub fn classify(&self, collection: &str, payload: &Payload) -> Result<String> {
        let mut best_match: Option<(&DomainRule, u32)> = None;

        for rule in &self.rules {
            let mut score = 0u32;

            // Check collection name
            for pattern in &rule.collection_patterns {
                if pattern.is_match(collection) {
                    score += 50;
                    break;
                }
            }

            // Check metadata
            for (meta_key, values) in &rule.metadata_matchers {
                if let Some(meta_value) = payload.data.get(meta_key).and_then(|v| v.as_str()) {
                    if values.iter().any(|v| v == meta_value) {
                        score += 30;
                    }
                }
            }

            // Check keywords in content
            if let Some(content) = payload.data.get("content").and_then(|v| v.as_str()) {
                let content_lower = content.to_lowercase();
                let keyword_matches = rule.keywords
                    .iter()
                    .filter(|kw| content_lower.contains(&kw.to_lowercase()))
                    .count();
                
                score += (keyword_matches * 10) as u32;
            }

            // Update best match
            if score > 0 {
                if let Some((_, best_score)) = best_match {
                    if score > best_score {
                        best_match = Some((rule, score));
                    }
                } else {
                    best_match = Some((rule, score));
                }
            }
        }

        Ok(best_match
            .map(|(rule, _)| rule.domain.clone())
            .unwrap_or_else(|| "general".to_string()))
    }
}
```

### 4. WebhookHandler (`src/nexus_sync/webhooks/mod.rs`)

Handles webhook events from Nexus for enrichment.

```rust
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};

pub struct WebhookManager {
    enrichment_engine: Arc<EnrichmentEngine>,
}

impl WebhookManager {
    pub fn routes() -> Router<Arc<Self>> {
        Router::new()
            .route("/webhooks/nexus/relationship", post(handle_relationship_created))
            .route("/webhooks/nexus/node", post(handle_node_updated))
    }
}

#[derive(Debug, Deserialize)]
struct RelationshipCreatedEvent {
    event_type: String,
    relationship: Relationship,
    source_node: Node,
    target_node: Node,
}

#[derive(Debug, Deserialize)]
struct Relationship {
    id: String,
    source: String,
    target: String,
    rel_type: String,
    properties: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    labels: Vec<String>,
    properties: HashMap<String, Value>,
}

async fn handle_relationship_created(
    State(manager): State<Arc<WebhookManager>>,
    Json(event): Json<RelationshipCreatedEvent>,
) -> Result<StatusCode, StatusCode> {
    info!("Received relationship created event: {}", event.relationship.id);

    // Extract vector_ids from nodes
    let source_vector_id = event.source_node.properties
        .get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let target_vector_id = event.target_node.properties
        .get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Trigger enrichment for both vectors
    manager.enrichment_engine
        .enrich_from_relationship(source_vector_id, target_vector_id, &event.relationship)
        .await
        .map_err(|e| {
            error!("Enrichment failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}
```

---

## API Specifications

### REST Endpoints

#### Enable Sync for Collection

```
POST /api/v1/sync/nexus/enable
```

**Request:**
```json
{
  "collection": "legal_documents",
  "domain": "legal",
  "config": {
    "similarity_threshold": 0.75,
    "similarity_top_k": 20,
    "auto_classify": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Sync enabled for collection 'legal_documents'",
  "collection": "legal_documents",
  "domain": "legal"
}
```

#### Get Sync Status

```
GET /api/v1/sync/nexus/status
```

**Response:**
```json
{
  "enabled": true,
  "total_synced": 15420,
  "total_errors": 12,
  "last_sync": "2024-01-15T10:30:45Z",
  "in_progress": 5,
  "worker_threads": 4,
  "queue_depth": 23,
  "collections": [
    {
      "name": "legal_documents",
      "domain": "legal",
      "synced": 8500,
      "last_sync": "2024-01-15T10:30:45Z"
    },
    {
      "name": "financial_documents",
      "domain": "financial",
      "synced": 6920,
      "last_sync": "2024-01-15T10:29:12Z"
    }
  ]
}
```

#### Trigger Manual Sync

```
POST /api/v1/sync/nexus/trigger
```

**Request:**
```json
{
  "collection": "legal_documents",
  "batch_size": 100,
  "force_resync": false
}
```

**Response:**
```json
{
  "success": true,
  "message": "Sync triggered for collection 'legal_documents'",
  "batch_size": 100,
  "estimated_total": 8500,
  "job_id": "sync-job-abc123"
}
```

---

## Configuration

### config.yml

```yaml
# Nexus Integration Configuration
nexus_integration:
  # Enable/disable integration
  enabled: true
  
  # Nexus server URL
  nexus_url: "http://localhost:15474"
  
  # API key for authentication
  api_key: "${NEXUS_API_KEY}"
  
  # Connection settings
  connection:
    timeout_seconds: 30
    pool_size: 10
    pool_idle_timeout_seconds: 90
  
  # Sync settings
  sync:
    mode: "async"  # "sync" or "async"
    worker_threads: 4
    batch_size: 100
    retry_attempts: 3
    retry_delay_ms: 1000
    max_queue_size: 10000
  
  # Similarity settings
  similarity:
    threshold: 0.75
    top_k: 20
    method: "embedding"  # "embedding", "keyword", or "hybrid"
  
  # Domain classification
  domains:
    auto_classify: true
    default_domain: "general"
    
    # Custom domain rules
    rules:
      - domain: "legal"
        collection_patterns: ["legal.*", ".*law.*", ".*contract.*"]
        keywords: ["contract", "agreement", "jurisdiction"]
        priority: 100
      
      - domain: "financial"
        collection_patterns: ["financ.*", ".*invoice.*"]
        keywords: ["invoice", "payment", "budget"]
        priority: 100
  
  # Collections to sync
  collections:
    - name: "legal_documents"
      domain: "legal"
      enabled: true
      similarity_threshold: 0.80
    
    - name: "financial_documents"
      domain: "financial"
      enabled: true
      similarity_threshold: 0.75
    
    - name: "hr_documents"
      domain: "hr"
      enabled: true
      similarity_threshold: 0.70
    
    - name: "engineering_documents"
      domain: "engineering"
      enabled: false
  
  # Enrichment settings
  enrichment:
    enabled: true
    batch_size: 50
    cache_ttl_seconds: 3600
  
  # Webhook settings
  webhooks:
    enabled: true
    secret: "${NEXUS_WEBHOOK_SECRET}"
    endpoints:
      - url: "${VECTORIZER_URL}/webhooks/nexus/relationship"
        events: ["relationship.created", "relationship.updated"]
      - url: "${VECTORIZER_URL}/webhooks/nexus/node"
        events: ["node.updated"]
```

### Environment Variables

```bash
# Nexus connection
NEXUS_URL=http://localhost:15474
NEXUS_API_KEY=your-api-key-here

# Vectorizer callback
VECTORIZER_URL=http://localhost:15002

# Webhook security
NEXUS_WEBHOOK_SECRET=your-webhook-secret

# Feature flags
NEXUS_SYNC_ENABLED=true
NEXUS_ENRICHMENT_ENABLED=true
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_classification() {
        let classifier = DomainClassifier::new();
        
        let payload = Payload::new(json!({
            "doc_type": "contract",
            "content": "This is a legal agreement between parties..."
        }));
        
        let domain = classifier.classify("legal_documents", &payload).unwrap();
        assert_eq!(domain, "legal");
    }

    #[tokio::test]
    async fn test_nexus_client_create_node() {
        let client = NexusClient::new(
            "http://localhost:15474".to_string(),
            None,
        ).unwrap();
        
        let doc = DocumentNode {
            vector_id: "test-123".to_string(),
            collection: "test".to_string(),
            title: "Test Document".to_string(),
            domain: "general".to_string(),
            doc_type: "document".to_string(),
            embedding: vec![0.1; 768],
        };
        
        let node_id = client.create_document_node(&doc).await.unwrap();
        assert!(!node_id.is_empty());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_sync_flow() {
    // Setup
    let store = create_test_vector_store();
    let nexus = create_test_nexus_client();
    let sync_engine = NexusSyncEngine::new(nexus.clone(), store.clone(), test_config());
    
    // Insert document
    let vector_id = "test-doc-1".to_string();
    let payload = Payload::new(json!({
        "title": "Test Document",
        "content": "This is a test document for legal purposes",
        "doc_type": "contract"
    }));
    
    store.insert("legal_documents", vec![
        Vector::with_payload(vector_id.clone(), test_embedding(), payload)
    ]).unwrap();
    
    // Wait for async sync
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify node created in Nexus
    let cypher = "MATCH (d:Document {vector_id: $vector_id}) RETURN d";
    let response = nexus.execute_cypher(cypher, hashmap!{
        "vector_id".to_string() => json!(vector_id),
    }).await.unwrap();
    
    assert_eq!(response.row_count, 1);
}
```

---

## Monitoring

### Prometheus Metrics

```rust
use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec};

lazy_static! {
    pub static ref NEXUS_SYNC_TOTAL: CounterVec = register_counter_vec!(
        "vectorizer_nexus_sync_total",
        "Total number of Nexus sync operations",
        &["collection", "status"]
    ).unwrap();

    pub static ref NEXUS_SYNC_DURATION: HistogramVec = register_histogram_vec!(
        "vectorizer_nexus_sync_duration_seconds",
        "Duration of Nexus sync operations",
        &["collection"]
    ).unwrap();

    pub static ref NEXUS_RELATIONSHIPS_CREATED: CounterVec = register_counter_vec!(
        "vectorizer_nexus_relationships_created_total",
        "Total number of relationships created in Nexus",
        &["relationship_type"]
    ).unwrap();
}
```

### Grafana Dashboard

See `monitoring/grafana-dashboard-nexus-sync.json` for complete dashboard.

---

## Deployment

### Docker Compose

```yaml
version: '3.8'

services:
  vectorizer:
    image: vectorizer:latest
    ports:
      - "15002:15002"
    environment:
      - NEXUS_URL=http://nexus:15474
      - NEXUS_API_KEY=${NEXUS_API_KEY}
      - NEXUS_SYNC_ENABLED=true
    depends_on:
      - nexus
    volumes:
      - ./vectorizer-data:/vectorizer/data
      - ./config.yml:/vectorizer/config.yml

  nexus:
    image: nexus:latest
    ports:
      - "15474:15474"
    environment:
      - VECTORIZER_URL=http://vectorizer:15002
      - VECTORIZER_API_KEY=${VECTORIZER_API_KEY}
    volumes:
      - ./nexus-data:/nexus/data

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ./grafana-data:/var/lib/grafana
```

---

## Troubleshooting

### Common Issues

**1. Sync Not Working**
- Check `NEXUS_SYNC_ENABLED=true`
- Verify Nexus URL is accessible
- Check API key is valid
- Review logs: `vectorizer_nexus_sync_errors_total`

**2. High Latency**
- Increase worker threads
- Reduce `similarity_top_k`
- Enable caching
- Check network latency to Nexus

**3. Memory Usage**
- Reduce `batch_size`
- Lower `max_queue_size`
- Monitor queue depth

---

This completes the technical implementation guide for the Vectorizer side of the integration. Next, I'll create the Nexus documentation.

