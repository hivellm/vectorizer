package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestClusterFailover(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/cluster/failover" {
			t.Errorf("expected path /cluster/failover, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["replica_id"] != "replica-42" {
			t.Errorf("expected replica_id replica-42, got %v", body["replica_id"])
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(FailoverReport{
			PromotedReplicaID:        "replica-42",
			MasterOffsetAtPromotion:  1000,
			ReplicaOffsetAtPromotion: 998,
			ResidualLagOperations:    2,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ClusterFailover("replica-42")
	if err != nil {
		t.Fatalf("ClusterFailover() returned error: %v", err)
	}
	if got.PromotedReplicaID != "replica-42" {
		t.Errorf("PromotedReplicaID: want replica-42, got %s", got.PromotedReplicaID)
	}
	if got.MasterOffsetAtPromotion != 1000 {
		t.Errorf("MasterOffsetAtPromotion: want 1000, got %d", got.MasterOffsetAtPromotion)
	}
	if got.ReplicaOffsetAtPromotion != 998 {
		t.Errorf("ReplicaOffsetAtPromotion: want 998, got %d", got.ReplicaOffsetAtPromotion)
	}
	if got.ResidualLagOperations != 2 {
		t.Errorf("ResidualLagOperations: want 2, got %d", got.ResidualLagOperations)
	}
}

func TestClusterResyncReplica(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/cluster/replicas/node-7/resync" {
			t.Errorf("expected path /cluster/replicas/node-7/resync, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(ResyncJob{
			ReplicaID:      "node-7",
			SnapshotOffset: 5000,
			FullSnapshot:   true,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ClusterResyncReplica("node-7")
	if err != nil {
		t.Fatalf("ClusterResyncReplica() returned error: %v", err)
	}
	if got.ReplicaID != "node-7" {
		t.Errorf("ReplicaID: want node-7, got %s", got.ReplicaID)
	}
	if got.SnapshotOffset != 5000 {
		t.Errorf("SnapshotOffset: want 5000, got %d", got.SnapshotOffset)
	}
	if !got.FullSnapshot {
		t.Errorf("FullSnapshot: want true, got false")
	}
}

func TestClusterAddPeer(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/cluster/peers" {
			t.Errorf("expected path /cluster/peers, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body AddPeerRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body.Address != "10.0.0.5:15002" {
			t.Errorf("expected address 10.0.0.5:15002, got %s", body.Address)
		}
		if body.Role != "replica" {
			t.Errorf("expected role replica, got %s", body.Role)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(PeerInfo{
			NodeID:  "peer-001",
			Address: "10.0.0.5:15002",
			Role:    "replica",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ClusterAddPeer(&AddPeerRequest{
		Address: "10.0.0.5:15002",
		Role:    "replica",
	})
	if err != nil {
		t.Fatalf("ClusterAddPeer() returned error: %v", err)
	}
	if got.NodeID != "peer-001" {
		t.Errorf("NodeID: want peer-001, got %s", got.NodeID)
	}
	if got.Address != "10.0.0.5:15002" {
		t.Errorf("Address: want 10.0.0.5:15002, got %s", got.Address)
	}
	if got.Role != "replica" {
		t.Errorf("Role: want replica, got %s", got.Role)
	}
}

func TestClusterRebalance(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/cluster/rebalance" {
			t.Errorf("expected path /cluster/rebalance, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(RebalanceJob{
			JobID:        "job-rebal-1",
			Status:       "running",
			ShardsToMove: 8,
			ShardsMoved:  0,
			Message:      "rebalance started",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ClusterRebalance()
	if err != nil {
		t.Fatalf("ClusterRebalance() returned error: %v", err)
	}
	if got.JobID != "job-rebal-1" {
		t.Errorf("JobID: want job-rebal-1, got %s", got.JobID)
	}
	if got.Status != "running" {
		t.Errorf("Status: want running, got %s", got.Status)
	}
	if got.ShardsToMove != 8 {
		t.Errorf("ShardsToMove: want 8, got %d", got.ShardsToMove)
	}
}

func TestClusterRebalanceStatusActive(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/cluster/rebalance/status" {
			t.Errorf("expected path /cluster/rebalance/status, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(RebalanceJob{
			JobID:        "job-rebal-1",
			Status:       "running",
			ShardsToMove: 8,
			ShardsMoved:  3,
			Message:      "in progress",
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ClusterRebalanceStatus()
	if err != nil {
		t.Fatalf("ClusterRebalanceStatus() returned error: %v", err)
	}
	if got == nil {
		t.Fatal("expected non-nil RebalanceJob for active status")
	}
	if got.JobID != "job-rebal-1" {
		t.Errorf("JobID: want job-rebal-1, got %s", got.JobID)
	}
	if got.ShardsMoved != 3 {
		t.Errorf("ShardsMoved: want 3, got %d", got.ShardsMoved)
	}
}

func TestClusterRebalanceStatusIdle(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/cluster/rebalance/status" {
			t.Errorf("expected path /cluster/rebalance/status, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		// Server returns the idle sentinel — client must return (nil, nil).
		w.Write([]byte(`{"status":"idle"}`))
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ClusterRebalanceStatus()
	if err != nil {
		t.Fatalf("ClusterRebalanceStatus() idle returned error: %v", err)
	}
	if got != nil {
		t.Errorf("expected nil RebalanceJob for idle sentinel, got %+v", got)
	}
}

func TestClusterRotateApiKey(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys/key-007/rotate" {
			t.Errorf("expected path /auth/keys/key-007/rotate, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(RotatedKey{
			OldKeyID:   "key-007",
			NewKeyID:   "key-008",
			NewToken:   "tok-new-xyz",
			GraceUntil: 1800000000,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.RotateApiKey("key-007")
	if err != nil {
		t.Fatalf("RotateApiKey() returned error: %v", err)
	}
	if got.OldKeyID != "key-007" {
		t.Errorf("OldKeyID: want key-007, got %s", got.OldKeyID)
	}
	if got.NewKeyID != "key-008" {
		t.Errorf("NewKeyID: want key-008, got %s", got.NewKeyID)
	}
	if got.NewToken != "tok-new-xyz" {
		t.Errorf("NewToken: want tok-new-xyz, got %s", got.NewToken)
	}
	if got.GraceUntil != 1800000000 {
		t.Errorf("GraceUntil: want 1800000000, got %d", got.GraceUntil)
	}
}

func TestClusterCreateScopedApiKey(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/keys" {
			t.Errorf("expected path /auth/keys, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body CreateScopedApiKeyRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body.Name != "scoped-key" {
			t.Errorf("expected name scoped-key, got %s", body.Name)
		}
		if len(body.Scopes) != 1 {
			t.Fatalf("expected 1 scope, got %d", len(body.Scopes))
		}
		if body.Scopes[0].Collection != "docs" {
			t.Errorf("expected scope collection docs, got %s", body.Scopes[0].Collection)
		}
		rawVal := "scoped-raw-token"
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(ApiKey{
			ID:          "key-scoped-1",
			Name:        "scoped-key",
			Permissions: []string{"read"},
			ApiKeyValue: &rawVal,
			CreatedAt:   1700000100,
			Active:      true,
			UsageCount:  0,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.CreateScopedApiKey(&CreateScopedApiKeyRequest{
		Name:        "scoped-key",
		Permissions: []string{"read"},
		Scopes: []TokenScope{
			{Collection: "docs", Permissions: []string{"read"}},
		},
	})
	if err != nil {
		t.Fatalf("CreateScopedApiKey() returned error: %v", err)
	}
	if got.ID != "key-scoped-1" {
		t.Errorf("ID: want key-scoped-1, got %s", got.ID)
	}
	if got.Name != "scoped-key" {
		t.Errorf("Name: want scoped-key, got %s", got.Name)
	}
	if got.ApiKeyValue == nil || *got.ApiKeyValue != "scoped-raw-token" {
		t.Errorf("ApiKeyValue: want scoped-raw-token, got %v", got.ApiKeyValue)
	}
	if !got.Active {
		t.Errorf("Active: want true, got false")
	}
}

func TestClusterIntrospectToken(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/introspect" {
			t.Errorf("expected path /auth/introspect, got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST, got %s", r.Method)
		}
		var body map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body["token"] != "tok-abc" {
			t.Errorf("expected token tok-abc, got %v", body["token"])
		}
		scope := "read write"
		sub := "u-001"
		exp := int64(1900000000)
		username := "alice"
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(TokenIntrospection{
			Active:   true,
			Scope:    &scope,
			Sub:      &sub,
			Exp:      &exp,
			Username: &username,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.IntrospectToken("tok-abc")
	if err != nil {
		t.Fatalf("IntrospectToken() returned error: %v", err)
	}
	if !got.Active {
		t.Errorf("Active: want true, got false")
	}
	if got.Sub == nil || *got.Sub != "u-001" {
		t.Errorf("Sub: want u-001, got %v", got.Sub)
	}
	if got.Username == nil || *got.Username != "alice" {
		t.Errorf("Username: want alice, got %v", got.Username)
	}
	if got.Scope == nil || *got.Scope != "read write" {
		t.Errorf("Scope: want 'read write', got %v", got.Scope)
	}
	if got.Exp == nil || *got.Exp != 1900000000 {
		t.Errorf("Exp: want 1900000000, got %v", got.Exp)
	}
}

func TestClusterListAuditLog(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/auth/audit" {
			t.Errorf("expected path /auth/audit, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		q := r.URL.Query()
		if q.Get("actor") != "alice" {
			t.Errorf("actor query param: want alice, got %s", q.Get("actor"))
		}
		if q.Get("action") != "delete" {
			t.Errorf("action query param: want delete, got %s", q.Get("action"))
		}
		if q.Get("since") != "2026-01-01T00:00:00Z" {
			t.Errorf("since query param: want 2026-01-01T00:00:00Z, got %s", q.Get("since"))
		}
		if q.Get("until") != "2026-05-03T00:00:00Z" {
			t.Errorf("until query param: want 2026-05-03T00:00:00Z, got %s", q.Get("until"))
		}
		if q.Get("limit") != "10" {
			t.Errorf("limit query param: want 10, got %s", q.Get("limit"))
		}
		corrID := "corr-xyz"
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"entries": []AuditEntry{
				{
					Actor:         "alice",
					Action:        "delete",
					Target:        "collection/docs",
					At:            "2026-03-10T12:00:00Z",
					CorrelationID: &corrID,
				},
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.ListAuditLog(AuditQuery{
		Actor:  "alice",
		Action: "delete",
		Since:  "2026-01-01T00:00:00Z",
		Until:  "2026-05-03T00:00:00Z",
		Limit:  10,
	})
	if err != nil {
		t.Fatalf("ListAuditLog() returned error: %v", err)
	}
	if len(got) != 1 {
		t.Fatalf("expected 1 audit entry, got %d", len(got))
	}
	if got[0].Actor != "alice" {
		t.Errorf("Actor: want alice, got %s", got[0].Actor)
	}
	if got[0].Action != "delete" {
		t.Errorf("Action: want delete, got %s", got[0].Action)
	}
	if got[0].Target != "collection/docs" {
		t.Errorf("Target: want collection/docs, got %s", got[0].Target)
	}
	if got[0].CorrelationID == nil || *got[0].CorrelationID != "corr-xyz" {
		t.Errorf("CorrelationID: want corr-xyz, got %v", got[0].CorrelationID)
	}
}
