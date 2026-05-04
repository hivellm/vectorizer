package vectorizer

import "net/url"

// DeleteByFilter deletes every vector in a collection that matches the given
// metadata filter.
//
// Calls POST /collections/{name}/vectors/delete_by_filter with body
// {"filter": filter}. An empty filter is rejected client-side to prevent
// accidental full-collection wipes.
//
// Response fields: Scanned, Matched, Deleted, Results.
func (c *Client) DeleteByFilter(collection string, filter map[string]interface{}) (*DeleteByFilterReport, error) {
	if len(filter) == 0 {
		return nil, &VectorizerError{
			Type:    "validation_error",
			Message: "filter must not be empty",
			Status:  0,
		}
	}
	body := map[string]interface{}{"filter": filter}
	path := "/collections/" + url.PathEscape(collection) + "/vectors/delete_by_filter"
	var report DeleteByFilterReport
	if err := c.request("POST", path, body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// BulkUpdateMetadata applies a JSON-merge-patch to every vector matching the
// given filter.
//
// Calls POST /collections/{name}/vectors/bulk_update_metadata with body
// {"filter": filter, "patch": patch}. An empty filter is rejected client-side
// to prevent accidental full-collection updates.
//
// Patch semantics follow RFC 7396: keys in patch overwrite existing payload
// values; null values remove keys.
func (c *Client) BulkUpdateMetadata(collection string, filter, patch map[string]interface{}) (*BulkUpdateReport, error) {
	if len(filter) == 0 {
		return nil, &VectorizerError{
			Type:    "validation_error",
			Message: "filter must not be empty",
			Status:  0,
		}
	}
	body := map[string]interface{}{"filter": filter, "patch": patch}
	path := "/collections/" + url.PathEscape(collection) + "/vectors/bulk_update_metadata"
	var report BulkUpdateReport
	if err := c.request("POST", path, body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// CopyVectors copies vectors from src to dst without re-embedding. Unlike
// MoveVectors, the source vectors are not deleted.
//
// Calls POST /collections/{src}/vectors/copy with body
// {"destination": dst, "ids": ids}.
//
// Per-id status values: ok | missing_in_src | dst_insert_failed.
func (c *Client) CopyVectors(src, dst string, ids []string) (*CopyReport, error) {
	body := map[string]interface{}{"destination": dst, "ids": ids}
	path := "/collections/" + url.PathEscape(src) + "/vectors/copy"
	var report CopyReport
	if err := c.request("POST", path, body, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

// ReencodeCollection re-quantizes an existing collection in-place without
// re-embedding.
//
// Calls POST /collections/{name}/reencode with body
// {"target_encoding": targetEncoding}. Valid encoding values: "sq8",
// "binary", "fp32".
//
// Returns a ReencodeJob with State == "completed" on success.
func (c *Client) ReencodeCollection(name, targetEncoding string) (*ReencodeJob, error) {
	body := map[string]interface{}{"target_encoding": targetEncoding}
	path := "/collections/" + url.PathEscape(name) + "/reencode"
	var job ReencodeJob
	if err := c.request("POST", path, body, &job); err != nil {
		return nil, err
	}
	return &job, nil
}

// SetCollectionTTL sets or clears a per-collection TTL.
//
// Calls POST /collections/{name}/ttl with body {"ttl_secs": ttlSecs}. Pass
// nil to clear the collection-level TTL. Existing vectors are not
// retroactively expired; only subsequent insertions that carry __expires_at
// in their payload are affected.
//
// For per-vector expiry use SetVectorExpiry.
func (c *Client) SetCollectionTTL(name string, ttlSecs *int64) error {
	body := map[string]interface{}{"ttl_secs": ttlSecs}
	path := "/collections/" + url.PathEscape(name) + "/ttl"
	return c.request("POST", path, body, nil)
}

// SetVectorExpiry sets or clears a per-vector expiry timestamp.
//
// Calls PATCH /collections/{name}/vectors/{id}/expiry with body
// {"expires_at": expiresAt}. Pass nil to clear an existing expiry. The
// timestamp is stored as __expires_at inside the vector payload and is
// read by the per-collection TTL reaper.
func (c *Client) SetVectorExpiry(collection, id string, expiresAt *int64) error {
	body := map[string]interface{}{"expires_at": expiresAt}
	path := "/collections/" + url.PathEscape(collection) + "/vectors/" + url.PathEscape(id) + "/expiry"
	return c.request("PATCH", path, body, nil)
}
