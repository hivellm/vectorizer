package vectorizer

// Metric represents distance metric
type Metric string

const (
	MetricCosine     Metric = "Cosine"
	MetricEuclidean  Metric = "Euclidean"
	MetricDotProduct Metric = "DotProduct"
)

// ===== CLIENT-SIDE REPLICATION CONFIGURATION =====

// ReadPreference represents read preference for routing read operations.
// Similar to MongoDB's read preferences.
type ReadPreference string

const (
	// ReadPreferenceMaster routes all reads to master
	ReadPreferenceMaster ReadPreference = "master"
	// ReadPreferenceReplica routes reads to replicas (round-robin)
	ReadPreferenceReplica ReadPreference = "replica"
	// ReadPreferenceNearest routes to the node with lowest latency
	ReadPreferenceNearest ReadPreference = "nearest"
)

// HostConfig holds host configuration for master/replica topology.
type HostConfig struct {
	// Master is the master node URL (receives all write operations)
	Master string
	// Replicas are the replica node URLs (receive read operations based on ReadPreference)
	Replicas []string
}

// ReadOptions holds options for read operations that can override default settings.
type ReadOptions struct {
	// ReadPreference overrides the default read preference for this operation
	ReadPreference ReadPreference
}

// CollectionConfig represents collection configuration
type CollectionConfig struct {
	Dimension int    `json:"dimension"`
	Metric    Metric `json:"metric"`
}

// CreateCollectionRequest represents a request to create a collection
type CreateCollectionRequest struct {
	Name   string            `json:"name"`
	Config *CollectionConfig `json:"config"`
}

// Collection represents a collection
type Collection struct {
	Name   string            `json:"name"`
	Config *CollectionConfig `json:"config"`
}

// Vector represents a vector
type Vector struct {
	ID        string                 `json:"id"`
	Data      []float32              `json:"data"`
	Payload   map[string]interface{} `json:"payload,omitempty"`
	PublicKey string                 `json:"publicKey,omitempty"` // Optional ECC public key for payload encryption
}

// SearchOptions represents search options
type SearchOptions struct {
	Limit   int                    `json:"limit,omitempty"`
	Filter  map[string]interface{} `json:"filter,omitempty"`
	Payload []string               `json:"payload,omitempty"`
}

// SearchResult represents a search result
type SearchResult struct {
	ID      string                 `json:"id"`
	Score   float64                `json:"score"`
	Payload map[string]interface{} `json:"payload,omitempty"`
	Vector  []float32              `json:"vector,omitempty"`
}

// InsertTextRequest represents a request to insert text
type InsertTextRequest struct {
	Text    string                 `json:"text"`
	Payload map[string]interface{} `json:"payload,omitempty"`
}

// InsertTextResponse represents a response from inserting text
type InsertTextResponse struct {
	Message    string `json:"message"`
	Text       string `json:"text,omitempty"`
	VectorID   string `json:"vector_id"`
	Collection string `json:"collection,omitempty"`
}

// ID returns the vector ID (for compatibility)
func (r *InsertTextResponse) ID() string {
	return r.VectorID
}

// DatabaseStats represents database statistics
type DatabaseStats struct {
	Collections int `json:"collections"`
	Vectors     int `json:"vectors"`
}

// CollectionInfo represents collection information.
//
// Phase25 §6 added VectorCountHistory: a per-collection ring buffer
// of (unix_ts, count) samples, sampled lazily on each
// GET /collections/{name} read. omitempty so older servers parse
// unchanged.
type CollectionInfo struct {
	Name                string              `json:"name"`
	VectorCount         int                 `json:"vector_count"`
	Dimension           int                 `json:"dimension"`
	Metric              string              `json:"metric"`
	VectorCountHistory  []VectorCountSample `json:"vector_count_history,omitempty"`
}

// CollectionsListResponse represents the response from listing collections
type CollectionsListResponse struct {
	Collections      []CollectionInfo `json:"collections"`
	TotalCollections int              `json:"total_collections"`
}

