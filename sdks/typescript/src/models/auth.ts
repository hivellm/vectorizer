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
