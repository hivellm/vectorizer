/**
 * Hook for polling `GET /metrics/runtime` (admin-gated).
 *
 * The endpoint (phase25 §§1+2+4) returns one current sample per tick:
 *   {
 *     cpu_percent, memory_rss_bytes, memory_total_bytes, memory_percent,
 *     active_connections, uptime_seconds, qps_window_60s,
 *     error_rate_5xx_60s, throughput_by_route: [{route, qps, p50_ms, p99_ms}]
 *   }
 *
 * In addition to the live snapshot, this hook maintains a client-side ring
 * buffer of up to 60 `qpsWindow60s` samples (one per tick) to feed the
 * MonitoringPage throughput Sparkline.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { useApiClient } from './useApiClient';

export interface ThroughputRoute {
  route: string;
  qps: number;
  p50Ms: number;
  p99Ms: number;
}

export interface RuntimeMetrics {
  cpuPercent: number;
  memoryPercent: number;
  memoryRssBytes: number;
  memoryTotalBytes: number;
  activeConnections: number;
  uptimeSeconds: number;
  qpsWindow60s: number;
  errorRate5xx60s: number;
  throughputByRoute: ThroughputRoute[];
}

const ZERO: RuntimeMetrics = {
  cpuPercent: 0,
  memoryPercent: 0,
  memoryRssBytes: 0,
  memoryTotalBytes: 0,
  activeConnections: 0,
  uptimeSeconds: 0,
  qpsWindow60s: 0,
  errorRate5xx60s: 0,
  throughputByRoute: [],
};

const QPS_BUFFER_SIZE = 60;

export interface UseRuntimeMetricsResult {
  metrics: RuntimeMetrics;
  /** Client-side ring buffer of qpsWindow60s — one entry per tick, capped at 60. */
  qpsHistory: number[];
  loading: boolean;
  error: Error | null;
}

interface Options {
  /** Polling interval in milliseconds. 0 disables polling. Defaults to 2000. */
  intervalMs?: number;
}

export function useRuntimeMetrics({ intervalMs = 2000 }: Options = {}): UseRuntimeMetricsResult {
  const api = useApiClient();
  const [metrics, setMetrics] = useState<RuntimeMetrics>(ZERO);
  const [qpsHistory, setQpsHistory] = useState<number[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const fetchMetrics = useCallback(async (cancelled: { current: boolean }) => {
    setLoading(true);
    try {
      const resp = await api.get<unknown>('/metrics/runtime');
      const payload = (resp as { data?: unknown }).data ?? resp;
      if (cancelled.current) return;
      const normalized = normalize(payload);
      setMetrics(normalized);
      setQpsHistory((prev) => {
        const next = [...prev, normalized.qpsWindow60s];
        return next.length > QPS_BUFFER_SIZE ? next.slice(next.length - QPS_BUFFER_SIZE) : next;
      });
      setError(null);
    } catch (err) {
      if (cancelled.current) return;
      setError(err instanceof Error ? err : new Error('Failed to fetch runtime metrics'));
    } finally {
      if (!cancelled.current) setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    const cancelled = { current: false };
    fetchMetrics(cancelled);
    if (intervalMs > 0) {
      timerRef.current = setInterval(() => fetchMetrics(cancelled), intervalMs);
    }
    return () => {
      cancelled.current = true;
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [intervalMs]);

  return { metrics, qpsHistory, loading, error };
}

function normalize(payload: unknown): RuntimeMetrics {
  if (!payload || typeof payload !== 'object') return ZERO;
  const p = payload as Record<string, unknown>;

  const routes: ThroughputRoute[] = [];
  if (Array.isArray(p.throughput_by_route)) {
    for (const r of p.throughput_by_route) {
      if (r && typeof r === 'object') {
        const row = r as Record<string, unknown>;
        routes.push({
          route: typeof row.route === 'string' ? row.route : '',
          qps: num(row.qps),
          p50Ms: num(row.p50_ms ?? row.p50Ms),
          p99Ms: num(row.p99_ms ?? row.p99Ms),
        });
      }
    }
  }

  return {
    cpuPercent: num(p.cpu_percent ?? p.cpuPercent),
    memoryPercent: num(p.memory_percent ?? p.memoryPercent),
    memoryRssBytes: num(p.memory_rss_bytes ?? p.memoryRssBytes),
    memoryTotalBytes: num(p.memory_total_bytes ?? p.memoryTotalBytes),
    activeConnections: num(p.active_connections ?? p.activeConnections),
    uptimeSeconds: num(p.uptime_seconds ?? p.uptimeSeconds),
    qpsWindow60s: num(p.qps_window_60s ?? p.qpsWindow60s),
    errorRate5xx60s: num(p.error_rate_5xx_60s ?? p.errorRate5xx60s),
    throughputByRoute: routes,
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
