/**
 * Hook for server identity polling.
 *
 * Reads `GET /status` which is served by the Rust handler
 * `crates/vectorizer-server/src/server/rest_handlers/meta.rs::get_status`.
 * The endpoint returns:
 *   { online: bool, version: string, uptime_seconds: number, collections_count: number }
 *
 * The hook is used by OverviewPage to display the live server version and
 * bind address in the System Health card, replacing the hardcoded literals
 * introduced in the initial dashboard implementation.
 */
import { useEffect, useRef, useState } from 'react';
import { useApiClient } from './useApiClient';

export interface ServerStatus {
  online: boolean;
  version: string;
  uptimeSeconds: number;
  collectionsCount: number;
}

const ZERO: ServerStatus = {
  online: false,
  version: '',
  uptimeSeconds: 0,
  collectionsCount: 0,
};

interface Options {
  /** Polling interval in milliseconds. 0 disables polling. */
  intervalMs?: number;
}

export interface UseStatusResult {
  status: ServerStatus;
  loading: boolean;
  error: string | null;
}

export function useStatus({ intervalMs = 30000 }: Options = {}): UseStatusResult {
  const api = useApiClient();
  const [status, setStatus] = useState<ServerStatus>(ZERO);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ref = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fetchStatus = async () => {
      setLoading(true);
      try {
        const resp = await api.get<unknown>('/status');
        // ApiClient already unwraps `{ data, error }` envelopes, but accept
        // both shapes to stay forwards-compatible.
        const payload = (resp as { data?: unknown }).data ?? resp;
        if (cancelled) return;
        setStatus(normalize(payload));
        setError(null);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to fetch status');
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    fetchStatus();
    if (intervalMs > 0) {
      ref.current = setInterval(fetchStatus, intervalMs);
    }
    return () => {
      cancelled = true;
      if (ref.current) {
        clearInterval(ref.current);
        ref.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [intervalMs]);

  return { status, loading, error };
}

function normalize(payload: unknown): ServerStatus {
  if (!payload || typeof payload !== 'object') return ZERO;
  const p = payload as Record<string, unknown>;
  return {
    online: p.online === true,
    version: typeof p.version === 'string' ? p.version : '',
    uptimeSeconds: num(p.uptime_seconds ?? p.uptimeSeconds),
    collectionsCount: num(p.collections_count ?? p.collectionsCount),
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
