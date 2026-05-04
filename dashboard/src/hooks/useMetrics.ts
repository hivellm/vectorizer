/**
 * Hook for dashboard metrics polling.
 *
 * Wires the console KPI strip (qps, p99, cpu, mem, conns, cache hit rate,
 * total vectors) to the live REST surface. The Rust server exposes a small
 * set of JSON endpoints today:
 *
 *   GET /stats   → { collections, total_vectors, uptime_seconds, version }
 *   GET /health  → { status, timestamp, version, cache: { hits, misses,
 *                    evictions, hit_rate, size, capacity }, hub?, backup? }
 *
 * Neither one currently surfaces qps, p99, cpu, mem or active connections —
 * those live in the Prometheus text export at /prometheus/metrics. Until a
 * dedicated dashboard-metrics JSON endpoint lands (tracked in Task 4.3),
 * this hook reads /stats by default so total_vectors lights up, and falls
 * back to zero for the synthetic-only KPIs. The `normalize()` mapping is
 * permissive so the same hook will pick up new fields without changes once
 * the backend grows them.
 */

import { useEffect, useRef, useState } from 'react';
import { useApiClient } from './useApiClient';

export interface DashboardMetrics {
  qps: number;
  p99Ms: number;
  cpuPercent: number;
  memPercent: number;
  connections: number;
  cacheHitRate: number;
  totalVectors: number;
}

const ZERO: DashboardMetrics = {
  qps: 0,
  p99Ms: 0,
  cpuPercent: 0,
  memPercent: 0,
  connections: 0,
  cacheHitRate: 0,
  totalVectors: 0,
};

interface Options {
  /** Polling interval in milliseconds. 0 disables polling. */
  intervalMs?: number;
  /** Path of the metrics endpoint. Defaults to `/stats`. */
  path?: string;
}

export interface UseMetricsResult {
  metrics: DashboardMetrics;
  loading: boolean;
  error: string | null;
}

export function useMetrics({ intervalMs = 5000, path = '/stats' }: Options = {}): UseMetricsResult {
  const api = useApiClient();
  const [metrics, setMetrics] = useState<DashboardMetrics>(ZERO);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ref = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fetchMetrics = async () => {
      setLoading(true);
      try {
        const resp = await api.get<unknown>(path);
        // ApiClient already unwraps `{ data, error }` envelopes for us, but
        // accept both shapes to stay forwards-compatible.
        const payload = (resp as { data?: unknown }).data ?? resp;
        if (cancelled) return;
        setMetrics(normalize(payload));
        setError(null);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to fetch metrics');
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    fetchMetrics();
    if (intervalMs > 0) {
      ref.current = setInterval(fetchMetrics, intervalMs);
    }
    return () => {
      cancelled = true;
      if (ref.current) {
        clearInterval(ref.current);
        ref.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [intervalMs, path]);

  return { metrics, loading, error };
}

function normalize(payload: unknown): DashboardMetrics {
  if (!payload || typeof payload !== 'object') return ZERO;
  const p = payload as Record<string, unknown>;
  // Cache stats may sit nested under `cache` (the /health shape).
  const cache = (p.cache && typeof p.cache === 'object') ? (p.cache as Record<string, unknown>) : undefined;
  return {
    qps: num(p.qps ?? p.queries_per_second ?? p.queries_per_sec ?? p.queriesPerSec),
    p99Ms: num(p.p99 ?? p.p99_ms ?? p.search_p99_ms ?? p.searchP99Ms),
    cpuPercent: num(p.cpu_percent ?? p.cpu ?? p.cpuPercent ?? p.cpu_usage),
    memPercent: num(p.memory_percent ?? p.mem ?? p.memPercent ?? p.memoryPercent ?? p.memory_usage_percent),
    connections: num(p.connections ?? p.active_connections ?? p.activeConnections),
    cacheHitRate: num(p.cache_hit_rate ?? p.cacheHitRate ?? cache?.hit_rate ?? cache?.hitRate),
    totalVectors: num(p.total_vectors ?? p.totalVectors ?? p.vectors_total),
  };
}

function num(v: unknown): number {
  if (typeof v === 'number' && Number.isFinite(v)) return v;
  if (typeof v === 'string') {
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
  }
  return 0;
}
