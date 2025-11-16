package vectorizer

// Metric represents distance metric
type Metric string

const (
	MetricCosine     Metric = "Cosine"
	MetricEuclidean Metric = "Euclidean"
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
	ID       string                 `json:"id"`
	Score    float64                `json:"score"`
	Payload  map[string]interface{} `json:"payload,omitempty"`
	Vector   []float32              `json:"vector,omitempty"`
}

// InsertTextRequest represents a request to insert text
type InsertTextRequest struct {
	Text    string                 `json:"text"`
	Payload map[string]interface{} `json:"payload,omitempty"`
}

// InsertTextResponse represents a response from inserting text
type InsertTextResponse struct {
	ID string `json:"id"`
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

