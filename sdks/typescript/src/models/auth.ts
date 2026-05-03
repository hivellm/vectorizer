/**
 * Authentication models.
 *
 * Types for user management, API key management, JWT tokens,
 * and password policy validation.
 */

/** User record returned by auth endpoints (`GET /auth/me`, `POST /auth/users`). */
export interface User {
  /** Opaque user identifier. */
  user_id: string;
  /** Username string. */
  username: string;
  /** Role names. */
  roles: string[];
}

/** JWT token returned by `POST /auth/refresh`. */
export interface JwtToken {
  /** Bearer access token. */
  access_token: string;
  /** Token type (always `"Bearer"`). */
  token_type: string;
  /** Lifetime in seconds. */
  expires_in: number;
}

/** Password policy report returned by `POST /auth/validate-password`. */
export interface PasswordPolicyReport {
  /** Whether the password satisfies all policy rules. */
  valid: boolean;
  /** Validation errors (empty when valid). */
  errors: string[];
  /** Strength score 0–100. */
  strength: number;
  /** Human-readable strength label. */
  strength_label: string;
}

/** Request body for `createApiKey` (`POST /auth/keys`). */
export interface CreateApiKeyRequest {
  /** Key name / description. */
  name: string;
  /** Permissions (defaults to Read). */
  permissions?: string[];
  /** TTL in seconds from now (undefined = never expires). */
  expires_in?: number;
}

/**
 * API key returned by `POST /auth/keys`.
 *
 * The `api_key` field is only present at creation time — store it securely.
 * `GET /auth/keys` returns entries with `api_key` omitted.
 */
export interface ApiKey {
  /** Key UUID. */
  id: string;
  /** Key name. */
  name: string;
  /** Permissions. */
  permissions: string[];
  /** The raw API key value (only present at creation time). */
  api_key?: string;
  /** Creation timestamp (Unix epoch seconds). */
  created_at: number;
  /** Last-used timestamp. */
  last_used?: number;
  /** Expiry timestamp (undefined = never expires). */
  expires_at?: number;
  /** Whether the key is currently active. */
  active: boolean;
  /** One-time warning message (present at creation). */
  warning?: string;
  /**
   * Total successful credential validations recorded against this key.
   * Defaults to 0 for keys that have never been used (or for servers
   * that don't yet emit the field).
   */
  usage_count?: number;
}

/** Per-collection scope attached to an API key. */
export interface ApiKeyScope {
  /** Collection this scope applies to. */
  collection: string;
  /** Permissions granted on that collection. */
  permissions: string[];
}

/** Request body for `PUT /auth/keys/{id}/permissions`. */
export interface UpdateApiKeyPermissionsRequest {
  /** New permission list. Server rejects an empty list with 400. */
  permissions: string[];
  /**
   * `undefined` leaves existing scopes untouched. `[]` clears scopes
   * (default-deny on scope-aware routes).
   */
  scopes?: ApiKeyScope[];
}

/** Flattened key view returned by the permission-update + usage endpoints. */
export interface ApiKeyView {
  id: string;
  name: string;
  user_id: string;
  permissions: string[];
  scopes: ApiKeyScope[];
  created_at: number;
  last_used?: number;
  expires_at?: number;
  active: boolean;
  usage_count: number;
}

/** One day's usage bucket from `GET /auth/keys/{id}/usage`. */
export interface ApiKeyUsageBucket {
  /** ISO-8601 date (UTC), e.g. `"2026-05-03"`. */
  date: string;
  /** Successful validations recorded for that day. */
  count: number;
}

/** Response body for `GET /auth/keys/{id}/usage`. */
export interface ApiKeyUsageReport {
  /** Live key view with up-to-date `usage_count`. */
  key: ApiKeyView;
  /**
   * Daily counter buckets, oldest first. Days with zero validations
   * are still present so the caller can render a continuous sparkline
   * without gap-fill logic.
   */
  buckets: ApiKeyUsageBucket[];
  /** Sum of `buckets[*].count`. */
  window_total: number;
}

/** Request body for `createUser` (`POST /auth/users`). */
export interface CreateUserRequest {
  /** Username. */
  username: string;
  /** Initial password. */
  password: string;
  /** Roles to assign (defaults to `["User"]`). */
  roles?: string[];
}
