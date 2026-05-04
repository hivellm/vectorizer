/**
 * Logs page — console-themed restyle.
 *
 * Visual restyle only: behaviour (polling /api/logs every 2s when
 * auto-refresh is on, level filter, auto-scroll, download, and clear)
 * is preserved from the pre-redesign version. The redesign brief has
 * no dedicated mockup for Logs, so this page applies the established
 * Phase 3 recipe:
 *   - `.page` + `.page-head` shell with title/sub + toolbar buttons
 *   - console `Card` / `CardHead` / `CardBody`
 *   - `Pill` tones for log levels (info=teal, warn=amber, error=red,
 *     debug/trace/other=muted)
 *   - `.code` block (already shipped in console.css) for the rolling
 *     log stream
 *   - `.input` / `.btn` for filter + actions, `Icons.*` instead of
 *     `@untitledui/icons`
 *   - no Tailwind utility classes, no `dark:` variants
 */

import { useEffect, useRef, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useToastContext } from '@/providers/ToastProvider';
import { useWsTopic } from '@/providers/WsDashboardProvider';
import {
  Icons,
  Pill,
  type PillTone,
  Card,
  CardHead,
  CardBody,
} from '@/components/console';

type LogLevel = 'all' | 'error' | 'warn' | 'info' | 'debug' | 'trace';

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  target?: string;
}

function levelTone(level: string): PillTone {
  const l = level.toLowerCase();
  if (l === 'error') return 'red';
  if (l === 'warn' || l === 'warning') return 'amber';
  if (l === 'info') return 'teal';
  return 'muted';
}

function formatTime(ts: string): string {
  const d = new Date(ts);
  if (Number.isNaN(d.getTime())) return ts;
  return d.toLocaleTimeString();
}

