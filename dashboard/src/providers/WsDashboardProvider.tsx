/**
 * Phase29 — single multiplexed WebSocket per app instance for the
 * dashboard's live data. Replaces the eight polling loops the
 * previous redesign fired off (1–30 s intervals on /metrics/runtime,
 * /stats, /status, /collections, /logs, etc.).
 *
 * Wire protocol (see crates/vectorizer-server/src/server/ws/dashboard.rs):
 *
 *   c→s  {"op":"subscribe",  "topics":["runtime", ...]}
 *   c→s  {"op":"unsubscribe","topics":["runtime", ...]}
 *   c→s  {"op":"ping"}
 *
 *   s→c  {"topic":"runtime", "data": {...}}
 *   s→c  {"op":"pong"}
 *   s→c  {"op":"error","code":"stream_lag" | "bad_frame"}
 *
 * The provider uses cookie auth (browser sends `vectorizer_session`
 * automatically on the upgrade GET) and reconnects with exponential
 * backoff (250 ms → 5 s) on `close` / `error`. Subscribers register
 * via `useWsTopic<T>(topic)` and receive the latest typed snapshot
 * via `useSyncExternalStore` so React only re-renders on actual
 * change.
 */

import {
  ReactNode,
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useSyncExternalStore,
} from 'react';

export type WsTopic = 'runtime' | 'status' | 'collections' | 'logs';

interface TopicStore<T> {
  /** Latest snapshot received for this topic. `null` until the first frame. */
  current: T | null;
  /** Listeners registered via useSyncExternalStore. */
  listeners: Set<() => void>;
  /** Reference count of mounted subscribers — drives `subscribe` /
   *  `unsubscribe` op frames so the server can stop publishing topics
   *  no tab is consuming. */
  refCount: number;
}

interface WsBus {
  /** Subscribe a render listener for a topic. Bumps the refcount and
   *  emits a `subscribe` frame on the wire if this is the first
   *  subscriber for the topic. Returns the unsubscribe function. */
  subscribe: (topic: WsTopic, listener: () => void) => () => void;
  /** Snapshot accessor — returns the latest typed payload (or `null`
   *  before the first frame). */
  snapshot: <T>(topic: WsTopic) => T | null;
  /** Connection state, exposed for diagnostic UI. */
  status: () => 'connecting' | 'open' | 'closed';
  /** Register a listener for status changes. Returns unsubscribe. */
  registerStatusListener: (listener: () => void) => () => void;
}

const WsContext = createContext<WsBus | null>(null);

const RECONNECT_INITIAL_MS = 250;
const RECONNECT_MAX_MS = 5_000;

