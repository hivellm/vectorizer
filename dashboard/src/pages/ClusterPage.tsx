/**
 * Cluster page - HA Cluster status and node management
 */

import { useState, useEffect, useRef } from 'react';
import { RefreshCw01, AlertCircle } from '@untitledui/icons';
import Card from '@/components/ui/Card';
import StatCard from '@/components/ui/StatCard';

interface ClusterNode {
  id: string;
  address: string;
  role: 'leader' | 'follower' | 'learner' | string;
  status: 'healthy' | 'unhealthy' | string;
  vector_count?: number;
  replication_lag?: number | null;
}

interface LeaderInfo {
  leader_url?: string;
  term?: number;
  epoch?: number;
}

interface ClusterRole {
  role?: string;
  is_leader?: boolean;
}

interface ClusterData {
  nodes: ClusterNode[];
  leader: LeaderInfo | null;
  role: ClusterRole | null;
  isHA: boolean;
  replicaCount: number;
  lastSyncTime: string | null;
  error: string | null;
  loading: boolean;
}

const REFRESH_INTERVAL_MS = 5000;

function getRoleBadgeClasses(role: string): string {
  switch (role.toLowerCase()) {
    case 'leader':
      return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400';
    case 'follower':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400';
    case 'learner':
      return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
    default:
      return 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-400';
  }
}

function getStatusDotClasses(status: string): string {
  switch (status.toLowerCase()) {
    case 'healthy':
      return 'bg-green-500';
    case 'unhealthy':
      return 'bg-red-500';
    default:
      return 'bg-neutral-400';
  }
}

function formatReplicationLag(lag: number | null | undefined): string {
  if (lag == null) return '—';
  if (lag === 0) return 'In sync';
  return `${lag} ops behind`;
}

async function fetchJSON<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }
  return response.json() as Promise<T>;
}

