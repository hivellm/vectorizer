/**
 * Hook for the live `runtime` topic on the dashboard WebSocket
 * (phase29). Replaces the prior 1-2 s polling loop on
 * `GET /metrics/runtime`.
 *
 * The server tick still produces the same JSON shape (phase25 §§1+2+4),
 * so the snake_case → camelCase mapping below stays unchanged. The
 * REST endpoint remains live for SDK callers and is still used as a
 * one-shot fallback before the first WS frame arrives — that initial
 * REST round-trip avoids the visible "loading…" flash on first paint
 * while the socket is still negotiating.
 *
 * In addition to the live snapshot, this hook keeps a client-side ring
 * buffer of up to 60 `qpsWindow60s` samples (one per WS push) to feed
 * the MonitoringPage throughput Sparkline.
 */

import { useEffect, useRef, useState } from 'react';
import { useWsTopic } from '../providers/WsDashboardProvider';
import { useApiClient } from './useApiClient';

export interface ThroughputRoute {
  route: string;
  qps: number;
  p50Ms: number;
  p99Ms: number;
}

export interface WalStats {
  currentSeq: number;
  sizeBytes: number;
  lastCheckpointAt: number;
  lastCheckpointSeq: number;
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
  wal: WalStats;
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
  wal: { currentSeq: 0, sizeBytes: 0, lastCheckpointAt: 0, lastCheckpointSeq: 0 },
};

const QPS_BUFFER_SIZE = 60;

export interface UseRuntimeMetricsResult {
  metrics: RuntimeMetrics;
  /** Client-side ring buffer of qpsWindow60s — one entry per WS push,
   *  capped at 60. */
  qpsHistory: number[];
  loading: boolean;
  error: Error | null;
}

export function useRuntimeMetrics(): UseRuntimeMetricsResult {
  const api = useApiClient();
  const wsPayload = useWsTopic<unknown>('runtime');
  const [restMetrics, setRestMetrics] = useState<RuntimeMetrics | null>(null);
  const [restError, setRestError] = useState<Error | null>(null);
  const [qpsHistory, setQpsHistory] = useState<number[]>([]);
  // Track which qpsWindow60s value was last appended so we don't push
  // duplicate ring-buffer entries on React re-renders that don't carry
  // a fresh WS frame.
  const lastAppendedRef = useRef<number | null>(null);

  // One-shot REST fetch on mount so the UI has a value before the
  // first WS frame lands. After that, all updates flow via the WS
  // store and this REST call is never repeated.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const resp = await api.get<unknown>('/metrics/runtime');
        const payload = (resp as { data?: unknown }).data ?? resp;
        if (!cancelled) {
          const norm = normalize(payload);
          setRestMetrics(norm);
          setRestError(null);
          appendQps(norm.qpsWindow60s);
        }
      } catch (err) {
        if (!cancelled) {
          setRestError(err instanceof Error ? err : new Error('Failed to fetch runtime metrics'));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const wsMetrics = wsPayload ? normalize(wsPayload) : null;
  const metrics = wsMetrics ?? restMetrics ?? ZERO;

  function appendQps(q: number) {
    if (lastAppendedRef.current === q) return;
    lastAppendedRef.current = q;
    setQpsHistory((prev) => {
      const next = [...prev, q];
      return next.length > QPS_BUFFER_SIZE ? next.slice(next.length - QPS_BUFFER_SIZE) : next;
    });
  }

  // Append qps to the ring buffer when the WS payload changes.
  useEffect(() => {
    if (!wsMetrics) return;
    appendQps(wsMetrics.qpsWindow60s);
  }, [wsMetrics]);

  return {
    metrics,
    qpsHistory,
    loading: wsMetrics === null && restMetrics === null && restError === null,
    error: restError,
  };
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

  const walRaw = (p.wal && typeof p.wal === 'object' ? p.wal : {}) as Record<string, unknown>;

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
    wal: {
      currentSeq: num(walRaw.current_seq ?? walRaw.currentSeq),
      sizeBytes: num(walRaw.size_bytes ?? walRaw.sizeBytes),
      lastCheckpointAt: num(walRaw.last_checkpoint_at ?? walRaw.lastCheckpointAt),
      lastCheckpointSeq: num(walRaw.last_checkpoint_seq ?? walRaw.lastCheckpointSeq),
    },
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
