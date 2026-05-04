package vectorizer

import (
	"encoding/json"
	"fmt"
	"net/url"
)

// AuditQuery holds optional filter parameters for GET /auth/audit.
// Empty string fields and a zero Limit are omitted from the request.
type AuditQuery struct {
	Actor  string
	Action string
	Since  string
	Until  string
	Limit  int
}

// ClusterFailover promotes a replica to primary.
//
// Calls POST /cluster/failover with {"replica_id": replicaID}.
// The server returns HTTP 409 when the replica's WAL lag exceeds the
// configured threshold.
func (c *Client) ClusterFailover(replicaID string) (*FailoverReport, error) {
	body := map[string]interface{}{"replica_id": replicaID}
	var result FailoverReport
	if err := c.request("POST", "/cluster/failover", body, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ClusterResyncReplica forces a full resync on the given replica.
//
// Calls POST /cluster/replicas/{id}/resync with an empty body.
func (c *Client) ClusterResyncReplica(replicaID string) (*ResyncJob, error) {
	path := "/cluster/replicas/" + url.PathEscape(replicaID) + "/resync"
	var result ResyncJob
	if err := c.request("POST", path, map[string]interface{}{}, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ClusterAddPeer registers a new peer in the cluster.
//
// Calls POST /cluster/peers with the AddPeerRequest body.
func (c *Client) ClusterAddPeer(req *AddPeerRequest) (*PeerInfo, error) {
	var result PeerInfo
	if err := c.request("POST", "/cluster/peers", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ClusterRebalance triggers a shard rebalance across all active cluster nodes.
//
// Calls POST /cluster/rebalance with an empty body.
// The server returns HTTP 400 when fewer than two active nodes are present
// or when a rebalance is already in progress.
func (c *Client) ClusterRebalance() (*RebalanceJob, error) {
	var result RebalanceJob
	if err := c.request("POST", "/cluster/rebalance", map[string]interface{}{}, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// rebalanceStatusRaw is an intermediate decode target that lets us inspect
// the server's idle sentinel {"status":"idle"} before committing to the
// full RebalanceJob shape.
type rebalanceStatusRaw struct {
	Status string `json:"status"`
}

// ClusterRebalanceStatus returns progress of the active or last completed rebalance job.
//
// Calls GET /cluster/rebalance/status.
// Returns nil when no rebalance has been triggered on this node (server
// returns {"status":"idle"}).
func (c *Client) ClusterRebalanceStatus() (*RebalanceJob, error) {
	// Capture the raw JSON so we can both probe the idle sentinel and
	// decode into the typed struct without an extra HTTP round-trip.
	var raw json.RawMessage
	if err := c.request("GET", "/cluster/rebalance/status", nil, &raw); err != nil {
		return nil, err
	}
	var sentinel rebalanceStatusRaw
	if err := json.Unmarshal(raw, &sentinel); err != nil {
		return nil, fmt.Errorf("parse rebalance status sentinel: %w", err)
	}
	if sentinel.Status == "idle" {
		return nil, nil
	}
	var result RebalanceJob
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("unmarshal rebalance status: %w", err)
	}
	return &result, nil
}

// RotateApiKey atomically rotates an API key (admin only).
//
// Calls POST /auth/keys/{id}/rotate with an empty body.
// Returns both the old and new tokens plus a grace window during which
// the old token remains valid.
func (c *Client) RotateApiKey(id string) (*RotatedKey, error) {
	path := "/auth/keys/" + url.PathEscape(id) + "/rotate"
	var result RotatedKey
	if err := c.request("POST", path, map[string]interface{}{}, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// CreateScopedApiKey creates an API key with optional per-collection scopes.
//
// Calls POST /auth/keys. When Scopes is non-empty the key is restricted to
// the listed collections. The ApiKeyValue field in the returned ApiKey is
// only present at creation time — store it securely.
func (c *Client) CreateScopedApiKey(req *CreateScopedApiKeyRequest) (*ApiKey, error) {
	var result ApiKey
	if err := c.request("POST", "/auth/keys", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// IntrospectToken introspects a token per RFC 7662.
//
// Calls POST /auth/introspect with {"token": token}.
// Returns active:false for any unrecognized token. Does not require admin.
func (c *Client) IntrospectToken(token string) (*TokenIntrospection, error) {
	body := map[string]interface{}{"token": token}
	var result TokenIntrospection
	if err := c.request("POST", "/auth/introspect", body, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// auditLogEnvelope is the response envelope from GET /auth/audit.
type auditLogEnvelope struct {
	Entries []AuditEntry `json:"entries"`
}

// ListAuditLog queries the admin audit log (admin only).
//
// Calls GET /auth/audit with optional query parameters drawn from query.
// Empty string fields and a zero Limit are omitted. Returns entries
// newest-first, bounded by query.Limit (server default 200).
func (c *Client) ListAuditLog(query AuditQuery) ([]AuditEntry, error) {
	params := url.Values{}
	if query.Actor != "" {
		params.Set("actor", query.Actor)
	}
	if query.Action != "" {
		params.Set("action", query.Action)
	}
	if query.Since != "" {
		params.Set("since", query.Since)
	}
	if query.Until != "" {
		params.Set("until", query.Until)
	}
	if query.Limit > 0 {
		params.Set("limit", fmt.Sprintf("%d", query.Limit))
	}

	path := "/auth/audit"
	if encoded := params.Encode(); encoded != "" {
		path = path + "?" + encoded
	}

	var envelope auditLogEnvelope
	if err := c.request("GET", path, nil, &envelope); err != nil {
		return nil, err
	}
	if envelope.Entries == nil {
		return []AuditEntry{}, nil
	}
	return envelope.Entries, nil
}

// UpdateApiKeyPermissions replaces the permission set (and optionally the
// scopes) of an API key without rotating its credential. Admin-only.
//
// Calls PUT /auth/keys/{id}/permissions. The key_hash, id, user_id and
// created_at fields stay immutable. The server rejects an empty
// permissions list with HTTP 400 and an unknown id with HTTP 404.
func (c *Client) UpdateApiKeyPermissions(id string, req *UpdateApiKeyPermissionsRequest) (*ApiKeyView, error) {
	path := "/auth/keys/" + url.PathEscape(id) + "/permissions"
	var result ApiKeyView
	if err := c.request("PUT", path, req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetApiKeyUsage returns the per-day usage time-series for an API key.
// Admin-only.
//
// Calls GET /auth/keys/{id}/usage?window=N. Pass windowDays=0 to use
// the server default (7). The server clamps the window to 1..=30 and
// returns the live key view, the bucket array (oldest first, including
// zero-count days), and the window total.
func (c *Client) GetApiKeyUsage(id string, windowDays int) (*ApiKeyUsageReport, error) {
	path := "/auth/keys/" + url.PathEscape(id) + "/usage"
	if windowDays > 0 {
		path = path + "?window=" + fmt.Sprintf("%d", windowDays)
	}
	var result ApiKeyUsageReport
	if err := c.request("GET", path, nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}
