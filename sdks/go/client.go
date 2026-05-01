package vectorizer

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// looksLikeJWT reports whether the given credential looks like a JWT:
// three non-empty base64url-encoded segments separated by `.`. Raw
// Vectorizer API keys (from `POST /auth/keys`) are a single
// alphanumeric string and fail this check, so they're routed to
// `X-API-Key` rather than `Authorization: Bearer`. The server's auth
// middleware treats every Bearer string as a JWT and never falls back
// to the API-key validator.
func looksLikeJWT(token string) bool {
	parts := strings.Split(token, ".")
	if len(parts) != 3 {
		return false
	}
	for _, p := range parts {
		if p == "" {
			return false
		}
	}
	return true
}

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
	baseURL        string
	apiKey         string
	httpClient     *http.Client
	masterClient   *http.Client
	replicaClients []*http.Client
	replicaURLs    []string
	masterURL      string
	replicaIndex   uint32
	readPreference ReadPreference
	isReplicaMode  bool
	config         *Config
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

// Issue #263 retry-after constants. Mirror the Rust + Python + TS
// SDK values so back-pressure behaves consistently across clients.
const (
	retryAfterMaxAttempts = 3
	retryAfterMaxSeconds  = 30
	retryAfterDefaultSecs = 1
)

// parseRetryAfterSeconds parses an HTTP `Retry-After` header value
// (seconds form only). Returns a sane default when missing/zero/junk
// and caps an unreasonably large server hint.
func parseRetryAfterSeconds(value string) int {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return retryAfterDefaultSecs
	}
	var secs int
	_, err := fmt.Sscanf(trimmed, "%d", &secs)
	if err != nil || secs <= 0 {
		return retryAfterDefaultSecs
	}
	if secs > retryAfterMaxSeconds {
		return retryAfterMaxSeconds
	}
	return secs
}

// request performs an HTTP request. Honors `Retry-After` on HTTP 429
// (issue #263): sleeps for the header value (capped) and retries up
// to retryAfterMaxAttempts times before surfacing the error.
func (c *Client) request(method, path string, body interface{}, result interface{}) error {
	u, err := url.Parse(c.baseURL + path)
	if err != nil {
		return fmt.Errorf("invalid URL: %w", err)
	}

	// Marshal body once outside the retry loop; each retry rewinds
	// from the same bytes via a fresh bytes.Reader.
	var bodyBytes []byte
	if body != nil {
		bodyBytes, err = json.Marshal(body)
		if err != nil {
			return fmt.Errorf("marshal request body: %w", err)
		}
	}

	attemptsRemaining := retryAfterMaxAttempts

	for {
		var reqBody io.Reader
		if bodyBytes != nil {
			reqBody = bytes.NewReader(bodyBytes)
		}

		req, err := http.NewRequest(method, u.String(), reqBody)
		if err != nil {
			return fmt.Errorf("create request: %w", err)
		}

		req.Header.Set("Content-Type", "application/json")
		if c.apiKey != "" {
			// JWT shape → `Authorization: Bearer`; raw API keys (from
			// `POST /auth/keys`) → `X-API-Key`. The server's auth
			// middleware treats every Bearer string as a JWT and never
			// falls back to the API-key validator, so routing must
			// happen client-side.
			if looksLikeJWT(c.apiKey) {
				req.Header.Set("Authorization", "Bearer "+c.apiKey)
			} else {
				req.Header.Set("X-API-Key", c.apiKey)
			}
		}

		resp, err := c.httpClient.Do(req)
		if err != nil {
			return fmt.Errorf("request failed: %w", err)
		}

		respBody, err := io.ReadAll(resp.Body)
		resp.Body.Close()
		if err != nil {
			return fmt.Errorf("read response: %w", err)
		}

		if resp.StatusCode == http.StatusTooManyRequests {
			retryAfterSecs := parseRetryAfterSeconds(resp.Header.Get("Retry-After"))
			if attemptsRemaining <= 0 {
				var errResp ErrorResponse
				if jerr := json.Unmarshal(respBody, &errResp); jerr == nil {
					return &VectorizerError{
						Type:    errResp.ErrorType,
						Message: fmt.Sprintf("HTTP 429 after %d retries: %s", retryAfterMaxAttempts, errResp.Message),
						Status:  resp.StatusCode,
						Details: errResp.Details,
					}
				}
				return fmt.Errorf("HTTP 429 after %d retries: %s", retryAfterMaxAttempts, string(respBody))
			}
			attemptsRemaining--
			time.Sleep(time.Duration(retryAfterSecs) * time.Second)
			continue
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
