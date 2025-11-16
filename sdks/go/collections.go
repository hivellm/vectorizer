package vectorizer

// ListCollections returns a list of all collections
func (c *Client) ListCollections() ([]string, error) {
	var collections []string
	if err := c.request("GET", "/collections", nil, &collections); err != nil {
		return nil, err
	}
	return collections, nil
}

// CreateCollection creates a new collection
func (c *Client) CreateCollection(req *CreateCollectionRequest) (*Collection, error) {
	var collection Collection
	if err := c.request("POST", "/collections", req, &collection); err != nil {
		return nil, err
	}
	return &collection, nil
}

// GetCollectionInfo returns information about a collection
func (c *Client) GetCollectionInfo(name string) (*CollectionInfo, error) {
	var info CollectionInfo
	if err := c.request("GET", "/collections/"+name, nil, &info); err != nil {
		return nil, err
	}
	return &info, nil
}

// DeleteCollection deletes a collection
func (c *Client) DeleteCollection(name string) error {
	return c.request("DELETE", "/collections/"+name, nil, nil)
}

