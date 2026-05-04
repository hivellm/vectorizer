package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// TestTierDeleteByFilter verifies that DeleteByFilter POSTs to the correct
// endpoint and deserialises the DeleteByFilterReport response fields.
func TestTierDeleteByFilter(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}
		if r.URL.Path != "/collections/c1/vectors/delete_by_filter" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}
		if _, ok := body["filter"]; !ok {
			t.Error("POST body should contain a filter key")
		}

		resp := DeleteByFilterReport{
			Scanned: 10,
			Matched: 4,
			Deleted: 4,
			Results: []VectorOpResult{
				{Status: "ok"},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	report, err := client.DeleteByFilterRaw("c1", map[string]interface{}{"status": "stale"})
	if err != nil {
		t.Fatalf("DeleteByFilter failed: %v", err)
	}
	if report == nil {
		t.Fatal("should return a non-nil DeleteByFilterReport")
	}
	if report.Scanned != 10 {
		t.Errorf("Scanned should be 10 but got %d", report.Scanned)
	}
	if report.Matched != 4 {
		t.Errorf("Matched should be 4 but got %d", report.Matched)
	}
	if report.Deleted != 4 {
		t.Errorf("Deleted should be 4 but got %d", report.Deleted)
	}
	if len(report.Results) != 1 {
		t.Errorf("Results should have 1 entry but got %d", len(report.Results))
	}
}

// TestTierDeleteByFilterEmptyFails verifies that an empty filter is rejected
// client-side before any HTTP request is issued (raw variant).
func TestTierDeleteByFilterEmptyFails(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("server should not be called")
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	// nil filter (len == 0) must be rejected client-side.
	_, err := client.DeleteByFilterRaw("c1", nil)
	if err == nil {
		t.Fatal("should return an error for nil filter")
	}
	ve, ok := err.(*VectorizerError)
	if !ok {
		t.Fatalf("error should be *VectorizerError but got %T", err)
	}
	if ve.Type != "validation_error" {
		t.Errorf("error Type should be validation_error but got %q", ve.Type)
	}

	// Empty non-nil map must also be rejected client-side.
	_, err = client.DeleteByFilterRaw("c1", map[string]interface{}{})
	if err == nil {
		t.Fatal("should return an error for empty filter map")
	}
	ve, ok = err.(*VectorizerError)
	if !ok {
		t.Fatalf("error should be *VectorizerError but got %T", err)
	}
	if ve.Type != "validation_error" {
		t.Errorf("error Type should be validation_error but got %q", ve.Type)
	}
}

// TestTierBulkUpdateMetadata verifies that BulkUpdateMetadata POSTs the
// correct body and deserialises the BulkUpdateReport response fields.
func TestTierBulkUpdateMetadata(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}
		if r.URL.Path != "/collections/col2/vectors/bulk_update_metadata" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}
		if _, ok := body["filter"]; !ok {
			t.Error("POST body should contain a filter key")
		}
		if _, ok := body["patch"]; !ok {
			t.Error("POST body should contain a patch key")
		}

		resp := BulkUpdateReport{
			Scanned: 5,
			Matched: 3,
			Updated: 3,
			Results: []VectorOpResult{
				{Status: "ok"},
				{Status: "ok"},
				{Status: "ok"},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	report, err := client.BulkUpdateMetadataRaw(
		"col2",
		map[string]interface{}{"category": "A"},
		map[string]interface{}{"tag": "updated"},
	)
	if err != nil {
		t.Fatalf("BulkUpdateMetadata failed: %v", err)
	}
	if report == nil {
		t.Fatal("should return a non-nil BulkUpdateReport")
	}
	if report.Scanned != 5 {
		t.Errorf("Scanned should be 5 but got %d", report.Scanned)
	}
	if report.Matched != 3 {
		t.Errorf("Matched should be 3 but got %d", report.Matched)
	}
	if report.Updated != 3 {
		t.Errorf("Updated should be 3 but got %d", report.Updated)
	}
}

// TestTierBulkUpdateMetadataEmptyFails verifies that an empty filter is
// rejected client-side before any HTTP request is issued (raw variant).
func TestTierBulkUpdateMetadataEmptyFails(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("server should not be called")
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	// nil filter must be rejected client-side.
	_, err := client.BulkUpdateMetadataRaw("col2", nil, map[string]interface{}{"tag": "x"})
	if err == nil {
		t.Fatal("should return an error for nil filter")
	}
	ve, ok := err.(*VectorizerError)
	if !ok {
		t.Fatalf("error should be *VectorizerError but got %T", err)
	}
	if ve.Type != "validation_error" {
		t.Errorf("error Type should be validation_error but got %q", ve.Type)
	}

	// Empty non-nil map must also be rejected client-side.
	_, err = client.BulkUpdateMetadataRaw("col2", map[string]interface{}{}, map[string]interface{}{"tag": "x"})
	if err == nil {
		t.Fatal("should return an error for empty filter map")
	}
	ve, ok = err.(*VectorizerError)
	if !ok {
		t.Fatalf("error should be *VectorizerError but got %T", err)
	}
	if ve.Type != "validation_error" {
		t.Errorf("error Type should be validation_error but got %q", ve.Type)
	}
}

// TestTierCopyVectors verifies that CopyVectors POSTs the correct body with
// destination and ids fields.
func TestTierCopyVectors(t *testing.T) {
	var capturedBody map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}
		if r.URL.Path != "/collections/src_col/vectors/copy" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}

		resp := CopyReport{
			Src:       "src_col",
			Dst:       "dst_col",
			Requested: 2,
			Copied:    2,
			Failed:    0,
			Results: []VectorOpResult{
				{Status: "ok"},
				{Status: "ok"},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	report, err := client.CopyVectors("src_col", "dst_col", []string{"id1", "id2"})
	if err != nil {
		t.Fatalf("CopyVectors failed: %v", err)
	}
	if report == nil {
		t.Fatal("should return a non-nil CopyReport")
	}
	if report.Src != "src_col" {
		t.Errorf("Src should be src_col but got %q", report.Src)
	}
	if report.Dst != "dst_col" {
		t.Errorf("Dst should be dst_col but got %q", report.Dst)
	}
	if report.Requested != 2 {
		t.Errorf("Requested should be 2 but got %d", report.Requested)
	}
	if report.Copied != 2 {
		t.Errorf("Copied should be 2 but got %d", report.Copied)
	}
	if report.Failed != 0 {
		t.Errorf("Failed should be 0 but got %d", report.Failed)
	}

	// Verify POST body shape.
	if capturedBody["destination"] != "dst_col" {
		t.Errorf("POST body destination should be dst_col but got %v", capturedBody["destination"])
	}
	ids, ok := capturedBody["ids"].([]interface{})
	if !ok {
		t.Fatalf("POST body ids should be an array but got %T", capturedBody["ids"])
	}
	if len(ids) != 2 {
		t.Errorf("POST body ids should have 2 elements but got %d", len(ids))
	}
}

// TestTierReencodeCollection verifies that ReencodeCollection POSTs the
// correct body and deserialises the ReencodeJob response.
func TestTierReencodeCollection(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}
		if r.URL.Path != "/collections/my_col/reencode" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}
		if body["target_encoding"] != "sq8" {
			t.Errorf("POST body target_encoding should be sq8 but got %v", body["target_encoding"])
		}

		resp := ReencodeJob{
			JobID:          "job-abc",
			Collection:     "my_col",
			State:          "completed",
			TargetEncoding: "sq8",
			Progress:       1.0,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	job, err := client.ReencodeCollection("my_col", "sq8")
	if err != nil {
		t.Fatalf("ReencodeCollection failed: %v", err)
	}
	if job == nil {
		t.Fatal("should return a non-nil ReencodeJob")
	}
	if job.JobID != "job-abc" {
		t.Errorf("JobID should be job-abc but got %q", job.JobID)
	}
	if job.State != "completed" {
		t.Errorf("State should be completed but got %q", job.State)
	}
	if job.TargetEncoding != "sq8" {
		t.Errorf("TargetEncoding should be sq8 but got %q", job.TargetEncoding)
	}
	if job.Progress != 1.0 {
		t.Errorf("Progress should be 1.0 but got %f", job.Progress)
	}
}

