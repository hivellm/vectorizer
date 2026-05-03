package rpc

// Phase 16 typed wrappers — full server command catalog.
//
// Mirrors sdks/rust/src/rpc/commands.rs and sdks/typescript/src/rpc/commands.ts.
// Each method follows the same pattern as the existing wrappers in commands.go:
//
//  1. Build the positional args array.
//  2. Call Client.Call.
//  3. Decode the VectorizerValue response into a typed Go value.

import (
	"context"
	"fmt"
)

// ── decode helpers ────────────────────────────────────────────────

func mapGetStr(v VectorizerValue, key string) (string, bool) {
	f, ok := v.MapGet(key)
	if !ok {
		return "", false
	}
	s, ok := f.AsStr()
	return s, ok
}

func mapGetInt(v VectorizerValue, key string) (int64, bool) {
	f, ok := v.MapGet(key)
	if !ok {
		return 0, false
	}
	i, ok := f.AsInt()
	return i, ok
}

func mapGetFloat(v VectorizerValue, key string) (float64, bool) {
	f, ok := v.MapGet(key)
	if !ok {
		return 0, false
	}
	fl, ok := f.AsFloat()
	return fl, ok
}

func mapGetBool(v VectorizerValue, key string) (bool, bool) {
	f, ok := v.MapGet(key)
	if !ok {
		return false, false
	}
	b, ok := f.AsBool()
	return b, ok
}

func requireStr(v VectorizerValue, cmd, key string) (string, error) {
	s, ok := mapGetStr(v, key)
	if !ok {
		return "", fmt.Errorf("%w: %s: missing string field '%s'", ErrServer, cmd, key)
	}
	return s, nil
}

func requireBool(v VectorizerValue, cmd, key string) (bool, error) {
	b, ok := mapGetBool(v, key)
	if !ok {
		return false, fmt.Errorf("%w: %s: missing bool field '%s'", ErrServer, cmd, key)
	}
	return b, nil
}

func requireInt(v VectorizerValue, cmd, key string) (int64, error) {
	i, ok := mapGetInt(v, key)
	if !ok {
		return 0, fmt.Errorf("%w: %s: missing int field '%s'", ErrServer, cmd, key)
	}
	return i, nil
}

func getStr(v VectorizerValue, key, fallback string) string {
	s, ok := mapGetStr(v, key)
	if !ok {
		return fallback
	}
	return s
}

func getInt(v VectorizerValue, key string) int64 {
	i, _ := mapGetInt(v, key)
	return i
}

func getFloat(v VectorizerValue, key string) float64 {
	f, _ := mapGetFloat(v, key)
	return f
}

func getBool(v VectorizerValue, key string) bool {
	b, _ := mapGetBool(v, key)
	return b
}

func getOptionalStr(v VectorizerValue, key string) *string {
	s, ok := mapGetStr(v, key)
	if !ok {
		return nil
	}
	return &s
}

func decodeBatchItems(arr []VectorizerValue) []BatchItemResult {
	items := make([]BatchItemResult, len(arr))
	for i, entry := range arr {
		items[i] = BatchItemResult{
			Index:  getInt(entry, "index"),
			ID:     getOptionalStr(entry, "id"),
			Status: getStr(entry, "status", "unknown"),
			Error:  getOptionalStr(entry, "error"),
		}
	}
	return items
}

func decodeSearchHitsPhase16(arr []VectorizerValue) []SearchHit {
	hits := make([]SearchHit, 0, len(arr))
	for _, entry := range arr {
		id, ok := mapGetStr(entry, "id")
		if !ok {
			continue
		}
		score, _ := mapGetFloat(entry, "score")
		hit := SearchHit{ID: id, Score: score}
		if p, ok := mapGetStr(entry, "payload"); ok {
			hit.Payload = &p
		}
		hits = append(hits, hit)
	}
	return hits
}

func stringsValue(ss []string) VectorizerValue {
	items := make([]VectorizerValue, len(ss))
	for i, s := range ss {
		items[i] = StrValue(s)
	}
	return ArrayValue(items)
}

// ═════════════════════════════════════════════════════════════════
// Collections
// ═════════════════════════════════════════════════════════════════

// CreateCollectionRpc issues collections.create.
//
// config is a Map with optional keys dimension (Int) and metric (Str:
// "cosine" | "euclidean" | "dot").
func (c *Client) CreateCollectionRpc(ctx context.Context, name string, config VectorizerValue) (CreateCollectionResult, error) {
	v, err := c.Call(ctx, "collections.create", []VectorizerValue{StrValue(name), config})
	if err != nil {
		return CreateCollectionResult{}, err
	}
	n, err := requireStr(v, "collections.create", "name")
	if err != nil {
		return CreateCollectionResult{}, err
	}
	dim, err := requireInt(v, "collections.create", "dimension")
	if err != nil {
		return CreateCollectionResult{}, err
	}
	metric, err := requireStr(v, "collections.create", "metric")
	if err != nil {
		return CreateCollectionResult{}, err
	}
	success, err := requireBool(v, "collections.create", "success")
	if err != nil {
		return CreateCollectionResult{}, err
	}
	return CreateCollectionResult{Name: n, Dimension: dim, Metric: metric, Success: success}, nil
}

// DeleteCollectionRpc issues collections.delete (admin-gated on server).
// Returns the success flag.
func (c *Client) DeleteCollectionRpc(ctx context.Context, name string) (bool, error) {
	v, err := c.Call(ctx, "collections.delete", []VectorizerValue{StrValue(name)})
	if err != nil {
		return false, err
	}
	return requireBool(v, "collections.delete", "success")
}

// ListEmptyCollections issues collections.list_empty and returns names of
// collections that contain zero vectors.
func (c *Client) ListEmptyCollections(ctx context.Context) ([]string, error) {
	v, err := c.Call(ctx, "collections.list_empty", nil)
	if err != nil {
		return nil, err
	}
	arr, ok := v.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: collections.list_empty: expected Array", ErrServer)
	}
	out := make([]string, 0, len(arr))
	for _, item := range arr {
		if s, ok := item.AsStr(); ok {
			out = append(out, s)
		}
	}
	return out, nil
}

// CleanupEmptyCollections issues collections.cleanup_empty.
//
// Pass dryRun=true to preview which collections would be removed without
// actually deleting them.
func (c *Client) CleanupEmptyCollections(ctx context.Context, dryRun bool) (CleanupEmptyResult, error) {
	config := MapValue([]MapPair{
		{Key: StrValue("dry_run"), Value: BoolValue(dryRun)},
	})
	v, err := c.Call(ctx, "collections.cleanup_empty", []VectorizerValue{config})
	if err != nil {
		return CleanupEmptyResult{}, err
	}
	removed, err := requireInt(v, "collections.cleanup_empty", "removed")
	if err != nil {
		return CleanupEmptyResult{}, err
	}
	dry, err := requireBool(v, "collections.cleanup_empty", "dry_run")
	if err != nil {
		return CleanupEmptyResult{}, err
	}
	return CleanupEmptyResult{Removed: removed, DryRun: dry}, nil
}

