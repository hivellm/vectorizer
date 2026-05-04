package vectorizer

import (
	"net/url"
)

// RenameCollection atomically renames a collection.
//
// Calls POST /collections/{name}/rename with body {"new_name": newName}.
// The server retains the old name as an in-memory alias for one minor
// version so existing clients continue to work without reconfiguration.
// The alias does not survive a restart.
func (c *Client) RenameCollection(oldName, newName string) error {
	path := "/collections/" + url.PathEscape(oldName) + "/rename"
	body := map[string]string{"new_name": newName}
	return c.request("POST", path, body, nil)
}

// ReindexCollection rebuilds the HNSW index for a collection with new
// parameters.
//
// Calls POST /collections/{name}/reindex with body
// {"m": ..., "ef_construction": ..., "ef_search": ...}.
// No re-embedding is required — existing stored vectors are reused.
// Returns a ReindexJob with State == "completed" on success.
func (c *Client) ReindexCollection(name string, params *ReindexParams) (*ReindexJob, error) {
	path := "/collections/" + url.PathEscape(name) + "/reindex"
	var result ReindexJob
	if err := c.request("POST", path, params, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// SnapshotCollectionNative creates a native per-collection snapshot.
//
// Calls POST /collections/{name}/snapshot with an empty body.
// The server writes a gzip-compressed JSON snapshot and returns snapshot
// metadata including ID, collection name, creation time, and size.
func (c *Client) SnapshotCollectionNative(name string) (*NativeSnapshotInfo, error) {
	path := "/collections/" + url.PathEscape(name) + "/snapshot"
	var result NativeSnapshotInfo
	if err := c.request("POST", path, map[string]interface{}{}, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ListCollectionSnapshotsNative lists all native snapshots for a collection.
//
// Calls GET /collections/{name}/snapshots.
// Returns snapshots newest-first as reported by the server.
func (c *Client) ListCollectionSnapshotsNative(name string) ([]NativeSnapshotInfo, error) {
	path := "/collections/" + url.PathEscape(name) + "/snapshots"
	var envelope struct {
		Snapshots []NativeSnapshotInfo `json:"snapshots"`
	}
	if err := c.request("GET", path, nil, &envelope); err != nil {
		return nil, err
	}
	return envelope.Snapshots, nil
}

// RestoreCollectionSnapshotNative restores a collection from a native snapshot.
//
// Calls POST /collections/{name}/snapshots/{id}/restore with an empty body.
// Drops the current in-memory state and replaces it with the snapshot data.
func (c *Client) RestoreCollectionSnapshotNative(name, snapshotID string) error {
	path := "/collections/" + url.PathEscape(name) + "/snapshots/" + url.PathEscape(snapshotID) + "/restore"
	return c.request("POST", path, map[string]interface{}{}, nil)
}

// ExplainSearch runs a vector search and returns the full HNSW execution trace.
//
// Calls POST /collections/{name}/explain with body {"vector": [...], "k": k}.
// The trace includes visited_nodes, ef_search, hnsw_search_ms,
// payload_filter_evals, quantization_score_ms, and total_ms.
// The results are identical to a normal search; the real code path is
// instrumented rather than a separate explain engine.
func (c *Client) ExplainSearch(collection string, vector []float32, k int) (*ExplainResponse, error) {
	path := "/collections/" + url.PathEscape(collection) + "/explain"
	body := map[string]interface{}{
		"vector": vector,
		"k":      k,
	}
	var result ExplainResponse
	if err := c.request("POST", path, body, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ListSlowQueries returns entries from the slow-query ring buffer.
//
// Calls GET /slow_queries. Returns entries in the order they were recorded
// (oldest first). Use SetSlowQueryConfig to tune the threshold and capacity.
func (c *Client) ListSlowQueries() ([]SlowQueryEntry, error) {
	var envelope struct {
		Entries []SlowQueryEntry `json:"entries"`
	}
	if err := c.request("GET", "/slow_queries", nil, &envelope); err != nil {
		return nil, err
	}
	return envelope.Entries, nil
}

// SetSlowQueryConfig reconfigures the slow-query ring buffer.
//
// Calls POST /slow_queries/config with body
// {"threshold_ms": ..., "capacity": ...}.
// Existing entries are retained. If the new capacity is smaller than the
// current entry count the oldest entries are evicted by the server.
// Returns the updated configuration as echoed back by the server.
func (c *Client) SetSlowQueryConfig(config *SlowQueryConfig) (*SlowQueryConfig, error) {
	var result SlowQueryConfig
	if err := c.request("POST", "/slow_queries/config", config, &result); err != nil {
		return nil, err
	}
	return &result, nil
}