// TestTierSetCollectionTTL verifies that SetCollectionTTL POSTs to the correct
// path and that a nil ttlSecs is serialised as JSON null.
func TestTierSetCollectionTTL(t *testing.T) {
	var capturedBody map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}
		if r.URL.Path != "/collections/ttl_col/ttl" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}

		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	// nil pointer → JSON null (clears TTL).
	err := client.SetCollectionTTL("ttl_col", nil)
	if err != nil {
		t.Fatalf("SetCollectionTTL(nil) failed: %v", err)
	}
	// json.Decoder decodes JSON null as nil interface{}.
	if ttlVal, exists := capturedBody["ttl_secs"]; exists {
		if ttlVal != nil {
			t.Errorf("ttl_secs should be null when pointer is nil but got %v", ttlVal)
		}
	} else {
		t.Error("POST body should contain ttl_secs key")
	}

	// Non-nil pointer → concrete integer value.
	var ttlSecs int64 = 3600
	err = client.SetCollectionTTL("ttl_col", &ttlSecs)
	if err != nil {
		t.Fatalf("SetCollectionTTL(3600) failed: %v", err)
	}
	// JSON numbers decode as float64 in map[string]interface{}.
	if capturedBody["ttl_secs"] != float64(3600) {
		t.Errorf("ttl_secs should be 3600 but got %v", capturedBody["ttl_secs"])
	}
}

