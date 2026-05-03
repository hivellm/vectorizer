/**
 * Hook for the recent-events feed.
 *
 * Polls a JSON `/events?limit=N` endpoint with a tolerant payload shape:
 *
 *   - bare array  -> [{ ts, level, msg }, ...]
 *   - envelope    -> { events: [{ ts, level, msg }, ...] }
 *
 * The Rust server does NOT currently expose `/events` (REST) or any SSE
 * route — verified against `crates/vectorizer-server/src/server/core/routing.rs`.
 * Until that endpoint lands, the first poll will 404; we set
 * `available = false`, stop polling, and the consumer renders a placeholder
 * instead of hammering the server with retries.
 */
import { useEffect, useRef, useState } from 'react';
import { useApiClient } from './useApiClient';

export interface ServerEvent {
  ts: string;
  level: 'info' | 'ok' | 'warn' | 'error' | string;
  msg: string;
}

interface Options {
  /** Polling interval in milliseconds. 0 disables polling. */
  intervalMs?: number;
  /** Path of the events endpoint. Defaults to `/events`. */
  path?: string;
  /** Maximum number of events to request. Defaults to 12. */
  limit?: number;
}

export interface UseEventsResult {
  events: ServerEvent[];
  loading: boolean;
  error: string | null;
  /** False when the endpoint 404s — consumer should show a placeholder. */
  available: boolean;
}

export function useEvents({
  intervalMs = 5000,
  path = '/events',
  limit = 12,
}: Options = {}): UseEventsResult {
  const api = useApiClient();
  const [events, setEvents] = useState<ServerEvent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [available, setAvailable] = useState(true);
  const ref = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fetchEvents = async () => {
      setLoading(true);
      try {
        const resp = await api.get<unknown>(`${path}?limit=${limit}`);
        const payload = (resp as { data?: unknown }).data ?? resp;
        const arr = Array.isArray(payload)
          ? (payload as ServerEvent[])
          : ((payload as { events?: ServerEvent[] })?.events ?? []);
        if (cancelled) return;
        setEvents(arr);
        setError(null);
        setAvailable(true);
      } catch (err) {
        if (cancelled) return;
        const msg = err instanceof Error ? err.message : 'Failed to fetch events';
        // 404 = endpoint not implemented on this backend; mark unavailable
        // and stop polling so we don't spam.
        if (/404|not.found/i.test(msg)) {
          setAvailable(false);
          if (ref.current) {
            clearInterval(ref.current);
            ref.current = null;
          }
        } else {
          setError(msg);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    fetchEvents();
    if (intervalMs > 0) {
      ref.current = setInterval(fetchEvents, intervalMs);
    }
    return () => {
      cancelled = true;
      if (ref.current) {
        clearInterval(ref.current);
        ref.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [intervalMs, path, limit]);

  return { events, loading, error, available };
}