function LogsPage() {
  const api = useApiClient();
  const toast = useToastContext();

  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [logLevel, setLogLevel] = useState<LogLevel>('all');
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);

  const loadLogs = async () => {
    try {
      const logsData = await api.get<unknown>('/api/logs');

      let parsedLogs: LogEntry[] = [];

      if (Array.isArray(logsData)) {
        parsedLogs = logsData as LogEntry[];
      } else if (typeof logsData === 'string') {
        const lines = logsData.split('\n').filter((line) => line.trim());
        parsedLogs = lines.map((line) => {
          try {
            const parsed = JSON.parse(line);
            return {
              timestamp: parsed.timestamp || new Date().toISOString(),
              level: parsed.level || 'info',
              message: parsed.message || parsed.msg || line,
              target: parsed.target,
            };
          } catch {
            return {
              timestamp: new Date().toISOString(),
              level: 'info',
              message: line,
            };
          }
        });
      } else if (
        logsData &&
        typeof logsData === 'object' &&
        Array.isArray((logsData as { logs?: LogEntry[] }).logs)
      ) {
        parsedLogs = (logsData as { logs: LogEntry[] }).logs;
      }

      setLogs(parsedLogs);
      setError(null);
    } catch (err) {
      console.error('Error loading logs:', err);
      // Don't surface error toast on every poll failure.
      if (!autoRefresh) {
        setError(err instanceof Error ? err.message : 'Failed to load logs');
      }
    } finally {
      setLoading(false);
    }
  };

  // One-shot REST fetch on mount populates the initial paint
  // (matching the `useRuntimeMetrics` / `useStatus` pattern from
  // phase29). Subsequent live updates arrive on the WS `logs` topic.
  useEffect(() => {
    loadLogs();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Phase30 — append each tailed log line into the rolling view when
  // auto-refresh is on. Cap at 1000 entries to avoid unbounded growth
  // on long-running sessions.
  const wsLog = useWsTopic<LogEntry>('logs');
  useEffect(() => {
    if (!autoRefresh) return;
    if (!wsLog) return;
    setLogs((prev) => {
      const next = prev.concat(wsLog);
      return next.length > 1000 ? next.slice(next.length - 1000) : next;
    });
  }, [wsLog, autoRefresh]);

  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  const filteredLogs = logs.filter((log) => {
    if (logLevel === 'all') return true;
    return log.level.toLowerCase() === logLevel.toLowerCase();
  });

  const handleDownload = () => {
    const logText = filteredLogs
      .map((log) => `[${log.timestamp}] ${log.level.toUpperCase()}: ${log.message}`)
      .join('\n');

    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `vectorizer-logs-${new Date().toISOString()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    toast.success('Logs downloaded successfully');
  };

  const handleClear = () => {
    if (
      window.confirm(
        'Are you sure you want to clear the logs? This action cannot be undone.',
      )
    ) {
      setLogs([]);
      toast.info('Logs cleared from view');
    }
  };

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Logs</h1>
          <p className="page-sub">View server logs and events</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <label
            className="row"
            style={{ gap: 6, color: 'var(--text-2)', fontSize: 12, cursor: 'pointer' }}
          >
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto-refresh
          </label>
          <label
            className="row"
            style={{ gap: 6, color: 'var(--text-2)', fontSize: 12, cursor: 'pointer' }}
          >
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
            />
            Auto-scroll
          </label>
          <button className="btn" onClick={loadLogs} disabled={loading && logs.length === 0}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button className="btn" onClick={handleDownload}>
            <Icons.arrowDown size={13} />
            Download
          </button>
          <button className="btn" onClick={handleClear}>
            <Icons.trash size={13} />
            Clear
          </button>
        </div>
      </div>

      {error && (
        <div style={{ marginBottom: 14 }}>
          <Card>
            <CardBody>
              <div className="row" style={{ gap: 8 }}>
                <Pill tone="red">error</Pill>
                <span style={{ color: 'var(--text-2)' }}>{error}</span>
              </div>
            </CardBody>
          </Card>
        </div>
      )}

      <Card>
        <CardHead
          title="Filters"
          sub={`Showing ${filteredLogs.length} of ${logs.length} entries`}
        />
        <CardBody>
          <div className="row" style={{ gap: 12, flexWrap: 'wrap' }}>
            <label
              className="col"
              style={{ display: 'flex', flexDirection: 'column', gap: 6, minWidth: 200 }}
            >
              <span style={{ color: 'var(--text-2)', fontSize: 12 }}>Log level</span>
              <select
                className="input"
                value={logLevel}
                onChange={(e) => setLogLevel(e.target.value as LogLevel)}
              >
                <option value="all">All levels</option>
                <option value="error">Error</option>
                <option value="warn">Warning</option>
                <option value="info">Info</option>
                <option value="debug">Debug</option>
                <option value="trace">Trace</option>
              </select>
            </label>
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <Card>
        <CardHead
          title="Stream"
          sub={loading && logs.length === 0 ? 'loading…' : autoRefresh ? 'live · 2s poll' : 'paused'}
        />
        <CardBody tight>
          {filteredLogs.length === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No logs available
            </div>
          ) : (
            <pre
              className="code"
              role="log"
              aria-live="polite"
              aria-label="Server log stream"
              style={{ maxHeight: 600, overflowY: 'auto', whiteSpace: 'pre-wrap', margin: 0 }}
            >
              {filteredLogs.map((log, idx) => (
                <div
                  key={idx}
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'auto auto 1fr',
                    gap: 10,
                    alignItems: 'baseline',
                    padding: '2px 0',
                  }}
                >
                  <span style={{ color: 'var(--text-3)', fontSize: 11 }}>
                    {formatTime(log.timestamp)}
                  </span>
                  <Pill tone={levelTone(log.level)}>{log.level.toLowerCase()}</Pill>
                  <span style={{ color: 'var(--text-1)' }}>
                    {log.message}
                    {log.target && (
                      <span style={{ color: 'var(--text-3)', marginLeft: 8 }}>
                        ({log.target})
                      </span>
                    )}
                  </span>
                </div>
              ))}
              <div ref={logsEndRef} />
            </pre>
          )}
        </CardBody>
      </Card>
    </div>
  );
}

export default LogsPage;