// ForceSaveCollection issues collections.force_save to flush a collection's
// in-memory state to disk. Returns the success flag.
func (c *Client) ForceSaveCollection(ctx context.Context, name string) (bool, error) {
	v, err := c.Call(ctx, "collections.force_save", []VectorizerValue{StrValue(name)})
	if err != nil {
		return false, err
	}
	return requireBool(v, "collections.force_save", "success")
}

// ═════════════════════════════════════════════════════════════════
// Vectors
// ═════════════════════════════════════════════════════════════════

// InsertVector issues vectors.insert with a pre-computed embedding.
//
// data must match the collection's configured dimension. id may be empty
// to let the server assign one. payload is optional and may be NullValue().
func (c *Client) InsertVectorRpc(ctx context.Context, collection, id string, data []float64, payload VectorizerValue) (VectorWriteResult, error) {
	var idVal VectorizerValue
	if id == "" {
		idVal = NullValue()
	} else {
		idVal = StrValue(id)
	}
	floats := make([]VectorizerValue, len(data))
	for i, f := range data {
		floats[i] = FloatValue(f)
	}
	args := []VectorizerValue{StrValue(collection), idVal, ArrayValue(floats)}
	if payload.Kind != ValueNull {
		args = append(args, payload)
	}
	v, err := c.Call(ctx, "vectors.insert", args)
	if err != nil {
		return VectorWriteResult{}, err
	}
	rid, err := requireStr(v, "vectors.insert", "id")
	if err != nil {
		return VectorWriteResult{}, err
	}
	success, err := requireBool(v, "vectors.insert", "success")
	if err != nil {
		return VectorWriteResult{}, err
	}
	return VectorWriteResult{ID: rid, Success: success}, nil
}

// InsertTextVectorRpc issues vectors.insert_text, embedding text server-side.
//
// The server auto-creates the collection if it does not exist. id may be
// empty; payload is optional.
func (c *Client) InsertTextVectorRpc(ctx context.Context, collection, id, text string, payload VectorizerValue) (VectorWriteResult, error) {
	var idVal VectorizerValue
	if id == "" {
		idVal = NullValue()
	} else {
		idVal = StrValue(id)
	}
	args := []VectorizerValue{StrValue(collection), idVal, StrValue(text)}
	if payload.Kind != ValueNull {
		args = append(args, payload)
	}
	v, err := c.Call(ctx, "vectors.insert_text", args)
	if err != nil {
		return VectorWriteResult{}, err
	}
	rid, err := requireStr(v, "vectors.insert_text", "id")
	if err != nil {
		return VectorWriteResult{}, err
	}
	success, err := requireBool(v, "vectors.insert_text", "success")
	if err != nil {
		return VectorWriteResult{}, err
	}
	return VectorWriteResult{ID: rid, Success: success}, nil
}

// UpdateVectorRpc issues vectors.update to replace a vector's data and/or payload.
func (c *Client) UpdateVectorRpc(ctx context.Context, collection, id string, data []float64, payload VectorizerValue) (VectorWriteResult, error) {
	floats := make([]VectorizerValue, len(data))
	for i, f := range data {
		floats[i] = FloatValue(f)
	}
	args := []VectorizerValue{StrValue(collection), StrValue(id), ArrayValue(floats)}
	if payload.Kind != ValueNull {
		args = append(args, payload)
	}
	v, err := c.Call(ctx, "vectors.update", args)
	if err != nil {
		return VectorWriteResult{}, err
	}
	rid, err := requireStr(v, "vectors.update", "id")
	if err != nil {
		return VectorWriteResult{}, err
	}
	success, err := requireBool(v, "vectors.update", "success")
	if err != nil {
		return VectorWriteResult{}, err
	}
	return VectorWriteResult{ID: rid, Success: success}, nil
}

// DeleteVectorRpc issues vectors.delete to remove one vector by id.
// Returns the success flag.
func (c *Client) DeleteVectorRpc(ctx context.Context, collection, id string) (bool, error) {
	v, err := c.Call(ctx, "vectors.delete", []VectorizerValue{StrValue(collection), StrValue(id)})
	if err != nil {
		return false, err
	}
	return requireBool(v, "vectors.delete", "success")
}

// ListVectors issues vectors.list to page through vectors in a collection.
//
// page is zero-based; limit is capped at 50 by the server.
func (c *Client) ListVectors(ctx context.Context, collection string, page, limit int64) (VectorListResult, error) {
	v, err := c.Call(ctx, "vectors.list", []VectorizerValue{
		StrValue(collection),
		IntValue(page),
		IntValue(limit),
	})
	if err != nil {
		return VectorListResult{}, err
	}
	var items []VectorizerValue
	if itemsV, ok := v.MapGet("items"); ok {
		if arr, ok := itemsV.AsArray(); ok {
			items = arr
		}
	}
	return VectorListResult{
		Items: items,
		Total: getInt(v, "total"),
		Page:  getInt(v, "page"),
		Limit: getInt(v, "limit"),
	}, nil
}

// EmbedText issues vectors.embed to embed text server-side and return the
// embedding. model may be empty to use the server default.
func (c *Client) EmbedText(ctx context.Context, text, model string) (EmbedResult, error) {
	args := []VectorizerValue{StrValue(text)}
	if model != "" {
		args = append(args, StrValue(model))
	}
	v, err := c.Call(ctx, "vectors.embed", args)
	if err != nil {
		return EmbedResult{}, err
	}
	var embedding []float64
	if embV, ok := v.MapGet("embedding"); ok {
		if arr, ok := embV.AsArray(); ok {
			embedding = make([]float64, 0, len(arr))
			for _, item := range arr {
				if f, ok := item.AsFloat(); ok {
					embedding = append(embedding, f)
				}
			}
		}
	}
	return EmbedResult{
		Embedding: embedding,
		Model:     getStr(v, "model", "bm25"),
		Dimension: getInt(v, "dimension"),
	}, nil
}

// BatchInsertVectors issues vectors.batch_insert. Each item in items is a
// Map with at least data (Array<Float>) and optionally id (Str) and payload (Map).
func (c *Client) BatchInsertVectors(ctx context.Context, collection string, items []VectorizerValue) (BatchInsertResult, error) {
	v, err := c.Call(ctx, "vectors.batch_insert", []VectorizerValue{
		StrValue(collection),
		ArrayValue(items),
	})
	if err != nil {
		return BatchInsertResult{}, err
	}
	var results []BatchItemResult
	if rv, ok := v.MapGet("results"); ok {
		if arr, ok := rv.AsArray(); ok {
			results = decodeBatchItems(arr)
		}
	}
	return BatchInsertResult{
		Inserted: getInt(v, "inserted"),
		Failed:   getInt(v, "failed"),
		Results:  results,
	}, nil
}

