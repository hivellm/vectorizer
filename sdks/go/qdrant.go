package vectorizer

// ===== QDRANT ADVANCED FEATURES (1.14.x) =====

// QdrantListCollectionSnapshots lists snapshots for a collection (Qdrant-compatible API)
func (c *Client) QdrantListCollectionSnapshots(collection string) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("GET", "/qdrant/collections/"+collection+"/snapshots", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantCreateCollectionSnapshot creates snapshot for a collection (Qdrant-compatible API)
func (c *Client) QdrantCreateCollectionSnapshot(collection string) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/snapshots", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantDeleteCollectionSnapshot deletes snapshot (Qdrant-compatible API)
func (c *Client) QdrantDeleteCollectionSnapshot(collection, snapshotName string) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("DELETE", "/qdrant/collections/"+collection+"/snapshots/"+snapshotName, nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantRecoverCollectionSnapshot recovers collection from snapshot (Qdrant-compatible API)
func (c *Client) QdrantRecoverCollectionSnapshot(collection, location string) (map[string]interface{}, error) {
	req := map[string]interface{}{
		"location": location,
	}
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/snapshots/recover", req, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantListAllSnapshots lists all snapshots (Qdrant-compatible API)
func (c *Client) QdrantListAllSnapshots() (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("GET", "/qdrant/snapshots", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantCreateFullSnapshot creates full snapshot (Qdrant-compatible API)
func (c *Client) QdrantCreateFullSnapshot() (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/snapshots", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantListShardKeys lists shard keys for a collection (Qdrant-compatible API)
func (c *Client) QdrantListShardKeys(collection string) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("GET", "/qdrant/collections/"+collection+"/shards", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantCreateShardKey creates shard key (Qdrant-compatible API)
func (c *Client) QdrantCreateShardKey(collection string, shardKey map[string]interface{}) (map[string]interface{}, error) {
	req := map[string]interface{}{
		"shard_key": shardKey,
	}
	var result map[string]interface{}
	if err := c.request("PUT", "/qdrant/collections/"+collection+"/shards", req, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantDeleteShardKey deletes shard key (Qdrant-compatible API)
func (c *Client) QdrantDeleteShardKey(collection string, shardKey map[string]interface{}) (map[string]interface{}, error) {
	req := map[string]interface{}{
		"shard_key": shardKey,
	}
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/shards/delete", req, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantGetClusterStatus gets cluster status (Qdrant-compatible API)
func (c *Client) QdrantGetClusterStatus() (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("GET", "/qdrant/cluster", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantClusterRecover recovers current peer (Qdrant-compatible API)
func (c *Client) QdrantClusterRecover() (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/cluster/recover", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantRemovePeer removes peer from cluster (Qdrant-compatible API)
func (c *Client) QdrantRemovePeer(peerID string) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("DELETE", "/qdrant/cluster/peer/"+peerID, nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantListMetadataKeys lists metadata keys (Qdrant-compatible API)
func (c *Client) QdrantListMetadataKeys() (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("GET", "/qdrant/cluster/metadata/keys", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantGetMetadataKey gets metadata key (Qdrant-compatible API)
func (c *Client) QdrantGetMetadataKey(key string) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("GET", "/qdrant/cluster/metadata/keys/"+key, nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantUpdateMetadataKey updates metadata key (Qdrant-compatible API)
func (c *Client) QdrantUpdateMetadataKey(key string, value map[string]interface{}) (map[string]interface{}, error) {
	req := map[string]interface{}{
		"value": value,
	}
	var result map[string]interface{}
	if err := c.request("PUT", "/qdrant/cluster/metadata/keys/"+key, req, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantQueryPoints queries points (Qdrant 1.7+ Query API)
func (c *Client) QdrantQueryPoints(collection string, request map[string]interface{}) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/points/query", request, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantBatchQueryPoints batch queries points (Qdrant 1.7+ Query API)
func (c *Client) QdrantBatchQueryPoints(collection string, request map[string]interface{}) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/points/query/batch", request, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantQueryPointsGroups queries points with groups (Qdrant 1.7+ Query API)
func (c *Client) QdrantQueryPointsGroups(collection string, request map[string]interface{}) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/points/query/groups", request, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantSearchPointsGroups searches points with groups (Qdrant Search Groups API)
func (c *Client) QdrantSearchPointsGroups(collection string, request map[string]interface{}) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/points/search/groups", request, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantSearchMatrixPairs searches matrix pairs (Qdrant Search Matrix API)
func (c *Client) QdrantSearchMatrixPairs(collection string, request map[string]interface{}) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/points/search/matrix/pairs", request, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// QdrantSearchMatrixOffsets searches matrix offsets (Qdrant Search Matrix API)
func (c *Client) QdrantSearchMatrixOffsets(collection string, request map[string]interface{}) (map[string]interface{}, error) {
	var result map[string]interface{}
	if err := c.request("POST", "/qdrant/collections/"+collection+"/points/search/matrix/offsets", request, &result); err != nil {
		return nil, err
	}
	return result, nil
}
