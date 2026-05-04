/**
 * Hook for server identity / liveness data — sourced from the dashboard
 * WebSocket's `status` topic (phase29 §2.1).
 *
 * Server publisher: `crates/vectorizer-server/src/server/core/bootstrap.rs`
 * spawns a 5 s tick that broadcasts the same JSON shape `GET /status`
 * returns. The REST endpoint stays live for SDK callers and is used as
 * a one-shot fallback before the first WS frame arrives so the System
 * Health card has something to render while the socket negotiates.
 */
import { useEffect, useState } from 'react';
import { useWsTopic } from '../providers/WsDashboardProvider';
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

export interface UseStatusResult {
  status: ServerStatus;
  loading: boolean;
  error: string | null;
}

export function useStatus(): UseStatusResult {
  const api = useApiClient();
  const wsPayload = useWsTopic<unknown>('status');
  const [restStatus, setRestStatus] = useState<ServerStatus | null>(null);
  const [restError, setRestError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const resp = await api.get<unknown>('/status');
        const payload = (resp as { data?: unknown }).data ?? resp;
        if (!cancelled) {
          setRestStatus(normalize(payload));
          setRestError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setRestError(err instanceof Error ? err.message : 'Failed to fetch status');
        }
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const wsStatus = wsPayload ? normalize(wsPayload) : null;
  const status = wsStatus ?? restStatus ?? ZERO;

  return {
    status,
    loading: wsStatus === null && restStatus === null && restError === null,
    error: restError,
  };
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
