package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestDiscoveryBroadDiscovery(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/discovery/broad_discovery" {
			t.Errorf("expected path '/discovery/broad_discovery', got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST method, got %s", r.Method)
		}

		var reqBody BroadDiscoveryRequest
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if len(reqBody.Queries) == 0 {
			t.Error("expected queries to be non-empty")
		}

		resp := BroadDiscoveryResponse{
			Chunks: []map[string]interface{}{
				{"id": "chunk-1", "text": "hello world"},
				{"id": "chunk-2", "text": "foo bar"},
			},
			Count: 2,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	result, err := client.BroadDiscovery(&BroadDiscoveryRequest{
		Queries: []string{"what is vectorizer", "how does HNSW work"},
	})
	if err != nil {
		t.Fatalf("BroadDiscovery failed: %v", err)
	}
	if result == nil {
		t.Fatal("result should not be nil")
	}
	if result.Count != 2 {
		t.Errorf("expected count 2, got %d", result.Count)
	}
	if len(result.Chunks) != 2 {
		t.Errorf("expected 2 chunks, got %d", len(result.Chunks))
	}
}

func TestDiscoverySemanticFocus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/discovery/semantic_focus" {
			t.Errorf("expected path '/discovery/semantic_focus', got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST method, got %s", r.Method)
		}

		var reqBody SemanticFocusRequest
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if reqBody.Collection == "" {
			t.Error("expected collection to be non-empty")
		}
		if len(reqBody.Queries) == 0 {
			t.Error("expected queries to be non-empty")
		}

		resp := SemanticFocusResponse{
			Chunks: []map[string]interface{}{
				{"id": "focused-1", "score": 0.95},
			},
			Count: 1,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	result, err := client.SemanticFocus(&SemanticFocusRequest{
		Collection: "docs",
		Queries:    []string{"embedding models"},
	})
	if err != nil {
		t.Fatalf("SemanticFocus failed: %v", err)
	}
	if result == nil {
		t.Fatal("result should not be nil")
	}
	if result.Count != 1 {
		t.Errorf("expected count 1, got %d", result.Count)
	}
	if len(result.Chunks) != 1 {
		t.Errorf("expected 1 chunk, got %d", len(result.Chunks))
	}
}

func TestDiscoveryPromoteReadme(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/discovery/promote_readme" {
			t.Errorf("expected path '/discovery/promote_readme', got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST method, got %s", r.Method)
		}

		var reqBody PromoteReadmeRequest
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if len(reqBody.Chunks) == 0 {
			t.Error("expected chunks to be non-empty")
		}

		resp := PromoteReadmeResponse{
			PromotedChunks: []map[string]interface{}{
				{"id": "readme-1", "source": "README.md"},
				{"id": "other-1", "source": "src/main.rs"},
			},
			Count: 2,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	result, err := client.PromoteReadme(&PromoteReadmeRequest{
		Chunks: []map[string]interface{}{
			{"id": "other-1", "source": "src/main.rs"},
			{"id": "readme-1", "source": "README.md"},
		},
	})
	if err != nil {
		t.Fatalf("PromoteReadme failed: %v", err)
	}
	if result == nil {
		t.Fatal("result should not be nil")
	}
	if result.Count != 2 {
		t.Errorf("expected count 2, got %d", result.Count)
	}
	if len(result.PromotedChunks) != 2 {
		t.Errorf("expected 2 promoted chunks, got %d", len(result.PromotedChunks))
	}
}

