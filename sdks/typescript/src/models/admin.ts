/**
 * Admin / observability models.
 *
 * Types for server statistics, status, logs, indexing progress,
 * collection maintenance, server config, backup management, and
 * workspace management.
 */

/** Server statistics returned by `GET /stats`. */
export interface Stats {
  /** Number of collections. */
  collections: number;
  /** Total vectors across all collections. */
  total_vectors: number;
  /** Server uptime in seconds. */
  uptime_seconds: number;
  /** Server version string. */
  version: string;
}

/** Server status returned by `GET /status`. */
export interface ServerStatus {
  /** Whether the server is online. */
  online: boolean;
  /** Version string. */
  version: string;
  /** Uptime in seconds. */
  uptime_seconds: number;
  /** Number of collections. */
  collections_count: number;
}

/** One entry in the collection-progress list inside `IndexingProgress`. */
export interface CollectionProgress {
  /** Collection name. */
  collection_name: string;
  /** Status string. */
  status: string;
  /** Progress percentage (0–100). */
  progress: number;
  /** Current vector count. */
  vector_count: number;
  /** Error message, if any. */
  error_message?: string;
  /** ISO-8601 last-updated timestamp. */
  last_updated: string;
}

/** Indexing progress returned by `GET /indexing/progress`. */
export interface IndexingProgress {
  /** Whether indexing is currently in progress. */
  is_indexing: boolean;
  /** Overall status string. */
  overall_status: string;
  /** Per-collection progress entries. */
  collections: CollectionProgress[];
}

/** Report returned by `DELETE /collections/cleanup`. */
export interface CleanupReport {
  /** Whether the cleanup succeeded. */
  success: boolean;
  /** Number of collections removed. */
  removed: number;
  /** Names of removed collections. */
  collections: string[];
  /** Optional message from the server. */
  message?: string;
}

/** One log entry returned by `GET /logs`. */
export interface LogEntry {
  /** ISO-8601 timestamp. */
  timestamp: string;
  /** Log level. */
  level: string;
  /** Log message. */
  message: string;
  /** Source component. */
  source: string;
}

/** Metadata for one server-side backup file returned by `GET /backups`. */
export interface BackupInfo {
  /** UUID of the backup. */
  id: string;
  /** Human-readable name. */
  name: string;
  /** Creation timestamp (RFC3339). */
  date: string;
  /** File size in bytes. */
  size: number;
  /** Collection names included in the backup. */
  collections: string[];
}

/** Request body for `createBackup` (`POST /backups/create`). */
export interface CreateBackupRequest {
  /** Backup name. */
  name: string;
  /** Collection names to include (empty = all). */
  collections?: string[];
}

/** Request body for `restoreBackup` (`POST /backups/restore`). */
export interface RestoreBackupRequest {
  /** ID of the backup to restore. */
  backup_id: string;
}

/** Request body for `addWorkspace` (`POST /workspace/add`). */
export interface AddWorkspaceRequest {
  /** File-system path to watch. */
  path: string;
  /** Collection name to index files into. */
  collection_name: string;
}
