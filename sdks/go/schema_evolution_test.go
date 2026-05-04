package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestSchemaRenameCollection(t *testing.T) {
	var capturedBody map[string]string

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/my-col/rename" {
			t.Errorf("should call /collections/my-col/rename but got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body but decode failed: %v", err)
		}

		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.RenameCollection("my-col", "my-col-v2")
	if err != nil {
		t.Fatalf("RenameCollection failed: %v", err)
	}

	if capturedBody["new_name"] != "my-col-v2" {
		t.Errorf("POST body should contain new_name=my-col-v2 but got %v", capturedBody["new_name"])
	}
}

func TestSchemaReindexCollection(t *testing.T) {
	var capturedBody map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/test-col/reindex" {
			t.Errorf("should call /collections/test-col/reindex but got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body but decode failed: %v", err)
		}

		resp := ReindexJob{
			JobID:      "job-001",
			Collection: "test-col",
			State:      "completed",
			Params: map[string]interface{}{
				"m":               float64(16),
				"ef_construction": float64(200),
				"ef_search":       float64(100),
			},
			Progress: 1.0,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	params := &ReindexParams{M: 16, EfConstruction: 200, EfSearch: 100}
	job, err := client.ReindexCollection("test-col", params)
	if err != nil {
		t.Fatalf("ReindexCollection failed: %v", err)
	}
	if job == nil {
		t.Fatal("should return non-nil ReindexJob")
	}
	if job.JobID != "job-001" {
		t.Errorf("should decode job_id as job-001 but got %s", job.JobID)
	}
	if job.State != "completed" {
		t.Errorf("should decode state as completed but got %s", job.State)
	}
	if job.Collection != "test-col" {
		t.Errorf("should decode collection as test-col but got %s", job.Collection)
	}

	if capturedBody["m"] != float64(16) {
		t.Errorf("POST body should contain m=16 but got %v", capturedBody["m"])
	}
	if capturedBody["ef_construction"] != float64(200) {
		t.Errorf("POST body should contain ef_construction=200 but got %v", capturedBody["ef_construction"])
	}
	if capturedBody["ef_search"] != float64(100) {
		t.Errorf("POST body should contain ef_search=100 but got %v", capturedBody["ef_search"])
	}
}

