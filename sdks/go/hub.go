package vectorizer

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
)

// listUserBackupsResponse is the JSON envelope returned by GET /hub/backups.
type listUserBackupsResponse struct {
	Backups []UserBackup `json:"backups"`
}

// createUserBackupResponse is the JSON envelope returned by POST /hub/backups.
type createUserBackupResponse struct {
	Backup UserBackup `json:"backup"`
}

// getOrUploadBackupResponse is the JSON envelope returned by
// GET /hub/backups/{id} and POST /hub/backups/upload.
type getOrUploadBackupResponse struct {
	Backup UserBackup `json:"backup"`
}

// uploadBackupBody is the request body for POST /hub/backups/upload.
// The Rust SDK encodes the binary payload as a base64 string under the
// "data" key, matching the server's expected JSON shape.
type uploadBackupBody struct {
	Data string `json:"data"`
}

// ListUserBackups returns all backups owned by the given user.
//
// Calls GET /hub/backups?user_id={userID}.
func (c *Client) ListUserBackups(userID string) ([]UserBackup, error) {
	qs := url.Values{"user_id": {userID}}.Encode()
	var envelope listUserBackupsResponse
	if err := c.request("GET", "/hub/backups?"+qs, nil, &envelope); err != nil {
		return nil, err
	}
	return envelope.Backups, nil
}

// CreateUserBackup creates a new backup for a user.
//
// Calls POST /hub/backups with the request body containing user_id, name,
// optional description, and optional collections slice.
func (c *Client) CreateUserBackup(req *CreateUserBackupRequest) (*UserBackup, error) {
	var envelope createUserBackupResponse
	if err := c.request("POST", "/hub/backups", req, &envelope); err != nil {
		return nil, err
	}
	return &envelope.Backup, nil
}

// RestoreUserBackup restores a previously created backup.
//
// Calls POST /hub/backups/restore with user_id, backup_id, and optional
// overwrite flag in the request body.
func (c *Client) RestoreUserBackup(req *RestoreUserBackupRequest) error {
	return c.request("POST", "/hub/backups/restore", req, nil)
}

// UploadUserBackup uploads raw backup data for a user.
//
// Calls POST /hub/backups/upload?user_id={userID}&name={name}.
// The binary data is base64-encoded and sent as {"data": "<base64>"} in the
// request body, matching the Rust SDK's encoding convention.
func (c *Client) UploadUserBackup(userID, name string, data []byte) (*UserBackup, error) {
	qs := url.Values{
		"user_id": {userID},
		"name":    {name},
	}.Encode()
	body := uploadBackupBody{
		Data: base64.StdEncoding.EncodeToString(data),
	}
	var envelope getOrUploadBackupResponse
	if err := c.request("POST", "/hub/backups/upload?"+qs, body, &envelope); err != nil {
		return nil, err
	}
	return &envelope.Backup, nil
}

// GetUserBackup fetches metadata for a single backup.
//
// Calls GET /hub/backups/{backupID}?user_id={userID}.
func (c *Client) GetUserBackup(userID, backupID string) (*UserBackup, error) {
	qs := url.Values{"user_id": {userID}}.Encode()
	path := fmt.Sprintf("/hub/backups/%s?%s", backupID, qs)
	var envelope getOrUploadBackupResponse
	if err := c.request("GET", path, nil, &envelope); err != nil {
		return nil, err
	}
	return &envelope.Backup, nil
}

// DeleteUserBackup deletes a backup by ID.
//
// Calls DELETE /hub/backups/{backupID}?user_id={userID}.
func (c *Client) DeleteUserBackup(userID, backupID string) error {
	qs := url.Values{"user_id": {userID}}.Encode()
	path := fmt.Sprintf("/hub/backups/%s?%s", backupID, qs)
	return c.request("DELETE", path, nil, nil)
}

