package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync"
	"testing"
)

func TestAuthMe(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/me" {
			t.Errorf("expected path /auth/me, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		user := User{
			UserID:   "u-001",
			Username: "alice",
			Roles:    []string{"admin"},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(user)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	user, err := client.Me()
	if err != nil {
		t.Fatalf("Me() returned error: %v", err)
	}
	if user.UserID != "u-001" {
		t.Errorf("expected user_id u-001, got %s", user.UserID)
	}
	if user.Username != "alice" {
		t.Errorf("expected username alice, got %s", user.Username)
	}
	if len(user.Roles) != 1 || user.Roles[0] != "admin" {
		t.Errorf("expected roles [admin], got %v", user.Roles)
	}
}

func TestAuthLogout(t *testing.T) {
	called := false
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/logout" {
			t.Errorf("expected path /auth/logout, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		called = true
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	if err := client.Logout(); err != nil {
		t.Fatalf("Logout() returned error: %v", err)
	}
	if !called {
		t.Error("mock server was not called")
	}
}

func TestAuthRefreshToken(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/refresh" {
			t.Errorf("expected path /auth/refresh, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		token := JwtToken{
			AccessToken: "new-token-xyz",
			TokenType:   "Bearer",
			ExpiresIn:   3600,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(token)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	token, err := client.RefreshToken()
	if err != nil {
		t.Fatalf("RefreshToken() returned error: %v", err)
	}
	if token.AccessToken != "new-token-xyz" {
		t.Errorf("expected access_token new-token-xyz, got %s", token.AccessToken)
	}
	if token.TokenType != "Bearer" {
		t.Errorf("expected token_type Bearer, got %s", token.TokenType)
	}
	if token.ExpiresIn != 3600 {
		t.Errorf("expected expires_in 3600, got %d", token.ExpiresIn)
	}
}

func TestAuthValidatePassword(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/validate-password" {
			t.Errorf("expected path /auth/validate-password, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["password"] != "s3cr3t!" {
			t.Errorf("expected password s3cr3t!, got %v", body["password"])
		}
		report := PasswordPolicyReport{
			Valid:         true,
			Errors:        []string{},
			Strength:      4,
			StrengthLabel: "strong",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(report)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	report, err := client.ValidatePassword("s3cr3t!")
	if err != nil {
		t.Fatalf("ValidatePassword() returned error: %v", err)
	}
	if !report.Valid {
		t.Error("expected valid to be true")
	}
	if report.Strength != 4 {
		t.Errorf("expected strength 4, got %d", report.Strength)
	}
	if report.StrengthLabel != "strong" {
		t.Errorf("expected strength_label strong, got %s", report.StrengthLabel)
	}
}

func TestAuthCreateApiKey(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys" {
			t.Errorf("expected path /auth/keys, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body CreateApiKeyRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body.Name != "my-key" {
			t.Errorf("expected name my-key, got %s", body.Name)
		}
		rawKey := "rawkey-abc123"
		key := ApiKey{
			ID:          "key-001",
			Name:        "my-key",
			Permissions: []string{"read", "write"},
			ApiKeyValue: &rawKey,
			CreatedAt:   1700000000,
			Active:      true,
			UsageCount:  0,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(key)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	key, err := client.CreateApiKey(&CreateApiKeyRequest{
		Name:        "my-key",
		Permissions: []string{"read", "write"},
	})
	if err != nil {
		t.Fatalf("CreateApiKey() returned error: %v", err)
	}
	if key.ID != "key-001" {
		t.Errorf("expected id key-001, got %s", key.ID)
	}
	if key.Name != "my-key" {
		t.Errorf("expected name my-key, got %s", key.Name)
	}
	if key.ApiKeyValue == nil || *key.ApiKeyValue != "rawkey-abc123" {
		t.Errorf("expected api_key rawkey-abc123, got %v", key.ApiKeyValue)
	}
	if !key.Active {
		t.Error("expected active to be true")
	}
}

func TestAuthListApiKeys(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys" {
			t.Errorf("expected path /auth/keys, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		keysJSON, _ := json.Marshal([]ApiKey{
			{ID: "key-001", Name: "my-key", Permissions: []string{"read"}, CreatedAt: 1700000000, Active: true},
			{ID: "key-002", Name: "other-key", Permissions: []string{"write"}, CreatedAt: 1700000001, Active: true},
		})
		envelope := map[string]json.RawMessage{"keys": keysJSON}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(envelope)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	keys, err := client.ListApiKeys()
	if err != nil {
		t.Fatalf("ListApiKeys() returned error: %v", err)
	}
	if len(keys) != 2 {
		t.Fatalf("expected 2 keys, got %d", len(keys))
	}
	if keys[0].ID != "key-001" {
		t.Errorf("expected first key id key-001, got %s", keys[0].ID)
	}
	if keys[1].ID != "key-002" {
		t.Errorf("expected second key id key-002, got %s", keys[1].ID)
	}
}

func TestAuthListApiKeysEmptyEnvelope(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Return envelope with null keys field — should yield empty slice, not error.
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{}`))
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	keys, err := client.ListApiKeys()
	if err != nil {
		t.Fatalf("ListApiKeys() with empty envelope returned error: %v", err)
	}
	if len(keys) != 0 {
		t.Errorf("expected 0 keys, got %d", len(keys))
	}
}

func TestAuthRevokeApiKey(t *testing.T) {
	called := false
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys/key-001" {
			t.Errorf("expected path /auth/keys/key-001, got %s", r.URL.Path)
		}
		if r.Method != "DELETE" {
			t.Errorf("expected DELETE, got %s", r.Method)
		}
		called = true
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	if err := client.RevokeApiKey("key-001"); err != nil {
		t.Fatalf("RevokeApiKey() returned error: %v", err)
	}
	if !called {
		t.Error("mock server was not called")
	}
}

func TestAuthCreateUser(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/users" {
			t.Errorf("expected path /auth/users, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body CreateUserRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body.Username != "bob" {
			t.Errorf("expected username bob, got %s", body.Username)
		}
		if body.Password != "pass123" {
			t.Errorf("expected password pass123, got %s", body.Password)
		}
		user := User{
			UserID:   "u-002",
			Username: "bob",
			Roles:    body.Roles,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(user)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	user, err := client.CreateUser(&CreateUserRequest{
		Username: "bob",
		Password: "pass123",
		Roles:    []string{"viewer"},
	})
	if err != nil {
		t.Fatalf("CreateUser() returned error: %v", err)
	}
	if user.UserID != "u-002" {
		t.Errorf("expected user_id u-002, got %s", user.UserID)
	}
	if user.Username != "bob" {
		t.Errorf("expected username bob, got %s", user.Username)
	}
}

func TestAuthListUsers(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/users" {
			t.Errorf("expected path /auth/users, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		usersJSON, _ := json.Marshal([]User{
			{UserID: "u-001", Username: "alice", Roles: []string{"admin"}},
			{UserID: "u-002", Username: "bob", Roles: []string{"viewer"}},
		})
		envelope := map[string]json.RawMessage{"users": usersJSON}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(envelope)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	users, err := client.ListUsers()
	if err != nil {
		t.Fatalf("ListUsers() returned error: %v", err)
	}
	if len(users) != 2 {
		t.Fatalf("expected 2 users, got %d", len(users))
	}
	if users[0].Username != "alice" {
		t.Errorf("expected first user alice, got %s", users[0].Username)
	}
	if users[1].Username != "bob" {
		t.Errorf("expected second user bob, got %s", users[1].Username)
	}
}

func TestAuthListUsersEmptyEnvelope(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{}`))
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	users, err := client.ListUsers()
	if err != nil {
		t.Fatalf("ListUsers() with empty envelope returned error: %v", err)
	}
	if len(users) != 0 {
		t.Errorf("expected 0 users, got %d", len(users))
	}
}

func TestAuthDeleteUser(t *testing.T) {
	called := false
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/users/alice" {
			t.Errorf("expected path /auth/users/alice, got %s", r.URL.Path)
		}
		if r.Method != "DELETE" {
			t.Errorf("expected DELETE, got %s", r.Method)
		}
		called = true
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	if err := client.DeleteUser("alice"); err != nil {
		t.Fatalf("DeleteUser() returned error: %v", err)
	}
	if !called {
		t.Error("mock server was not called")
	}
}

func TestAuthChangePassword(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/users/alice/password" {
			t.Errorf("expected path /auth/users/alice/password, got %s", r.URL.Path)
		}
		if r.Method != "PUT" {
			t.Errorf("expected PUT, got %s", r.Method)
		}
		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["new_password"] != "newpass456" {
			t.Errorf("expected new_password newpass456, got %v", body["new_password"])
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	if err := client.ChangePassword("alice", "newpass456"); err != nil {
		t.Fatalf("ChangePassword() returned error: %v", err)
	}
}

// TestAuthApiKeyLifecycle verifies the create → list → revoke sequence using a
// single stateful mock server. The created key must appear in the list and be
// absent after revoke.
func TestAuthApiKeyLifecycle(t *testing.T) {
	var mu sync.Mutex
	store := make(map[string]ApiKey) // keyed by id

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")

		switch {
		// POST /auth/keys — create
		case r.Method == "POST" && r.URL.Path == "/auth/keys":
			var req CreateApiKeyRequest
			if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
				t.Errorf("failed to decode CreateApiKey body: %v", err)
				http.Error(w, "bad request", http.StatusBadRequest)
				return
			}
			rawVal := "lifecycle-raw-key"
			key := ApiKey{
				ID:          "lc-key-001",
				Name:        req.Name,
				Permissions: req.Permissions,
				ApiKeyValue: &rawVal,
				CreatedAt:   1700000000,
				Active:      true,
				UsageCount:  0,
			}
			mu.Lock()
			store[key.ID] = key
			mu.Unlock()
			json.NewEncoder(w).Encode(key)

		// GET /auth/keys — list
		case r.Method == "GET" && r.URL.Path == "/auth/keys":
			mu.Lock()
			keys := make([]ApiKey, 0, len(store))
			for _, k := range store {
				keys = append(keys, k)
			}
			mu.Unlock()
			keysJSON, _ := json.Marshal(keys)
			envelope := map[string]json.RawMessage{"keys": keysJSON}
			json.NewEncoder(w).Encode(envelope)

		// DELETE /auth/keys/{id} — revoke
		case r.Method == "DELETE" && len(r.URL.Path) > len("/auth/keys/"):
			id := r.URL.Path[len("/auth/keys/"):]
			mu.Lock()
			delete(store, id)
			mu.Unlock()
			w.WriteHeader(http.StatusOK)

		default:
			t.Errorf("unexpected request: %s %s", r.Method, r.URL.Path)
			http.Error(w, "not found", http.StatusNotFound)
		}
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	// Step 1: Create
	created, err := client.CreateApiKey(&CreateApiKeyRequest{
		Name:        "lifecycle-key",
		Permissions: []string{"read"},
	})
	if err != nil {
		t.Fatalf("CreateApiKey() returned error: %v", err)
	}
	if created.ID != "lc-key-001" {
		t.Fatalf("expected created id lc-key-001, got %s", created.ID)
	}

	// Step 2: List — key must appear
	keys, err := client.ListApiKeys()
	if err != nil {
		t.Fatalf("ListApiKeys() returned error: %v", err)
	}
	found := false
	for _, k := range keys {
		if k.ID == created.ID {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("created key %s not found in list: %v", created.ID, keys)
	}

	// Step 3: Revoke
	if err := client.RevokeApiKey(created.ID); err != nil {
		t.Fatalf("RevokeApiKey() returned error: %v", err)
	}

	// Step 4: List again — key must be absent
	keys, err = client.ListApiKeys()
	if err != nil {
		t.Fatalf("ListApiKeys() after revoke returned error: %v", err)
	}
	for _, k := range keys {
		if k.ID == created.ID {
			t.Errorf("revoked key %s still present in list", created.ID)
		}
	}
}
