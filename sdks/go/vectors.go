package vectorizer

import (
	"encoding/json"
	"fmt"
	"net/url"
)

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

// DeleteVectors deletes a batch of vectors from a single collection.
//
// Calls POST /batch_delete with {"collection", "ids"}. Per-id failures
// (e.g. not-found) populate DeleteReport.Results without aborting the
// batch. Companion to MoveToCollection for tier-demotion (issue #265).
func (c *Client) DeleteVectors(collection string, ids []string) (*DeleteReport, error) {
	body := map[string]interface{}{
		"collection": collection,
		"ids":        ids,
	}
	var report DeleteReport
	if err := c.request("POST", "/batch_delete", body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// MoveToCollection moves vectors from src to dst without re-embedding
// (issue #265).
//
// Calls POST /collections/{src}/vectors/move with {"destination", "ids"}.
// Server invariant: the destination insert lands BEFORE the source
// delete, so a mid-batch crash leaves a recoverable duplicate (never
// data loss). Per-id outcomes (ok, missing_in_src, dst_insert_failed,
// src_delete_failed) populate MoveReport.Results without aborting the
// batch. Typical use: tier-demotion pruner that walks a hot collection
// and relocates aged vectors to a warm/cold collection.
func (c *Client) MoveToCollection(src, dst string, ids []string) (*MoveReport, error) {
	body := map[string]interface{}{
		"destination": dst,
		"ids":         ids,
	}
	path := "/collections/" + url.PathEscape(src) + "/vectors/move"
	var report MoveReport
	if err := c.request("POST", path, body, &report); err != nil {
		return nil, err
	}
	return &report, nil
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

// UpdateVectorPayload updates a vector's metadata payload in-place.
//
// Calls POST /update with {"collection", "id", "metadata": req}.
// This is distinct from UpdateVector (PUT /collections/{c}/vectors/{id}) which
// replaces the full Vector struct including the dense embedding.
func (c *Client) UpdateVectorPayload(collection, id string, req map[string]interface{}) (*Vector, error) {
	body := map[string]interface{}{
		"collection": collection,
		"id":         id,
		"metadata":   req,
	}
	// The server returns {message} — synthesise a minimal Vector from the request parameters.
	if err := c.request("POST", "/update", body, nil); err != nil {
		return nil, err
	}
	return &Vector{ID: id}, nil
}

// InsertTextWithID inserts a single text document with an explicit client-supplied ID.
//
// Calls POST /insert with {"collection", "id", "text", "metadata"?}.
// Unlike InsertText, the caller controls the id sent to the server (the server
// may still reassign a UUID; the returned Vector carries the server-assigned id).
func (c *Client) InsertTextWithID(collection, id, text string, metadata map[string]interface{}) (*Vector, error) {
	body := map[string]interface{}{
		"collection": collection,
		"id":         id,
		"text":       text,
	}
	if metadata != nil {
		body["metadata"] = metadata
	}
	var resp InsertTextResponse
	if err := c.request("POST", "/insert", body, &resp); err != nil {
		return nil, err
	}
	assignedID := resp.VectorID
	if assignedID == "" {
		assignedID = id
	}
	return &Vector{ID: assignedID}, nil
}

// ListVectors returns a paginated list of vectors in a collection.
//
// Calls GET /collections/{name}/vectors?limit=&offset=.
func (c *Client) ListVectors(collection string, limit, offset int) (*VectorPage, error) {
	q := url.Values{}
	q.Set("limit", fmt.Sprintf("%d", limit))
	q.Set("offset", fmt.Sprintf("%d", offset))
	path := "/collections/" + url.PathEscape(collection) + "/vectors?" + q.Encode()
	var page VectorPage
	if err := c.request("GET", path, nil, &page); err != nil {
		return nil, err
	}
	return &page, nil
}

// BatchInsertTexts batch-inserts text documents via the /batch_insert endpoint.
//
// Calls POST /batch_insert with {"collection", "texts": items}.
// Each item is a free-form map (e.g. {"text": "...", "id": "...", "metadata": {...}}).
// This is distinct from BatchInsert which targets /insert_texts with typed BatchInsertTextEntry items.
func (c *Client) BatchInsertTexts(collection string, items []map[string]interface{}) (*BatchInsertReport, error) {
	body := map[string]interface{}{
		"collection": collection,
		"texts":      items,
	}
	var report BatchInsertReport
	if err := c.request("POST", "/batch_insert", body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// InsertVectors bulk-inserts pre-computed embeddings, skipping server-side embedding.
//
// Calls POST /insert_vectors with {"collection", "vectors": vectors}.
// Use this when you already have raw float32 embeddings and want to bypass the
// server's embedding pipeline.
func (c *Client) InsertVectors(collection string, vectors []Vector) (*BatchInsertReport, error) {
	body := map[string]interface{}{
		"collection": collection,
		"vectors":    vectors,
	}
	var report BatchInsertReport
	if err := c.request("POST", "/insert_vectors", body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// BatchSearchQueries runs multiple search queries in a single round-trip.
//
// Calls POST /batch_search with {"collection", "queries": queries}.
// Each query is a free-form map that may carry either a "query" text string
// (embedded server-side) or a raw "vector". Returns one SearchResponse per query.
// This is distinct from BatchSearch which accepts typed string-only queries via BatchSearchRequest.
func (c *Client) BatchSearchQueries(collection string, queries []map[string]interface{}) ([]SearchResponse, error) {
	body := map[string]interface{}{
		"collection": collection,
		"queries":    queries,
	}
	var raw map[string]interface{}
	if err := c.request("POST", "/batch_search", body, &raw); err != nil {
		return nil, err
	}
	// Server returns {collection, count, succeeded, failed, results: [...]}
	resultsRaw, _ := raw["results"].([]interface{})
	out := make([]SearchResponse, 0, len(resultsRaw))
	for _, entry := range resultsRaw {
		entryMap, ok := entry.(map[string]interface{})
		if !ok {
			continue
		}
		b, err := json.Marshal(entryMap)
		if err != nil {
			return nil, fmt.Errorf("marshal batch_search entry: %w", err)
		}
		var sr SearchResponse
		if err := json.Unmarshal(b, &sr); err != nil {
			return nil, fmt.Errorf("parse batch_search entry: %w", err)
		}
		out = append(out, sr)
	}
	return out, nil
}

// BatchUpdateVectors batch-updates vector payloads and/or dense vectors.
//
// Calls POST /batch_update with {"collection", "updates": updates}.
// Each update map should carry at minimum an "id" key plus the fields to patch.
func (c *Client) BatchUpdateVectors(collection string, updates []map[string]interface{}) (*BatchUpdateReport, error) {
	body := map[string]interface{}{
		"collection": collection,
		"updates":    updates,
	}
	var report BatchUpdateReport
	if err := c.request("POST", "/batch_update", body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// SearchByText searches a collection using a plain text query with a simple limit.
//
// Calls POST /collections/{name}/search/text with {"query", "limit"}.
// Unlike SearchText, no filter or payload options are accepted — use SearchText
// for full control. Returns the raw SearchResponse (including aggregate metadata).
func (c *Client) SearchByText(collection, query string, limit int) (*SearchResponse, error) {
	body := map[string]interface{}{
		"query": query,
		"limit": limit,
	}
	var resp SearchResponse
	if err := c.request("POST", "/collections/"+url.PathEscape(collection)+"/search/text", body, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

// SearchByFile searches a collection for vectors associated with a given file path.
//
// Calls POST /collections/{name}/search/file with {"file_path", "limit"}.
// Returns a SearchResponse that may be empty if the file has not been indexed.
func (c *Client) SearchByFile(collection, filePath string, limit int) (*SearchResponse, error) {
	body := map[string]interface{}{
		"file_path": filePath,
		"limit":     limit,
	}
	var resp SearchResponse
	if err := c.request("POST", "/collections/"+url.PathEscape(collection)+"/search/file", body, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}
