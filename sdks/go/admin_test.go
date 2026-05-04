package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestAdminGetServerStats(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/stats" {
			t.Errorf("expected path /stats, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"collections":    3,
			"total_vectors":  100,
			"uptime_seconds": 60,
			"version":        "3.3.0",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetServerStats()
	if err != nil {
		t.Fatalf("GetServerStats returned error: %v", err)
	}
	if got.Collections != 3 {
		t.Errorf("Collections: want 3, got %d", got.Collections)
	}
	if got.TotalVectors != 100 {
		t.Errorf("TotalVectors: want 100, got %d", got.TotalVectors)
	}
	if got.UptimeSeconds != 60 {
		t.Errorf("UptimeSeconds: want 60, got %d", got.UptimeSeconds)
	}
	if got.Version != "3.3.0" {
		t.Errorf("Version: want 3.3.0, got %s", got.Version)
	}
}

func TestAdminGetStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/status" {
			t.Errorf("expected path /status, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"online":            true,
			"version":           "3.3.0",
			"uptime_seconds":    120,
			"collections_count": 5,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetStatus()
	if err != nil {
		t.Fatalf("GetStatus returned error: %v", err)
	}
	if !got.Online {
		t.Errorf("Online: want true, got false")
	}
	if got.Version != "3.3.0" {
		t.Errorf("Version: want 3.3.0, got %s", got.Version)
	}
	if got.UptimeSeconds != 120 {
		t.Errorf("UptimeSeconds: want 120, got %d", got.UptimeSeconds)
	}
	if got.CollectionsCount != 5 {
		t.Errorf("CollectionsCount: want 5, got %d", got.CollectionsCount)
	}
}

func TestAdminGetLogs(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/logs" {
			t.Errorf("expected path /logs, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		q := r.URL.Query()
		if q.Get("lines") != "50" {
			t.Errorf("lines query param: want 50, got %s", q.Get("lines"))
		}
		if q.Get("level") != "error" {
			t.Errorf("level query param: want error, got %s", q.Get("level"))
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"logs": []map[string]interface{}{
				{
					"timestamp": "2026-05-03T10:00:00Z",
					"level":     "error",
					"message":   "disk full",
					"source":    "storage",
				},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetLogs(50, "error")
	if err != nil {
		t.Fatalf("GetLogs returned error: %v", err)
	}
	if len(got) != 1 {
		t.Fatalf("expected 1 log entry, got %d", len(got))
	}
	if got[0].Level != "error" {
		t.Errorf("Level: want error, got %s", got[0].Level)
	}
	if got[0].Message != "disk full" {
		t.Errorf("Message: want 'disk full', got %s", got[0].Message)
	}
	if got[0].Source != "storage" {
		t.Errorf("Source: want storage, got %s", got[0].Source)
	}
}

func TestAdminGetLogsNoParams(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/logs" {
			t.Errorf("expected path /logs, got %s", r.URL.Path)
		}
		if r.URL.RawQuery != "" {
			t.Errorf("expected no query params, got %s", r.URL.RawQuery)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"logs": []map[string]interface{}{},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetLogs(0, "")
	if err != nil {
		t.Fatalf("GetLogs(0,'') returned error: %v", err)
	}
	if len(got) != 0 {
		t.Errorf("expected 0 log entries, got %d", len(got))
	}
}

func TestAdminGetIndexingProgress(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/indexing/progress" {
			t.Errorf("expected path /indexing/progress, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"is_indexing":    true,
			"overall_status": "running",
			"collections": []map[string]interface{}{
				{
					"collection_name": "docs",
					"status":          "indexing",
					"progress":        0.75,
					"vector_count":    1000,
					"last_updated":    "2026-05-03T10:00:00Z",
				},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetIndexingProgress()
	if err != nil {
		t.Fatalf("GetIndexingProgress returned error: %v", err)
	}
	if !got.IsIndexing {
		t.Errorf("IsIndexing: want true, got false")
	}
	if got.OverallStatus != "running" {
		t.Errorf("OverallStatus: want running, got %s", got.OverallStatus)
	}
	if len(got.Collections) != 1 {
		t.Fatalf("expected 1 collection progress entry, got %d", len(got.Collections))
	}
	if got.Collections[0].CollectionName != "docs" {
		t.Errorf("CollectionName: want docs, got %s", got.Collections[0].CollectionName)
	}
	if got.Collections[0].Progress != 0.75 {
		t.Errorf("Progress: want 0.75, got %f", got.Collections[0].Progress)
	}
}

func TestAdminForceSaveCollection(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/my-col/force-save" {
			t.Errorf("expected path /collections/my-col/force-save, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.ForceSaveCollection("my-col")
	if err != nil {
		t.Fatalf("ForceSaveCollection returned error: %v", err)
	}
}

func TestAdminListEmptyCollectionsBareArray(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/empty" {
			t.Errorf("expected path /collections/empty, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		// Server returns a bare array
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode([]string{"alpha", "beta"})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ListEmptyCollections()
	if err != nil {
		t.Fatalf("ListEmptyCollections (bare array) returned error: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("expected 2 collections, got %d", len(got))
	}
	if got[0] != "alpha" {
		t.Errorf("got[0]: want alpha, got %s", got[0])
	}
	if got[1] != "beta" {
		t.Errorf("got[1]: want beta, got %s", got[1])
	}
}

func TestAdminListEmptyCollectionsEnvelope(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Server returns {"collections":[...]} envelope
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"collections": []string{"gamma"},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ListEmptyCollections()
	if err != nil {
		t.Fatalf("ListEmptyCollections (envelope) returned error: %v", err)
	}
	if len(got) != 1 {
		t.Fatalf("expected 1 collection, got %d", len(got))
	}
	if got[0] != "gamma" {
		t.Errorf("got[0]: want gamma, got %s", got[0])
	}
}

func TestAdminCleanupEmptyCollections(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/collections/cleanup" {
			t.Errorf("expected path /collections/cleanup, got %s", r.URL.Path)
		}
		if r.Method != "DELETE" {
			t.Errorf("expected DELETE, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"success":     true,
			"removed":     2,
			"collections": []string{"alpha", "beta"},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.CleanupEmptyCollections()
	if err != nil {
		t.Fatalf("CleanupEmptyCollections returned error: %v", err)
	}
	if !got.Success {
		t.Errorf("Success: want true, got false")
	}
	if got.Removed != 2 {
		t.Errorf("Removed: want 2, got %d", got.Removed)
	}
	if len(got.Collections) != 2 {
		t.Errorf("Collections: want 2 entries, got %d", len(got.Collections))
	}
}

func TestAdminGetConfig(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/config" {
			t.Errorf("expected path /config, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"port":      15002,
			"log_level": "info",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetConfig()
	if err != nil {
		t.Fatalf("GetConfig returned error: %v", err)
	}
	portVal, ok := got["port"]
	if !ok {
		t.Fatalf("GetConfig response missing 'port' key")
	}
	// JSON numbers unmarshal as float64 into interface{}
	if portVal.(float64) != 15002 {
		t.Errorf("port: want 15002, got %v", portVal)
	}
}

func TestAdminUpdateConfig(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/config" {
			t.Errorf("expected path /config, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var payload map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
			t.Errorf("decode request body: %v", err)
		}
		if payload["log_level"] != "debug" {
			t.Errorf("log_level: want debug, got %v", payload["log_level"])
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"port":      15002,
			"log_level": "debug",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.UpdateConfig(map[string]interface{}{"log_level": "debug"})
	if err != nil {
		t.Fatalf("UpdateConfig returned error: %v", err)
	}
	if got["log_level"] != "debug" {
		t.Errorf("log_level: want debug, got %v", got["log_level"])
	}
}

func TestAdminUpdateConfigForbidden(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusForbidden)
		json.NewEncoder(w).Encode(map[string]interface{}{
			"error_type": "forbidden",
			"message":    "admin only",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	_, err := client.UpdateConfig(map[string]interface{}{"log_level": "debug"})
	if err == nil {
		t.Fatal("expected error for 403 response, got nil")
	}
	ve, ok := err.(*VectorizerError)
	if !ok {
		t.Fatalf("expected *VectorizerError, got %T: %v", err, err)
	}
	if ve.Type != "forbidden" {
		t.Errorf("Type: want forbidden, got %s", ve.Type)
	}
	if ve.Status != http.StatusForbidden {
		t.Errorf("Status: want 403, got %d", ve.Status)
	}
}

func TestAdminListBackups(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/backups" {
			t.Errorf("expected path /backups, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"backups": []map[string]interface{}{
				{
					"id":          "bkp-001",
					"name":        "nightly",
					"date":        "2026-05-03",
					"size":        4096,
					"collections": []string{"docs"},
				},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ListBackups()
	if err != nil {
		t.Fatalf("ListBackups returned error: %v", err)
	}
	if len(got) != 1 {
		t.Fatalf("expected 1 backup, got %d", len(got))
	}
	if got[0].ID != "bkp-001" {
		t.Errorf("ID: want bkp-001, got %s", got[0].ID)
	}
	if got[0].Name != "nightly" {
		t.Errorf("Name: want nightly, got %s", got[0].Name)
	}
	if got[0].Size != 4096 {
		t.Errorf("Size: want 4096, got %d", got[0].Size)
	}
}

func TestAdminCreateBackup(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/backups/create" {
			t.Errorf("expected path /backups/create, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var payload CreateBackupRequest
		if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
			t.Errorf("decode request body: %v", err)
		}
		if payload.Name != "manual-backup" {
			t.Errorf("Name: want manual-backup, got %s", payload.Name)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"id":          "bkp-002",
			"name":        "manual-backup",
			"date":        "2026-05-03",
			"size":        8192,
			"collections": []string{"docs", "images"},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.CreateBackup(&CreateBackupRequest{
		Name:        "manual-backup",
		Collections: []string{"docs", "images"},
	})
	if err != nil {
		t.Fatalf("CreateBackup returned error: %v", err)
	}
	if got.ID != "bkp-002" {
		t.Errorf("ID: want bkp-002, got %s", got.ID)
	}
	if got.Name != "manual-backup" {
		t.Errorf("Name: want manual-backup, got %s", got.Name)
	}
}

func TestAdminRestoreBackup(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/backups/restore" {
			t.Errorf("expected path /backups/restore, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var payload RestoreBackupRequest
		if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
			t.Errorf("decode request body: %v", err)
		}
		if payload.BackupID != "bkp-001" {
			t.Errorf("BackupID: want bkp-001, got %s", payload.BackupID)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.RestoreBackup(&RestoreBackupRequest{BackupID: "bkp-001"})
	if err != nil {
		t.Fatalf("RestoreBackup returned error: %v", err)
	}
}

func TestAdminRestartServer(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/admin/restart" {
			t.Errorf("expected path /admin/restart, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.RestartServer()
	if err != nil {
		t.Fatalf("RestartServer returned error: %v", err)
	}
}

func TestAdminListWorkspaces(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/workspace/list" {
			t.Errorf("expected path /workspace/list, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"workspaces": []map[string]interface{}{
				{"path": "/data/ws1", "collection_name": "ws1"},
				{"path": "/data/ws2", "collection_name": "ws2"},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ListWorkspaces()
	if err != nil {
		t.Fatalf("ListWorkspaces returned error: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("expected 2 workspaces, got %d", len(got))
	}
	// WorkspaceConfig is map[string]interface{}, verify the path key
	if got[0]["path"] != "/data/ws1" {
		t.Errorf("workspace[0].path: want /data/ws1, got %v", got[0]["path"])
	}
}

func TestAdminGetWorkspaceConfig(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/workspace/config" {
			t.Errorf("expected path /workspace/config, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"path":            "/data/default",
			"collection_name": "default",
			"auto_index":      true,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetWorkspaceConfig()
	if err != nil {
		t.Fatalf("GetWorkspaceConfig returned error: %v", err)
	}
	if (*got)["path"] != "/data/default" {
		t.Errorf("path: want /data/default, got %v", (*got)["path"])
	}
	if (*got)["collection_name"] != "default" {
		t.Errorf("collection_name: want default, got %v", (*got)["collection_name"])
	}
}

func TestAdminAddWorkspace(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/workspace/add" {
			t.Errorf("expected path /workspace/add, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var payload AddWorkspaceRequest
		if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
			t.Errorf("decode request body: %v", err)
		}
		if payload.Path != "/data/new-ws" {
			t.Errorf("Path: want /data/new-ws, got %s", payload.Path)
		}
		if payload.CollectionName != "new-ws" {
			t.Errorf("CollectionName: want new-ws, got %s", payload.CollectionName)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.AddWorkspace(&AddWorkspaceRequest{
		Path:           "/data/new-ws",
		CollectionName: "new-ws",
	})
	if err != nil {
		t.Fatalf("AddWorkspace returned error: %v", err)
	}
}

func TestAdminRemoveWorkspace(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/workspace/remove" {
			t.Errorf("expected path /workspace/remove, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var payload map[string]string
		if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
			t.Errorf("decode request body: %v", err)
		}
		if payload["path"] != "/data/old-ws" {
			t.Errorf("path: want /data/old-ws, got %s", payload["path"])
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.RemoveWorkspace("/data/old-ws")
	if err != nil {
		t.Fatalf("RemoveWorkspace returned error: %v", err)
	}
}