// SearchResponse represents the response from search operations
type SearchResponse struct {
	Results      []SearchResult `json:"results"`
	Query        string         `json:"query,omitempty"`
	Limit        int            `json:"limit,omitempty"`
	Collection   string         `json:"collection,omitempty"`
	TotalResults int            `json:"total_results,omitempty"`
}

// Graph models

// GraphNode represents a graph node
type GraphNode struct {
	ID       string                 `json:"id"`
	NodeType string                 `json:"node_type"`
	Metadata map[string]interface{} `json:"metadata"`
}

// GraphEdge represents a graph edge
type GraphEdge struct {
	ID               string                 `json:"id"`
	Source           string                 `json:"source"`
	Target           string                 `json:"target"`
	RelationshipType string                 `json:"relationship_type"`
	Weight           float32                `json:"weight"`
	Metadata         map[string]interface{} `json:"metadata"`
	CreatedAt        string                 `json:"created_at"`
}

// NeighborInfo represents neighbor information
type NeighborInfo struct {
	Node GraphNode `json:"node"`
	Edge GraphEdge `json:"edge"`
}

// RelatedNodeInfo represents related node information
type RelatedNodeInfo struct {
	Node     GraphNode `json:"node"`
	Distance int       `json:"distance"`
	Weight   float32   `json:"weight"`
}

// FindRelatedRequest represents a request to find related nodes
type FindRelatedRequest struct {
	MaxHops          *int    `json:"max_hops,omitempty"`
	RelationshipType *string `json:"relationship_type,omitempty"`
}

// FindRelatedResponse represents the response from finding related nodes
type FindRelatedResponse struct {
	Related []RelatedNodeInfo `json:"related"`
}

// FindPathRequest represents a request to find path between nodes
type FindPathRequest struct {
	Collection string `json:"collection"`
	Source     string `json:"source"`
	Target     string `json:"target"`
}

// FindPathResponse represents the response from finding path
type FindPathResponse struct {
	Path  []GraphNode `json:"path"`
	Found bool        `json:"found"`
}

// CreateEdgeRequest represents a request to create an edge
type CreateEdgeRequest struct {
	Collection       string   `json:"collection"`
	Source           string   `json:"source"`
	Target           string   `json:"target"`
	RelationshipType string   `json:"relationship_type"`
	Weight           *float32 `json:"weight,omitempty"`
}

// CreateEdgeResponse represents the response from creating an edge
type CreateEdgeResponse struct {
	EdgeID  string `json:"edge_id"`
	Success bool   `json:"success"`
	Message string `json:"message"`
}

// ListNodesResponse represents the response from listing nodes
type ListNodesResponse struct {
	Nodes []GraphNode `json:"nodes"`
	Count int         `json:"count"`
}

// GetNeighborsResponse represents the response from getting neighbors
type GetNeighborsResponse struct {
	Neighbors []NeighborInfo `json:"neighbors"`
}

// ListEdgesResponse represents the response from listing edges
type ListEdgesResponse struct {
	Edges []GraphEdge `json:"edges"`
	Count int         `json:"count"`
}

// DiscoverEdgesRequest represents a request to discover edges
type DiscoverEdgesRequest struct {
	SimilarityThreshold *float32 `json:"similarity_threshold,omitempty"`
	MaxPerNode          *int     `json:"max_per_node,omitempty"`
}

// DiscoverEdgesResponse represents the response from discovering edges
type DiscoverEdgesResponse struct {
	Success      bool   `json:"success"`
	EdgesCreated int    `json:"edges_created"`
	Message      string `json:"message"`
}

// DiscoveryStatusResponse represents the response from getting discovery status
type DiscoveryStatusResponse struct {
	TotalNodes         int     `json:"total_nodes"`
	NodesWithEdges     int     `json:"nodes_with_edges"`
	TotalEdges         int     `json:"total_edges"`
	ProgressPercentage float64 `json:"progress_percentage"`
}

// ===== Admin / Observability =====