// BatchInsertTexts issues vectors.batch_insert_texts to embed and insert
// multiple text items. Each item is a Map with at least text (Str) and
// optionally id (Str) and payload (Map).
func (c *Client) BatchInsertTexts(ctx context.Context, collection string, items []VectorizerValue) (BatchInsertResult, error) {
	v, err := c.Call(ctx, "vectors.batch_insert_texts", []VectorizerValue{
		StrValue(collection),
		ArrayValue(items),
	})
	if err != nil {
		return BatchInsertResult{}, err
	}
	var results []BatchItemResult
	if rv, ok := v.MapGet("results"); ok {
		if arr, ok := rv.AsArray(); ok {
			results = decodeBatchItems(arr)
		}
	}
	return BatchInsertResult{
		Inserted: getInt(v, "inserted"),
		Failed:   getInt(v, "failed"),
		Results:  results,
	}, nil
}

// BatchSearch issues vectors.batch_search to run multiple searches in one
// round-trip. Each request is a Map with collection (Str), query (Str), and
// optional limit (Int).
func (c *Client) BatchSearch(ctx context.Context, requests []VectorizerValue) ([]BatchSearchResult, error) {
	v, err := c.Call(ctx, "vectors.batch_search", []VectorizerValue{ArrayValue(requests)})
	if err != nil {
		return nil, err
	}
	arr, ok := v.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: vectors.batch_search: expected Array", ErrServer)
	}
	out := make([]BatchSearchResult, len(arr))
	for i, entry := range arr {
		var hits []SearchHit
		if rv, ok := entry.MapGet("results"); ok {
			if ha, ok := rv.AsArray(); ok {
				hits = decodeSearchHitsPhase16(ha)
			}
		}
		out[i] = BatchSearchResult{
			Index:   getInt(entry, "index"),
			Status:  getStr(entry, "status", "unknown"),
			Results: hits,
			Error:   getOptionalStr(entry, "error"),
		}
	}
	return out, nil
}

// BatchUpdateVectors issues vectors.batch_update. Each item in updates is a
// Map with id (Str) and optionally data (Array<Float>) and payload (Map).
func (c *Client) BatchUpdateVectors(ctx context.Context, collection string, updates []VectorizerValue) (BatchUpdateResult, error) {
	v, err := c.Call(ctx, "vectors.batch_update", []VectorizerValue{
		StrValue(collection),
		ArrayValue(updates),
	})
	if err != nil {
		return BatchUpdateResult{}, err
	}
	var results []BatchItemResult
	if rv, ok := v.MapGet("results"); ok {
		if arr, ok := rv.AsArray(); ok {
			results = decodeBatchItems(arr)
		}
	}
	return BatchUpdateResult{
		Updated: getInt(v, "updated"),
		Failed:  getInt(v, "failed"),
		Results: results,
	}, nil
}

// BatchDeleteVectors issues vectors.batch_delete to delete multiple vectors.
func (c *Client) BatchDeleteVectors(ctx context.Context, collection string, ids []string) (BatchDeleteResult, error) {
	v, err := c.Call(ctx, "vectors.batch_delete", []VectorizerValue{
		StrValue(collection),
		stringsValue(ids),
	})
	if err != nil {
		return BatchDeleteResult{}, err
	}
	var results []BatchItemResult
	if rv, ok := v.MapGet("results"); ok {
		if arr, ok := rv.AsArray(); ok {
			results = decodeBatchItems(arr)
		}
	}
	return BatchDeleteResult{
		Deleted: getInt(v, "deleted"),
		Failed:  getInt(v, "failed"),
		Results: results,
	}, nil
}

// MoveVectorsRpc issues vectors.move to move vectors from src to dst collection.
// Named MoveVectorsRpc to avoid collision with the REST SDK.
func (c *Client) MoveVectorsRpc(ctx context.Context, src, dst string, ids []string) (MoveVectorsResult, error) {
	v, err := c.Call(ctx, "vectors.move", []VectorizerValue{
		StrValue(src),
		StrValue(dst),
		stringsValue(ids),
	})
	if err != nil {
		return MoveVectorsResult{}, err
	}
	s, err := requireStr(v, "vectors.move", "src")
	if err != nil {
		return MoveVectorsResult{}, err
	}
	d, err := requireStr(v, "vectors.move", "dst")
	if err != nil {
		return MoveVectorsResult{}, err
	}
	return MoveVectorsResult{
		Src:    s,
		Dst:    d,
		Moved:  getInt(v, "moved"),
		Failed: getInt(v, "failed"),
	}, nil
}

// CopyVectorsRpc issues vectors.copy to copy vectors from src to dst without
// deleting the originals.
func (c *Client) CopyVectorsRpc(ctx context.Context, src, dst string, ids []string) (CopyVectorsResult, error) {
	v, err := c.Call(ctx, "vectors.copy", []VectorizerValue{
		StrValue(src),
		StrValue(dst),
		stringsValue(ids),
	})
	if err != nil {
		return CopyVectorsResult{}, err
	}
	s, err := requireStr(v, "vectors.copy", "src")
	if err != nil {
		return CopyVectorsResult{}, err
	}
	d, err := requireStr(v, "vectors.copy", "dst")
	if err != nil {
		return CopyVectorsResult{}, err
	}
	return CopyVectorsResult{
		Src:    s,
		Dst:    d,
		Copied: getInt(v, "copied"),
		Failed: getInt(v, "failed"),
	}, nil
}

// DeleteByFilterRpc issues vectors.delete_by_filter to delete all vectors
// matching a Qdrant-style filter predicate.
func (c *Client) DeleteByFilterRpc(ctx context.Context, collection string, filter VectorizerValue) (DeleteByFilterResult, error) {
	v, err := c.Call(ctx, "vectors.delete_by_filter", []VectorizerValue{StrValue(collection), filter})
	if err != nil {
		return DeleteByFilterResult{}, err
	}
	return DeleteByFilterResult{
		Scanned: getInt(v, "scanned"),
		Matched: getInt(v, "matched"),
		Deleted: getInt(v, "deleted"),
	}, nil
}

// BulkUpdateMetadataRpc issues vectors.bulk_update_metadata to apply a
// JSON-merge-patch to all vectors matching filter.
func (c *Client) BulkUpdateMetadataRpc(ctx context.Context, collection string, filter, patch VectorizerValue) (BulkUpdateMetadataResult, error) {
	v, err := c.Call(ctx, "vectors.bulk_update_metadata", []VectorizerValue{
		StrValue(collection),
		filter,
		patch,
	})
	if err != nil {
		return BulkUpdateMetadataResult{}, err
	}
	return BulkUpdateMetadataResult{
		Scanned: getInt(v, "scanned"),
		Matched: getInt(v, "matched"),
		Updated: getInt(v, "updated"),
	}, nil
}

