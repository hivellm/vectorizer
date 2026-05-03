package rpc

// Phase 16 response types. Mirrors sdks/rust/src/rpc/commands.rs
// and sdks/typescript/src/rpc/commands.ts.

// ── Collections ───────────────────────────────────────────────────

// CreateCollectionResult is returned by collections.create.
type CreateCollectionResult struct {
	Name      string
	Dimension int64
	Metric    string
	Success   bool
}

// CleanupEmptyResult is returned by collections.cleanup_empty.
type CleanupEmptyResult struct {
	Removed int64
	DryRun  bool
}

// ── Vectors ───────────────────────────────────────────────────────

// VectorWriteResult is returned by vectors.insert, vectors.insert_text,
// and vectors.update.
type VectorWriteResult struct {
	ID      string
	Success bool
}

// BatchItemResult is one per-item result inside batch operation responses.
type BatchItemResult struct {
	Index  int64
	ID     *string
	Status string
	Error  *string
}

// BatchInsertResult is returned by vectors.batch_insert and
// vectors.batch_insert_texts.
type BatchInsertResult struct {
	Inserted int64
	Failed   int64
	Results  []BatchItemResult
}

// BatchUpdateResult is returned by vectors.batch_update.
type BatchUpdateResult struct {
	Updated int64
	Failed  int64
	Results []BatchItemResult
}

// BatchDeleteResult is returned by vectors.batch_delete.
type BatchDeleteResult struct {
	Deleted int64
	Failed  int64
	Results []BatchItemResult
}

// BatchSearchResult is one per-query result from vectors.batch_search.
type BatchSearchResult struct {
	Index   int64
	Status  string
	Results []SearchHit
	Error   *string
}

// MoveVectorsResult is returned by vectors.move.
type MoveVectorsResult struct {
	Src    string
	Dst    string
	Moved  int64
	Failed int64
}

// CopyVectorsResult is returned by vectors.copy.
type CopyVectorsResult struct {
	Src    string
	Dst    string
	Copied int64
	Failed int64
}

// DeleteByFilterResult is returned by vectors.delete_by_filter.
type DeleteByFilterResult struct {
	Scanned int64
	Matched int64
	Deleted int64
}

// BulkUpdateMetadataResult is returned by vectors.bulk_update_metadata.
type BulkUpdateMetadataResult struct {
	Scanned int64
	Matched int64
	Updated int64
}

// SetExpiryResult is returned by vectors.set_expiry.
type SetExpiryResult struct {
	ID        string
	ExpiresAt int64
	Success   bool
}

// EmbedResult is returned by vectors.embed.
type EmbedResult struct {
	Embedding []float64
	Model     string
	Dimension int64
}

// VectorListResult is returned by vectors.list.
type VectorListResult struct {
	Items []VectorizerValue
	Total int64
	Page  int64
	Limit int64
}

// ── Search ────────────────────────────────────────────────────────

// SearchTrace is the HNSW traversal trace from search.explain.
type SearchTrace struct {
	VisitedNodes int64
	EfSearch     int64
	HnswSearchMs float64
	TotalMs      float64
}

// SearchExplainResult is returned by search.explain.
type SearchExplainResult struct {
	Hits       []SearchHit
	Collection string
	K          int64
	Trace      SearchTrace
}

// ── Discovery ─────────────────────────────────────────────────────

// DiscoverResult is returned by discovery.discover.
type DiscoverResult struct {
	AnswerPrompt string
	Sections     int64
	Bullets      int64
	Chunks       int64
}

// ScoredCollection is one entry from discovery.score_collections.
type ScoredCollection struct {
	Name        string
	Score       float64
	VectorCount int64
}

// ExpandQueriesResult is returned by discovery.expand_queries.
type ExpandQueriesResult struct {
	OriginalQuery   string
	ExpandedQueries []string
	Count           int64
}

// DiscoveryChunk is one item from discovery.broad_discovery and
// discovery.semantic_focus.
type DiscoveryChunk struct {
	Collection     string
	Score          float64
	ContentPreview string
}

// CompressBullet is one bullet from discovery.compress_evidence.
type CompressBullet struct {
	Text     string
	SourceID string
	Score    float64
}

// AnswerPlanSection is one section inside an answer plan.
type AnswerPlanSection struct {
	Title        string
	BulletsCount int64
}

// AnswerPlanResult is returned by discovery.build_answer_plan.
type AnswerPlanResult struct {
	Sections     []AnswerPlanSection
	TotalBullets int64
}

// RenderPromptResult is returned by discovery.render_llm_prompt.
type RenderPromptResult struct {
	Prompt          string
	Length          int64
	EstimatedTokens int64
}

// ── Graph ─────────────────────────────────────────────────────────

// GraphDiscoveryStatus is returned by graph.discovery_status.
type GraphDiscoveryStatus struct {
	TotalNodes         int64
	NodesWithEdges     int64
	TotalEdges         int64
	ProgressPercentage float64
}

// DiscoverEdgesResult is returned by graph.discover_edges.
type DiscoverEdgesResult struct {
	Success           bool
	TotalNodes        int64
	NodesProcessed    int64
	NodesWithEdges    int64
	TotalEdgesCreated int64
}

// DiscoverEdgesForNodeResult is returned by graph.discover_edges_for_node.
type DiscoverEdgesForNodeResult struct {
	Success      bool
	NodeID       string
	EdgesCreated int64
}

// ── Admin ─────────────────────────────────────────────────────────

// AdminStats is returned by admin.stats.
type AdminStats struct {
	CollectionsCount int64
	TotalVectors     int64
	Version          string
}

// AdminStatus is returned by admin.status.
type AdminStatus struct {
	Ready            bool
	CollectionsCount int64
	Version          string
}

// SlowQueryConfigResult is returned by admin.slow_queries_config.
type SlowQueryConfigResult struct {
	ThresholdMs int64
	Capacity    int64
	Status      string
}

// ── Auth ──────────────────────────────────────────────────────────

// AuthMeResult is returned by auth.me.
type AuthMeResult struct {
	Username      string
	Authenticated bool
}

// RefreshTokenResult is returned by auth.refresh_token.
type RefreshTokenResult struct {
	AccessToken string
	TokenType   string
}

// ValidatePasswordResult is returned by auth.validate_password.
type ValidatePasswordResult struct {
	Valid  bool
	Errors []string
}

// ApiKeyCreated is returned by auth.api_keys_create and
// auth.api_keys_create_scoped.
type ApiKeyCreated struct {
	APIKey string
	ID     string
	Name   string
}

// RotatedApiKey is returned by auth.api_keys_rotate.
type RotatedApiKey struct {
	OldKeyID   string
	NewKeyID   string
	NewToken   string
	GraceUntil *string
}

// ── Replication ───────────────────────────────────────────────────

// ReplicationConfigureResult is returned by replication.configure.
type ReplicationConfigureResult struct {
	Success bool
	Role    string
	Message string
}

// ── Cluster ───────────────────────────────────────────────────────

// RebalanceStatus is returned by cluster.rebalance_status.
type RebalanceStatus struct {
	Status  *string
	Message *string
}