// Stats represents aggregate server statistics returned by GET /stats.
//
// Phase25 §5 added DefaultQuantization and CompressionRatio. Both are
// emitted as omitempty so older servers parse unchanged; consumers that
// want a default value should fall back to ("none", 1.0).
type Stats struct {
	Collections         int     `json:"collections"`
	TotalVectors        int     `json:"total_vectors"`
	UptimeSeconds       int64   `json:"uptime_seconds"`
	Version             string  `json:"version"`
	DefaultQuantization string  `json:"default_quantization,omitempty"`
	CompressionRatio    float32 `json:"compression_ratio,omitempty"`
}

// VectorCountSample is one entry in CollectionInfo.VectorCountHistory
// (phase25 §6). Sampled at most once per minute on the read path.
type VectorCountSample struct {
	At    int64 `json:"at"`
	Count int   `json:"count"`
}

// RouteStats is one row inside RuntimeMetrics.ThroughputByRoute.
type RouteStats struct {
	Route  string  `json:"route"`
	QPS    float64 `json:"qps"`
	P50Ms  float64 `json:"p50_ms"`
	P99Ms  float64 `json:"p99_ms"`
}

// WalSnapshot is the WAL section inside RuntimeMetrics (phase25 §3).
// All fields are zero on standalone servers without replication.
type WalSnapshot struct {
	CurrentSeq        uint64 `json:"current_seq"`
	SizeBytes         uint64 `json:"size_bytes"`
	LastCheckpointAt  uint64 `json:"last_checkpoint_at"`
	LastCheckpointSeq uint64 `json:"last_checkpoint_seq"`
}

// RuntimeMetrics is the snapshot returned by GET /metrics/runtime
// (phase25). Older servers without phase25 §4 may omit any field; the
// JSON tags use omitempty + the consumer pattern is "treat zero as
// unknown".
type RuntimeMetrics struct {
	CPUPercent         float64      `json:"cpu_percent,omitempty"`
	MemoryRSSBytes     uint64       `json:"memory_rss_bytes,omitempty"`
	MemoryTotalBytes   uint64       `json:"memory_total_bytes,omitempty"`
	MemoryPercent      float64      `json:"memory_percent,omitempty"`
	ActiveConnections  int          `json:"active_connections,omitempty"`
	UptimeSeconds      uint64       `json:"uptime_seconds,omitempty"`
	QPSWindow60s       float64      `json:"qps_window_60s,omitempty"`
	ErrorRate5xx60s    float64      `json:"error_rate_5xx_60s,omitempty"`
	ThroughputByRoute  []RouteStats `json:"throughput_by_route,omitempty"`
	WAL                WalSnapshot  `json:"wal,omitempty"`
}

// ServerStatus represents server liveness state returned by GET /status.
type ServerStatus struct {
	Online           bool   `json:"online"`
	Version          string `json:"version"`
	UptimeSeconds    int64  `json:"uptime_seconds"`
	CollectionsCount int    `json:"collections_count"`
}

// LogEntry represents one log line returned by GET /logs.
type LogEntry struct {
	Timestamp string `json:"timestamp"`
	Level     string `json:"level"`
	Message   string `json:"message"`
	Source    string `json:"source,omitempty"`
}

// CollectionProgress represents the indexing progress for a single collection.
type CollectionProgress struct {
	CollectionName string  `json:"collection_name"`
	Status         string  `json:"status"`
	Progress       float64 `json:"progress"`
	VectorCount    int     `json:"vector_count"`
	ErrorMessage   *string `json:"error_message,omitempty"`
	LastUpdated    string  `json:"last_updated"`
}

// IndexingProgress represents per-collection indexing progress returned by
// GET /indexing/progress.
type IndexingProgress struct {
	IsIndexing    bool                 `json:"is_indexing"`
	OverallStatus string               `json:"overall_status"`
	Collections   []CollectionProgress `json:"collections"`
}

// ConfigSnapshot represents the server configuration as free-form JSON.
type ConfigSnapshot map[string]interface{}

// BackupInfo represents metadata for one server-side backup file.
type BackupInfo struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	Date        string   `json:"date"`
	Size        int64    `json:"size"`
	Collections []string `json:"collections"`
}

