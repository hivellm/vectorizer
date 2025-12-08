package vectorizer

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestUploadFile(t *testing.T) {
	// Mock server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/files/upload" {
			t.Errorf("Expected path '/files/upload', got %s", r.URL.Path)
		}

		if r.Method != "POST" {
			t.Errorf("Expected POST method, got %s", r.Method)
		}

		// Parse multipart form
		err := r.ParseMultipartForm(10 << 20) // 10 MB
		if err != nil {
			t.Fatalf("Failed to parse multipart form: %v", err)
		}

		// Check collection name
		collectionName := r.FormValue("collection_name")
		if collectionName != "test-collection" {
			t.Errorf("Expected collection_name 'test-collection', got %s", collectionName)
		}

		// Check file
		file, header, err := r.FormFile("file")
		if err != nil {
			t.Fatalf("Failed to get file: %v", err)
		}
		defer file.Close()

		if header.Filename != "test.txt" {
			t.Errorf("Expected filename 'test.txt', got %s", header.Filename)
		}

		// Send response
		response := FileUploadResponse{
			Success:          true,
			Filename:         "test.txt",
			CollectionName:   "test-collection",
			ChunksCreated:    5,
			VectorsCreated:   5,
			FileSize:         1024,
			Language:         "text",
			ProcessingTimeMs: 100,
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	// Create client
	client := NewClient(&Config{
		BaseURL: server.URL,
	})

	// Test upload
	content := []byte("This is a test file content for upload testing.")
	response, err := client.UploadFile(content, "test.txt", "test-collection", nil)
	if err != nil {
		t.Fatalf("UploadFile failed: %v", err)
	}

	if !response.Success {
		t.Error("Expected success to be true")
	}

	if response.Filename != "test.txt" {
		t.Errorf("Expected filename 'test.txt', got %s", response.Filename)
	}

	if response.CollectionName != "test-collection" {
		t.Errorf("Expected collection 'test-collection', got %s", response.CollectionName)
	}

	if response.ChunksCreated != 5 {
		t.Errorf("Expected 5 chunks created, got %d", response.ChunksCreated)
	}

	if response.VectorsCreated != 5 {
		t.Errorf("Expected 5 vectors created, got %d", response.VectorsCreated)
	}
}

func TestUploadFileWithOptions(t *testing.T) {
	// Mock server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		err := r.ParseMultipartForm(10 << 20)
		if err != nil {
			t.Fatalf("Failed to parse multipart form: %v", err)
		}

		// Check optional fields
		chunkSize := r.FormValue("chunk_size")
		if chunkSize != "512" {
			t.Errorf("Expected chunk_size '512', got %s", chunkSize)
		}

		chunkOverlap := r.FormValue("chunk_overlap")
		if chunkOverlap != "50" {
			t.Errorf("Expected chunk_overlap '50', got %s", chunkOverlap)
		}

		metadata := r.FormValue("metadata")
		if metadata == "" {
			t.Error("Expected metadata to be present")
		}

		response := FileUploadResponse{
			Success:          true,
			Filename:         "test.txt",
			CollectionName:   "test-collection",
			ChunksCreated:    3,
			VectorsCreated:   3,
			FileSize:         512,
			Language:         "text",
			ProcessingTimeMs: 50,
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	client := NewClient(&Config{
		BaseURL: server.URL,
	})

	chunkSize := 512
	chunkOverlap := 50
	options := &UploadFileOptions{
		ChunkSize:    &chunkSize,
		ChunkOverlap: &chunkOverlap,
		Metadata: map[string]interface{}{
			"source": "test",
			"type":   "document",
		},
	}

	content := []byte("Test content with options")
	response, err := client.UploadFile(content, "test.txt", "test-collection", options)
	if err != nil {
		t.Fatalf("UploadFile with options failed: %v", err)
	}

	if !response.Success {
		t.Error("Expected success to be true")
	}
}

func TestUploadFileContent(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		response := FileUploadResponse{
			Success:          true,
			Filename:         "content.txt",
			CollectionName:   "test-collection",
			ChunksCreated:    2,
			VectorsCreated:   2,
			FileSize:         256,
			Language:         "text",
			ProcessingTimeMs: 30,
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	client := NewClient(&Config{
		BaseURL: server.URL,
	})

	content := "This is direct string content for upload"
	response, err := client.UploadFileContent(content, "content.txt", "test-collection", nil)
	if err != nil {
		t.Fatalf("UploadFileContent failed: %v", err)
	}

	if !response.Success {
		t.Error("Expected success to be true")
	}

	if response.Filename != "content.txt" {
		t.Errorf("Expected filename 'content.txt', got %s", response.Filename)
	}
}

func TestGetUploadConfig(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/files/config" {
			t.Errorf("Expected path '/files/config', got %s", r.URL.Path)
		}

		if r.Method != "GET" {
			t.Errorf("Expected GET method, got %s", r.Method)
		}

		config := FileUploadConfig{
			MaxFileSize:         10485760,
			MaxFileSizeMb:       10,
			AllowedExtensions:   []string{".txt", ".pdf", ".md", ".doc"},
			RejectBinary:        true,
			DefaultChunkSize:    1000,
			DefaultChunkOverlap: 200,
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(config)
	}))
	defer server.Close()

	client := NewClient(&Config{
		BaseURL: server.URL,
	})

	config, err := client.GetUploadConfig()
	if err != nil {
		t.Fatalf("GetUploadConfig failed: %v", err)
	}

	if config.MaxFileSizeMb != 10 {
		t.Errorf("Expected max file size 10MB, got %d", config.MaxFileSizeMb)
	}

	if config.DefaultChunkSize != 1000 {
		t.Errorf("Expected default chunk size 1000, got %d", config.DefaultChunkSize)
	}

	if config.DefaultChunkOverlap != 200 {
		t.Errorf("Expected default chunk overlap 200, got %d", config.DefaultChunkOverlap)
	}

	if !config.RejectBinary {
		t.Error("Expected reject_binary to be true")
	}

	if len(config.AllowedExtensions) != 4 {
		t.Errorf("Expected 4 allowed extensions, got %d", len(config.AllowedExtensions))
	}
}
