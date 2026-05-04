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
  /**
   * Most-common quantization label across active collections (one of
   * `none`, `binary`, `sq-4bit`, `sq-8bit`, `sq-16bit`, `sq`, `pq`).
   * Older servers without phase25 §5 fall back to `none`.
   */
  default_quantization?: string;
  /**
   * Mean compression ratio (uncompressed_bytes / compressed_bytes)
   * across the collections sharing `default_quantization`. `1.0` on
   * older servers or when the store is empty.
   */
  compression_ratio?: number;
}

/** Per-route latency / throughput line in {@link RuntimeMetrics}. */
export interface RouteStats {
  /** Route path (raw URI; templated routes are normalised by the server). */
  route: string;
  /** Queries per second for this route over the last 60 s. */
  qps: number;
  /** 50th-percentile latency in milliseconds. */
  p50_ms: number;
  /** 99th-percentile latency in milliseconds. */
  p99_ms: number;
}

/** WAL state surfaced inside {@link RuntimeMetrics} (phase25 §3). */
export interface WalSnapshot {
  /** Latest offset appended to the WAL. */
  current_seq: number;
  /** On-disk WAL file size in bytes (0 in memory-only mode). */
  size_bytes: number;
  /**
   * Unix timestamp (seconds) at which `last_checkpoint_seq` last
   * advanced. 0 when no replica has confirmed an offset.
   */
  last_checkpoint_at: number;
  /** Lowest offset that has been confirmed by all replicas. */
  last_checkpoint_seq: number;
}

/**
 * Runtime metrics snapshot returned by `GET /metrics/runtime`
 * (phase25). Every field is optional so the SDK tolerates older
 * servers that do not emit the route or partial payloads.
 */
export interface RuntimeMetrics {
  /** CPU usage of the server process, 0–100 %. */
  cpu_percent?: number;
  /** Resident-set size of the server process in bytes. */
  memory_rss_bytes?: number;
  /** Total physical memory of the host in bytes. */
  memory_total_bytes?: number;
  /** RSS as a fraction of total memory, 0–100 %. */
  memory_percent?: number;
  /** Active HTTP connections at the moment of sampling. */
  active_connections?: number;
  /** Seconds since the server process started. */
  uptime_seconds?: number;
  /** Rolling 60-second queries-per-second across all routes. */
  qps_window_60s?: number;
  /** Fraction of requests in the last 60 s with HTTP 5xx status, 0–1. */
  error_rate_5xx_60s?: number;
  /** Per-route latency / throughput. Sorted descending by QPS. */
  throughput_by_route?: RouteStats[];
  /** WAL state. Zero-initialised on standalone servers without replication. */
  wal?: WalSnapshot;
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
