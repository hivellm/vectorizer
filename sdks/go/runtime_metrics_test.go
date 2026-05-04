package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// Phase25 §7 — Go SDK runtime metrics + extended Stats / CollectionInfo.

func TestAdminGetRuntimeMetricsTargetsRoute(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/metrics/runtime" {
			t.Errorf("expected path /metrics/runtime, got %s", r.URL.Path)
		}
		if r.Method != "GET" {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	if _, err := client.GetRuntimeMetrics(); err != nil {
		t.Fatalf("GetRuntimeMetrics returned error: %v", err)
	}
}

func TestAdminGetRuntimeMetricsDecodesFullSnapshot(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"cpu_percent":         12.4,
			"memory_rss_bytes":    124857600,
			"memory_total_bytes":  17179869184,
			"memory_percent":      0.73,
			"active_connections":  8,
			"uptime_seconds":      3712,
			"qps_window_60s":      142.3,
			"error_rate_5xx_60s":  0.001,
			"throughput_by_route": []map[string]interface{}{{"route": "/insert_texts", "qps": 12.0, "p50_ms": 8.2, "p99_ms": 41.0}},
			"wal": map[string]interface{}{
				"current_seq":         482919,
				"size_bytes":          12582912,
				"last_checkpoint_at":  1714828800,
				"last_checkpoint_seq": 482800,
			},
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetRuntimeMetrics()
	if err != nil {
		t.Fatalf("GetRuntimeMetrics returned error: %v", err)
	}
	if got.CPUPercent != 12.4 {
		t.Errorf("CPUPercent: want 12.4, got %f", got.CPUPercent)
	}
	if got.ActiveConnections != 8 {
		t.Errorf("ActiveConnections: want 8, got %d", got.ActiveConnections)
	}
	if len(got.ThroughputByRoute) != 1 {
		t.Fatalf("ThroughputByRoute: want 1 entry, got %d", len(got.ThroughputByRoute))
	}
	if got.ThroughputByRoute[0].Route != "/insert_texts" {
		t.Errorf("ThroughputByRoute[0].Route: want /insert_texts, got %s", got.ThroughputByRoute[0].Route)
	}
	if got.ThroughputByRoute[0].P99Ms != 41.0 {
		t.Errorf("ThroughputByRoute[0].P99Ms: want 41.0, got %f", got.ThroughputByRoute[0].P99Ms)
	}
	if got.WAL.CurrentSeq != 482919 {
		t.Errorf("WAL.CurrentSeq: want 482919, got %d", got.WAL.CurrentSeq)
	}
	if got.WAL.LastCheckpointSeq != 482800 {
		t.Errorf("WAL.LastCheckpointSeq: want 482800, got %d", got.WAL.LastCheckpointSeq)
	}
}

func TestAdminGetRuntimeMetricsTolerantPartialPayload(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		// Older / standalone server: no routes, no WAL block.
		json.NewEncoder(w).Encode(map[string]interface{}{
			"cpu_percent":        1.0,
			"memory_total_bytes": 8000000000,
		})
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})
	got, err := client.GetRuntimeMetrics()
	if err != nil {
		t.Fatalf("GetRuntimeMetrics returned error: %v", err)
	}
	if got.CPUPercent != 1.0 {
		t.Errorf("CPUPercent: want 1.0, got %f", got.CPUPercent)
	}
	if len(got.ThroughputByRoute) != 0 {
		t.Errorf("ThroughputByRoute: want empty, got %d entries", len(got.ThroughputByRoute))
	}
	if got.WAL.CurrentSeq != 0 {
		t.Errorf("WAL.CurrentSeq: want 0, got %d", got.WAL.CurrentSeq)
	}
}

func TestStatsDecodesPhase25QuantizationFields(t *testing.T) {
	raw := []byte(`{
		"collections": 3,
		"total_vectors": 12000,
		"uptime_seconds": 60,
		"version": "3.4.0",
		"default_quantization": "sq-8bit",
		"compression_ratio": 4.0
	}`)
	var s Stats
	if err := json.Unmarshal(raw, &s); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if s.DefaultQuantization != "sq-8bit" {
		t.Errorf("DefaultQuantization: want sq-8bit, got %s", s.DefaultQuantization)
	}
	if s.CompressionRatio != 4.0 {
		t.Errorf("CompressionRatio: want 4.0, got %f", s.CompressionRatio)
	}
}

func TestStatsDecodesPre25Server(t *testing.T) {
	// Older servers omit the new fields entirely. omitempty + zero
	// value gives ("", 0.0) — consumers can fall back to ("none", 1.0).
	raw := []byte(`{
		"collections": 0,
		"total_vectors": 0,
		"uptime_seconds": 0,
		"version": "3.3.0"
	}`)
	var s Stats
	if err := json.Unmarshal(raw, &s); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if s.DefaultQuantization != "" {
		t.Errorf("DefaultQuantization: want empty, got %s", s.DefaultQuantization)
	}
	if s.CompressionRatio != 0.0 {
		t.Errorf("CompressionRatio: want 0.0, got %f", s.CompressionRatio)
	}
}

func TestCollectionInfoDecodesVectorCountHistory(t *testing.T) {
	raw := []byte(`{
		"name": "docs",
		"vector_count": 482919,
		"dimension": 768,
		"metric": "cosine",
		"vector_count_history": [
			{"at": 1714828740, "count": 482900},
			{"at": 1714828800, "count": 482919}
		]
	}`)
	var ci CollectionInfo
	if err := json.Unmarshal(raw, &ci); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if len(ci.VectorCountHistory) != 2 {
		t.Fatalf("VectorCountHistory: want 2 entries, got %d", len(ci.VectorCountHistory))
	}
	if ci.VectorCountHistory[0].Count != 482900 {
		t.Errorf("VectorCountHistory[0].Count: want 482900, got %d", ci.VectorCountHistory[0].Count)
	}
	if ci.VectorCountHistory[1].At != 1714828800 {
		t.Errorf("VectorCountHistory[1].At: want 1714828800, got %d", ci.VectorCountHistory[1].At)
	}
}

func TestCollectionInfoOlderServerHasEmptyHistory(t *testing.T) {
	raw := []byte(`{
		"name": "older",
		"vector_count": 0,
		"dimension": 384,
		"metric": "cosine"
	}`)
	var ci CollectionInfo
	if err := json.Unmarshal(raw, &ci); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if len(ci.VectorCountHistory) != 0 {
		t.Errorf("VectorCountHistory: want empty, got %d entries", len(ci.VectorCountHistory))
	}
}