// SetVectorExpiry issues vectors.set_expiry to attach a TTL to one vector.
// expiresAt may be a Unix millisecond timestamp string or an RFC3339 string.
func (c *Client) SetVectorExpiry(ctx context.Context, collection, id, expiresAt string) (SetExpiryResult, error) {
	v, err := c.Call(ctx, "vectors.set_expiry", []VectorizerValue{
		StrValue(collection),
		StrValue(id),
		StrValue(expiresAt),
	})
	if err != nil {
		return SetExpiryResult{}, err
	}
	rid, err := requireStr(v, "vectors.set_expiry", "id")
	if err != nil {
		return SetExpiryResult{}, err
	}
	exp, err := requireInt(v, "vectors.set_expiry", "expires_at")
	if err != nil {
		return SetExpiryResult{}, err
	}
	success, err := requireBool(v, "vectors.set_expiry", "success")
	if err != nil {
		return SetExpiryResult{}, err
	}
	return SetExpiryResult{ID: rid, ExpiresAt: exp, Success: success}, nil
}

// ═════════════════════════════════════════════════════════════════
// Search
// ═════════════════════════════════════════════════════════════════

// SearchIntelligent issues search.intelligent for multi-collection intelligent
// search. request must contain query and optionally collections, max_results,
// domain_expansion.
func (c *Client) SearchIntelligent(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "search.intelligent", []VectorizerValue{request})
}

// SearchByText issues search.by_text for one collection.
func (c *Client) SearchByText(ctx context.Context, collection, query string, limit int) ([]SearchHit, error) {
	v, err := c.Call(ctx, "search.by_text", []VectorizerValue{
		StrValue(collection),
		StrValue(query),
		IntValue(int64(limit)),
	})
	if err != nil {
		return nil, err
	}
	rv, ok := v.MapGet("results")
	if !ok {
		return nil, fmt.Errorf("%w: search.by_text: missing results array", ErrServer)
	}
	arr, ok := rv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: search.by_text: results is not an Array", ErrServer)
	}
	return decodeSearchHitsPhase16(arr), nil
}

// SearchByFile issues search.by_file for file-content-based search.
// request is a Map describing the file query.
func (c *Client) SearchByFile(ctx context.Context, collection string, request VectorizerValue) ([]SearchHit, error) {
	v, err := c.Call(ctx, "search.by_file", []VectorizerValue{StrValue(collection), request})
	if err != nil {
		return nil, err
	}
	rv, ok := v.MapGet("results")
	if !ok {
		return nil, fmt.Errorf("%w: search.by_file: missing results array", ErrServer)
	}
	arr, ok := rv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: search.by_file: results is not an Array", ErrServer)
	}
	return decodeSearchHitsPhase16(arr), nil
}

// SearchHybrid issues search.hybrid for RRF / weighted-combination
// dense+sparse search. request must contain query and optionally alpha,
// dense_k, sparse_k, final_k, algorithm.
func (c *Client) SearchHybrid(ctx context.Context, collection string, request VectorizerValue) ([]SearchHit, error) {
	v, err := c.Call(ctx, "search.hybrid", []VectorizerValue{StrValue(collection), request})
	if err != nil {
		return nil, err
	}
	rv, ok := v.MapGet("results")
	if !ok {
		return nil, fmt.Errorf("%w: search.hybrid: missing results array", ErrServer)
	}
	arr, ok := rv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: search.hybrid: results is not an Array", ErrServer)
	}
	return decodeSearchHitsPhase16(arr), nil
}

// SearchSemantic issues search.semantic for semantic re-ranking search.
// request must contain query and collection.
func (c *Client) SearchSemantic(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "search.semantic", []VectorizerValue{request})
}

// SearchContextual issues search.contextual for context-filtered semantic
// search. request must contain query and collection.
func (c *Client) SearchContextual(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "search.contextual", []VectorizerValue{request})
}

// SearchMultiCollection issues search.multi_collection for fan-out search
// across multiple collections. request must contain query and collections.
func (c *Client) SearchMultiCollection(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "search.multi_collection", []VectorizerValue{request})
}

// SearchExplain issues search.explain to run a vector search and return an
// HNSW traversal trace. request must contain vector (Array<Float>) and
// optionally k (Int).
func (c *Client) SearchExplain(ctx context.Context, collection string, request VectorizerValue) (SearchExplainResult, error) {
	v, err := c.Call(ctx, "search.explain", []VectorizerValue{StrValue(collection), request})
	if err != nil {
		return SearchExplainResult{}, err
	}
	var hits []SearchHit
	if hv, ok := v.MapGet("hits"); ok {
		if arr, ok := hv.AsArray(); ok {
			hits = decodeSearchHitsPhase16(arr)
		}
	}
	var trace SearchTrace
	if tv, ok := v.MapGet("trace"); ok {
		trace = SearchTrace{
			VisitedNodes: getInt(tv, "visited_nodes"),
			EfSearch:     getInt(tv, "ef_search"),
			HnswSearchMs: getFloat(tv, "hnsw_search_ms"),
			TotalMs:      getFloat(tv, "total_ms"),
		}
	}
	return SearchExplainResult{
		Hits:       hits,
		Collection: getStr(v, "collection", ""),
		K:          getInt(v, "k"),
		Trace:      trace,
	}, nil
}

// ═════════════════════════════════════════════════════════════════
// Discovery
// ═════════════════════════════════════════════════════════════════

// Discover issues discovery.discover to run the full discovery pipeline:
// embed → search → compress → build plan → render prompt.
// request must contain query and optionally include_collections,
// exclude_collections, max_bullets.
func (c *Client) Discover(ctx context.Context, request VectorizerValue) (DiscoverResult, error) {
	v, err := c.Call(ctx, "discovery.discover", []VectorizerValue{request})
	if err != nil {
		return DiscoverResult{}, err
	}
	ap, err := requireStr(v, "discovery.discover", "answer_prompt")
	if err != nil {
		return DiscoverResult{}, err
	}
	return DiscoverResult{
		AnswerPrompt: ap,
		Sections:     getInt(v, "sections"),
		Bullets:      getInt(v, "bullets"),
		Chunks:       getInt(v, "chunks"),
	}, nil
}

// FilterCollections issues discovery.filter_collections to filter collection
// list by query relevance. request must contain query.
func (c *Client) FilterCollections(ctx context.Context, request VectorizerValue) ([]string, error) {
	v, err := c.Call(ctx, "discovery.filter_collections", []VectorizerValue{request})
	if err != nil {
		return nil, err
	}
	fv, ok := v.MapGet("filtered_collections")
	if !ok {
		return nil, fmt.Errorf("%w: discovery.filter_collections: missing filtered_collections", ErrServer)
	}
	arr, ok := fv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: discovery.filter_collections: not an Array", ErrServer)
	}
	out := make([]string, 0, len(arr))
	for _, entry := range arr {
		if n, ok := mapGetStr(entry, "name"); ok {
			out = append(out, n)
		}
	}
	return out, nil
}

