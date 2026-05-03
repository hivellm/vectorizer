/**
 * Replication surface.
 *
 * Covers the `/replication/*` REST endpoints (status, configuration,
 * statistics, replica listing) and the `/cluster/*` admin endpoints
 * added in phase15 (failover, resync, add-peer, rebalance).
 */

import { BaseClient } from './_base';
import type { ReplicaInfo, ReplicationStats } from '../models/replication';
import type { ReplicationConfig, ReplicationStatus } from '../models/replication-sdk';
import type {
  AddPeerRequest,
  FailoverReport,
  PeerInfo,
  RebalanceJob,
  ResyncJob,
} from '../models/cluster-auth-admin';

export class ReplicationClient extends BaseClient {
  /**
   * Get the current replication status and role of this node.
   * Calls `GET /replication/status`.
   */
  public async getReplicationStatus(): Promise<ReplicationStatus> {
    this.logger.debug('Getting replication status');
    return this.transport.get<ReplicationStatus>('/replication/status');
  }

  /**
   * Configure this node's replication role and parameters.
   * Calls `POST /replication/configure`.
   */
  public async configureReplication(config: ReplicationConfig): Promise<void> {
    this.logger.debug('Configuring replication', { role: config.role });
    await this.transport.post('/replication/configure', config);
  }

  /**
   * Get raw replication statistics for the active replication node.
   * Calls `GET /replication/stats`.
   */
  public async getReplicationStats(): Promise<ReplicationStats> {
    this.logger.debug('Getting replication stats');
    return this.transport.get<ReplicationStats>('/replication/stats');
  }

  /**
   * List the replica nodes connected to this master.
   * Calls `GET /replication/replicas`. Only available on master nodes.
   */
  public async listReplicas(): Promise<ReplicaInfo[]> {
    this.logger.debug('Listing replicas');
    const response = await this.transport.get<{ replicas: ReplicaInfo[] }>(
      '/replication/replicas',
    );
    return response.replicas ?? [];
  }

  // ── phase15 cluster admin ───────────────────────────────────────────────────

  /**
   * Trigger a failover — promote a replica to primary.
   * Calls `POST /cluster/failover` with `{replica_id}`.
   * Returns 409 from the server when the replica's WAL lag exceeds the threshold.
   */
  public async clusterFailover(replicaId: string): Promise<FailoverReport> {
    this.logger.debug('Triggering cluster failover', { replicaId });
    return this.transport.post<FailoverReport>('/cluster/failover', { replica_id: replicaId });
  }

  /**
   * Force a full resync on a replica.
   * Calls `POST /cluster/replicas/{id}/resync` with an empty body.
   */
  public async clusterResyncReplica(replicaId: string): Promise<ResyncJob> {
    this.logger.debug('Forcing replica resync', { replicaId });
    return this.transport.post<ResyncJob>(`/cluster/replicas/${replicaId}/resync`, {});
  }

  /**
   * Add a peer to the cluster.
   * Calls `POST /cluster/peers` with `{address, role}`.
   */
  public async clusterAddPeer(request: AddPeerRequest): Promise<PeerInfo> {
    this.logger.debug('Adding cluster peer', { address: request.address });
    return this.transport.post<PeerInfo>('/cluster/peers', request);
  }

  /**
   * Trigger a shard rebalance across all active cluster nodes.
   * Calls `POST /cluster/rebalance` with an empty body.
   */
  public async clusterRebalance(): Promise<RebalanceJob> {
    this.logger.debug('Triggering cluster rebalance');
    return this.transport.post<RebalanceJob>('/cluster/rebalance', {});
  }

  /**
   * Return progress of the active (or last completed) rebalance job.
   * Calls `GET /cluster/rebalance/status`.
   * Returns `null` when no rebalance has been triggered on this node.
   */
  public async clusterRebalanceStatus(): Promise<RebalanceJob | null> {
    this.logger.debug('Getting cluster rebalance status');
    const response = await this.transport.get<RebalanceJob | { status: string }>(
      '/cluster/rebalance/status',
    );
    if ('status' in response && (response as { status: string }).status === 'idle') {
      return null;
    }
    return response as RebalanceJob;
  }
}
