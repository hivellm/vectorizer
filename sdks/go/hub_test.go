package vectorizer

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestHubListUserBackups(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("should use GET method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/backups" {
			t.Errorf("should call /hub/backups, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-123" {
			t.Errorf("should pass user_id=user-123, got %q", got)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(listUserBackupsResponse{
			Backups: []UserBackup{
				{ID: "bk-1", UserID: "user-123", Name: "backup-one", Status: "ready"},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	backups, err := client.ListUserBackups("user-123")
	if err != nil {
		t.Fatalf("ListUserBackups returned unexpected error: %v", err)
	}
	if len(backups) != 1 {
		t.Fatalf("should return 1 backup, got %d", len(backups))
	}
	if backups[0].ID != "bk-1" {
		t.Errorf("backup ID should be bk-1, got %q", backups[0].ID)
	}
}

func TestHubCreateUserBackup(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/backups" {
			t.Errorf("should call /hub/backups, got %s", r.URL.Path)
		}
		var body CreateUserBackupRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should decode request body: %v", err)
		}
		if body.UserID != "user-456" {
			t.Errorf("body user_id should be user-456, got %q", body.UserID)
		}
		if body.Name != "my-backup" {
			t.Errorf("body name should be my-backup, got %q", body.Name)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(createUserBackupResponse{
			Backup: UserBackup{ID: "bk-2", UserID: "user-456", Name: "my-backup", Status: "pending"},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	req := &CreateUserBackupRequest{UserID: "user-456", Name: "my-backup"}
	backup, err := client.CreateUserBackup(req)
	if err != nil {
		t.Fatalf("CreateUserBackup returned unexpected error: %v", err)
	}
	if backup.ID != "bk-2" {
		t.Errorf("backup ID should be bk-2, got %q", backup.ID)
	}
	if backup.Name != "my-backup" {
		t.Errorf("backup Name should be my-backup, got %q", backup.Name)
	}
}

func TestHubRestoreUserBackup(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/backups/restore" {
			t.Errorf("should call /hub/backups/restore, got %s", r.URL.Path)
		}
		var body RestoreUserBackupRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should decode request body: %v", err)
		}
		if body.UserID != "user-789" {
			t.Errorf("body user_id should be user-789, got %q", body.UserID)
		}
		if body.BackupID != "bk-3" {
			t.Errorf("body backup_id should be bk-3, got %q", body.BackupID)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.RestoreUserBackup(&RestoreUserBackupRequest{UserID: "user-789", BackupID: "bk-3"})
	if err != nil {
		t.Fatalf("RestoreUserBackup returned unexpected error: %v", err)
	}
}

func TestHubUploadUserBackup(t *testing.T) {
	rawPayload := []byte("raw-backup-bytes")
	expectedB64 := base64.StdEncoding.EncodeToString(rawPayload)

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/backups/upload" {
			t.Errorf("should call /hub/backups/upload, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-111" {
			t.Errorf("query param user_id should be user-111, got %q", got)
		}
		if got := r.URL.Query().Get("name"); got != "upload-bk" {
			t.Errorf("query param name should be upload-bk, got %q", got)
		}
		var body uploadBackupBody
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should decode request body: %v", err)
		}
		if body.Data != expectedB64 {
			t.Errorf("body data should be base64-encoded payload, got %q", body.Data)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(getOrUploadBackupResponse{
			Backup: UserBackup{ID: "bk-upload", UserID: "user-111", Name: "upload-bk", Status: "ready"},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	backup, err := client.UploadUserBackup("user-111", "upload-bk", rawPayload)
	if err != nil {
		t.Fatalf("UploadUserBackup returned unexpected error: %v", err)
	}
	if backup.ID != "bk-upload" {
		t.Errorf("backup ID should be bk-upload, got %q", backup.ID)
	}
}

func TestHubGetUserBackup(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("should use GET method, got %s", r.Method)
		}
		if !strings.HasPrefix(r.URL.Path, "/hub/backups/bk-42") {
			t.Errorf("should call /hub/backups/bk-42, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-222" {
			t.Errorf("query param user_id should be user-222, got %q", got)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(getOrUploadBackupResponse{
			Backup: UserBackup{ID: "bk-42", UserID: "user-222", Name: "named-bk", Status: "ready"},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	backup, err := client.GetUserBackup("user-222", "bk-42")
	if err != nil {
		t.Fatalf("GetUserBackup returned unexpected error: %v", err)
	}
	if backup.ID != "bk-42" {
		t.Errorf("backup ID should be bk-42, got %q", backup.ID)
	}
}

func TestHubDeleteUserBackup(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "DELETE" {
			t.Errorf("should use DELETE method, got %s", r.Method)
		}
		if !strings.HasPrefix(r.URL.Path, "/hub/backups/bk-del") {
			t.Errorf("should call /hub/backups/bk-del, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-333" {
			t.Errorf("query param user_id should be user-333, got %q", got)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	err := client.DeleteUserBackup("user-333", "bk-del")
	if err != nil {
		t.Fatalf("DeleteUserBackup returned unexpected error: %v", err)
	}
}

func TestHubDownloadUserBackup(t *testing.T) {
	binaryPayload := []byte("binary-payload")

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("should use GET method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/backups/bk-dl/download" {
			t.Errorf("should call /hub/backups/bk-dl/download, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-444" {
			t.Errorf("query param user_id should be user-444, got %q", got)
		}
		w.Header().Set("Content-Type", "application/octet-stream")
		w.Write(binaryPayload)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	data, err := client.DownloadUserBackup("user-444", "bk-dl")
	if err != nil {
		t.Fatalf("DownloadUserBackup returned unexpected error: %v", err)
	}
	if string(data) != string(binaryPayload) {
		t.Errorf("returned bytes should match binary-payload, got %q", string(data))
	}
}

func TestHubGetUsageStatistics(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("should use GET method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/usage/statistics" {
			t.Errorf("should call /hub/usage/statistics, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-555" {
			t.Errorf("query param user_id should be user-555, got %q", got)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(UsageStatistics{
			Success: true,
			Message: "ok",
			Stats:   map[string]interface{}{"vectors": float64(42)},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	stats, err := client.GetUsageStatistics("user-555")
	if err != nil {
		t.Fatalf("GetUsageStatistics returned unexpected error: %v", err)
	}
	if !stats.Success {
		t.Errorf("success should be true")
	}
	if stats.Stats["vectors"] != float64(42) {
		t.Errorf("stats[vectors] should be 42, got %v", stats.Stats["vectors"])
	}
}

func TestHubGetQuotaInfo(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("should use GET method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/usage/quota" {
			t.Errorf("should call /hub/usage/quota, got %s", r.URL.Path)
		}
		if got := r.URL.Query().Get("user_id"); got != "user-666" {
			t.Errorf("query param user_id should be user-666, got %q", got)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(QuotaInfo{
			Success: true,
			Message: "ok",
			Quota:   map[string]interface{}{"max_backups": float64(10)},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	quota, err := client.GetQuotaInfo("user-666")
	if err != nil {
		t.Fatalf("GetQuotaInfo returned unexpected error: %v", err)
	}
	if !quota.Success {
		t.Errorf("success should be true")
	}
	if quota.Quota["max_backups"] != float64(10) {
		t.Errorf("quota[max_backups] should be 10, got %v", quota.Quota["max_backups"])
	}
}

func TestHubValidateApiKey(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("should use POST method, got %s", r.Method)
		}
		if r.URL.Path != "/hub/validate-key" {
			t.Errorf("should call /hub/validate-key, got %s", r.URL.Path)
		}
		var body map[string]string
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("should decode request body: %v", err)
		}
		if body["key"] != "test-api-key" {
			t.Errorf("body key should be test-api-key, got %q", body["key"])
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(HubApiKeyValidation{
			Valid:       true,
			TenantID:    "tenant-1",
			TenantName:  "Acme Corp",
			Permissions: []string{"read", "write"},
			ValidatedAt: "2026-05-03T00:00:00Z",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	result, err := client.ValidateHubAPIKey("test-api-key")
	if err != nil {
		t.Fatalf("ValidateHubAPIKey returned unexpected error: %v", err)
	}
	if !result.Valid {
		t.Errorf("valid should be true")
	}
	if result.TenantID != "tenant-1" {
		t.Errorf("tenant_id should be tenant-1, got %q", result.TenantID)
	}
	if len(result.Permissions) != 2 {
		t.Errorf("should have 2 permissions, got %d", len(result.Permissions))
	}
}
