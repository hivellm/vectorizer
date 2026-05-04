package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReplicationGetStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/replication/status" {
			t.Errorf("should call /replication/status but got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("should use GET method but got %s", r.Method)
		}

		resp := ReplicationStatus{
			Role:    "master",
			Enabled: true,
			Replicas: []ReplicaInfo{
				{
					ReplicaID:        "replica-1",
					Host:             "10.0.0.2",
					Port:             15002,
					Status:           "connected",
					LastHeartbeat:    "2026-05-03T00:00:00Z",
					OperationsSynced: 42,
				},
			},
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	status, err := client.GetReplicationStatus()
	if err != nil {
		t.Fatalf("GetReplicationStatus failed: %v", err)
	}
	if status == nil {
		t.Fatal("should return non-nil ReplicationStatus")
	}
	if len(status.Replicas) == 0 {
		t.Error("should decode ReplicationStatus with a non-empty Replicas slice")
	}
	if status.Role != "master" {
		t.Errorf("should decode role as master but got %s", status.Role)
	}
	if !status.Enabled {
		t.Error("should decode enabled as true")
	}
}

func TestReplicationConfigure(t *testing.T) {
	var capturedBody map[string]interface{}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/replication/configure" {
			t.Errorf("should call /replication/configure but got %s", r.URL.Path)
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

	bindAddr := "0.0.0.0:15003"
	cfg := &ReplicationConfig{
		Role:        "master",
		BindAddress: &bindAddr,
	}

	err := client.ConfigureReplication(cfg)
	if err != nil {
		t.Fatalf("ConfigureReplication failed: %v", err)
	}

	if capturedBody["role"] != "master" {
		t.Errorf("POST body should contain role=master but got %v", capturedBody["role"])
	}
	if capturedBody["bind_address"] != bindAddr {
		t.Errorf("POST body should contain bind_address=%s but got %v", bindAddr, capturedBody["bind_address"])
	}
}

func TestReplicationGetStats(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/replication/stats" {
			t.Errorf("should call /replication/stats but got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("should use GET method but got %s", r.Method)
		}

		role := "master"
		connected := 2
		resp := ReplicationStats{
			Role:              &role,
			MasterOffset:      1000,
			ReplicaOffset:     998,
			LagOperations:     2,
			TotalReplicated:   5000,
			ConnectedReplicas: &connected,
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	stats, err := client.GetReplicationStats()
	if err != nil {
		t.Fatalf("GetReplicationStats failed: %v", err)
	}
	if stats == nil {
		t.Fatal("should return non-nil ReplicationStats")
	}
	if stats.MasterOffset != 1000 {
		t.Errorf("should decode master_offset as 1000 but got %d", stats.MasterOffset)
	}
	if stats.TotalReplicated != 5000 {
		t.Errorf("should decode total_replicated as 5000 but got %d", stats.TotalReplicated)
	}
}

func TestReplicationListReplicas(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/replication/replicas" {
			t.Errorf("should call /replication/replicas but got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("should use GET method but got %s", r.Method)
		}

		// Envelope shape: {"replicas":[...]}
		type envelope struct {
			Replicas []ReplicaInfo `json:"replicas"`
		}
		resp := envelope{
			Replicas: []ReplicaInfo{
				{
					ReplicaID:        "replica-1",
					Host:             "10.0.0.2",
					Port:             15002,
					Status:           "connected",
					LastHeartbeat:    "2026-05-03T00:00:00Z",
					OperationsSynced: 10,
				},
				{
					ReplicaID:        "replica-2",
					Host:             "10.0.0.3",
					Port:             15002,
					Status:           "connected",
					LastHeartbeat:    "2026-05-03T00:00:01Z",
					OperationsSynced: 9,
				},
			},
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	replicas, err := client.ListReplicas()
	if err != nil {
		t.Fatalf("ListReplicas failed: %v", err)
	}
	if len(replicas) != 2 {
		t.Fatalf("should unwrap replicas envelope and return 2 items but got %d", len(replicas))
	}
	if replicas[0].ReplicaID != "replica-1" {
		t.Errorf("first replica should have ID replica-1 but got %s", replicas[0].ReplicaID)
	}
	if replicas[1].ReplicaID != "replica-2" {
		t.Errorf("second replica should have ID replica-2 but got %s", replicas[1].ReplicaID)
	}
}
