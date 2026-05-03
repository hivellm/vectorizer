/**
 * HiveHub models.
 *
 * Types for user-scoped backup management, usage statistics, quota
 * information, and HiveHub API key validation.
 *
 * These endpoints are only meaningful when the server is running in
 * HiveHub cluster mode.
 */

/** A user-scoped backup entry returned by `GET /hub/backups`. */
export interface UserBackup {
  /** Backup UUID. */
  id: string;
  /** User UUID (owner). */
  user_id: string;
  /** Human-readable backup name. */
  name: string;
  /** Optional description. */
  description?: string;
  /** Collections included. */
  collections: string[];
  /** Creation timestamp (RFC3339). */
  created_at: string;
  /** Backup size in bytes. */
  size: number;
  /** Status string (`"active"`, `"creating"`, etc.). */
  status: string;
}

/** Request for `createUserBackup` (`POST /hub/backups`). */
export interface CreateUserBackupRequest {
  /** User UUID who owns the backup. */
  user_id: string;
  /** Backup name. */
  name: string;
  /** Optional description. */
  description?: string;
  /** Collections to include (undefined = all user collections). */
  collections?: string[];
}

/** Request for `restoreUserBackup` (`POST /hub/backups/restore`). */
export interface RestoreUserBackupRequest {
  /** User UUID. */
  user_id: string;
  /** Backup UUID to restore. */
  backup_id: string;
  /** Whether to overwrite existing collections. */
  overwrite?: boolean;
}

/** Request for `uploadUserBackup` (`POST /hub/backups/upload`). */
export interface UploadUserBackupRequest {
  /** User UUID. */
  user_id: string;
  /** Optional backup name override. */
  name?: string;
  /** Binary backup data (base64-encoded string for JSON transport). */
  data: string;
}

/** Usage statistics returned by `GET /hub/usage/statistics`. */
export interface UsageStatistics {
  /** Whether the call succeeded. */
  success: boolean;
  /** Human-readable message. */
  message: string;
  /** The statistics payload (free-form per server). */
  stats?: Record<string, unknown>;
}

/** Quota information returned by `GET /hub/usage/quota`. */
export interface QuotaInfo {
  /** Whether the call succeeded. */
  success: boolean;
  /** Human-readable message. */
  message: string;
  /** The quota payload (free-form per server). */
  quota?: Record<string, unknown>;
}

/** Validation result returned by `POST /hub/validate-key`. */
export interface HubApiKeyValidation {
  /** Whether the key is valid. */
  valid: boolean;
  /** Tenant id the key belongs to. */
  tenant_id: string;
  /** Tenant name. */
  tenant_name: string;
  /** Permissions granted by the key. */
  permissions: string[];
  /** Validation timestamp (RFC3339). */
  validated_at: string;
}
