/**
 * Phase30 §4.3 — useWsTopic('collections') / useWsTopic('logs') tests.
 *
 * Verifies that when the multiplexer pushes a frame on a topic, the
 * mounted consumers receive the typed payload and re-render via
 * `useSyncExternalStore`. Uses a minimal stub WebSocket that the
 * provider opens against `url`.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { WsDashboardProvider, useWsTopic } from '../WsDashboardProvider';

interface CollectionsSnapshot {
  collections: Array<{ name: string; vector_count: number; dimension: number }>;
}
interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  source: string;
}

// Minimal stub of the global WebSocket — captures the most recent
// instance so individual tests can simulate `onmessage` / `onclose`.
class StubWebSocket {
  static last: StubWebSocket | null = null;
  static OPEN = 1;
  readyState = StubWebSocket.OPEN;
  url: string;
  onopen: (() => void) | null = null;
  onmessage: ((ev: { data: string }) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  sent: string[] = [];

  constructor(url: string) {
    this.url = url;
    StubWebSocket.last = this;
    // Schedule onopen synchronously inside a microtask so React
    // useEffect has flushed first.
    queueMicrotask(() => {
      if (this.onopen) this.onopen();
    });
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = 3;
    if (this.onclose) this.onclose();
  }

  emit(payload: unknown) {
    if (this.onmessage) {
      this.onmessage({ data: JSON.stringify(payload) });
    }
  }
}

function CollectionsConsumer() {
  const snap = useWsTopic<CollectionsSnapshot>('collections');
  if (!snap) return <span data-testid="state">empty</span>;
  return (
    <span data-testid="state">
      {snap.collections.map((c) => `${c.name}:${c.vector_count}`).join(',')}
    </span>
  );
}

function LogsConsumer() {
  const entry = useWsTopic<LogEntry>('logs');
  if (!entry) return <span data-testid="state">empty</span>;
  return <span data-testid="state">{entry.level}:{entry.message}</span>;
}

describe('WsDashboardProvider — phase30 topics', () => {
  let originalWs: typeof WebSocket;

  beforeEach(() => {
    originalWs = globalThis.WebSocket;
    // @ts-expect-error — stub sufficient for the provider's surface.
    globalThis.WebSocket = StubWebSocket;
    StubWebSocket.last = null;
  });

  afterEach(() => {
    globalThis.WebSocket = originalWs;
    vi.restoreAllMocks();
  });

  it('delivers a collections snapshot to a consumer', async () => {
    render(
      <WsDashboardProvider url="ws://stub/dashboard">
        <CollectionsConsumer />
      </WsDashboardProvider>,
    );

    // Yield so the provider's effect runs and `onopen` fires.
    await act(async () => {
      await Promise.resolve();
    });

    const ws = StubWebSocket.last!;
    expect(ws).toBeTruthy();
    expect(ws.sent.some((f) => f.includes('"collections"'))).toBe(true);

    await act(async () => {
      ws.emit({
        topic: 'collections',
        data: {
          collections: [{ name: 'docs', vector_count: 7, dimension: 384 }],
        },
      });
    });

    expect(screen.getByTestId('state').textContent).toBe('docs:7');
  });

  it('delivers a logs entry to a consumer', async () => {
    render(
      <WsDashboardProvider url="ws://stub/dashboard">
        <LogsConsumer />
      </WsDashboardProvider>,
    );
    await act(async () => {
      await Promise.resolve();
    });

    const ws = StubWebSocket.last!;
    expect(ws.sent.some((f) => f.includes('"logs"'))).toBe(true);

    await act(async () => {
      ws.emit({
        topic: 'logs',
        data: {
          timestamp: '2026-05-04T10:00:00Z',
          level: 'ERROR',
          message: 'boom',
          source: 'vectorizer',
        },
      });
    });

    expect(screen.getByTestId('state').textContent).toBe('ERROR:boom');
  });

  it('subscribes only on first mount and unsubscribes on last unmount', async () => {
    const { unmount } = render(
      <WsDashboardProvider url="ws://stub/dashboard">
        <CollectionsConsumer />
        <CollectionsConsumer />
      </WsDashboardProvider>,
    );
    await act(async () => {
      await Promise.resolve();
    });

    const ws = StubWebSocket.last!;
    const subscribeCount = ws.sent.filter((f) =>
      f.includes('"op":"subscribe"') && f.includes('"collections"'),
    ).length;
    // Two consumers share a single subscribe frame because the
    // provider refcounts.
    expect(subscribeCount).toBe(1);

    unmount();
    // After both consumers unmount the underlying socket is closed
    // by the provider's effect cleanup. The subscribe frame was sent
    // exactly once on mount which already proves refcounting; the
    // unsubscribe path is exercised when consumers unmount while the
    // socket is still open (covered by the integration test below).
    expect(StubWebSocket.last?.readyState).toBe(3);
  });

  it('emits unsubscribe when the last consumer unmounts but the socket stays open', async () => {
    function Holder({ show }: { show: boolean }) {
      return (
        <WsDashboardProvider url="ws://stub/dashboard">
          {show ? <CollectionsConsumer /> : null}
        </WsDashboardProvider>
      );
    }

    const { rerender } = render(<Holder show={true} />);
    await act(async () => {
      await Promise.resolve();
    });
    const ws = StubWebSocket.last!;

    rerender(<Holder show={false} />);
    await act(async () => {
      await Promise.resolve();
    });

    const unsubscribeCount = ws.sent.filter(
      (f) => f.includes('"op":"unsubscribe"') && f.includes('"collections"'),
    ).length;
    expect(unsubscribeCount).toBe(1);
  });
});