// CleanupReport represents the result of DELETE /collections/cleanup.
type CleanupReport struct {
	Success     bool     `json:"success"`
	Removed     int      `json:"removed"`
	Collections []string `json:"collections"`
	Message     *string  `json:"message,omitempty"`
}

// WorkspaceConfig represents a workspace configuration entry as free-form JSON.
type WorkspaceConfig map[string]interface{}

// AddWorkspaceRequest represents the request body for POST /workspace/add.
type AddWorkspaceRequest struct {
	Path           string `json:"path"`
	CollectionName string `json:"collection_name"`
}

// CreateBackupRequest represents the request body for POST /backups/create.
type CreateBackupRequest struct {
	Name        string   `json:"name,omitempty"`
	Collections []string `json:"collections,omitempty"`
}

// RestoreBackupRequest represents the request body for POST /backups/restore.
type RestoreBackupRequest struct {
	BackupID string `json:"backup_id"`
}

// SlowQueryEntry represents one entry in the slow-query ring buffer.
type SlowQueryEntry struct {
	Timestamp  string  `json:"timestamp"`
	Collection string  `json:"collection"`
	K          int     `json:"k"`
	DurationMs float64 `json:"duration_ms"`
}

// SlowQueryConfig represents the slow-query ring-buffer configuration.
type SlowQueryConfig struct {
	ThresholdMs int64 `json:"threshold_ms"`
	Capacity    int   `json:"capacity"`
}

// ===== Tier-Control =====

// VectorOpResult represents the per-vector outcome of a batch operation.
type VectorOpResult struct {
	ID     *string `json:"id,omitempty"`
	Status string  `json:"status"`
	Error  *string `json:"error,omitempty"`
	Index  *int    `json:"index,omitempty"`
}

// DeleteByFilterReport represents the result of
// POST /collections/{name}/vectors/delete_by_filter.
type DeleteByFilterReport struct {
	Scanned int              `json:"scanned"`
	Matched int              `json:"matched"`
	Deleted int              `json:"deleted"`
	Results []VectorOpResult `json:"results"`
}

// BulkUpdateReport represents the result of
// POST /collections/{name}/vectors/bulk_update_metadata.
type BulkUpdateReport struct {
	Scanned int              `json:"scanned"`
	Matched int              `json:"matched"`
	Updated int              `json:"updated"`
	Results []VectorOpResult `json:"results"`
}

// CopyReport represents the result of POST /collections/{src}/vectors/copy.
type CopyReport struct {
	Src       string           `json:"src"`
	Dst       string           `json:"dst"`
	Requested int              `json:"requested"`
	Copied    int              `json:"copied"`
	Failed    int              `json:"failed"`
	Results   []VectorOpResult `json:"results"`
}

// DeleteReport represents the result of POST /batch_delete.
type DeleteReport struct {
	Collection string           `json:"collection"`
	Count      int              `json:"count"`
	Deleted    int              `json:"deleted"`
	Failed     int              `json:"failed"`
	Results    []VectorOpResult `json:"results"`
}

// MoveReport represents the result of POST /collections/{src}/vectors/move.
type MoveReport struct {
	Src       string           `json:"src"`
	Dst       string           `json:"dst"`
	Requested int              `json:"requested"`
	Moved     int              `json:"moved"`
	Failed    int              `json:"failed"`
	Results   []VectorOpResult `json:"results"`
}

// ReencodeJob represents the job descriptor returned by
// POST /collections/{name}/reencode.
type ReencodeJob struct {
	JobID          string  `json:"job_id"`
	Collection     string  `json:"collection"`
	State          string  `json:"state"`
	TargetEncoding string  `json:"target_encoding"`
	Progress       float64 `json:"progress"`
}

// TtlConfig represents a per-collection TTL configuration.
type TtlConfig struct {
	TtlSecs *int64 `json:"ttl_secs"`
}

// VectorExpiryRequest represents the request body for
// PATCH /collections/{name}/vectors/{id}/expiry.
type VectorExpiryRequest struct {
	ExpiresAt *int64 `json:"expires_at"`
}

