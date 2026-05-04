package vectorizer

// GetReplicationStatus returns the current replication status and role of this node.
//
// Calls GET /replication/status.
func (c *Client) GetReplicationStatus() (*ReplicationStatus, error) {
	var result ReplicationStatus
	if err := c.request("GET", "/replication/status", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ConfigureReplication configures this node's replication role and parameters.
//
// Calls POST /replication/configure with the provided ReplicationConfig.
// A server restart is required for the new configuration to take effect.
func (c *Client) ConfigureReplication(config *ReplicationConfig) error {
	return c.request("POST", "/replication/configure", config, nil)
}

// GetReplicationStats returns raw replication statistics for the active replication node.
//
// Calls GET /replication/stats. Returns an error when replication is not
// enabled on this node.
func (c *Client) GetReplicationStats() (*ReplicationStats, error) {
	var result ReplicationStats
	if err := c.request("GET", "/replication/stats", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ListReplicas returns the replica nodes connected to this master.
//
// Calls GET /replication/replicas. Only available on master nodes; returns
// an error otherwise.
func (c *Client) ListReplicas() ([]ReplicaInfo, error) {
	var result struct {
		Replicas []ReplicaInfo `json:"replicas"`
	}
	if err := c.request("GET", "/replication/replicas", nil, &result); err != nil {
		return nil, err
	}
	return result.Replicas, nil
}
