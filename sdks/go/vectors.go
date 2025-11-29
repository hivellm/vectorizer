package vectorizer

// InsertText inserts text into a collection (with automatic embedding)
func (c *Client) InsertText(collection, text string, payload map[string]interface{}) (*InsertTextResponse, error) {
	req := map[string]interface{}{
		"collection": collection,
		"text":       text,
	}
	if payload != nil {
		req["metadata"] = payload
	}
	var resp InsertTextResponse
	if err := c.request("POST", "/insert", req, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

// GetVector retrieves a vector by ID
func (c *Client) GetVector(collection, id string) (*Vector, error) {
	var vector Vector
	if err := c.request("GET", "/collections/"+collection+"/vectors/"+id, nil, &vector); err != nil {
		return nil, err
	}
	return &vector, nil
}

// UpdateVector updates a vector
func (c *Client) UpdateVector(collection, id string, vector *Vector) error {
	return c.request("PUT", "/collections/"+collection+"/vectors/"+id, vector, nil)
}

// DeleteVector deletes a vector
func (c *Client) DeleteVector(collection, id string) error {
	return c.request("DELETE", "/collections/"+collection+"/vectors/"+id, nil, nil)
}

// Search performs a vector search
func (c *Client) Search(collection string, query []float32, options *SearchOptions) ([]SearchResult, error) {
	req := map[string]interface{}{
		"vector": query,
	}
	if options != nil {
		if options.Limit > 0 {
			req["limit"] = options.Limit
		}
		if options.Filter != nil {
			req["filter"] = options.Filter
		}
		if options.Payload != nil {
			req["payload"] = options.Payload
		}
	}

	var response SearchResponse
	if err := c.request("POST", "/collections/"+collection+"/search", req, &response); err != nil {
		return nil, err
	}
	return response.Results, nil
}

// SearchText performs a text search (with automatic embedding)
func (c *Client) SearchText(collection, query string, options *SearchOptions) ([]SearchResult, error) {
	req := map[string]interface{}{
		"query": query,
	}
	if options != nil {
		if options.Limit > 0 {
			req["limit"] = options.Limit
		}
		if options.Filter != nil {
			req["filter"] = options.Filter
		}
		if options.Payload != nil {
			req["payload"] = options.Payload
		}
	}

	var response SearchResponse
	if err := c.request("POST", "/collections/"+collection+"/search/text", req, &response); err != nil {
		return nil, err
	}
	return response.Results, nil
}