// VectorPage represents a paginated vector listing from
// GET /collections/{name}/vectors.
type VectorPage struct {
	Total   int                      `json:"total"`
	Vectors []map[string]interface{} `json:"vectors"`
	Limit   int                      `json:"limit,omitempty"`
	Offset  int                      `json:"offset,omitempty"`
}

// BatchInsertItem represents one item in a batch_insert_texts call.
type BatchInsertItem struct {
	ID       *string                `json:"id,omitempty"`
	Text     string                 `json:"text"`
	Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// BatchInsertReport represents the result of POST /batch_insert or
// POST /insert_vectors.
type BatchInsertReport struct {
	Collection string                   `json:"collection"`
	Total      int                      `json:"count,omitempty"`
	Inserted   int                      `json:"inserted"`
	Failed     int                      `json:"failed"`
	Results    []map[string]interface{} `json:"results,omitempty"`
}

// BatchUpdateReport represents the result of POST /batch_update.
type BatchUpdateReport struct {
	Collection string                   `json:"collection"`
	Total      int                      `json:"count,omitempty"`
	Updated    int                      `json:"updated"`
	Failed     int                      `json:"failed"`
	Results    []map[string]interface{} `json:"results,omitempty"`
}

// ===== Schema Evolution =====

// ReindexParams represents the parameters for POST /collections/{name}/reindex.
type ReindexParams struct {
	M              int `json:"m"`
	EfConstruction int `json:"ef_construction"`
	EfSearch       int `json:"ef_search"`
}

// ReindexJob represents the job descriptor returned by
// POST /collections/{name}/reindex.
type ReindexJob struct {
	JobID      string                 `json:"job_id"`
	Collection string                 `json:"collection"`
	State      string                 `json:"state"`
	Params     map[string]interface{} `json:"params"`
	Progress   float64                `json:"progress"`
}

// NativeSnapshotInfo represents metadata for a native collection snapshot.
type NativeSnapshotInfo struct {
	ID         string `json:"id"`
	Collection string `json:"collection"`
	CreatedAt  string `json:"created_at"`
	SizeBytes  int64  `json:"size_bytes"`
}

// ExplainTrace represents the HNSW execution trace from
// POST /collections/{name}/explain.
type ExplainTrace struct {
	VisitedNodes        int     `json:"visited_nodes"`
	EfSearch            int     `json:"ef_search"`
	HnswSearchMs        float64 `json:"hnsw_search_ms"`
	PayloadFilterEvals  int     `json:"payload_filter_evals"`
	QuantizationScoreMs float64 `json:"quantization_score_ms"`
	TotalMs             float64 `json:"total_ms"`
}

// ExplainResponse represents the response from
// POST /collections/{name}/explain.
type ExplainResponse struct {
	Collection string                   `json:"collection"`
	K          int                      `json:"k"`
	Results    []map[string]interface{} `json:"results"`
	Trace      ExplainTrace             `json:"trace"`
}

// ===== Cluster + Auth =====

// FailoverReport represents the result of POST /cluster/failover.
type FailoverReport struct {
	PromotedReplicaID        string `json:"promoted_replica_id"`
	MasterOffsetAtPromotion  int64  `json:"master_offset_at_promotion"`
	ReplicaOffsetAtPromotion int64  `json:"replica_offset_at_promotion"`
	ResidualLagOperations    int64  `json:"residual_lag_operations"`
}

// ResyncJob represents the result of
// POST /cluster/replicas/{id}/resync.
type ResyncJob struct {
	ReplicaID      string `json:"replica_id"`
	SnapshotOffset int64  `json:"snapshot_offset"`
	FullSnapshot   bool   `json:"full_snapshot"`
}

// PeerInfo represents a cluster peer returned by POST /cluster/peers.
type PeerInfo struct {
	NodeID  string `json:"node_id"`
	Address string `json:"address"`
	Role    string `json:"role"`
}

// AddPeerRequest represents the request body for POST /cluster/peers.
type AddPeerRequest struct {
	Address string `json:"address"`
	Role    string `json:"role,omitempty"`
}

// RebalanceJob represents the rebalance job descriptor returned by
// POST /cluster/rebalance.
type RebalanceJob struct {
	JobID              string  `json:"job_id"`
	Status             string  `json:"status"`
	ShardsToMove       int     `json:"shards_to_move"`
	ShardsMoved        int     `json:"shards_moved"`
	LastCheckpointNode *string `json:"last_checkpoint_node,omitempty"`
	Message            string  `json:"message"`
}

// RotatedKey represents the result of POST /auth/keys/{id}/rotate.
type RotatedKey struct {
	OldKeyID   string `json:"old_key_id"`
	NewKeyID   string `json:"new_key_id"`
	NewToken   string `json:"new_token"`
	GraceUntil int64  `json:"grace_until"`
}

// TokenIntrospection represents the RFC 7662 introspection response from
// POST /auth/introspect.
type TokenIntrospection struct {
	Active   bool    `json:"active"`
	Scope    *string `json:"scope,omitempty"`
	Sub      *string `json:"sub,omitempty"`
	Exp      *int64  `json:"exp,omitempty"`
	Username *string `json:"username,omitempty"`
}

// AuditEntry represents one entry in the admin audit log.
type AuditEntry struct {
	Actor         string  `json:"actor"`
	Action        string  `json:"action"`
	Target        string  `json:"target"`
	At            string  `json:"at"`
	CorrelationID *string `json:"correlation_id,omitempty"`
}

// ApiKeyScope represents a per-collection permission scope on an API key.
type ApiKeyScope struct {
	Collection  string   `json:"collection"`
	Permissions []string `json:"permissions"`
}

// ApiKey represents an API key returned by POST /auth/keys or GET /auth/keys.
type ApiKey struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	Permissions []string `json:"permissions"`
	ApiKeyValue *string  `json:"api_key,omitempty"`
	CreatedAt   int64    `json:"created_at"`
	ExpiresAt   *int64   `json:"expires_at,omitempty"`
	Active      bool     `json:"active"`
	Warning     *string  `json:"warning,omitempty"`
	UsageCount  int64    `json:"usage_count"`
}

