package vectorizer

import (
	"fmt"
	"net/url"
)

// GetServerStats returns aggregate collection and vector counts, uptime, and
// version from the server. Calls GET /stats.
//
// Note: GetStats() (returning *DatabaseStats) already exists on *Client in
// client.go for the minimal /stats shape. This method returns the richer
// Stats type that includes UptimeSeconds and Version fields.
func (c *Client) GetServerStats() (*Stats, error) {
	var result Stats
	if err := c.request("GET", "/stats", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetStatus returns server liveness state including version and uptime.
// Calls GET /status.
func (c *Client) GetStatus() (*ServerStatus, error) {
	var result ServerStatus
	if err := c.request("GET", "/status", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetLogs tails recent server log lines. lines controls how many lines to
// return (0 = server default); level filters by log level (empty = all).
// Calls GET /logs?lines=N&level=LEVEL.
func (c *Client) GetLogs(lines int, level string) ([]LogEntry, error) {
	params := url.Values{}
	if lines > 0 {
		params.Set("lines", fmt.Sprintf("%d", lines))
	}
	if level != "" {
		params.Set("level", level)
	}

	path := "/logs"
	if len(params) > 0 {
		path = "/logs?" + params.Encode()
	}

	var envelope struct {
		Logs []LogEntry `json:"logs"`
	}
	if err := c.request("GET", path, nil, &envelope); err != nil {
		return nil, err
	}
	return envelope.Logs, nil
}

// GetIndexingProgress returns per-collection indexing progress.
// Calls GET /indexing/progress.
func (c *Client) GetIndexingProgress() (*IndexingProgress, error) {
	var result IndexingProgress
	if err := c.request("GET", "/indexing/progress", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ForceSaveCollection flushes the named collection to disk immediately.
// Calls POST /collections/{name}/force-save.
func (c *Client) ForceSaveCollection(name string) error {
	path := "/collections/" + url.PathEscape(name) + "/force-save"
	return c.request("POST", path, nil, nil)
}

// ListEmptyCollections returns the names of collections that contain zero
// vectors. Calls GET /collections/empty.
func (c *Client) ListEmptyCollections() ([]string, error) {
	// The server may return either a bare array or {"collections":[...]}
	var raw interface{}
	if err := c.request("GET", "/collections/empty", nil, &raw); err != nil {
		return nil, err
	}

	extractStrings := func(arr []interface{}) []string {
		out := make([]string, 0, len(arr))
		for _, v := range arr {
			if s, ok := v.(string); ok {
				out = append(out, s)
			}
		}
		return out
	}

	switch v := raw.(type) {
	case []interface{}:
		return extractStrings(v), nil
	case map[string]interface{}:
		if cols, ok := v["collections"].([]interface{}); ok {
			return extractStrings(cols), nil
		}
		return []string{}, nil
	default:
		return []string{}, nil
	}
}

// CleanupEmptyCollections deletes all empty collections in one call and
// returns a report of what was removed. Calls DELETE /collections/cleanup.
func (c *Client) CleanupEmptyCollections() (*CleanupReport, error) {
	var result CleanupReport
	if err := c.request("DELETE", "/collections/cleanup", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// GetConfig reads the server's current config.yml as a free-form snapshot.
// Calls GET /config.
func (c *Client) GetConfig() (ConfigSnapshot, error) {
	var result ConfigSnapshot
	if err := c.request("GET", "/config", nil, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// UpdateConfig overwrites the server's config.yml with the provided patch
// and returns the config as echoed back by the server. Calls POST /config.
func (c *Client) UpdateConfig(patch map[string]interface{}) (ConfigSnapshot, error) {
	var result ConfigSnapshot
	if err := c.request("POST", "/config", patch, &result); err != nil {
		return nil, err
	}
	return result, nil
}

// ListBackups returns metadata for all server-side backup files.
// Calls GET /backups.
func (c *Client) ListBackups() ([]BackupInfo, error) {
	var envelope struct {
		Backups []BackupInfo `json:"backups"`
	}
	if err := c.request("GET", "/backups", nil, &envelope); err != nil {
		return nil, err
	}
	return envelope.Backups, nil
}

// CreateBackup creates a new server-side backup and returns its metadata.
// Calls POST /backups/create.
func (c *Client) CreateBackup(req *CreateBackupRequest) (*BackupInfo, error) {
	var result BackupInfo
	if err := c.request("POST", "/backups/create", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// RestoreBackup restores a backup from the server's backup directory.
// Calls POST /backups/restore.
func (c *Client) RestoreBackup(req *RestoreBackupRequest) error {
	return c.request("POST", "/backups/restore", req, nil)
}

// RestartServer initiates a graceful server restart. The server responds
// before the process actually restarts; callers should poll Health() until
// the server is back. Calls POST /admin/restart.
func (c *Client) RestartServer() error {
	return c.request("POST", "/admin/restart", nil, nil)
}

// ListWorkspaces returns all configured workspace directory entries.
// Calls GET /workspace/list.
func (c *Client) ListWorkspaces() ([]WorkspaceConfig, error) {
	var envelope struct {
		Workspaces []WorkspaceConfig `json:"workspaces"`
	}
	if err := c.request("GET", "/workspace/list", nil, &envelope); err != nil {
		return nil, err
	}
	return envelope.Workspaces, nil
}

// GetWorkspaceConfig reads the workspace configuration file.
// Calls GET /workspace/config.
func (c *Client) GetWorkspaceConfig() (*WorkspaceConfig, error) {
	var result WorkspaceConfig
	if err := c.request("GET", "/workspace/config", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// AddWorkspace registers a new workspace directory on the server.
// Calls POST /workspace/add.
func (c *Client) AddWorkspace(req *AddWorkspaceRequest) error {
	return c.request("POST", "/workspace/add", req, nil)
}

// RemoveWorkspace removes a registered workspace directory by path.
// Calls POST /workspace/remove with body {"path": name}.
func (c *Client) RemoveWorkspace(name string) error {
	body := map[string]string{"path": name}
	return c.request("POST", "/workspace/remove", body, nil)
}
