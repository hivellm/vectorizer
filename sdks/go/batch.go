package vectorizer

// BatchInsertRequest represents a batch insert request
type BatchInsertRequest struct {
	Texts   []string               `json:"texts"`
	Payload []map[string]interface{} `json:"payload,omitempty"`
}

// BatchInsertResponse represents a batch insert response
type BatchInsertResponse struct {
	Inserted int      `json:"inserted"`
	IDs      []string `json:"ids"`
}

// BatchInsert performs batch insertion
func (c *Client) BatchInsert(collection string, req *BatchInsertRequest) (*BatchInsertResponse, error) {
	var resp BatchInsertResponse
	if err := c.request("POST", "/collections/"+collection+"/batch/insert", req, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

// BatchSearchRequest represents a batch search request
type BatchSearchRequest struct {
	Queries []string               `json:"queries"`
	Limit   int                    `json:"limit,omitempty"`
	Filter  map[string]interface{} `json:"filter,omitempty"`
}

// BatchSearchResponse represents a batch search response
type BatchSearchResponse struct {
	Results [][]SearchResult `json:"results"`
}

// BatchSearch performs batch search
func (c *Client) BatchSearch(collection string, req *BatchSearchRequest) (*BatchSearchResponse, error) {
	var resp BatchSearchResponse
	if err := c.request("POST", "/collections/"+collection+"/batch/search", req, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

