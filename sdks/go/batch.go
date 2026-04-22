package vectorizer

// BatchInsertTextEntry is one entry in a `/insert_texts` batch. The
// server embeds `Text` with the collection's configured provider and
// reassigns a server-generated UUID; the caller's `ID` is returned on
// the response as `client_id` for idempotency tracking.
type BatchInsertTextEntry struct {
	ID       string                 `json:"id"`
	Text     string                 `json:"text"`
	Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// BatchInsertRequest is the `/insert_texts` payload. `Collection` is a
// top-level field alongside `Texts`, not a path segment — the earlier
// `POST /collections/{c}/batch/insert` path this SDK used was never
// served by the v3.0.x server.
type BatchInsertRequest struct {
	Collection string                 `json:"collection"`
	Texts      []BatchInsertTextEntry `json:"texts"`
}

// BatchInsertResult captures a single entry's outcome on
// `/insert_texts`. `ClientID` is the caller-supplied id; `VectorIDs`
// holds the server-assigned UUID(s) (more than one when the server
// chunked a long text).
type BatchInsertResult struct {
	ClientID       string   `json:"client_id"`
	Index          int      `json:"index"`
	Status         string   `json:"status"`
	Chunked        bool     `json:"chunked"`
	VectorIDs      []string `json:"vector_ids"`
	VectorsCreated int      `json:"vectors_created"`
	Error          string   `json:"error,omitempty"`
}

// BatchInsertResponse is the `/insert_texts` response shape.
type BatchInsertResponse struct {
	Collection string              `json:"collection"`
	Count      int                 `json:"count"`
	Inserted   int                 `json:"inserted"`
	Failed     int                 `json:"failed"`
	Results    []BatchInsertResult `json:"results"`
}

// BatchInsert performs batch text insertion. The server embeds each
// entry with the collection's configured provider.
func (c *Client) BatchInsert(collection string, req *BatchInsertRequest) (*BatchInsertResponse, error) {
	// Ensure `Collection` on the request body matches the caller's
	// explicit `collection` argument even if the struct was zero-valued.
	if req.Collection == "" {
		req.Collection = collection
	}
	var resp BatchInsertResponse
	if err := c.request("POST", "/insert_texts", req, &resp); err != nil {
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

// BatchSearch performs batch search against `/batch_search`. The
// previous `/collections/{c}/batch/search` path was never served by
// the v3.0.x server.
func (c *Client) BatchSearch(collection string, req *BatchSearchRequest) (*BatchSearchResponse, error) {
	var resp BatchSearchResponse
	// Attach the collection in the body since `/batch_search` doesn't
	// encode it in the path.
	body := struct {
		Collection string                 `json:"collection"`
		Queries    []string               `json:"queries"`
		Limit      int                    `json:"limit,omitempty"`
		Filter     map[string]interface{} `json:"filter,omitempty"`
	}{
		Collection: collection,
		Queries:    req.Queries,
		Limit:      req.Limit,
		Filter:     req.Filter,
	}
	if err := c.request("POST", "/batch_search", body, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}