// ApiKeyView represents the flattened key view returned by
// PUT /auth/keys/{id}/permissions and GET /auth/keys/{id}/usage.
type ApiKeyView struct {
	ID          string        `json:"id"`
	Name        string        `json:"name"`
	UserID      string        `json:"user_id"`
	Permissions []string      `json:"permissions"`
	Scopes      []ApiKeyScope `json:"scopes"`
	CreatedAt   int64         `json:"created_at"`
	LastUsed    *int64        `json:"last_used,omitempty"`
	ExpiresAt   *int64        `json:"expires_at,omitempty"`
	Active      bool          `json:"active"`
	UsageCount  int64         `json:"usage_count"`
}

// ApiKeyUsageBucket represents one day's usage bucket from
// GET /auth/keys/{id}/usage.
type ApiKeyUsageBucket struct {
	Date  string `json:"date"`
	Count int64  `json:"count"`
}

// ApiKeyUsageReport represents the response from
// GET /auth/keys/{id}/usage.
type ApiKeyUsageReport struct {
	Key         ApiKeyView          `json:"key"`
	Buckets     []ApiKeyUsageBucket `json:"buckets"`
	WindowTotal int64               `json:"window_total"`
}

// User represents a user record returned by auth endpoints.
type User struct {
	UserID   string   `json:"user_id"`
	Username string   `json:"username"`
	Roles    []string `json:"roles"`
}

// JwtToken represents a JWT token returned by POST /auth/refresh.
type JwtToken struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
	ExpiresIn   int64  `json:"expires_in"`
}

// PasswordPolicyReport represents the password policy report returned by
// POST /auth/validate-password.
type PasswordPolicyReport struct {
	Valid         bool     `json:"valid"`
	Errors        []string `json:"errors"`
	Strength      int      `json:"strength"`
	StrengthLabel string   `json:"strength_label"`
}

