package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestVectorsUpdateVectorPayload(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/update" {
			t.Errorf("should call /update, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["collection"] != "my-col" {
			t.Errorf("should send collection=my-col, got %v", body["collection"])
		}
		if body["id"] != "vec-1" {
			t.Errorf("should send id=vec-1, got %v", body["id"])
		}
		meta, ok := body["metadata"].(map[string]interface{})
		if !ok {
			t.Errorf("should send metadata object, got %T", body["metadata"])
		} else if meta["tag"] != "test" {
			t.Errorf("should send metadata.tag=test, got %v", meta["tag"])
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"message":"updated"}`))
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	vec, err := client.UpdateVectorPayload("my-col", "vec-1", map[string]interface{}{"tag": "test"})
	if err != nil {
		t.Fatalf("UpdateVectorPayload returned error: %v", err)
	}
	if vec == nil {
		t.Fatal("should return non-nil Vector")
	}
	if vec.ID != "vec-1" {
		t.Errorf("should return Vector with ID=vec-1, got %s", vec.ID)
	}
}

func TestVectorsInsertTextWithID(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/insert" {
			t.Errorf("should call /insert, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["collection"] != "col1" {
			t.Errorf("should send collection=col1, got %v", body["collection"])
		}
		if body["id"] != "my-id" {
			t.Errorf("should send id=my-id, got %v", body["id"])
		}
		if body["text"] != "hello world" {
			t.Errorf("should send text=hello world, got %v", body["text"])
		}
		meta, ok := body["metadata"].(map[string]interface{})
		if !ok {
			t.Errorf("should send metadata object, got %T", body["metadata"])
		} else if meta["lang"] != "en" {
			t.Errorf("should send metadata.lang=en, got %v", meta["lang"])
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(InsertTextResponse{
			Message:    "inserted",
			VectorID:   "server-assigned-id",
			Collection: "col1",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	vec, err := client.InsertTextWithID("col1", "my-id", "hello world", map[string]interface{}{"lang": "en"})
	if err != nil {
		t.Fatalf("InsertTextWithID returned error: %v", err)
	}
	if vec == nil {
		t.Fatal("should return non-nil Vector")
	}
	if vec.ID != "server-assigned-id" {
		t.Errorf("should return server-assigned ID, got %s", vec.ID)
	}
}

func TestVectorsInsertTextWithID_FallsBackToClientID(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		// server returns empty vector_id
		json.NewEncoder(w).Encode(InsertTextResponse{
			Message:    "inserted",
			VectorID:   "",
			Collection: "col1",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	vec, err := client.InsertTextWithID("col1", "client-id", "text", nil)
	if err != nil {
		t.Fatalf("InsertTextWithID returned error: %v", err)
	}
	if vec.ID != "client-id" {
		t.Errorf("should fall back to client-supplied ID when server returns empty, got %s", vec.ID)
	}
}

func TestVectorsListVectors(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("should use GET method, got %s", r.Method)
		}
		if !strings.HasPrefix(r.URL.Path, "/collections/myCol/vectors") {
			t.Errorf("should call /collections/myCol/vectors, got %s", r.URL.Path)
		}
		q := r.URL.Query()
		if q.Get("limit") != "10" {
			t.Errorf("should send limit=10, got %s", q.Get("limit"))
		}
		if q.Get("offset") != "5" {
			t.Errorf("should send offset=5, got %s", q.Get("offset"))
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(VectorPage{
			Total:  42,
			Limit:  10,
			Offset: 5,
			Vectors: []map[string]interface{}{
				{"id": "v1"},
				{"id": "v2"},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	page, err := client.ListVectors("myCol", 10, 5)
	if err != nil {
		t.Fatalf("ListVectors returned error: %v", err)
	}
	if page == nil {
		t.Fatal("should return non-nil VectorPage")
	}
	if page.Total != 42 {
		t.Errorf("should return total=42, got %d", page.Total)
	}
	if len(page.Vectors) != 2 {
		t.Errorf("should return 2 vectors, got %d", len(page.Vectors))
	}
}

func TestVectorsBatchInsertTexts(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/batch_insert" {
			t.Errorf("should call /batch_insert, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["collection"] != "batch-col" {
			t.Errorf("should send collection=batch-col, got %v", body["collection"])
		}
		texts, ok := body["texts"].([]interface{})
		if !ok {
			t.Errorf("should send texts array, got %T", body["texts"])
		} else if len(texts) != 2 {
			t.Errorf("should send 2 texts, got %d", len(texts))
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(BatchInsertReport{
			Collection: "batch-col",
			Inserted:   2,
			Failed:     0,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	items := []map[string]interface{}{
		{"text": "first doc"},
		{"text": "second doc"},
	}
	report, err := client.BatchInsertTexts("batch-col", items)
	if err != nil {
		t.Fatalf("BatchInsertTexts returned error: %v", err)
	}
	if report == nil {
		t.Fatal("should return non-nil BatchInsertReport")
	}
	if report.Inserted != 2 {
		t.Errorf("should report inserted=2, got %d", report.Inserted)
	}
	if report.Failed != 0 {
		t.Errorf("should report failed=0, got %d", report.Failed)
	}
}

func TestVectorsInsertVectors(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/insert_vectors" {
			t.Errorf("should call /insert_vectors, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["collection"] != "vec-col" {
			t.Errorf("should send collection=vec-col, got %v", body["collection"])
		}
		vecs, ok := body["vectors"].([]interface{})
		if !ok {
			t.Errorf("should send vectors array, got %T", body["vectors"])
		} else if len(vecs) != 2 {
			t.Errorf("should send 2 vectors, got %d", len(vecs))
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(BatchInsertReport{
			Collection: "vec-col",
			Inserted:   2,
			Failed:     0,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	vecs := []Vector{
		{ID: "v1", Data: []float32{0.1, 0.2}},
		{ID: "v2", Data: []float32{0.3, 0.4}},
	}
	report, err := client.InsertVectors("vec-col", vecs)
	if err != nil {
		t.Fatalf("InsertVectors returned error: %v", err)
	}
	if report == nil {
		t.Fatal("should return non-nil BatchInsertReport")
	}
	if report.Inserted != 2 {
		t.Errorf("should report inserted=2, got %d", report.Inserted)
	}
	if report.Collection != "vec-col" {
		t.Errorf("should return collection=vec-col, got %s", report.Collection)
	}
}

func TestVectorsBatchSearchQueries(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/batch_search" {
			t.Errorf("should call /batch_search, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["collection"] != "search-col" {
			t.Errorf("should send collection=search-col, got %v", body["collection"])
		}
		queries, ok := body["queries"].([]interface{})
		if !ok {
			t.Errorf("should send queries array, got %T", body["queries"])
		} else if len(queries) != 2 {
			t.Errorf("should send 2 queries, got %d", len(queries))
		}

		w.Header().Set("Content-Type", "application/json")
		// Server returns {collection, count, succeeded, failed, results: [...]}
		json.NewEncoder(w).Encode(map[string]interface{}{
			"collection": "search-col",
			"count":      2,
			"succeeded":  2,
			"failed":     0,
			"results": []map[string]interface{}{
				{
					"results":       []interface{}{},
					"query":         "query one",
					"collection":    "search-col",
					"total_results": 0,
				},
				{
					"results":       []interface{}{},
					"query":         "query two",
					"collection":    "search-col",
					"total_results": 0,
				},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	queries := []map[string]interface{}{
		{"query": "query one"},
		{"query": "query two"},
	}
	responses, err := client.BatchSearchQueries("search-col", queries)
	if err != nil {
		t.Fatalf("BatchSearchQueries returned error: %v", err)
	}
	if len(responses) != 2 {
		t.Errorf("should return 2 SearchResponses, got %d", len(responses))
	}
	if responses[0].Query != "query one" {
		t.Errorf("should return first query=query one, got %s", responses[0].Query)
	}
	if responses[1].Query != "query two" {
		t.Errorf("should return second query=query two, got %s", responses[1].Query)
	}
}

func TestVectorsBatchUpdateVectors(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/batch_update" {
			t.Errorf("should call /batch_update, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["collection"] != "upd-col" {
			t.Errorf("should send collection=upd-col, got %v", body["collection"])
		}
		updates, ok := body["updates"].([]interface{})
		if !ok {
			t.Errorf("should send updates array, got %T", body["updates"])
		} else if len(updates) != 3 {
			t.Errorf("should send 3 updates, got %d", len(updates))
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(BatchUpdateReport{
			Collection: "upd-col",
			Updated:    3,
			Failed:     0,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	updates := []map[string]interface{}{
		{"id": "v1", "metadata": map[string]interface{}{"k": "a"}},
		{"id": "v2", "metadata": map[string]interface{}{"k": "b"}},
		{"id": "v3", "metadata": map[string]interface{}{"k": "c"}},
	}
	report, err := client.BatchUpdateVectors("upd-col", updates)
	if err != nil {
		t.Fatalf("BatchUpdateVectors returned error: %v", err)
	}
	if report == nil {
		t.Fatal("should return non-nil BatchUpdateReport")
	}
	if report.Updated != 3 {
		t.Errorf("should report updated=3, got %d", report.Updated)
	}
	if report.Failed != 0 {
		t.Errorf("should report failed=0, got %d", report.Failed)
	}
}

func TestVectorsSearchByText(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/collections/txt-col/search/text" {
			t.Errorf("should call /collections/txt-col/search/text, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["query"] != "find me" {
			t.Errorf("should send query=find me, got %v", body["query"])
		}
		// limit arrives as float64 after JSON decode
		if body["limit"] != float64(7) {
			t.Errorf("should send limit=7, got %v", body["limit"])
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(SearchResponse{
			Collection:   "txt-col",
			Query:        "find me",
			Limit:        7,
			TotalResults: 1,
			Results: []SearchResult{
				{ID: "r1", Score: 0.99},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	resp, err := client.SearchByText("txt-col", "find me", 7)
	if err != nil {
		t.Fatalf("SearchByText returned error: %v", err)
	}
	if resp == nil {
		t.Fatal("should return non-nil SearchResponse")
	}
	if resp.TotalResults != 1 {
		t.Errorf("should return total_results=1, got %d", resp.TotalResults)
	}
	if len(resp.Results) != 1 {
		t.Fatalf("should return 1 result, got %d", len(resp.Results))
	}
	if resp.Results[0].ID != "r1" {
		t.Errorf("should return result ID=r1, got %s", resp.Results[0].ID)
	}
}

func TestVectorsSearchByFile(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/collections/file-col/search/file" {
			t.Errorf("should call /collections/file-col/search/file, got %s", r.URL.Path)
		}

		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["file_path"] != "/docs/readme.md" {
			t.Errorf("should send file_path=/docs/readme.md, got %v", body["file_path"])
		}
		if body["limit"] != float64(5) {
			t.Errorf("should send limit=5, got %v", body["limit"])
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(SearchResponse{
			Collection:   "file-col",
			TotalResults: 2,
			Results: []SearchResult{
				{ID: "f1", Score: 0.9},
				{ID: "f2", Score: 0.8},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	resp, err := client.SearchByFile("file-col", "/docs/readme.md", 5)
	if err != nil {
		t.Fatalf("SearchByFile returned error: %v", err)
	}
	if resp == nil {
		t.Fatal("should return non-nil SearchResponse")
	}
	if resp.TotalResults != 2 {
		t.Errorf("should return total_results=2, got %d", resp.TotalResults)
	}
	if len(resp.Results) != 2 {
		t.Fatalf("should return 2 results, got %d", len(resp.Results))
	}
	if resp.Results[0].ID != "f1" {
		t.Errorf("should return first result ID=f1, got %s", resp.Results[0].ID)
	}
}
