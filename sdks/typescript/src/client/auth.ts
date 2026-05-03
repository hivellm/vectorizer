/**
 * Authentication surface.
 *
 * Covers the `/auth/*` REST endpoints: current-user info, logout,
 * token refresh, password validation, API key management (CRUD), user
 * management (admin), and the phase15 admin endpoints: key rotation,
 * scoped key creation, token introspection, and audit-log query.
 */

import { BaseClient } from './_base';
import type {
  ApiKey,
  ApiKeyUsageReport,
  ApiKeyView,
  CreateApiKeyRequest,
  CreateUserRequest,
  JwtToken,
  PasswordPolicyReport,
  UpdateApiKeyPermissionsRequest,
  User,
} from '../models';
import type {
  AuditEntry,
  AuditQuery,
  CreateScopedApiKeyRequest,
  RotatedKey,
  TokenIntrospection,
} from '../models/cluster-auth-admin';

export class AuthClient extends BaseClient {
  /** Return the authenticated user's claims. Calls `GET /auth/me`. */
  public async me(): Promise<User> {
    this.logger.debug('Getting current user');
    return this.transport.get<User>('/auth/me');
  }

  /**
   * Invalidate the current session token.
   * Calls `POST /auth/logout`.
   */
  public async logout(): Promise<void> {
    this.logger.debug('Logging out');
    await this.transport.post('/auth/logout', {});
  }

  /**
   * Exchange the current token for a fresh one with an extended TTL.
   * Calls `POST /auth/refresh`.
   */
  public async refreshToken(): Promise<JwtToken> {
    this.logger.debug('Refreshing token');
    return this.transport.post<JwtToken>('/auth/refresh', {});
  }

  /**
   * Validate a password against the server's password policy without
   * creating an account. Calls `POST /auth/validate-password`.
   */
  public async validatePassword(password: string): Promise<PasswordPolicyReport> {
    this.logger.debug('Validating password');
    return this.transport.post<PasswordPolicyReport>('/auth/validate-password', { password });
  }

  /**
   * Create a new API key for the calling user.
   * Calls `POST /auth/keys`. The `api_key` field in the response is
   * only present at creation time — store it securely.
   */
  public async createApiKey(request: CreateApiKeyRequest): Promise<ApiKey> {
    this.logger.debug('Creating API key', { name: request.name });
    return this.transport.post<ApiKey>('/auth/keys', request);
  }

  /**
   * List the API keys belonging to the calling user.
   * Calls `GET /auth/keys`. The `api_key` field is omitted in list responses.
   */
  public async listApiKeys(): Promise<ApiKey[]> {
    this.logger.debug('Listing API keys');
    const response = await this.transport.get<{ keys: ApiKey[] }>('/auth/keys');
    return response.keys ?? [];
  }

  /**
   * Revoke an API key by id.
   * Calls `DELETE /auth/keys/{id}`.
   */
  public async revokeApiKey(id: string): Promise<void> {
    this.logger.debug('Revoking API key', { id });
    await this.transport.delete(`/auth/keys/${id}`);
  }

  /**
   * Create a new user (admin only).
   * Calls `POST /auth/users`.
   */
  public async createUser(request: CreateUserRequest): Promise<User> {
    this.logger.debug('Creating user', { username: request.username });
    return this.transport.post<User>('/auth/users', request);
  }

  /**
   * List all users (admin only).
   * Calls `GET /auth/users`.
   */
  public async listUsers(): Promise<User[]> {
    this.logger.debug('Listing users');
    const response = await this.transport.get<{ users: User[] }>('/auth/users');
    return response.users ?? [];
  }

  /**
   * Delete a user (admin only).
   * Calls `DELETE /auth/users/{username}`.
   */
  public async deleteUser(username: string): Promise<void> {
    this.logger.debug('Deleting user', { username });
    await this.transport.delete(`/auth/users/${username}`);
  }

