package vectorizer

// IntelligentSearchRequest represents an intelligent search request
type IntelligentSearchRequest struct {
	Query           string   `json:"query"`
	Collections     []string `json:"collections"`
	MaxResults      int      `json:"max_results,omitempty"`
	MMREnabled      bool     `json:"mmr_enabled,omitempty"`
	DomainExpansion bool     `json:"domain_expansion,omitempty"`
	TechnicalFocus  bool     `json:"technical_focus,omitempty"`
	MMRLambda       float64  `json:"mmr_lambda,omitempty"`
}

// IntelligentSearchResult represents an intelligent search result
type IntelligentSearchResult struct {
	ID         string                 `json:"id"`
	Score      float64                `json:"score"`
	Payload    map[string]interface{} `json:"payload,omitempty"`
	Vector     []float32              `json:"vector,omitempty"`
	Collection string                 `json:"collection,omitempty"`
}

// IntelligentSearch performs an intelligent search
func (c *Client) IntelligentSearch(req *IntelligentSearchRequest) ([]IntelligentSearchResult, error) {
	var results []IntelligentSearchResult
	if err := c.request("POST", "/intelligent_search", req, &results); err != nil {
		return nil, err
	}
	return results, nil
}

// SemanticSearchRequest represents a semantic search request
type SemanticSearchRequest struct {
	Collection          string  `json:"collection"`
	Query               string  `json:"query"`
	MaxResults          int     `json:"max_results,omitempty"`
	SemanticReranking   bool    `json:"semantic_reranking,omitempty"`
	SimilarityThreshold float64 `json:"similarity_threshold,omitempty"`
}

// SemanticSearch performs a semantic search
func (c *Client) SemanticSearch(req *SemanticSearchRequest) ([]SearchResult, error) {
	var results []SearchResult
	if err := c.request("POST", "/semantic_search", req, &results); err != nil {
		return nil, err
	}
	return results, nil
}