// CreateApiKeyRequest represents the request body for POST /auth/keys.
type CreateApiKeyRequest struct {
	Name        string   `json:"name"`
	Permissions []string `json:"permissions,omitempty"`
	ExpiresIn   *int64   `json:"expires_in,omitempty"`
}

// TokenScope represents a per-collection permission scope in a scoped key request.
type TokenScope struct {
	Collection  string   `json:"collection"`
	Permissions []string `json:"permissions"`
}

// CreateScopedApiKeyRequest represents the request body for
// POST /auth/keys with per-collection scopes.
type CreateScopedApiKeyRequest struct {
	Name        string       `json:"name"`
	Permissions []string     `json:"permissions,omitempty"`
	ExpiresIn   *int64       `json:"expires_in,omitempty"`
	Scopes      []TokenScope `json:"scopes,omitempty"`
}

// UpdateApiKeyPermissionsRequest represents the request body for
// PUT /auth/keys/{id}/permissions.
type UpdateApiKeyPermissionsRequest struct {
	Permissions []string      `json:"permissions"`
	Scopes      []ApiKeyScope `json:"scopes,omitempty"`
}

// CreateUserRequest represents the request body for POST /auth/users.
type CreateUserRequest struct {
	Username string   `json:"username"`
	Password string   `json:"password"`
	Roles    []string `json:"roles,omitempty"`
}

// ===== Replication =====

// ReplicationStatus represents the node's replication role and state from
// GET /replication/status.
type ReplicationStatus struct {
	Role     string            `json:"role"`
	Enabled  bool              `json:"enabled"`
	Stats    *ReplicationStats `json:"stats,omitempty"`
	Replicas []ReplicaInfo     `json:"replicas,omitempty"`
}

// ReplicationConfig represents the request body for
// POST /replication/configure.
type ReplicationConfig struct {
	Role              string  `json:"role"`
	BindAddress       *string `json:"bind_address,omitempty"`
	MasterAddress     *string `json:"master_address,omitempty"`
	HeartbeatInterval *int64  `json:"heartbeat_interval,omitempty"`
	LogSize           *int    `json:"log_size,omitempty"`
}

// ReplicationStats represents raw replication statistics from
// GET /replication/stats.
type ReplicationStats struct {
	Role              *string `json:"role,omitempty"`
	BytesSent         *int64  `json:"bytes_sent,omitempty"`
	BytesReceived     *int64  `json:"bytes_received,omitempty"`
	LastSync          *string `json:"last_sync,omitempty"`
	OperationsPending *int    `json:"operations_pending,omitempty"`
	SnapshotSize      *int    `json:"snapshot_size,omitempty"`
	ConnectedReplicas *int    `json:"connected_replicas,omitempty"`
	MasterOffset      int64   `json:"master_offset"`
	ReplicaOffset     int64   `json:"replica_offset"`
	LagOperations     int64   `json:"lag_operations"`
	TotalReplicated   int64   `json:"total_replicated"`
}

// ReplicaInfo represents information about a replica node from
// GET /replication/replicas.
type ReplicaInfo struct {
	ReplicaID        string `json:"replica_id"`
	Host             string `json:"host"`
	Port             int    `json:"port"`
	Status           string `json:"status"`
	LastHeartbeat    string `json:"last_heartbeat"`
	OperationsSynced int64  `json:"operations_synced"`
	Offset           *int64 `json:"offset,omitempty"`
	Lag              *int64 `json:"lag,omitempty"`
}

// ===== HiveHub =====

// UserBackup represents a user-scoped backup entry from GET /hub/backups.
type UserBackup struct {
	ID          string   `json:"id"`
	UserID      string   `json:"user_id"`
	Name        string   `json:"name"`
	Description *string  `json:"description,omitempty"`
	Collections []string `json:"collections"`
	CreatedAt   string   `json:"created_at"`
	Size        int64    `json:"size"`
	Status      string   `json:"status"`
}

// CreateUserBackupRequest represents the request body for POST /hub/backups.
type CreateUserBackupRequest struct {
	UserID      string   `json:"user_id"`
	Name        string   `json:"name"`
	Description *string  `json:"description,omitempty"`
	Collections []string `json:"collections,omitempty"`
}

