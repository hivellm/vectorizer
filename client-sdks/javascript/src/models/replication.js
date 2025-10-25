/**
 * Replication Models
 *
 * Data models for replication monitoring and status tracking.
 * Compatible with Vectorizer v1.2.0+
 */

/**
 * Status enum for replica nodes
 * @readonly
 * @enum {string}
 */
const ReplicaStatus = {
  Connected: 'Connected',
  Syncing: 'Syncing',
  Lagging: 'Lagging',
  Disconnected: 'Disconnected',
};

/**
 * Information about a replica node
 * @typedef {Object} ReplicaInfo
 * @property {string} replica_id - Unique identifier for the replica
 * @property {string} host - Hostname or IP address of the replica
 * @property {number} port - Port number of the replica
 * @property {string} status - Current status of the replica
 * @property {Date|string} last_heartbeat - Timestamp of last heartbeat
 * @property {number} operations_synced - Number of operations successfully synced
 * @property {number} [offset] - Legacy: Current offset on replica (deprecated)
 * @property {number} [lag] - Legacy: Lag in operations (deprecated)
 */

/**
 * Statistics for replication status
 * @typedef {Object} ReplicationStats
 * @property {('Master'|'Replica')} [role] - Role of the node: Master or Replica
 * @property {number} [bytes_sent] - Total bytes sent to replicas (Master only)
 * @property {number} [bytes_received] - Total bytes received from master (Replica only)
 * @property {Date|string} [last_sync] - Timestamp of last synchronization
 * @property {number} [operations_pending] - Number of operations pending replication
 * @property {number} [snapshot_size] - Size of snapshot data in bytes
 * @property {number} [connected_replicas] - Number of connected replicas (Master only)
 * @property {number} master_offset - Current offset on master node
 * @property {number} replica_offset - Current offset on replica node
 * @property {number} lag_operations - Number of operations behind
 * @property {number} total_replicated - Total operations replicated
 */

/**
 * Response for replication status endpoint
 * @typedef {Object} ReplicationStatusResponse
 * @property {string} status - Overall status message
 * @property {ReplicationStats} stats - Detailed replication statistics
 * @property {string} [message] - Optional message with additional information
 */

/**
 * Response for listing replicas
 * @typedef {Object} ReplicaListResponse
 * @property {ReplicaInfo[]} replicas - List of replica nodes
 * @property {number} count - Total count of replicas
 * @property {string} message - Status message
 */

/**
 * Validates a ReplicaInfo object
 * @param {Partial<ReplicaInfo>} replica - The replica info to validate
 * @returns {boolean} True if valid, false otherwise
 */
function validateReplicaInfo(replica) {
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
 * @param {Partial<ReplicationStats>} stats - The stats to validate
 * @returns {boolean} True if valid, false otherwise
 */
function validateReplicationStats(stats) {
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

/**
 * Checks if a value is a valid ReplicaStatus
 * @param {any} value - The value to check
 * @returns {boolean} True if valid ReplicaStatus, false otherwise
 */
function isReplicaStatus(value) {
  return Object.values(ReplicaStatus).includes(value);
}

module.exports = {
  ReplicaStatus,
  validateReplicaInfo,
  validateReplicationStats,
  isReplicaStatus,
};