// ScoreCollections issues discovery.score_collections to score all collections
// for a query. request must contain query.
func (c *Client) ScoreCollections(ctx context.Context, request VectorizerValue) ([]ScoredCollection, error) {
	v, err := c.Call(ctx, "discovery.score_collections", []VectorizerValue{request})
	if err != nil {
		return nil, err
	}
	sv, ok := v.MapGet("scored_collections")
	if !ok {
		return nil, fmt.Errorf("%w: discovery.score_collections: missing scored_collections", ErrServer)
	}
	arr, ok := sv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: discovery.score_collections: not an Array", ErrServer)
	}
	out := make([]ScoredCollection, len(arr))
	for i, entry := range arr {
		out[i] = ScoredCollection{
			Name:        getStr(entry, "name", ""),
			Score:       getFloat(entry, "score"),
			VectorCount: getInt(entry, "vector_count"),
		}
	}
	return out, nil
}

// ExpandQueries issues discovery.expand_queries to generate query variants
// via baseline expansion. request must contain query.
func (c *Client) ExpandQueries(ctx context.Context, request VectorizerValue) (ExpandQueriesResult, error) {
	v, err := c.Call(ctx, "discovery.expand_queries", []VectorizerValue{request})
	if err != nil {
		return ExpandQueriesResult{}, err
	}
	orig, err := requireStr(v, "discovery.expand_queries", "original_query")
	if err != nil {
		return ExpandQueriesResult{}, err
	}
	var expanded []string
	if ev, ok := v.MapGet("expanded_queries"); ok {
		if arr, ok := ev.AsArray(); ok {
			expanded = make([]string, 0, len(arr))
			for _, item := range arr {
				if s, ok := item.AsStr(); ok && s != "" {
					expanded = append(expanded, s)
				}
			}
		}
	}
	return ExpandQueriesResult{
		OriginalQuery:   orig,
		ExpandedQueries: expanded,
		Count:           getInt(v, "count"),
	}, nil
}

// BroadDiscovery issues discovery.broad_discovery for multi-query broad search
// across all collections. request must contain queries.
func (c *Client) BroadDiscovery(ctx context.Context, request VectorizerValue) ([]DiscoveryChunk, error) {
	v, err := c.Call(ctx, "discovery.broad_discovery", []VectorizerValue{request})
	if err != nil {
		return nil, err
	}
	cv, ok := v.MapGet("chunks")
	if !ok {
		return nil, fmt.Errorf("%w: discovery.broad_discovery: missing chunks", ErrServer)
	}
	arr, ok := cv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: discovery.broad_discovery: chunks is not an Array", ErrServer)
	}
	out := make([]DiscoveryChunk, len(arr))
	for i, entry := range arr {
		out[i] = DiscoveryChunk{
			Collection:     getStr(entry, "collection", ""),
			Score:          getFloat(entry, "score"),
			ContentPreview: getStr(entry, "content_preview", ""),
		}
	}
	return out, nil
}

// SemanticFocus issues discovery.semantic_focus for deep semantic search
// within one collection. request must contain collection and queries.
func (c *Client) SemanticFocus(ctx context.Context, request VectorizerValue) ([]DiscoveryChunk, error) {
	v, err := c.Call(ctx, "discovery.semantic_focus", []VectorizerValue{request})
	if err != nil {
		return nil, err
	}
	cv, ok := v.MapGet("chunks")
	if !ok {
		return nil, fmt.Errorf("%w: discovery.semantic_focus: missing chunks", ErrServer)
	}
	arr, ok := cv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: discovery.semantic_focus: chunks is not an Array", ErrServer)
	}
	out := make([]DiscoveryChunk, len(arr))
	for i, entry := range arr {
		out[i] = DiscoveryChunk{
			Collection:     getStr(entry, "collection", ""),
			Score:          getFloat(entry, "score"),
			ContentPreview: getStr(entry, "content_preview", ""),
		}
	}
	return out, nil
}

// PromoteReadme issues discovery.promote_readme to promote README chunks to
// the top of a chunk set. Returns the raw VectorizerValue response.
func (c *Client) PromoteReadme(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "discovery.promote_readme", []VectorizerValue{request})
}

// CompressEvidence issues discovery.compress_evidence to compress a chunk
// set into ranked bullets. request must contain chunks.
func (c *Client) CompressEvidence(ctx context.Context, request VectorizerValue) ([]CompressBullet, error) {
	v, err := c.Call(ctx, "discovery.compress_evidence", []VectorizerValue{request})
	if err != nil {
		return nil, err
	}
	bv, ok := v.MapGet("bullets")
	if !ok {
		return nil, fmt.Errorf("%w: discovery.compress_evidence: missing bullets", ErrServer)
	}
	arr, ok := bv.AsArray()
	if !ok {
		return nil, fmt.Errorf("%w: discovery.compress_evidence: bullets is not an Array", ErrServer)
	}
	out := make([]CompressBullet, len(arr))
	for i, entry := range arr {
		out[i] = CompressBullet{
			Text:     getStr(entry, "text", ""),
			SourceID: getStr(entry, "source_id", ""),
			Score:    getFloat(entry, "score"),
		}
	}
	return out, nil
}

// BuildAnswerPlan issues discovery.build_answer_plan to organise bullets into
// a structured answer plan. request must contain bullets.
func (c *Client) BuildAnswerPlan(ctx context.Context, request VectorizerValue) (AnswerPlanResult, error) {
	v, err := c.Call(ctx, "discovery.build_answer_plan", []VectorizerValue{request})
	if err != nil {
		return AnswerPlanResult{}, err
	}
	var sections []AnswerPlanSection
	if sv, ok := v.MapGet("sections"); ok {
		if arr, ok := sv.AsArray(); ok {
			sections = make([]AnswerPlanSection, len(arr))
			for i, entry := range arr {
				sections[i] = AnswerPlanSection{
					Title:        getStr(entry, "title", ""),
					BulletsCount: getInt(entry, "bullets_count"),
				}
			}
		}
	}
	return AnswerPlanResult{
		Sections:     sections,
		TotalBullets: getInt(v, "total_bullets"),
	}, nil
}

// RenderLlmPrompt issues discovery.render_llm_prompt to render an answer
// plan into an LLM prompt string. request must contain plan.
func (c *Client) RenderLlmPrompt(ctx context.Context, request VectorizerValue) (RenderPromptResult, error) {
	v, err := c.Call(ctx, "discovery.render_llm_prompt", []VectorizerValue{request})
	if err != nil {
		return RenderPromptResult{}, err
	}
	prompt, err := requireStr(v, "discovery.render_llm_prompt", "prompt")
	if err != nil {
		return RenderPromptResult{}, err
	}
	return RenderPromptResult{
		Prompt:          prompt,
		Length:          getInt(v, "length"),
		EstimatedTokens: getInt(v, "estimated_tokens"),
	}, nil
}

// ═════════════════════════════════════════════════════════════════
// File ops
// ═════════════════════════════════════════════════════════════════