function buildWsUrl(): string {
  // Same-origin upgrade — `vectorizer_session` cookie travels
  // automatically. Falls back to the API base URL when the dashboard
  // is hosted on a different origin.
  if (typeof window === 'undefined') return '';
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${window.location.host}/ws/dashboard`;
}

interface WsDashboardProviderProps {
  children: ReactNode;
  /** Override the upgrade URL (mainly for tests). */
  url?: string;
}

export function WsDashboardProvider({ children, url }: WsDashboardProviderProps) {
  const stores = useRef<Map<WsTopic, TopicStore<unknown>>>(new Map());
  const socketRef = useRef<WebSocket | null>(null);
  const statusRef = useRef<'connecting' | 'open' | 'closed'>('connecting');
  const reconnectMsRef = useRef<number>(RECONNECT_INITIAL_MS);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const statusListenersRef = useRef<Set<() => void>>(new Set());

  const ensureStore = (topic: WsTopic): TopicStore<unknown> => {
    let s = stores.current.get(topic);
    if (!s) {
      s = { current: null, listeners: new Set(), refCount: 0 };
      stores.current.set(topic, s);
    }
    return s;
  };

  const notifyTopic = (topic: WsTopic) => {
    const s = stores.current.get(topic);
    if (!s) return;
    for (const l of s.listeners) l();
  };

  const setStatus = (next: 'connecting' | 'open' | 'closed') => {
    if (statusRef.current === next) return;
    statusRef.current = next;
    for (const l of statusListenersRef.current) l();
  };

  const wsUrl = url ?? buildWsUrl();

  // Stable bus exposed to children. It only depends on refs, so a
  // useMemo with an empty dep list keeps the identity stable across
  // renders.
  const bus = useMemo<WsBus>(() => {
    return {
      subscribe(topic, listener) {
        const s = ensureStore(topic);
        s.listeners.add(listener);
        s.refCount += 1;
        if (s.refCount === 1 && socketRef.current?.readyState === WebSocket.OPEN) {
          sendFrame({ op: 'subscribe', topics: [topic] });
        }
        return () => {
          s.listeners.delete(listener);
          s.refCount -= 1;
          if (s.refCount === 0 && socketRef.current?.readyState === WebSocket.OPEN) {
            sendFrame({ op: 'unsubscribe', topics: [topic] });
          }
        };
      },
      snapshot<T>(topic: WsTopic) {
        return (stores.current.get(topic)?.current ?? null) as T | null;
      },
      status: () => statusRef.current,
      registerStatusListener: (l) => {
        statusListenersRef.current.add(l);
        return () => {
          statusListenersRef.current.delete(l);
        };
      },
    };
  }, []);

  const sendFrame = (frame: unknown) => {
    const ws = socketRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    try {
      ws.send(JSON.stringify(frame));
    } catch {
      // Socket transitioned to CLOSING between the readyState check and
      // the send call — drop silently; reconnect will replay the
      // subscriptions.
    }
  };

  const connect = () => {
    if (!wsUrl) return;
    setStatus('connecting');
    let ws: WebSocket;
    try {
      ws = new WebSocket(wsUrl);
    } catch {
      scheduleReconnect();
      return;
    }
    socketRef.current = ws;

    ws.onopen = () => {
      reconnectMsRef.current = RECONNECT_INITIAL_MS;
      setStatus('open');
      // Resubscribe every topic that has live consumers — this also
      // handles the initial handshake when subscribers mounted before
      // the socket opened.
      const toResub: WsTopic[] = [];
      for (const [topic, s] of stores.current.entries()) {
        if (s.refCount > 0) toResub.push(topic);
      }
      if (toResub.length > 0) {
        sendFrame({ op: 'subscribe', topics: toResub });
      }
    };

    ws.onmessage = (ev) => {
      if (typeof ev.data !== 'string') return;
      let frame: unknown;
      try {
        frame = JSON.parse(ev.data);
      } catch {
        return;
      }
      if (!frame || typeof frame !== 'object') return;
      const f = frame as Record<string, unknown>;
      if (typeof f.topic === 'string') {
        const topic = f.topic as WsTopic;
        const s = ensureStore(topic);
        s.current = f.data;
        notifyTopic(topic);
        return;
      }
      // op-style frames: pong, error. Errors trigger a soft reconnect
      // because the server has already closed (e.g. stream_lag).
      if (f.op === 'error') {
        ws.close();
      }
      // pong: no-op for now (server doesn't send unsolicited pings yet)
    };

    ws.onerror = () => {
      // The error event fires before close in some browsers — defer
      // the reconnect to onclose so we don't double-schedule.
    };

    ws.onclose = () => {
      setStatus('closed');
      socketRef.current = null;
      scheduleReconnect();
    };
  };

  const scheduleReconnect = () => {
    if (reconnectTimerRef.current !== null) return;
    const delay = reconnectMsRef.current;
    reconnectMsRef.current = Math.min(delay * 2, RECONNECT_MAX_MS);
    reconnectTimerRef.current = setTimeout(() => {
      reconnectTimerRef.current = null;
      connect();
    }, delay);
  };

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimerRef.current !== null) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
      const ws = socketRef.current;
      socketRef.current = null;
      if (ws) {
        ws.close();
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wsUrl]);

  return <WsContext.Provider value={bus}>{children}</WsContext.Provider>;
}

function noopUnsubscribe() {
  return undefined;
}

function noopSubscribe() {
  return noopUnsubscribe;
}

/**
 * Subscribe to a typed topic on the dashboard WebSocket. Returns the
 * latest snapshot (or `null` before the first frame). Unsubscribes on
 * unmount; the underlying socket sends a `subscribe` op only when the
 * topic transitions from 0→1 mounted consumers.
 */
export function useWsTopic<T>(topic: WsTopic): T | null {
  const bus = useContext(WsContext);
  const subscribe = bus
    ? (listener: () => void) => bus.subscribe(topic, listener)
    : noopSubscribe;
  const getSnapshot = () => (bus ? bus.snapshot<T>(topic) : null);
  return useSyncExternalStore(subscribe, getSnapshot, () => null);
}

/**
 * Diagnostic accessor for the WS connection state. Used by the
 * Topbar status pill to surface reconnect activity without forcing
 * every consumer to re-render on every state flip.
 */
export function useWsStatus(): 'connecting' | 'open' | 'closed' {
  const bus = useContext(WsContext);
  const subscribe = bus
    ? (listener: () => void) => bus.registerStatusListener(listener)
    : noopSubscribe;
  const getSnapshot = () => (bus ? bus.status() : 'closed');
  return useSyncExternalStore(subscribe, getSnapshot, () => 'closed');
}
