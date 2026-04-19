package rpc

import (
	"context"
	"fmt"
)

// Typed wrappers around the v1 RPC command catalog.
//
// Each method corresponds to one entry in the wire spec's command
// catalog (§ 6). The wrapper:
//
//  1. Builds the positional args array per the spec.
//  2. Calls Client.Call.
//  3. Decodes the VectorizerValue response into a typed Go value
//     with explicit field handling (no JSON detour — the wire is
//     MessagePack, not JSON).

// CollectionInfo is the metadata returned by collections.get_info.
type CollectionInfo struct {
	Name          string
	VectorCount   int64
	DocumentCount int64
	Dimension     int64
	Metric        string
	CreatedAt     string
	UpdatedAt     string
}

// SearchHit is one result from search.basic.
//
// Payload is an optional JSON string. The server stores payloads as
// serde_json::Value; the RPC layer ships them as a string because
// the wire VectorizerValue enum doesn't model JSON directly. Decode
// with encoding/json if structured access is needed.
type SearchHit struct {
	ID      string
	Score   float64
	Payload *string
}

// ListCollections issues collections.list and returns every
// collection name visible to the authenticated principal.
func (c *Client) ListCollections(ctx context.Context) ([]string, error) {
	v, err := c.Call(ctx, "collections.list", nil)
	if err != nil {
		return nil, err
	}
	arr, ok := v.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: collections.list: expected Array", ErrServer)
	}
	out := make([]string, 0, len(arr))
	for _, item := range arr {
		if s, ok := item.AsStr(); ok {
			out = append(out, s)
		}
	}
	return out, nil
}

// GetCollectionInfo issues collections.get_info for one collection.
func (c *Client) GetCollectionInfo(ctx context.Context, name string) (CollectionInfo, error) {
	v, err := c.Call(ctx, "collections.get_info", []VectorizerValue{StrValue(name)})
	if err != nil {
		return CollectionInfo{}, err
	}
	needStr := func(key string) (string, error) {
		field, ok := v.MapGet(key)
		if !ok {
			return "", fmt.Errorf("%w: collections.get_info: missing string field '%s'", ErrServer, key)
		}
		s, ok := field.AsStr()
		if !ok {
			return "", fmt.Errorf("%w: collections.get_info: field '%s' is not a string", ErrServer, key)
		}
		return s, nil
	}
	needInt := func(key string) (int64, error) {
		field, ok := v.MapGet(key)
		if !ok {
			return 0, fmt.Errorf("%w: collections.get_info: missing int field '%s'", ErrServer, key)
		}
		i, ok := field.AsInt()
		if !ok {
			return 0, fmt.Errorf("%w: collections.get_info: field '%s' is not an int", ErrServer, key)
		}
		return i, nil
	}
	info := CollectionInfo{}
	var err2 error
	if info.Name, err2 = needStr("name"); err2 != nil {
		return CollectionInfo{}, err2
	}
	if info.VectorCount, err2 = needInt("vector_count"); err2 != nil {
		return CollectionInfo{}, err2
	}
	if info.DocumentCount, err2 = needInt("document_count"); err2 != nil {
		return CollectionInfo{}, err2
	}
	if info.Dimension, err2 = needInt("dimension"); err2 != nil {
		return CollectionInfo{}, err2
	}
	if info.Metric, err2 = needStr("metric"); err2 != nil {
		return CollectionInfo{}, err2
	}
	if info.CreatedAt, err2 = needStr("created_at"); err2 != nil {
		return CollectionInfo{}, err2
	}
	if info.UpdatedAt, err2 = needStr("updated_at"); err2 != nil {
		return CollectionInfo{}, err2
	}
	return info, nil
}

// GetVector issues vectors.get and returns the raw VectorizerValue so
// callers can read whichever fields they care about (id, data,
// payload, document_id).
func (c *Client) GetVector(ctx context.Context, collection, vectorID string) (VectorizerValue, error) {
	return c.Call(ctx, "vectors.get", []VectorizerValue{
		StrValue(collection),
		StrValue(vectorID),
	})
}

// SearchBasic issues search.basic and returns up to limit hits sorted
// by descending similarity.
func (c *Client) SearchBasic(ctx context.Context, collection, query string, limit int) ([]SearchHit, error) {
	args := []VectorizerValue{
		StrValue(collection),
		StrValue(query),
		IntValue(int64(limit)),
	}
	v, err := c.Call(ctx, "search.basic", args)
	if err != nil {
		return nil, err
	}
	arr, ok := v.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: search.basic: expected Array", ErrServer)
	}
	hits := make([]SearchHit, 0, len(arr))
	for _, entry := range arr {
		idV, ok := entry.MapGet("id")
		if !ok {
			return nil, fmt.Errorf("%w: search.basic: hit missing 'id'", ErrServer)
		}
		id, ok := idV.AsStr()
		if !ok {
			return nil, fmt.Errorf("%w: search.basic: hit 'id' is not a string", ErrServer)
		}
		scoreV, ok := entry.MapGet("score")
		if !ok {
			return nil, fmt.Errorf("%w: search.basic: hit missing 'score'", ErrServer)
		}
		score, ok := scoreV.AsFloat()
		if !ok {
			return nil, fmt.Errorf("%w: search.basic: hit 'score' is not numeric", ErrServer)
		}
		hit := SearchHit{ID: id, Score: score}
		if payloadV, ok := entry.MapGet("payload"); ok {
			if p, ok := payloadV.AsStr(); ok {
				hit.Payload = &p
			}
		}
		hits = append(hits, hit)
	}
	return hits, nil
}