// FileContent issues file.content to retrieve raw file content stored in a
// collection. request must contain collection and file_path.
func (c *Client) FileContent(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.content", []VectorizerValue{request})
}

// FileList issues file.list to list files indexed in a collection.
// request must contain collection.
func (c *Client) FileList(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.list", []VectorizerValue{request})
}

// FileSummary issues file.summary for extractive or structural summary of one
// file. request must contain collection and file_path.
func (c *Client) FileSummary(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.summary", []VectorizerValue{request})
}

// FileChunks issues file.chunks to retrieve ordered chunks for one file.
// request must contain collection and file_path.
func (c *Client) FileChunks(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.chunks", []VectorizerValue{request})
}

// FileOutline issues file.outline to get a directory-tree outline of a
// collection's files. request must contain collection.
func (c *Client) FileOutline(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.outline", []VectorizerValue{request})
}

// FileRelated issues file.related to find files semantically related to a
// given file. request must contain collection and file_path.
func (c *Client) FileRelated(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.related", []VectorizerValue{request})
}

// FileSearchByType issues file.search_by_type to search within files of
// specific extension types. request must contain collection, query, and file_types.
func (c *Client) FileSearchByType(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "file.search_by_type", []VectorizerValue{request})
}

// ═════════════════════════════════════════════════════════════════
// Graph
// ═════════════════════════════════════════════════════════════════

// GraphListNodes issues graph.list_nodes to list all graph nodes in a
// collection. Returns the raw VectorizerValue response.
func (c *Client) GraphListNodes(ctx context.Context, collection string) (VectorizerValue, error) {
	return c.Call(ctx, "graph.list_nodes", []VectorizerValue{StrValue(collection)})
}

// GraphNeighbors issues graph.neighbors to fetch direct neighbors of a node.
func (c *Client) GraphNeighbors(ctx context.Context, collection, nodeID string) (VectorizerValue, error) {
	return c.Call(ctx, "graph.neighbors", []VectorizerValue{StrValue(collection), StrValue(nodeID)})
}

// GraphFindRelated issues graph.find_related to find nodes reachable within
// maxHops of a given node.
func (c *Client) GraphFindRelated(ctx context.Context, collection, nodeID string, maxHops int64) (VectorizerValue, error) {
	return c.Call(ctx, "graph.find_related", []VectorizerValue{
		StrValue(collection),
		StrValue(nodeID),
		IntValue(maxHops),
	})
}

// GraphFindPath issues graph.find_path to find the shortest path between two
// graph nodes.
func (c *Client) GraphFindPath(ctx context.Context, collection, from, to string) (VectorizerValue, error) {
	return c.Call(ctx, "graph.find_path", []VectorizerValue{
		StrValue(collection),
		StrValue(from),
		StrValue(to),
	})
}

// GraphCreateEdge issues graph.create_edge to create a directed edge between
// two nodes. edge must contain source, target, and relationship_type.
func (c *Client) GraphCreateEdge(ctx context.Context, collection string, edge VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "graph.create_edge", []VectorizerValue{StrValue(collection), edge})
}

// GraphDeleteEdge issues graph.delete_edge to remove an edge by its id.
func (c *Client) GraphDeleteEdge(ctx context.Context, collection, edgeID string) (VectorizerValue, error) {
	return c.Call(ctx, "graph.delete_edge", []VectorizerValue{StrValue(collection), StrValue(edgeID)})
}

// GraphListEdges issues graph.list_edges to list all edges in a collection's
// graph.
func (c *Client) GraphListEdges(ctx context.Context, collection string) (VectorizerValue, error) {
	return c.Call(ctx, "graph.list_edges", []VectorizerValue{StrValue(collection)})
}

// GraphDiscoverEdges issues graph.discover_edges to auto-discover edges by
// vector similarity across the whole collection. request is a Map with
// optional similarity_threshold (Float) and max_per_node (Int).
func (c *Client) GraphDiscoverEdges(ctx context.Context, collection string, request VectorizerValue) (DiscoverEdgesResult, error) {
	v, err := c.Call(ctx, "graph.discover_edges", []VectorizerValue{StrValue(collection), request})
	if err != nil {
		return DiscoverEdgesResult{}, err
	}
	return DiscoverEdgesResult{
		Success:           getBool(v, "success"),
		TotalNodes:        getInt(v, "total_nodes"),
		NodesProcessed:    getInt(v, "nodes_processed"),
		NodesWithEdges:    getInt(v, "nodes_with_edges"),
		TotalEdgesCreated: getInt(v, "total_edges_created"),
	}, nil
}

// GraphDiscoverEdgesForNode issues graph.discover_edges_for_node to
// auto-discover edges for one specific node.
func (c *Client) GraphDiscoverEdgesForNode(ctx context.Context, collection, nodeID string, request VectorizerValue) (DiscoverEdgesForNodeResult, error) {
	v, err := c.Call(ctx, "graph.discover_edges_for_node", []VectorizerValue{
		StrValue(collection),
		StrValue(nodeID),
		request,
	})
	if err != nil {
		return DiscoverEdgesForNodeResult{}, err
	}
	return DiscoverEdgesForNodeResult{
		Success:      getBool(v, "success"),
		NodeID:       getStr(v, "node_id", nodeID),
		EdgesCreated: getInt(v, "edges_created"),
	}, nil
}

// GraphDiscoveryStatus issues graph.discovery_status to get the percentage of
// nodes that have been assigned edges.
func (c *Client) GraphDiscoveryStatus(ctx context.Context, collection string) (GraphDiscoveryStatus, error) {
	v, err := c.Call(ctx, "graph.discovery_status", []VectorizerValue{StrValue(collection)})
	if err != nil {
		return GraphDiscoveryStatus{}, err
	}
	return GraphDiscoveryStatus{
		TotalNodes:         getInt(v, "total_nodes"),
		NodesWithEdges:     getInt(v, "nodes_with_edges"),
		TotalEdges:         getInt(v, "total_edges"),
		ProgressPercentage: getFloat(v, "progress_percentage"),
	}, nil
}

// ═════════════════════════════════════════════════════════════════
// Admin / observability
// ═════════════════════════════════════════════════════════════════

// AdminStats issues admin.stats to retrieve aggregate vector and collection
// counts.
func (c *Client) AdminStats(ctx context.Context) (AdminStats, error) {
	v, err := c.Call(ctx, "admin.stats", nil)
	if err != nil {
		return AdminStats{}, err
	}
	return AdminStats{
		CollectionsCount: getInt(v, "collections_count"),
		TotalVectors:     getInt(v, "total_vectors"),
		Version:          getStr(v, "version", ""),
	}, nil
}

// AdminStatus issues admin.status as a readiness probe with basic counts.
func (c *Client) AdminStatus(ctx context.Context) (AdminStatus, error) {
	v, err := c.Call(ctx, "admin.status", nil)
	if err != nil {
		return AdminStatus{}, err
	}
	return AdminStatus{
		Ready:            getBool(v, "ready"),
		CollectionsCount: getInt(v, "collections_count"),
		Version:          getStr(v, "version", ""),
	}, nil
}