// RestoreUserBackupRequest represents the request body for
// POST /hub/backups/restore.
type RestoreUserBackupRequest struct {
	UserID    string `json:"user_id"`
	BackupID  string `json:"backup_id"`
	Overwrite *bool  `json:"overwrite,omitempty"`
}

// UsageStatistics represents usage statistics from GET /hub/usage/statistics.
type UsageStatistics struct {
	Success bool                   `json:"success"`
	Message string                 `json:"message"`
	Stats   map[string]interface{} `json:"stats,omitempty"`
}

// QuotaInfo represents quota information from GET /hub/usage/quota.
type QuotaInfo struct {
	Success bool                   `json:"success"`
	Message string                 `json:"message"`
	Quota   map[string]interface{} `json:"quota,omitempty"`
}

// HubApiKeyValidation represents the result of POST /hub/validate-key.
type HubApiKeyValidation struct {
	Valid       bool     `json:"valid"`
	TenantID    string   `json:"tenant_id"`
	TenantName  string   `json:"tenant_name"`
	Permissions []string `json:"permissions"`
	ValidatedAt string   `json:"validated_at"`
}

// ===== Discovery Pipeline =====

// BroadDiscoveryRequest represents the request body for
// POST /discovery/broad_discovery.
type BroadDiscoveryRequest struct {
	Queries []string `json:"queries"`
	K       *int     `json:"k,omitempty"`
}

// BroadDiscoveryResponse represents the response from broad_discovery.
type BroadDiscoveryResponse struct {
	Chunks []map[string]interface{} `json:"chunks"`
	Count  int                      `json:"count"`
}

// SemanticFocusRequest represents the request body for
// POST /discovery/semantic_focus.
type SemanticFocusRequest struct {
	Collection string   `json:"collection"`
	Queries    []string `json:"queries"`
	K          *int     `json:"k,omitempty"`
}

// SemanticFocusResponse represents the response from semantic_focus.
type SemanticFocusResponse struct {
	Chunks []map[string]interface{} `json:"chunks"`
	Count  int                      `json:"count"`
}

// PromoteReadmeRequest represents the request body for
// POST /discovery/promote_readme.
type PromoteReadmeRequest struct {
	Chunks []map[string]interface{} `json:"chunks"`
}

// PromoteReadmeResponse represents the response from promote_readme.
type PromoteReadmeResponse struct {
	PromotedChunks []map[string]interface{} `json:"promoted_chunks"`
	Count          int                      `json:"count"`
}

// CompressEvidenceRequest represents the request body for
// POST /discovery/compress_evidence.
type CompressEvidenceRequest struct {
	Chunks     []map[string]interface{} `json:"chunks"`
	MaxBullets *int                     `json:"max_bullets,omitempty"`
	MaxPerDoc  *int                     `json:"max_per_doc,omitempty"`
}

// CompressEvidenceResponse represents the response from compress_evidence.
type CompressEvidenceResponse struct {
	Bullets []map[string]interface{} `json:"bullets"`
	Count   int                      `json:"count"`
}

// AnswerPlanRequest represents the request body for
// POST /discovery/build_answer_plan.
type AnswerPlanRequest struct {
	Bullets []map[string]interface{} `json:"bullets"`
}

// AnswerPlan represents the structured answer plan returned by build_answer_plan.
type AnswerPlan struct {
	Sections     []map[string]interface{} `json:"sections"`
	TotalBullets int                      `json:"total_bullets"`
	Sources      []string                 `json:"sources"`
}

// RenderPromptRequest represents the request body for
// POST /discovery/render_llm_prompt.
type RenderPromptRequest struct {
	Plan AnswerPlan `json:"plan"`
}

// LlmPrompt represents the rendered LLM prompt returned by render_llm_prompt.
type LlmPrompt struct {
	Prompt          string `json:"prompt"`
	Length          int    `json:"length"`
	EstimatedTokens int    `json:"estimated_tokens"`
}
