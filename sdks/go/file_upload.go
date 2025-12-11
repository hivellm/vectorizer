package vectorizer

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
)

// FileUploadRequest represents the request to upload a file
type FileUploadRequest struct {
	CollectionName string                 `json:"collection_name"`
	ChunkSize      *int                   `json:"chunk_size,omitempty"`
	ChunkOverlap   *int                   `json:"chunk_overlap,omitempty"`
	Metadata       map[string]interface{} `json:"metadata,omitempty"`
	PublicKey      string                 `json:"public_key,omitempty"` // Optional ECC public key for payload encryption
}

// FileUploadResponse represents the response from file upload
type FileUploadResponse struct {
	Success          bool   `json:"success"`
	Filename         string `json:"filename"`
	CollectionName   string `json:"collection_name"`
	ChunksCreated    int    `json:"chunks_created"`
	VectorsCreated   int    `json:"vectors_created"`
	FileSize         int64  `json:"file_size"`
	Language         string `json:"language"`
	ProcessingTimeMs int64  `json:"processing_time_ms"`
}

// FileUploadConfig represents the server's file upload configuration
type FileUploadConfig struct {
	MaxFileSize         int64    `json:"max_file_size"`
	MaxFileSizeMb       int      `json:"max_file_size_mb"`
	AllowedExtensions   []string `json:"allowed_extensions"`
	RejectBinary        bool     `json:"reject_binary"`
	DefaultChunkSize    int      `json:"default_chunk_size"`
	DefaultChunkOverlap int      `json:"default_chunk_overlap"`
}

// UploadFileOptions contains optional parameters for file upload
type UploadFileOptions struct {
	ChunkSize    *int
	ChunkOverlap *int
	Metadata     map[string]interface{}
	PublicKey    string // Optional ECC public key for payload encryption (PEM, base64, or hex format)
}

// UploadFile uploads a file for automatic text extraction, chunking, and indexing
func (c *Client) UploadFile(fileContent []byte, filename, collectionName string, options *UploadFileOptions) (*FileUploadResponse, error) {
	// Create multipart form
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	// Add file
	part, err := writer.CreateFormFile("file", filename)
	if err != nil {
		return nil, fmt.Errorf("failed to create form file: %w", err)
	}
	if _, err := part.Write(fileContent); err != nil {
		return nil, fmt.Errorf("failed to write file content: %w", err)
	}

	// Add collection name
	if err := writer.WriteField("collection_name", collectionName); err != nil {
		return nil, fmt.Errorf("failed to write collection_name: %w", err)
	}

	// Add optional fields
	if options != nil {
		if options.ChunkSize != nil {
			if err := writer.WriteField("chunk_size", fmt.Sprintf("%d", *options.ChunkSize)); err != nil {
				return nil, fmt.Errorf("failed to write chunk_size: %w", err)
			}
		}

		if options.ChunkOverlap != nil {
			if err := writer.WriteField("chunk_overlap", fmt.Sprintf("%d", *options.ChunkOverlap)); err != nil {
				return nil, fmt.Errorf("failed to write chunk_overlap: %w", err)
			}
		}

		if options.Metadata != nil {
			metadataJSON, err := json.Marshal(options.Metadata)
			if err != nil {
				return nil, fmt.Errorf("failed to marshal metadata: %w", err)
			}
			if err := writer.WriteField("metadata", string(metadataJSON)); err != nil {
				return nil, fmt.Errorf("failed to write metadata: %w", err)
			}
		}

		if options.PublicKey != "" {
			if err := writer.WriteField("public_key", options.PublicKey); err != nil {
				return nil, fmt.Errorf("failed to write public_key: %w", err)
			}
		}
	}

	if err := writer.Close(); err != nil {
		return nil, fmt.Errorf("failed to close multipart writer: %w", err)
	}

	// Create request
	httpClient, baseURL := c.getWriteClient()
	fullURL := baseURL + "/files/upload"

	req, err := http.NewRequest("POST", fullURL, body)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", writer.FormDataContentType())
	if c.apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	// Send request
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	// Read response
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("upload failed with status %d: %s", resp.StatusCode, string(respBody))
	}

	// Parse response
	var uploadResp FileUploadResponse
	if err := json.Unmarshal(respBody, &uploadResp); err != nil {
		return nil, fmt.Errorf("failed to parse response: %w", err)
	}

	return &uploadResp, nil
}

// UploadFileContent uploads file content directly as a string
func (c *Client) UploadFileContent(content, filename, collectionName string, options *UploadFileOptions) (*FileUploadResponse, error) {
	return c.UploadFile([]byte(content), filename, collectionName, options)
}

// GetUploadConfig retrieves the file upload configuration from the server
func (c *Client) GetUploadConfig() (*FileUploadConfig, error) {
	httpClient, baseURL := c.getReadClient(nil)
	fullURL := baseURL + "/files/config"

	req, err := http.NewRequest("GET", fullURL, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	if c.apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("request failed with status %d: %s", resp.StatusCode, string(respBody))
	}

	var config FileUploadConfig
	if err := json.Unmarshal(respBody, &config); err != nil {
		return nil, fmt.Errorf("failed to parse response: %w", err)
	}

	return &config, nil
}