// TestTierSetVectorExpiry verifies that SetVectorExpiry uses PATCH, targets
// the correct path, and serialises a nil expiresAt as JSON null.
func TestTierSetVectorExpiry(t *testing.T) {
	var capturedBody map[string]interface{}
	var capturedMethod string

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		capturedMethod = r.Method
		if r.URL.Path != "/collections/exp_col/vectors/vec1/expiry" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}

		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	// nil pointer → JSON null (clears expiry).
	err := client.SetVectorExpiry("exp_col", "vec1", nil)
	if err != nil {
		t.Fatalf("SetVectorExpiry(nil) failed: %v", err)
	}
	if capturedMethod != "PATCH" {
		t.Errorf("should use PATCH method but got %s", capturedMethod)
	}
	if expiresVal, exists := capturedBody["expires_at"]; exists {
		if expiresVal != nil {
			t.Errorf("expires_at should be null when pointer is nil but got %v", expiresVal)
		}
	} else {
		t.Error("POST body should contain expires_at key")
	}

	// Non-nil pointer → concrete integer value.
	var expiresAt int64 = 9999999999
	err = client.SetVectorExpiry("exp_col", "vec1", &expiresAt)
	if err != nil {
		t.Fatalf("SetVectorExpiry(9999999999) failed: %v", err)
	}
	if capturedMethod != "PATCH" {
		t.Errorf("should use PATCH method but got %s", capturedMethod)
	}
	if capturedBody["expires_at"] != float64(9999999999) {
		t.Errorf("expires_at should be 9999999999 but got %v", capturedBody["expires_at"])
	}
}

// TestTierDeleteByFilterTypedHappy verifies that DeleteByFilter (typed) POSTs
// the correct JSON body shape and decodes the response correctly.
func TestTierDeleteByFilterTypedHappy(t *testing.T) {
	var capturedFilter map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}
		if r.URL.Path != "/collections/typed_col/vectors/delete_by_filter" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should receive valid JSON body: %v", err)
		}

		filterVal, ok := body["filter"]
		if !ok {
			t.Fatal("POST body should contain a filter key")
		}
		capturedFilter, ok = filterVal.(map[string]interface{})
		if !ok {
			t.Fatalf("filter should be an object but got %T", filterVal)
		}

		resp := DeleteByFilterReport{
			Scanned: 8,
			Matched: 2,
			Deleted: 2,
			Results: []VectorOpResult{{Status: "ok"}, {Status: "ok"}},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	f := MustFilter(FilterEq("status", "stale"))
	report, err := client.DeleteByFilter("typed_col", &f)
	if err != nil {
		t.Fatalf("DeleteByFilter typed failed: %v", err)
	}
	if report == nil {
		t.Fatal("should return a non-nil DeleteByFilterReport")
	}
	if report.Deleted != 2 {
		t.Errorf("Deleted should be 2 but got %d", report.Deleted)
	}

	// Assert the wire shape: filter.must[0].key == "status",
	// filter.must[0].match.value == "stale".
	mustArr, ok := capturedFilter["must"].([]interface{})
	if !ok || len(mustArr) == 0 {
		t.Fatalf("filter.must should be a non-empty array but got %v", capturedFilter["must"])
	}
	cond, ok := mustArr[0].(map[string]interface{})
	if !ok {
		t.Fatalf("must[0] should be an object but got %T", mustArr[0])
	}
	if cond["key"] != "status" {
		t.Errorf("must[0].key should be 'status' but got %v", cond["key"])
	}
	matchObj, ok := cond["match"].(map[string]interface{})
	if !ok {
		t.Fatalf("must[0].match should be an object but got %T", cond["match"])
	}
	if matchObj["value"] != "stale" {
		t.Errorf("must[0].match.value should be 'stale' but got %v", matchObj["value"])
	}
}

// TestTierDeleteByFilterTypedEmptyRejectedClientSide verifies that a nil
// filter and an empty *QdrantFilter are both rejected client-side without
// touching the server.
func TestTierDeleteByFilterTypedEmptyRejectedClientSide(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("server should not be called for empty typed filter")
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	// nil pointer must be rejected client-side.
	_, err := client.DeleteByFilter("col", nil)
	if err == nil {
		t.Fatal("should return an error for nil *QdrantFilter")
	}
	ve, ok := err.(*VectorizerError)
	if !ok {
		t.Fatalf("error should be *VectorizerError but got %T", err)
	}
	if ve.Type != "validation_error" {
		t.Errorf("error Type should be validation_error but got %q", ve.Type)
	}

	// Empty (no conditions) filter must also be rejected client-side.
	empty := &QdrantFilter{}
	_, err = client.DeleteByFilter("col", empty)
	if err == nil {
		t.Fatal("should return an error for empty *QdrantFilter")
	}
	ve, ok = err.(*VectorizerError)
	if !ok {
		t.Fatalf("error should be *VectorizerError but got %T", err)
	}
	if ve.Type != "validation_error" {
		t.Errorf("error Type should be validation_error but got %q", ve.Type)
	}
}