// DownloadUserBackup downloads the raw binary content of a backup.
//
// Calls GET /hub/backups/{backupID}/download?user_id={userID}.
//
// This method bypasses c.request because that helper JSON-decodes every
// response body; backup downloads are raw binary (not JSON). Instead it
// constructs the HTTP request manually using c.httpClient, applies the
// same Authorization/X-API-Key header selection as c.request (via
// looksLikeJWT), and returns the response body bytes directly.
//
// Rate-limiting (HTTP 429): one attempt only. If the server returns 429
// the error is surfaced immediately rather than retried, because the
// retry-after sleep loop in c.request is not replicated here to keep
// this helper minimal.
func (c *Client) DownloadUserBackup(userID, backupID string) ([]byte, error) {
	qs := url.Values{"user_id": {userID}}.Encode()
	rawURL := fmt.Sprintf("%s/hub/backups/%s/download?%s", c.baseURL, backupID, qs)

	req, err := http.NewRequest("GET", rawURL, nil)
	if err != nil {
		return nil, fmt.Errorf("create download request: %w", err)
	}

	if c.apiKey != "" {
		if looksLikeJWT(c.apiKey) {
			req.Header.Set("Authorization", "Bearer "+c.apiKey)
		} else {
			req.Header.Set("X-API-Key", c.apiKey)
		}
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("download request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read download response: %w", err)
	}

	if resp.StatusCode >= 400 {
		var errResp ErrorResponse
		if jerr := json.Unmarshal(body, &errResp); jerr == nil {
			return nil, &VectorizerError{
				Type:    errResp.ErrorType,
				Message: errResp.Message,
				Status:  resp.StatusCode,
				Details: errResp.Details,
			}
		}
		return nil, fmt.Errorf("download failed with status %d: %s", resp.StatusCode, string(body))
	}

	return body, nil
}

// GetUsageStatistics returns aggregate usage statistics for a user.
//
// Calls GET /hub/usage/statistics?user_id={userID}.
func (c *Client) GetUsageStatistics(userID string) (*UsageStatistics, error) {
	qs := url.Values{"user_id": {userID}}.Encode()
	var result UsageStatistics
	if err := c.request("GET", "/hub/usage/statistics?"+qs, nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetQuotaInfo returns quota information for a user.
//
// Calls GET /hub/usage/quota?user_id={userID}.
func (c *Client) GetQuotaInfo(userID string) (*QuotaInfo, error) {
	qs := url.Values{"user_id": {userID}}.Encode()
	var result QuotaInfo
	if err := c.request("GET", "/hub/usage/quota?"+qs, nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ValidateHubAPIKey validates a HiveHub API key.
//
// Calls POST /hub/validate-key with {"key": key} in the request body.
// The key being validated is passed in the body and may differ from the
// credential configured on the client itself.
func (c *Client) ValidateHubAPIKey(key string) (*HubApiKeyValidation, error) {
	body := map[string]string{"key": key}
	var result HubApiKeyValidation
	if err := c.request("POST", "/hub/validate-key", body, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// hubDownloadRaw performs a raw GET using a pre-built URL string and
// returns the response bytes. It is factored out so tests can call it
// with a custom base URL.
func hubDownloadRaw(httpClient *http.Client, rawURL, apiKey string) ([]byte, error) {
	req, err := http.NewRequest("GET", rawURL, nil)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	if apiKey != "" {
		if looksLikeJWT(apiKey) {
			req.Header.Set("Authorization", "Bearer "+apiKey)
		} else {
			req.Header.Set("X-API-Key", apiKey)
		}
	}
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}
	if resp.StatusCode >= 400 {
		var errResp ErrorResponse
		if jerr := json.Unmarshal(body, &errResp); jerr == nil {
			return nil, &VectorizerError{
				Type:    errResp.ErrorType,
				Message: errResp.Message,
				Status:  resp.StatusCode,
				Details: errResp.Details,
			}
		}
		return nil, fmt.Errorf("status %d: %s", resp.StatusCode, string(body))
	}
	return body, nil
}