// AdminLogs issues admin.logs to retrieve in-process log entries.
// request is optional; pass NullValue() for no filter.
func (c *Client) AdminLogs(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	var args []VectorizerValue
	if request.Kind != ValueNull {
		args = []VectorizerValue{request}
	}
	return c.Call(ctx, "admin.logs", args)
}

// AdminIndexingProgress issues admin.indexing_progress to report how many
// collections have been indexed.
func (c *Client) AdminIndexingProgress(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "admin.indexing_progress", nil)
}

// AdminConfigGet issues admin.config_get to read the server's config.yml.
func (c *Client) AdminConfigGet(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "admin.config_get", nil)
}

// AdminConfigUpdate issues admin.config_update to write a patch map to
// config.yml. patch is a Map of config keys to new values. Returns success.
func (c *Client) AdminConfigUpdate(ctx context.Context, patch VectorizerValue) (bool, error) {
	v, err := c.Call(ctx, "admin.config_update", []VectorizerValue{patch})
	if err != nil {
		return false, err
	}
	return requireBool(v, "admin.config_update", "success")
}

// AdminBackupsList issues admin.backups_list to list available backup files.
func (c *Client) AdminBackupsList(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "admin.backups_list", nil)
}

// AdminBackupsCreate issues admin.backups_create to create a backup.
// request must contain name. Returns the backup_id.
func (c *Client) AdminBackupsCreate(ctx context.Context, request VectorizerValue) (string, error) {
	v, err := c.Call(ctx, "admin.backups_create", []VectorizerValue{request})
	if err != nil {
		return "", err
	}
	return requireStr(v, "admin.backups_create", "backup_id")
}

// AdminBackupsRestore issues admin.backups_restore to restore a backup.
// request must contain backup_id. Returns success.
func (c *Client) AdminBackupsRestore(ctx context.Context, request VectorizerValue) (bool, error) {
	v, err := c.Call(ctx, "admin.backups_restore", []VectorizerValue{request})
	if err != nil {
		return false, err
	}
	return requireBool(v, "admin.backups_restore", "success")
}

// AdminWorkspacesList issues admin.workspaces_list to list configured
// workspaces.
func (c *Client) AdminWorkspacesList(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "admin.workspaces_list", nil)
}

// AdminWorkspaceGet issues admin.workspace_get to read workspace.yml.
func (c *Client) AdminWorkspaceGet(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "admin.workspace_get", nil)
}

// AdminWorkspaceAdd issues admin.workspace_add to register a new workspace
// directory. request must contain path and collection_name.
func (c *Client) AdminWorkspaceAdd(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "admin.workspace_add", []VectorizerValue{request})
}

// AdminWorkspaceRemove issues admin.workspace_remove to remove a workspace by
// name. Returns success.
func (c *Client) AdminWorkspaceRemove(ctx context.Context, name string) (bool, error) {
	v, err := c.Call(ctx, "admin.workspace_remove", []VectorizerValue{StrValue(name)})
	if err != nil {
		return false, err
	}
	return requireBool(v, "admin.workspace_remove", "success")
}

// AdminRestart issues admin.restart to schedule a server restart.
func (c *Client) AdminRestart(ctx context.Context) (bool, error) {
	v, err := c.Call(ctx, "admin.restart", nil)
	if err != nil {
		return false, err
	}
	return requireBool(v, "admin.restart", "success")
}

// AdminSlowQueriesList issues admin.slow_queries_list to retrieve the
// slow-query ring buffer.
func (c *Client) AdminSlowQueriesList(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "admin.slow_queries_list", nil)
}

// AdminSlowQueriesConfig issues admin.slow_queries_config to configure the
// slow-query threshold and capacity. config must contain threshold_ms.
func (c *Client) AdminSlowQueriesConfig(ctx context.Context, config VectorizerValue) (SlowQueryConfigResult, error) {
	v, err := c.Call(ctx, "admin.slow_queries_config", []VectorizerValue{config})
	if err != nil {
		return SlowQueryConfigResult{}, err
	}
	return SlowQueryConfigResult{
		ThresholdMs: getInt(v, "threshold_ms"),
		Capacity:    getInt(v, "capacity"),
		Status:      getStr(v, "status", "ok"),
	}, nil
}

// ═════════════════════════════════════════════════════════════════
// Auth / RBAC
// ═════════════════════════════════════════════════════════════════

// AuthMe issues auth.me to return the authenticated principal's identity.
func (c *Client) AuthMe(ctx context.Context) (AuthMeResult, error) {
	v, err := c.Call(ctx, "auth.me", nil)
	if err != nil {
		return AuthMeResult{}, err
	}
	return AuthMeResult{
		Username:      getStr(v, "username", "unknown"),
		Authenticated: getBool(v, "authenticated"),
	}, nil
}

// AuthLogout issues auth.logout to blacklist the supplied JWT so it cannot be
// reused.
func (c *Client) AuthLogout(ctx context.Context, token string) (VectorizerValue, error) {
	return c.Call(ctx, "auth.logout", []VectorizerValue{StrValue(token)})
}

// AuthRefreshToken issues auth.refresh_token to exchange a valid JWT for a
// fresh one.
func (c *Client) AuthRefreshToken(ctx context.Context, token string) (RefreshTokenResult, error) {
	v, err := c.Call(ctx, "auth.refresh_token", []VectorizerValue{StrValue(token)})
	if err != nil {
		return RefreshTokenResult{}, err
	}
	at, err := requireStr(v, "auth.refresh_token", "access_token")
	if err != nil {
		return RefreshTokenResult{}, err
	}
	return RefreshTokenResult{
		AccessToken: at,
		TokenType:   getStr(v, "token_type", "Bearer"),
	}, nil
}

// AuthValidatePassword issues auth.validate_password to check a plaintext
// password against the server's password policy.
func (c *Client) AuthValidatePassword(ctx context.Context, password string) (ValidatePasswordResult, error) {
	v, err := c.Call(ctx, "auth.validate_password", []VectorizerValue{StrValue(password)})
	if err != nil {
		return ValidatePasswordResult{}, err
	}
	var errors []string
	if ev, ok := v.MapGet("errors"); ok {
		if arr, ok := ev.AsArray(); ok {
			for _, item := range arr {
				if s, ok := item.AsStr(); ok && s != "" {
					errors = append(errors, s)
				}
			}
		}
	}
	return ValidatePasswordResult{
		Valid:  getBool(v, "valid"),
		Errors: errors,
	}, nil
}

