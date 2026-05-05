/**
 * Hook for system stats polling (cache + optional WAL fields).
 *
 * Reads `/health` by default, since that's where the Rust server exposes
 * the QueryCache stats today (size/capacity/hits/misses/evictions/hit_rate).
 * The hook also looks for WAL fields (`wal.sequence`, `wal.size_bytes`,
 * `wal.last_checkpoint`) — they are not currently emitted by `/health`,
 * but the normalizer is permissive so the hook will pick them up
 * automatically once the backend grows them.
 *
 * Backend reference: `crates/vectorizer-server/src/server/rest_handlers/meta.rs::health_check`.
 */
import { useEffect, useRef, useState } from 'react';
import { useApiClient } from './useApiClient';

export interface CacheStats {
  size: number;
  capacity: number;
  hits: number;
  misses: number;
  evictions: number;
  hitRate: number; // 0..1
}

export interface SystemStats {
  status: string;
  /** Server version reported by /health (e.g. "3.2.1"). Undefined until the
   *  first response lands or when the backend doesn't emit it. */
  version?: string;
  cache: CacheStats;
  walSequence?: number;
  walSizeBytes?: number;
  walLastCheckpointAt?: string;
}

const ZERO_CACHE: CacheStats = {
  size: 0,
  capacity: 0,
  hits: 0,
  misses: 0,
  evictions: 0,
  hitRate: 0,
};
const ZERO: SystemStats = { status: 'unknown', cache: ZERO_CACHE };

interface Options {
  /** Polling interval in milliseconds. 0 disables polling. */
  intervalMs?: number;
  /** Path of the stats endpoint. Defaults to `/health`. */
  path?: string;
}

export interface UseStatsResult {
  stats: SystemStats;
  loading: boolean;
  error: string | null;
}

export function useStats({ intervalMs = 5000, path = '/health' }: Options = {}): UseStatsResult {
  const api = useApiClient();
  const [stats, setStats] = useState<SystemStats>(ZERO);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ref = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fetchStats = async () => {
      setLoading(true);
      try {
        const resp = await api.get<unknown>(path);
        // ApiClient already unwraps `{ data, error }` envelopes, but accept
        // both shapes to stay forwards-compatible.
        const payload = (resp as { data?: unknown }).data ?? resp;
        if (cancelled) return;
        setStats(normalize(payload));
        setError(null);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to fetch stats');
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    fetchStats();
    if (intervalMs > 0) {
      ref.current = setInterval(fetchStats, intervalMs);
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

  return { stats, loading, error };
}

function normalize(payload: unknown): SystemStats {
  if (!payload || typeof payload !== 'object') return ZERO;
  const p = payload as Record<string, unknown>;
  const cacheSrc =
    p.cache && typeof p.cache === 'object' ? (p.cache as Record<string, unknown>) : {};
  const wal = p.wal && typeof p.wal === 'object' ? (p.wal as Record<string, unknown>) : {};
  return {
    status: String(p.status ?? 'unknown'),
    version: typeof p.version === 'string' && p.version.length > 0 ? p.version : undefined,
    cache: {
      size: num(cacheSrc.size),
      capacity: num(cacheSrc.capacity),
      hits: num(cacheSrc.hits),
      misses: num(cacheSrc.misses),
      evictions: num(cacheSrc.evictions),
      hitRate: num(cacheSrc.hit_rate ?? cacheSrc.hitRate),
    },
    walSequence: numOrUndef(wal.sequence ?? wal.seq ?? p.wal_sequence ?? p.walSequence),
    walSizeBytes: numOrUndef(wal.size_bytes ?? wal.size ?? p.wal_size_bytes ?? p.walSizeBytes),
    walLastCheckpointAt:
      typeof wal.last_checkpoint === 'string' ? (wal.last_checkpoint as string) : undefined,
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

function numOrUndef(v: unknown): number | undefined {
  const n = num(v);
  return n || undefined;
}