func TestDiscoveryCompressEvidence(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/discovery/compress_evidence" {
			t.Errorf("expected path '/discovery/compress_evidence', got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST method, got %s", r.Method)
		}

		var reqBody CompressEvidenceRequest
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if len(reqBody.Chunks) == 0 {
			t.Error("expected chunks to be non-empty")
		}

		resp := CompressEvidenceResponse{
			Bullets: []map[string]interface{}{
				{"text": "Vectorizer uses HNSW for indexing", "source": "README.md"},
				{"text": "Sub-3ms search latency", "source": "docs/perf.md"},
			},
			Count: 2,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	maxBullets := 10
	result, err := client.CompressEvidence(&CompressEvidenceRequest{
		Chunks: []map[string]interface{}{
			{"id": "c1", "text": "long chunk text about HNSW"},
			{"id": "c2", "text": "another chunk about latency"},
		},
		MaxBullets: &maxBullets,
	})
	if err != nil {
		t.Fatalf("CompressEvidence failed: %v", err)
	}
	if result == nil {
		t.Fatal("result should not be nil")
	}
	if result.Count != 2 {
		t.Errorf("expected count 2, got %d", result.Count)
	}
	if len(result.Bullets) != 2 {
		t.Errorf("expected 2 bullets, got %d", len(result.Bullets))
	}
}

func TestDiscoveryBuildAnswerPlan(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/discovery/build_answer_plan" {
			t.Errorf("expected path '/discovery/build_answer_plan', got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST method, got %s", r.Method)
		}

		var reqBody AnswerPlanRequest
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if len(reqBody.Bullets) == 0 {
			t.Error("expected bullets to be non-empty")
		}

		resp := AnswerPlan{
			Sections: []map[string]interface{}{
				{"title": "Introduction", "bullets": []string{"Vectorizer is a vector DB"}},
				{"title": "Performance", "bullets": []string{"Sub-3ms search"}},
			},
			TotalBullets: 2,
			Sources:      []string{"README.md", "docs/perf.md"},
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	result, err := client.BuildAnswerPlan(&AnswerPlanRequest{
		Bullets: []map[string]interface{}{
			{"text": "Vectorizer is a vector DB", "source": "README.md"},
			{"text": "Sub-3ms search", "source": "docs/perf.md"},
		},
	})
	if err != nil {
		t.Fatalf("BuildAnswerPlan failed: %v", err)
	}
	if result == nil {
		t.Fatal("result should not be nil")
	}
	if result.TotalBullets != 2 {
		t.Errorf("expected total_bullets 2, got %d", result.TotalBullets)
	}
	if len(result.Sections) != 2 {
		t.Errorf("expected 2 sections, got %d", len(result.Sections))
	}
	if len(result.Sources) != 2 {
		t.Errorf("expected 2 sources, got %d", len(result.Sources))
	}
}

func TestDiscoveryRenderLlmPrompt(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/discovery/render_llm_prompt" {
			t.Errorf("expected path '/discovery/render_llm_prompt', got %s", r.URL.Path)
		}
		if r.Method != "POST" {
			t.Errorf("expected POST method, got %s", r.Method)
		}

		var reqBody RenderPromptRequest
		if err := json.NewDecoder(r.Body).Decode(&reqBody); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if len(reqBody.Plan.Sections) == 0 {
			t.Error("expected plan.sections to be non-empty")
		}

		resp := LlmPrompt{
			Prompt:          "Answer the following question based on the context below:\n\n## Introduction\n- Vectorizer is a vector DB",
			Length:          80,
			EstimatedTokens: 20,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient(&Config{BaseURL: server.URL})

	result, err := client.RenderLlmPrompt(&RenderPromptRequest{
		Plan: AnswerPlan{
			Sections: []map[string]interface{}{
				{"title": "Introduction", "bullets": []string{"Vectorizer is a vector DB"}},
			},
			TotalBullets: 1,
			Sources:      []string{"README.md"},
		},
	})
	if err != nil {
		t.Fatalf("RenderLlmPrompt failed: %v", err)
	}
	if result == nil {
		t.Fatal("result should not be nil")
	}
	if result.Prompt == "" {
		t.Error("expected prompt to be non-empty")
	}
	if result.Length <= 0 {
		t.Errorf("expected length > 0, got %d", result.Length)
	}
	if result.EstimatedTokens <= 0 {
		t.Errorf("expected estimated_tokens > 0, got %d", result.EstimatedTokens)
	}
}
