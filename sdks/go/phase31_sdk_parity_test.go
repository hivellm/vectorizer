// phase31_sdk_parity_test.go
//
// Unit tests for the four phase31 client methods that close the v3.3.0
// parity gap with the Rust / TypeScript / Python SDKs:
//
//   - UpdateApiKeyPermissions  (PUT  /auth/keys/{id}/permissions)
//   - GetApiKeyUsage           (GET  /auth/keys/{id}/usage[?window=N])
//   - DeleteVectors            (POST /batch_delete)
//   - MoveToCollection         (POST /collections/{src}/vectors/move)

package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestUpdateApiKeyPermissions(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys/key-42/permissions" {
			t.Errorf("expected path /auth/keys/key-42/permissions, got %s", r.URL.Path)
		}
		if r.Method != "PUT" {
			t.Errorf("expected PUT, got %s", r.Method)
		}
		var body UpdateApiKeyPermissionsRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("decode body: %v", err)
		}
		if len(body.Permissions) != 2 || body.Permissions[0] != "read" || body.Permissions[1] != "write" {
			t.Errorf("permissions: want [read write], got %v", body.Permissions)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(ApiKeyView{
			ID:          "key-42",
			Name:        "ci-bot",
			UserID:      "user-1",
			Permissions: []string{"read", "write"},
			Scopes:      []ApiKeyScope{},
			CreatedAt:   1700000000,
			Active:      true,
			UsageCount:  17,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.UpdateApiKeyPermissions("key-42", &UpdateApiKeyPermissionsRequest{
		Permissions: []string{"read", "write"},
	})
	if err != nil {
		t.Fatalf("UpdateApiKeyPermissions: %v", err)
	}
	if got.ID != "key-42" {
		t.Errorf("ID: want key-42, got %s", got.ID)
	}
	if len(got.Permissions) != 2 {
		t.Errorf("permissions: want 2, got %d", len(got.Permissions))
	}
	if got.UsageCount != 17 {
		t.Errorf("UsageCount: want 17, got %d", got.UsageCount)
	}
}

func TestGetApiKeyUsageDefaultWindow(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys/key-42/usage" {
			t.Errorf("expected path /auth/keys/key-42/usage, got %s", r.URL.Path)
		}
		// windowDays=0 ⇒ no `?window=` query param so the server applies its default.
		if r.URL.RawQuery != "" {
			t.Errorf("expected empty query, got %q", r.URL.RawQuery)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(ApiKeyUsageReport{
			Key: ApiKeyView{ID: "key-42", Name: "ci-bot", Active: true},
			Buckets: []ApiKeyUsageBucket{
				{Date: "2026-04-28", Count: 10},
				{Date: "2026-04-29", Count: 0},
			},
			WindowTotal: 10,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetApiKeyUsage("key-42", 0)
	if err != nil {
		t.Fatalf("GetApiKeyUsage: %v", err)
	}
	if len(got.Buckets) != 2 {
		t.Errorf("buckets: want 2, got %d", len(got.Buckets))
	}
	if got.WindowTotal != 10 {
		t.Errorf("WindowTotal: want 10, got %d", got.WindowTotal)
	}
}

func TestGetApiKeyUsageCustomWindow(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Query().Get("window") != "14" {
			t.Errorf("expected window=14, got %q", r.URL.Query().Get("window"))
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(ApiKeyUsageReport{
			Key:         ApiKeyView{ID: "key-42"},
			Buckets:     []ApiKeyUsageBucket{},
			WindowTotal: 0,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	if _, err := client.GetApiKeyUsage("key-42", 14); err != nil {
		t.Fatalf("GetApiKeyUsage(14): %v", err)
	}
}

func TestDeleteVectors(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/batch_delete" {
			t.Errorf("expected path /batch_delete, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("decode body: %v", err)
		}
		if body["collection"] != "docs" {
			t.Errorf("collection: want docs, got %v", body["collection"])
		}
		ids, _ := body["ids"].([]interface{})
		if len(ids) != 3 {
			t.Errorf("ids: want 3, got %d", len(ids))
		}
		w.Header().Set("Content-Type", "application/json")
		idA, idB, idC := "a", "b", "c"
		json.NewEncoder(w).Encode(DeleteReport{
			Collection: "docs",
			Count:      3,
			Deleted:    2,
			Failed:     1,
			Results: []VectorOpResult{
				{ID: &idA, Status: "ok"},
				{ID: &idB, Status: "ok"},
				{ID: &idC, Status: "missing_in_src"},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.DeleteVectors("docs", []string{"a", "b", "c"})
	if err != nil {
		t.Fatalf("DeleteVectors: %v", err)
	}
	if got.Deleted != 2 || got.Failed != 1 {
		t.Errorf("counts: want 2/1, got %d/%d", got.Deleted, got.Failed)
	}
	if len(got.Results) != 3 {
		t.Errorf("results: want 3, got %d", len(got.Results))
	}
}

func TestMoveToCollection(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/hot/vectors/move" {
			t.Errorf("expected path /collections/hot/vectors/move, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("decode body: %v", err)
		}
		if body["destination"] != "cold" {
			t.Errorf("destination: want cold, got %v", body["destination"])
		}
		ids, _ := body["ids"].([]interface{})
		if len(ids) != 2 {
			t.Errorf("ids: want 2, got %d", len(ids))
		}
		w.Header().Set("Content-Type", "application/json")
		idX, idY := "x", "y"
		json.NewEncoder(w).Encode(MoveReport{
			Src:       "hot",
			Dst:       "cold",
			Requested: 2,
			Moved:     2,
			Failed:    0,
			Results: []VectorOpResult{
				{ID: &idX, Status: "ok"},
				{ID: &idY, Status: "ok"},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.MoveToCollection("hot", "cold", []string{"x", "y"})
	if err != nil {
		t.Fatalf("MoveToCollection: %v", err)
	}
	if got.Src != "hot" || got.Dst != "cold" {
		t.Errorf("src/dst: want hot/cold, got %s/%s", got.Src, got.Dst)
	}
	if got.Moved != 2 {
		t.Errorf("Moved: want 2, got %d", got.Moved)
	}
}