// AuthApiKeysCreate issues auth.api_keys_create. request must contain name.
func (c *Client) AuthApiKeysCreate(ctx context.Context, request VectorizerValue) (ApiKeyCreated, error) {
	v, err := c.Call(ctx, "auth.api_keys_create", []VectorizerValue{request})
	if err != nil {
		return ApiKeyCreated{}, err
	}
	key, err := requireStr(v, "auth.api_keys_create", "api_key")
	if err != nil {
		return ApiKeyCreated{}, err
	}
	id, err := requireStr(v, "auth.api_keys_create", "id")
	if err != nil {
		return ApiKeyCreated{}, err
	}
	name, err := requireStr(v, "auth.api_keys_create", "name")
	if err != nil {
		return ApiKeyCreated{}, err
	}
	return ApiKeyCreated{APIKey: key, ID: id, Name: name}, nil
}

// AuthApiKeysList issues auth.api_keys_list to list API keys for the current
// principal.
func (c *Client) AuthApiKeysList(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "auth.api_keys_list", nil)
}

// AuthApiKeysRevoke issues auth.api_keys_revoke to permanently revoke an API
// key by id.
func (c *Client) AuthApiKeysRevoke(ctx context.Context, keyID string) (bool, error) {
	v, err := c.Call(ctx, "auth.api_keys_revoke", []VectorizerValue{StrValue(keyID)})
	if err != nil {
		return false, err
	}
	return requireBool(v, "auth.api_keys_revoke", "success")
}

// RotateApiKeyRpc issues auth.api_keys_rotate to rotate an API key with a
// 5-minute grace period. Named RotateApiKeyRpc to avoid collision with the
// REST SDK's RotateApiKey.
func (c *Client) RotateApiKeyRpc(ctx context.Context, keyID string) (RotatedApiKey, error) {
	v, err := c.Call(ctx, "auth.api_keys_rotate", []VectorizerValue{StrValue(keyID)})
	if err != nil {
		return RotatedApiKey{}, err
	}
	oldID, err := requireStr(v, "auth.api_keys_rotate", "old_key_id")
	if err != nil {
		return RotatedApiKey{}, err
	}
	newID, err := requireStr(v, "auth.api_keys_rotate", "new_key_id")
	if err != nil {
		return RotatedApiKey{}, err
	}
	newToken, err := requireStr(v, "auth.api_keys_rotate", "new_token")
	if err != nil {
		return RotatedApiKey{}, err
	}
	return RotatedApiKey{
		OldKeyID:   oldID,
		NewKeyID:   newID,
		NewToken:   newToken,
		GraceUntil: getOptionalStr(v, "grace_until"),
	}, nil
}

// AuthApiKeysCreateScoped issues auth.api_keys_create_scoped to create a
// collection-scoped API key. request must contain name.
func (c *Client) AuthApiKeysCreateScoped(ctx context.Context, request VectorizerValue) (ApiKeyCreated, error) {
	v, err := c.Call(ctx, "auth.api_keys_create_scoped", []VectorizerValue{request})
	if err != nil {
		return ApiKeyCreated{}, err
	}
	key, err := requireStr(v, "auth.api_keys_create_scoped", "api_key")
	if err != nil {
		return ApiKeyCreated{}, err
	}
	id, err := requireStr(v, "auth.api_keys_create_scoped", "id")
	if err != nil {
		return ApiKeyCreated{}, err
	}
	name, err := requireStr(v, "auth.api_keys_create_scoped", "name")
	if err != nil {
		return ApiKeyCreated{}, err
	}
	return ApiKeyCreated{APIKey: key, ID: id, Name: name}, nil
}

// AuthIntrospect issues auth.introspect to inspect a token's claims and
// blacklist status.
func (c *Client) AuthIntrospect(ctx context.Context, token string) (VectorizerValue, error) {
	return c.Call(ctx, "auth.introspect", []VectorizerValue{StrValue(token)})
}

// AuthAudit issues auth.audit to query the auth audit log. request is a Map
// with optional from, to, actor, action, limit.
func (c *Client) AuthAudit(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "auth.audit", []VectorizerValue{request})
}

// ═════════════════════════════════════════════════════════════════
// Replication
// ═════════════════════════════════════════════════════════════════

// ReplicationStatus issues replication.status to return the current
// replication role and replica list.
func (c *Client) ReplicationStatus(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "replication.status", nil)
}

// ReplicationConfigure issues replication.configure to set the replication
// role for this node. config must contain role.
func (c *Client) ReplicationConfigure(ctx context.Context, config VectorizerValue) (ReplicationConfigureResult, error) {
	v, err := c.Call(ctx, "replication.configure", []VectorizerValue{config})
	if err != nil {
		return ReplicationConfigureResult{}, err
	}
	success, err := requireBool(v, "replication.configure", "success")
	if err != nil {
		return ReplicationConfigureResult{}, err
	}
	role, err := requireStr(v, "replication.configure", "role")
	if err != nil {
		return ReplicationConfigureResult{}, err
	}
	return ReplicationConfigureResult{
		Success: success,
		Role:    role,
		Message: getStr(v, "message", ""),
	}, nil
}

// ReplicationStats issues replication.stats to retrieve replication
// throughput and lag statistics.
func (c *Client) ReplicationStats(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "replication.stats", nil)
}

// ReplicationReplicasList issues replication.replicas_list to list connected
// replicas (master only).
func (c *Client) ReplicationReplicasList(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "replication.replicas_list", nil)
}

// ═════════════════════════════════════════════════════════════════
// Cluster
// ═════════════════════════════════════════════════════════════════

// ClusterFailover issues cluster.failover to promote a replica to master.
func (c *Client) ClusterFailover(ctx context.Context, replicaID string) (VectorizerValue, error) {
	return c.Call(ctx, "cluster.failover", []VectorizerValue{StrValue(replicaID)})
}

// ClusterReplicaResync issues cluster.replica_resync to force a replica to
// resync from master.
func (c *Client) ClusterReplicaResync(ctx context.Context, replicaID string) (VectorizerValue, error) {
	return c.Call(ctx, "cluster.replica_resync", []VectorizerValue{StrValue(replicaID)})
}

// ClusterPeerAdd issues cluster.peer_add to add a new peer to the cluster.
// request must contain address.
func (c *Client) ClusterPeerAdd(ctx context.Context, request VectorizerValue) (VectorizerValue, error) {
	return c.Call(ctx, "cluster.peer_add", []VectorizerValue{request})
}

// ClusterRebalance issues cluster.rebalance to trigger a shard rebalance
// across peers.
func (c *Client) ClusterRebalance(ctx context.Context) (VectorizerValue, error) {
	return c.Call(ctx, "cluster.rebalance", nil)
}

// ClusterRebalanceStatus issues cluster.rebalance_status to check the status
// of an in-progress rebalance, or confirm idle.
func (c *Client) ClusterRebalanceStatus(ctx context.Context) (RebalanceStatus, error) {
	v, err := c.Call(ctx, "cluster.rebalance_status", nil)
	if err != nil {
		return RebalanceStatus{}, err
	}
	return RebalanceStatus{
		Status:  getOptionalStr(v, "status"),
		Message: getOptionalStr(v, "message"),
	}, nil
}