function ClusterPage() {
  const [data, setData] = useState<ClusterData>({
    nodes: [],
    leader: null,
    role: null,
    isHA: false,
    replicaCount: 0,
    lastSyncTime: null,
    error: null,
    loading: true,
  });

  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchClusterData = async () => {
    try {
      const [nodesResult, leaderResult, roleResult] = await Promise.allSettled([
        fetchJSON<{ nodes?: ClusterNode[] } | ClusterNode[]>('/api/v1/cluster/nodes'),
        fetchJSON<LeaderInfo>('/api/v1/cluster/leader'),
        fetchJSON<ClusterRole>('/api/v1/cluster/role'),
      ]);

      const rawNodes =
        nodesResult.status === 'fulfilled'
          ? Array.isArray(nodesResult.value)
            ? nodesResult.value
            : (nodesResult.value as { nodes?: ClusterNode[] }).nodes ?? []
          : [];

      const leader = leaderResult.status === 'fulfilled' ? leaderResult.value : null;
      const role = roleResult.status === 'fulfilled' ? roleResult.value : null;

      const healthyNodes = rawNodes.filter((n) => n.status?.toLowerCase() === 'healthy');
      const isHA = rawNodes.length > 1;

      setData({
        nodes: rawNodes,
        leader,
        role,
        isHA,
        replicaCount: healthyNodes.filter((n) => n.role?.toLowerCase() !== 'leader').length,
        lastSyncTime: new Date().toLocaleTimeString(),
        error: null,
        loading: false,
      });
    } catch (err) {
      setData((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : 'Failed to load cluster data',
        loading: false,
      }));
    }
  };

  useEffect(() => {
    fetchClusterData();

    intervalRef.current = setInterval(() => {
      fetchClusterData();
    }, REFRESH_INTERVAL_MS);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  const { nodes, leader, isHA, replicaCount, lastSyncTime, error, loading } = data;

  const healthyCount = nodes.filter((n) => n.status?.toLowerCase() === 'healthy').length;
  const leaderNode = nodes.find((n) => n.role?.toLowerCase() === 'leader');

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">
            Cluster Status
          </h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            High-availability cluster health and node information
          </p>
        </div>
        <button
          onClick={fetchClusterData}
          disabled={loading}
          className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium text-neutral-700 dark:text-neutral-300 border border-neutral-300 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors disabled:opacity-50"
        >
          <RefreshCw01 className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Cluster Status Banner */}
      <div
        className={`flex items-center gap-4 p-4 rounded-lg border ${
          isHA
            ? 'bg-green-50 dark:bg-green-900/10 border-green-200 dark:border-green-800/50'
            : 'bg-neutral-50 dark:bg-neutral-800/50 border-neutral-200 dark:border-neutral-800/50'
        }`}
      >
        <span
          className={`inline-flex items-center gap-2 px-3 py-1 rounded-full text-sm font-semibold ${
            isHA
              ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
              : 'bg-neutral-200 text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300'
          }`}
        >
          <span
            className={`w-2 h-2 rounded-full ${isHA ? 'bg-green-500' : 'bg-neutral-400'}`}
          />
          {isHA ? 'HA Active' : 'Standalone'}
        </span>
        <p className="text-sm text-neutral-600 dark:text-neutral-400">
          {isHA
            ? `High-availability cluster with ${nodes.length} node${nodes.length !== 1 ? 's' : ''}`
            : 'Running as a single-node instance'}
        </p>
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Total Nodes"
          value={nodes.length}
          subtitle="Registered cluster nodes"
        />
        <StatCard
          title="Healthy Nodes"
          value={healthyCount}
          subtitle={`${nodes.length - healthyCount} unhealthy`}
        />
        <StatCard
          title="Connected Replicas"
          value={replicaCount}
          subtitle="Healthy followers / learners"
        />
        <StatCard
          title="Last Sync"
          value={lastSyncTime ?? '—'}
          subtitle="Last successful data fetch"
        />
      </div>

      {/* Nodes Table */}
      <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50">
        <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">Nodes</h2>
        {loading && nodes.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <RefreshCw01 className="w-8 h-8 text-neutral-400 dark:text-neutral-500 animate-spin" />
          </div>
        ) : nodes.length === 0 ? (
          <div className="text-center py-10 text-neutral-500 dark:text-neutral-400">
            <AlertCircle className="w-10 h-10 mx-auto mb-3 text-neutral-400 dark:text-neutral-500" />
            <p className="text-sm">No cluster nodes found</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-neutral-200 dark:divide-neutral-700">
              <thead className="bg-neutral-50 dark:bg-neutral-800/50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                    Node
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                    Role
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                    Address
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                    Vectors
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                    Replication Lag
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-neutral-900 divide-y divide-neutral-200 dark:divide-neutral-800/50">
                {nodes.map((node) => (
                  <tr key={node.id} className="hover:bg-neutral-50 dark:hover:bg-neutral-800/50">
                    <td className="px-4 py-3 whitespace-nowrap text-sm font-mono font-medium text-neutral-900 dark:text-white">
                      {node.id}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <span
                        className={`px-2 py-1 text-xs font-medium rounded capitalize ${getRoleBadgeClasses(node.role ?? 'unknown')}`}
                      >
                        {node.role ?? 'unknown'}
                      </span>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm font-mono text-neutral-600 dark:text-neutral-400">
                      {node.address}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <span className="flex items-center gap-2 text-sm text-neutral-700 dark:text-neutral-300">
                        <span
                          className={`w-2 h-2 rounded-full flex-shrink-0 ${getStatusDotClasses(node.status ?? 'unknown')}`}
                        />
                        <span className="capitalize">{node.status ?? 'unknown'}</span>
                      </span>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-right text-neutral-600 dark:text-neutral-400">
                      {node.vector_count != null ? node.vector_count.toLocaleString() : '—'}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-neutral-600 dark:text-neutral-400">
                      {node.role?.toLowerCase() === 'leader'
                        ? '—'
                        : formatReplicationLag(node.replication_lag)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {/* Bottom Cards Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Leader Info Card */}
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50">
          <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">
            Leader Information
          </h2>
          {leader || leaderNode ? (
            <dl className="space-y-3 text-sm">
              <div className="flex justify-between gap-4">
                <dt className="text-neutral-500 dark:text-neutral-400">Leader URL</dt>
                <dd className="font-mono text-neutral-900 dark:text-white text-right break-all">
                  {leader?.leader_url ?? leaderNode?.address ?? '—'}
                </dd>
              </div>
              {leader?.term != null && (
                <div className="flex justify-between gap-4">
                  <dt className="text-neutral-500 dark:text-neutral-400">Term</dt>
                  <dd className="font-mono text-neutral-900 dark:text-white">{leader.term}</dd>
                </div>
              )}
              {leader?.epoch != null && (
                <div className="flex justify-between gap-4">
                  <dt className="text-neutral-500 dark:text-neutral-400">Epoch</dt>
                  <dd className="font-mono text-neutral-900 dark:text-white">{leader.epoch}</dd>
                </div>
              )}
              <div className="flex justify-between gap-4">
                <dt className="text-neutral-500 dark:text-neutral-400">Node ID</dt>
                <dd className="font-mono text-neutral-900 dark:text-white">
                  {leaderNode?.id ?? '—'}
                </dd>
              </div>
            </dl>
          ) : (
            <p className="text-sm text-neutral-500 dark:text-neutral-400">
              No leader information available
            </p>
          )}
        </Card>

        {/* Replication Status Card */}
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50">
          <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">
            Replication Status
          </h2>
          <dl className="space-y-3 text-sm">
            <div className="flex justify-between gap-4">
              <dt className="text-neutral-500 dark:text-neutral-400">Connected Replicas</dt>
              <dd className="font-semibold text-neutral-900 dark:text-white">{replicaCount}</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-neutral-500 dark:text-neutral-400">Last Sync</dt>
              <dd className="text-neutral-900 dark:text-white">{lastSyncTime ?? '—'}</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-neutral-500 dark:text-neutral-400">Cluster Mode</dt>
              <dd>
                <span
                  className={`px-2 py-1 text-xs font-medium rounded ${
                    isHA
                      ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                      : 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-400'
                  }`}
                >
                  {isHA ? 'HA Cluster' : 'Standalone'}
                </span>
              </dd>
            </div>
            {nodes.length > 0 && (
              <div className="flex justify-between gap-4">
                <dt className="text-neutral-500 dark:text-neutral-400">Unhealthy Nodes</dt>
                <dd
                  className={`font-semibold ${
                    nodes.length - healthyCount > 0
                      ? 'text-red-600 dark:text-red-400'
                      : 'text-green-600 dark:text-green-400'
                  }`}
                >
                  {nodes.length - healthyCount}
                </dd>
              </div>
            )}
          </dl>
        </Card>
      </div>
    </div>
  );
}

export default ClusterPage;
