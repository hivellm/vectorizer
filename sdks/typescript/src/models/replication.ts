/**
 * Replication Models
 *
 * Data models for replication monitoring and status tracking.
 * Compatible with Vectorizer v1.3.0+
 */

/**
 * Status of a replica node
 */
export enum ReplicaStatus {
  Connected = 'Connected',
  Syncing = 'Syncing',
  Lagging = 'Lagging',
  Disconnected = 'Disconnected',
}

/**
 * Information about a replica node
 */
export interface ReplicaInfo {
  /** Unique identifier for the replica */
  replica_id: string;
  /** Hostname or IP address of the replica */
  host: string;
  /** Port number of the replica */
  port: number;
  /** Current status of the replica */
  status: ReplicaStatus | string;
  /** Timestamp of last heartbeat */
  last_heartbeat: Date | string;
  /** Number of operations successfully synced */
  operations_synced: number;
  
  // Legacy fields (backwards compatible)
  /** Legacy: Current offset on replica (deprecated, use operations_synced) */
  offset?: number;
  /** Legacy: Lag in operations (deprecated, use status) */
  lag?: number;
}

/**
 * Statistics for replication status
 */
export interface ReplicationStats {
  // New fields (v1.2.0+)
  /** Role of the node: Master or Replica */
  role?: 'Master' | 'Replica';
  /** Total bytes sent to replicas (Master only) */
  bytes_sent?: number;
  /** Total bytes received from master (Replica only) */
  bytes_received?: number;
  /** Timestamp of last synchronization */
  last_sync?: Date | string;
  /** Number of operations pending replication */
  operations_pending?: number;
  /** Size of snapshot data in bytes */
  snapshot_size?: number;
  /** Number of connected replicas (Master only) */
  connected_replicas?: number;
  
  // Legacy fields (backwards compatible)
  /** Current offset on master node */
  master_offset: number;
  /** Current offset on replica node */
  replica_offset: number;
  /** Number of operations behind */
  lag_operations: number;
  /** Total operations replicated */
  total_replicated: number;
}

/**
 * Response for replication status endpoint
 */
export interface ReplicationStatusResponse {
  /** Overall status message */
  status: string;
  /** Detailed replication statistics */
  stats: ReplicationStats;
  /** Optional message with additional information */
  message?: string;
}

/**
 * Response for listing replicas
 */
export interface ReplicaListResponse {
  /** List of replica nodes */
  replicas: ReplicaInfo[];
  /** Total count of replicas */
  count: number;
  /** Status message */
  message: string;
}

/**
 * Type guard to check if a value is a valid ReplicaStatus
 */
export function isReplicaStatus(value: any): value is ReplicaStatus {
  return Object.values(ReplicaStatus).includes(value as ReplicaStatus);
}

/**
 * Validates a ReplicaInfo object
 */
export function validateReplicaInfo(replica: Partial<ReplicaInfo>): replica is ReplicaInfo {
  if (!replica.replica_id || typeof replica.replica_id !== 'string') {
    return false;
  }
  if (!replica.host || typeof replica.host !== 'string') {
    return false;
  }
  if (!replica.port || typeof replica.port !== 'number' || replica.port <= 0 || replica.port > 65535) {
    return false;
  }
  if (!replica.status || typeof replica.status !== 'string') {
    return false;
  }
  if (replica.operations_synced === undefined || typeof replica.operations_synced !== 'number' || replica.operations_synced < 0) {
    return false;
  }
  return true;
}

/**
 * Validates a ReplicationStats object
 */
export function validateReplicationStats(stats: Partial<ReplicationStats>): stats is ReplicationStats {
  // Validate legacy fields (required for backwards compatibility)
  if (typeof stats.master_offset !== 'number' || stats.master_offset < 0) {
    return false;
  }
  if (typeof stats.replica_offset !== 'number' || stats.replica_offset < 0) {
    return false;
  }
  if (typeof stats.lag_operations !== 'number' || stats.lag_operations < 0) {
    return false;
  }
  if (typeof stats.total_replicated !== 'number' || stats.total_replicated < 0) {
    return false;
  }
  
  // Validate new fields if present
  if (stats.role !== undefined && stats.role !== 'Master' && stats.role !== 'Replica') {
    return false;
  }
  if (stats.bytes_sent !== undefined && (typeof stats.bytes_sent !== 'number' || stats.bytes_sent < 0)) {
    return false;
  }
  if (stats.bytes_received !== undefined && (typeof stats.bytes_received !== 'number' || stats.bytes_received < 0)) {
    return false;
  }
  if (stats.operations_pending !== undefined && (typeof stats.operations_pending !== 'number' || stats.operations_pending < 0)) {
    return false;
  }
  if (stats.snapshot_size !== undefined && (typeof stats.snapshot_size !== 'number' || stats.snapshot_size < 0)) {
    return false;
  }
  if (stats.connected_replicas !== undefined && (typeof stats.connected_replicas !== 'number' || stats.connected_replicas < 0)) {
    return false;
  }
  
  return true;
}