func TestSchemaSnapshotCollectionNative(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/my-col/snapshot" {
			t.Errorf("should call /collections/my-col/snapshot but got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}

		resp := NativeSnapshotInfo{
			ID:         "snap-abc",
			Collection: "my-col",
			CreatedAt:  "2026-05-03T00:00:00Z",
			SizeBytes:  4096,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	info, err := client.SnapshotCollectionNative("my-col")
	if err != nil {
		t.Fatalf("SnapshotCollectionNative failed: %v", err)
	}
	if info == nil {
		t.Fatal("should return non-nil NativeSnapshotInfo")
	}
	if info.ID != "snap-abc" {
		t.Errorf("should decode id as snap-abc but got %s", info.ID)
	}
	if info.Collection != "my-col" {
		t.Errorf("should decode collection as my-col but got %s", info.Collection)
	}
	if info.SizeBytes != 4096 {
		t.Errorf("should decode size_bytes as 4096 but got %d", info.SizeBytes)
	}
}

func TestSchemaListCollectionSnapshotsNative(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/my-col/snapshots" {
			t.Errorf("should call /collections/my-col/snapshots but got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("should use GET method but got %s", r.Method)
		}

		type envelope struct {
			Snapshots []NativeSnapshotInfo `json:"snapshots"`
		}
		resp := envelope{
			Snapshots: []NativeSnapshotInfo{
				{ID: "snap-1", Collection: "my-col", CreatedAt: "2026-05-03T00:00:00Z", SizeBytes: 1024},
				{ID: "snap-2", Collection: "my-col", CreatedAt: "2026-05-03T00:01:00Z", SizeBytes: 2048},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	snapshots, err := client.ListCollectionSnapshotsNative("my-col")
	if err != nil {
		t.Fatalf("ListCollectionSnapshotsNative failed: %v", err)
	}
	if len(snapshots) != 2 {
		t.Fatalf("should unwrap snapshots envelope and return 2 items but got %d", len(snapshots))
	}
	if snapshots[0].ID != "snap-1" {
		t.Errorf("first snapshot should have ID snap-1 but got %s", snapshots[0].ID)
	}
	if snapshots[1].ID != "snap-2" {
		t.Errorf("second snapshot should have ID snap-2 but got %s", snapshots[1].ID)
	}
}

func TestSchemaRestoreCollectionSnapshotNative(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/my-col/snapshots/snap-abc/restore" {
			t.Errorf("should call /collections/my-col/snapshots/snap-abc/restore but got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}

		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.RestoreCollectionSnapshotNative("my-col", "snap-abc")
	if err != nil {
		t.Fatalf("RestoreCollectionSnapshotNative failed: %v", err)
	}
}

func TestSchemaExplainSearch(t *testing.T) {
	var capturedBody map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/my-col/explain" {
			t.Errorf("should call /collections/my-col/explain but got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body but decode failed: %v", err)
		}

		resp := ExplainResponse{
			Collection: "my-col",
			K:          5,
			Results:    []map[string]interface{}{},
			Trace: ExplainTrace{
				VisitedNodes:        42,
				EfSearch:            100,
				HnswSearchMs:        1.5,
				PayloadFilterEvals:  0,
				QuantizationScoreMs: 0.3,
				TotalMs:             2.1,
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	vector := []float32{0.1, 0.2, 0.3}
	result, err := client.ExplainSearch("my-col", vector, 5)
	if err != nil {
		t.Fatalf("ExplainSearch failed: %v", err)
	}
	if result == nil {
		t.Fatal("should return non-nil ExplainResponse")
	}
	if result.Collection != "my-col" {
		t.Errorf("should decode collection as my-col but got %s", result.Collection)
	}
	if result.K != 5 {
		t.Errorf("should decode k as 5 but got %d", result.K)
	}
	if result.Trace.VisitedNodes != 42 {
		t.Errorf("should decode trace.visited_nodes as 42 but got %d", result.Trace.VisitedNodes)
	}
	if result.Trace.TotalMs != 2.1 {
		t.Errorf("should decode trace.total_ms as 2.1 but got %f", result.Trace.TotalMs)
	}

	if capturedBody["k"] != float64(5) {
		t.Errorf("POST body should contain k=5 but got %v", capturedBody["k"])
	}
	rawVec, ok := capturedBody["vector"].([]interface{})
	if !ok {
		t.Fatalf("POST body should contain a vector array but got %T", capturedBody["vector"])
	}
	if len(rawVec) != 3 {
		t.Errorf("POST body vector should have 3 elements but got %d", len(rawVec))
	}
}

func TestSchemaListSlowQueries(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/slow_queries" {
			t.Errorf("should call /slow_queries but got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("should use GET method but got %s", r.Method)
		}

		type envelope struct {
			Entries []SlowQueryEntry `json:"entries"`
		}
		resp := envelope{
			Entries: []SlowQueryEntry{
				{Timestamp: "2026-05-03T00:00:00Z", Collection: "col-a", K: 10, DurationMs: 150.5},
				{Timestamp: "2026-05-03T00:01:00Z", Collection: "col-b", K: 5, DurationMs: 200.0},
			},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	entries, err := client.ListSlowQueries()
	if err != nil {
		t.Fatalf("ListSlowQueries failed: %v", err)
	}
	if len(entries) != 2 {
		t.Fatalf("should unwrap entries envelope and return 2 items but got %d", len(entries))
	}
	if entries[0].Collection != "col-a" {
		t.Errorf("first entry should have collection col-a but got %s", entries[0].Collection)
	}
	if entries[0].DurationMs != 150.5 {
		t.Errorf("first entry should have duration_ms 150.5 but got %f", entries[0].DurationMs)
	}
	if entries[1].Collection != "col-b" {
		t.Errorf("second entry should have collection col-b but got %s", entries[1].Collection)
	}
}

func TestSchemaSetSlowQueryConfig(t *testing.T) {
	var capturedBody map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/slow_queries/config" {
			t.Errorf("should call /slow_queries/config but got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("should use POST method but got %s", r.Method)
		}

		if err := json.NewDecoder(r.Body).Decode(&capturedBody); err != nil {
			t.Fatalf("should receive valid JSON body but decode failed: %v", err)
		}

		resp := SlowQueryConfig{
			ThresholdMs: 500,
			Capacity:    200,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	cfg := &SlowQueryConfig{ThresholdMs: 500, Capacity: 200}
	result, err := client.SetSlowQueryConfig(cfg)
	if err != nil {
		t.Fatalf("SetSlowQueryConfig failed: %v", err)
	}
	if result == nil {
		t.Fatal("should return non-nil SlowQueryConfig")
	}
	if result.ThresholdMs != 500 {
		t.Errorf("should decode threshold_ms as 500 but got %d", result.ThresholdMs)
	}
	if result.Capacity != 200 {
		t.Errorf("should decode capacity as 200 but got %d", result.Capacity)
	}

	if capturedBody["threshold_ms"] != float64(500) {
		t.Errorf("POST body should contain threshold_ms=500 but got %v", capturedBody["threshold_ms"])
	}
	if capturedBody["capacity"] != float64(200) {
		t.Errorf("POST body should contain capacity=200 but got %v", capturedBody["capacity"])
	}
}
