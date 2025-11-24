package vectorizer

// Metric represents distance metric
type Metric string

const (
	MetricCosine     Metric = "Cosine"
	MetricEuclidean  Metric = "Euclidean"
	MetricDotProduct Metric = "DotProduct"
)

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
	ID      string                 `json:"id"`
	Data    []float32              `json:"data"`
	Payload map[string]interface{} `json:"payload,omitempty"`
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

// CollectionInfo represents collection information
type CollectionInfo struct {
	Name        string `json:"name"`
	VectorCount int    `json:"vector_count"`
	Dimension   int    `json:"dimension"`
	Metric      string `json:"metric"`
}

// CollectionsListResponse represents the response from listing collections
type CollectionsListResponse struct {
	Collections     []CollectionInfo `json:"collections"`
	TotalCollections int             `json:"total_collections"`
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
	ID              string                 `json:"id"`
	Source          string                 `json:"source"`
	Target          string                 `json:"target"`
	RelationshipType string                `json:"relationship_type"`
	Weight          float32                `json:"weight"`
	Metadata        map[string]interface{} `json:"metadata"`
	CreatedAt       string                 `json:"created_at"`
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
	TotalNodes        int     `json:"total_nodes"`
	NodesWithEdges    int     `json:"nodes_with_edges"`
	TotalEdges        int     `json:"total_edges"`
	ProgressPercentage float64 `json:"progress_percentage"`
}
