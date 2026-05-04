package vectorizer

import (
	"encoding/json"
	"fmt"
	"net/url"
)

// Me returns the current user's claims (GET /auth/me).
func (c *Client) Me() (*User, error) {
	var result User
	if err := c.request("GET", "/auth/me", nil, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// Logout invalidates the current session token (POST /auth/logout).
func (c *Client) Logout() error {
	return c.request("POST", "/auth/logout", nil, nil)
}

// RefreshToken exchanges the current token for a fresh one (POST /auth/refresh).
func (c *Client) RefreshToken() (*JwtToken, error) {
	var result JwtToken
	if err := c.request("POST", "/auth/refresh", map[string]interface{}{}, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// ValidatePassword checks a password against the server's password policy
// without creating an account (POST /auth/validate-password).
func (c *Client) ValidatePassword(password string) (*PasswordPolicyReport, error) {
	var result PasswordPolicyReport
	body := map[string]interface{}{"password": password}
	if err := c.request("POST", "/auth/validate-password", body, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// CreateApiKey creates a new API key for the calling user (POST /auth/keys).
// The ApiKeyValue field in the returned ApiKey is only present at creation time.
func (c *Client) CreateApiKey(req *CreateApiKeyRequest) (*ApiKey, error) {
	var result ApiKey
	if err := c.request("POST", "/auth/keys", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// listApiKeysResponse is the envelope returned by GET /auth/keys.
type listApiKeysResponse struct {
	Keys json.RawMessage `json:"keys"`
}

// ListApiKeys returns the API keys belonging to the calling user (GET /auth/keys).
func (c *Client) ListApiKeys() ([]ApiKey, error) {
	var envelope listApiKeysResponse
	if err := c.request("GET", "/auth/keys", nil, &envelope); err != nil {
		return nil, err
	}
	if envelope.Keys == nil {
		return []ApiKey{}, nil
	}
	var keys []ApiKey
	if err := json.Unmarshal(envelope.Keys, &keys); err != nil {
		return nil, fmt.Errorf("unmarshal api keys: %w", err)
	}
	return keys, nil
}

// RevokeApiKey revokes an API key by id (DELETE /auth/keys/{id}).
func (c *Client) RevokeApiKey(id string) error {
	return c.request("DELETE", "/auth/keys/"+url.PathEscape(id), nil, nil)
}

// CreateUser creates a new user (POST /auth/users). Requires admin role.
func (c *Client) CreateUser(req *CreateUserRequest) (*User, error) {
	var result User
	if err := c.request("POST", "/auth/users", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// listUsersResponse is the envelope returned by GET /auth/users.
type listUsersResponse struct {
	Users json.RawMessage `json:"users"`
}

// ListUsers returns all users (GET /auth/users). Requires admin role.
func (c *Client) ListUsers() ([]User, error) {
	var envelope listUsersResponse
	if err := c.request("GET", "/auth/users", nil, &envelope); err != nil {
		return nil, err
	}
	if envelope.Users == nil {
		return []User{}, nil
	}
	var users []User
	if err := json.Unmarshal(envelope.Users, &users); err != nil {
		return nil, fmt.Errorf("unmarshal users: %w", err)
	}
	return users, nil
}

// DeleteUser deletes a user by username (DELETE /auth/users/{username}).
// Requires admin role. The server refuses to delete self or the last admin.
func (c *Client) DeleteUser(username string) error {
	return c.request("DELETE", "/auth/users/"+url.PathEscape(username), nil, nil)
}

// ChangePassword sets a new password for the given user
// (PUT /auth/users/{username}/password).
// Admins can change any password; non-admins must also supply their current
// password at the server level.
func (c *Client) ChangePassword(username, newPassword string) error {
	body := map[string]interface{}{"new_password": newPassword}
	return c.request("PUT", "/auth/users/"+url.PathEscape(username)+"/password", body, nil)
}
