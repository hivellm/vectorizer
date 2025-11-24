package vectorizer

// ListGraphNodes lists all nodes in a collection's graph
func (c *Client) ListGraphNodes(collection string) (*ListNodesResponse, error) {
	var result ListNodesResponse
	if err := c.request("GET", "/graph/nodes/"+collection, nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetGraphNeighbors gets neighbors of a specific node
func (c *Client) GetGraphNeighbors(collection, nodeID string) (*GetNeighborsResponse, error) {
	var result GetNeighborsResponse
	if err := c.request("GET", "/graph/nodes/"+collection+"/"+nodeID+"/neighbors", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// FindRelatedNodes finds related nodes within N hops
func (c *Client) FindRelatedNodes(collection, nodeID string, request *FindRelatedRequest) (*FindRelatedResponse, error) {
	var result FindRelatedResponse
	if err := c.request("POST", "/graph/nodes/"+collection+"/"+nodeID+"/related", request, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// FindGraphPath finds shortest path between two nodes
func (c *Client) FindGraphPath(request *FindPathRequest) (*FindPathResponse, error) {
	var result FindPathResponse
	if err := c.request("POST", "/graph/path", request, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// CreateGraphEdge creates an explicit edge between two nodes
func (c *Client) CreateGraphEdge(request *CreateEdgeRequest) (*CreateEdgeResponse, error) {
	var result CreateEdgeResponse
	if err := c.request("POST", "/graph/edges", request, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// DeleteGraphEdge deletes an edge by ID
func (c *Client) DeleteGraphEdge(edgeID string) error {
	return c.request("DELETE", "/graph/edges/"+edgeID, nil, nil)
}

// ListGraphEdges lists all edges in a collection
func (c *Client) ListGraphEdges(collection string) (*ListEdgesResponse, error) {
	var result ListEdgesResponse
	if err := c.request("GET", "/graph/collections/"+collection+"/edges", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// DiscoverGraphEdges discovers SIMILAR_TO edges for entire collection
func (c *Client) DiscoverGraphEdges(collection string, request *DiscoverEdgesRequest) (*DiscoverEdgesResponse, error) {
	var result DiscoverEdgesResponse
	if err := c.request("POST", "/graph/discover/"+collection, request, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// DiscoverGraphEdgesForNode discovers SIMILAR_TO edges for a specific node
func (c *Client) DiscoverGraphEdgesForNode(collection, nodeID string, request *DiscoverEdgesRequest) (*DiscoverEdgesResponse, error) {
	var result DiscoverEdgesResponse
	if err := c.request("POST", "/graph/discover/"+collection+"/"+nodeID, request, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetGraphDiscoveryStatus gets discovery status for a collection
func (c *Client) GetGraphDiscoveryStatus(collection string) (*DiscoveryStatusResponse, error) {
	var result DiscoveryStatusResponse
	if err := c.request("GET", "/graph/discover/"+collection+"/status", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

