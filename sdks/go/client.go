package vectorizer

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"
)

// Config holds the client configuration
type Config struct {
	BaseURL        string
	APIKey         string
	Timeout        time.Duration
	HTTPClient     *http.Client
	Hosts          *HostConfig
	ReadPreference ReadPreference
}

// Client is the main Vectorizer client with optional master/replica topology support
type Client struct {
	baseURL          string
	apiKey           string
	httpClient       *http.Client
	masterClient     *http.Client
	replicaClients   []*http.Client
	replicaURLs      []string
	masterURL        string
	replicaIndex     uint32
	readPreference   ReadPreference
	isReplicaMode    bool
	config           *Config
}

// NewClient creates a new Vectorizer client
func NewClient(config *Config) *Client {
	if config == nil {
		config = &Config{}
	}

	if config.BaseURL == "" {
		config.BaseURL = "http://localhost:15002"
	}

	if config.Timeout == 0 {
		config.Timeout = 60 * time.Second
	}

	httpClient := config.HTTPClient
	if httpClient == nil {
		httpClient = &http.Client{
			Timeout: config.Timeout,
		}
	}

	readPreference := config.ReadPreference
	if readPreference == "" {
		readPreference = ReadPreferenceReplica
	}

	client := &Client{
		baseURL:        config.BaseURL,
		apiKey:         config.APIKey,
		httpClient:     httpClient,
		readPreference: readPreference,
		config:         config,
	}

	// Initialize replica mode if hosts are configured
	if config.Hosts != nil {
		client.initializeReplicaMode(config)
	}

	return client
}

// initializeReplicaMode sets up master/replica transports
func (c *Client) initializeReplicaMode(config *Config) {
	c.isReplicaMode = true
	c.masterURL = config.Hosts.Master
	c.replicaURLs = config.Hosts.Replicas

	// Create master HTTP client
	c.masterClient = &http.Client{
		Timeout: config.Timeout,
	}

	// Create replica HTTP clients
	c.replicaClients = make([]*http.Client, len(config.Hosts.Replicas))
	for i := range config.Hosts.Replicas {
		c.replicaClients[i] = &http.Client{
			Timeout: config.Timeout,
		}
	}
}

// getWriteClient returns the HTTP client and base URL for write operations (always master)
func (c *Client) getWriteClient() (*http.Client, string) {
	if c.isReplicaMode && c.masterClient != nil {
		return c.masterClient, c.masterURL
	}
	return c.httpClient, c.baseURL
}

// getReadClient returns the HTTP client and base URL for read operations based on preference
func (c *Client) getReadClient(opts *ReadOptions) (*http.Client, string) {
	if !c.isReplicaMode {
		return c.httpClient, c.baseURL
	}

	preference := c.readPreference
	if opts != nil && opts.ReadPreference != "" {
		preference = opts.ReadPreference
	}

	switch preference {
	case ReadPreferenceMaster:
		return c.masterClient, c.masterURL
	case ReadPreferenceReplica, ReadPreferenceNearest:
		if len(c.replicaClients) == 0 {
			return c.masterClient, c.masterURL
		}
		// Round-robin selection using atomic increment
		idx := c.replicaIndex
		c.replicaIndex = (c.replicaIndex + 1) % uint32(len(c.replicaClients))
		return c.replicaClients[idx], c.replicaURLs[idx]
	default:
		return c.masterClient, c.masterURL
	}
}

// WithMaster creates a new client that always routes reads to master.
// Useful for read-your-writes scenarios.
func (c *Client) WithMaster() *Client {
	masterConfig := *c.config
	masterConfig.ReadPreference = ReadPreferenceMaster
	return NewClient(&masterConfig)
}

// request performs an HTTP request
func (c *Client) request(method, path string, body interface{}, result interface{}) error {
	u, err := url.Parse(c.baseURL + path)
	if err != nil {
		return fmt.Errorf("invalid URL: %w", err)
	}

	var reqBody io.Reader
	if body != nil {
		jsonData, err := json.Marshal(body)
		if err != nil {
			return fmt.Errorf("marshal request body: %w", err)
		}
		reqBody = bytes.NewBuffer(jsonData)
	}

	req, err := http.NewRequest(method, u.String(), reqBody)
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	if c.apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("read response: %w", err)
	}

	if resp.StatusCode >= 400 {
		var errResp ErrorResponse
		if err := json.Unmarshal(respBody, &errResp); err == nil {
			return &VectorizerError{
				Type:    errResp.ErrorType,
				Message: errResp.Message,
				Status:  resp.StatusCode,
				Details: errResp.Details,
			}
		}
		return fmt.Errorf("request failed with status %d: %s", resp.StatusCode, string(respBody))
	}

	if result != nil {
		if err := json.Unmarshal(respBody, result); err != nil {
			return fmt.Errorf("unmarshal response: %w", err)
		}
	}

	return nil
}

// Health checks the server health
func (c *Client) Health() error {
	return c.request("GET", "/health", nil, nil)
}

// GetStats returns database statistics
func (c *Client) GetStats() (*DatabaseStats, error) {
	var stats DatabaseStats
	if err := c.request("GET", "/stats", nil, &stats); err != nil {
		return nil, err
	}
	return &stats, nil
}