  /**
   * Change a user's password.
   * Calls `PUT /auth/users/{username}/password` with `{new_password}`.
   * Admins can change any password; non-admins must also supply `current_password`.
   */
  public async changePassword(username: string, newPassword: string): Promise<void> {
    this.logger.debug('Changing password', { username });
    await this.transport.put(`/auth/users/${username}/password`, {
      new_password: newPassword,
    });
  }

  // ── phase15 admin endpoints ─────────────────────────────────────────────────

  /**
   * Atomically rotate an API key (admin only).
   * Calls `POST /auth/keys/{id}/rotate` with an empty body.
   * Returns both the old and new tokens plus a grace window.
   */
  public async rotateApiKey(id: string): Promise<RotatedKey> {
    this.logger.debug('Rotating API key', { id });
    return this.transport.post<RotatedKey>(`/auth/keys/${id}/rotate`, {});
  }

  /**
   * Create an API key with optional per-collection scopes.
   * Calls `POST /auth/keys`. When `scopes` is non-empty the key is restricted
   * to the listed collections.
   */
  public async createScopedApiKey(request: CreateScopedApiKeyRequest): Promise<ApiKey> {
    this.logger.debug('Creating scoped API key', { name: request.name });
    return this.transport.post<ApiKey>('/auth/keys', request);
  }

  /**
   * Replace `permissions` (and optionally `scopes`) on an existing API
   * key without rotating the credential. Admin-only.
   *
   * Calls `PUT /auth/keys/{id}/permissions`. Returns the updated key
   * view with the live `usage_count` stamped in.
   *
   * `key_hash`, `id`, `user_id`, and `created_at` are immutable —
   * rotate the key with `rotateApiKey` if those need to change.
   */
  public async updateApiKeyPermissions(
    id: string,
    request: UpdateApiKeyPermissionsRequest,
  ): Promise<ApiKeyView> {
    this.logger.debug('Updating API key permissions', { id });
    return this.transport.put<ApiKeyView>(`/auth/keys/${id}/permissions`, request);
  }

  /**
   * Fetch the per-day usage time-series for an API key. Admin-only.
   *
   * Calls `GET /auth/keys/{id}/usage?window={days}`. `days` is clamped
   * server-side to 1..=30; `undefined` defaults to 7. The response
   * carries the live key view, the bucket array (oldest first,
   * including zero-count days), and the window total.
   */
  public async getApiKeyUsage(id: string, windowDays?: number): Promise<ApiKeyUsageReport> {
    this.logger.debug('Getting API key usage', { id, windowDays });
    const path =
      windowDays !== undefined
        ? `/auth/keys/${id}/usage?window=${windowDays}`
        : `/auth/keys/${id}/usage`;
    return this.transport.get<ApiKeyUsageReport>(path);
  }

  /**
   * Introspect a token — RFC 7662.
   * Calls `POST /auth/introspect` with `{token}`. Requires authentication but
   * not admin. Returns `active: false` for any unrecognized token.
   */
  public async introspectToken(token: string): Promise<TokenIntrospection> {
    this.logger.debug('Introspecting token');
    return this.transport.post<TokenIntrospection>('/auth/introspect', { token });
  }

  /**
   * Query the admin audit log (admin only).
   * Calls `GET /auth/audit` with optional query parameters.
   * Returns entries newest-first, bounded by `query.limit` (server default 200).
   */
  public async listAuditLog(query: AuditQuery = {}): Promise<AuditEntry[]> {
    this.logger.debug('Listing audit log');
    const params = new URLSearchParams();
    if (query.actor !== undefined) params.set('actor', query.actor);
    if (query.action !== undefined) params.set('action', query.action);
    if (query.since !== undefined) params.set('since', query.since);
    if (query.until !== undefined) params.set('until', query.until);
    if (query.limit !== undefined) params.set('limit', String(query.limit));
    const qs = params.toString();
    const path = qs ? `/auth/audit?${qs}` : '/auth/audit';
    const response = await this.transport.get<{ entries: AuditEntry[] }>(path);
    return response.entries ?? [];
  }
}
