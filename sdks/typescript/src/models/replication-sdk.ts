/**
 * Replication SDK models (phase12).
 *
 * These complement the existing `replication.ts` models by adding
 * the phase12 typed structs for `getReplicationStatus`,
 * `configureReplication`, `getReplicationStats`, and `listReplicas`.
 */

import type { ReplicaInfo, ReplicationStats } from './replication';

/** Replication status returned by `GET /replication/status`. */
export interface ReplicationStatus {
  /** Node role: `"Master"`, `"Replica"`, or `"Standalone"`. */
  role: string;
  /** Whether replication is enabled on this node. */
  enabled: boolean;
  /** Replication stats (master or replica depending on role). */
  stats?: ReplicationStats;
  /** Connected replicas (master only). */
  replicas?: ReplicaInfo[];
}

/** Request body for `configureReplication` (`POST /replication/configure`). */
export interface ReplicationConfig {
  /** Target role: `"master"`, `"replica"`, or `"standalone"`. */
  role: string;
  /** Bind address for master nodes. */
  bind_address?: string;
  /** Master address for replica nodes. */
  master_address?: string;
  /** Heartbeat interval in milliseconds. */
  heartbeat_interval?: number;
  /** Replication log size. */
  log_size?: number;
}
