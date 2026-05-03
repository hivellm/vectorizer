/**
 * File Watcher page — console-themed restyle.
 *
 * Visual restyle only: behaviour (status polling, start/stop,
 * configuration mutation, metrics rendering) is preserved from the
 * pre-redesign version. The redesign brief has no dedicated mockup for
 * File Watcher, so this page applies the established Phase 3 recipe:
 *   - `.page` + `.page-head` shell
 *   - console `Card` / `CardHead` / `CardBody`
 *   - console `Tbl` / `Th` / `Td` for the watched-paths list
 *   - `.btn` + `Icons.*` for actions
 *   - `StatusPill` / `Pill` for the runtime status
 *   - `Kpi` cards for the metrics dashboards
 *   - no Tailwind utility classes, no `dark:` variants
 */

import { useEffect, useState } from 'react';
import { useFileWatcher, FileWatcherStatus, FileWatcherMetrics } from '@/hooks/useFileWatcher';
import { useToastContext } from '@/providers/ToastProvider';
import { formatNumber } from '@/utils/formatters';
import {
  Icons,
  Pill,
  StatusPill,
  Card,
  CardHead,
  CardBody,
  Kpi,
  Tbl,
  Th,
  Td,
} from '@/components/console';

function formatUptime(seconds: number | null | undefined): string {
  const sec = seconds ?? 0;
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m ${sec % 60}s`;
  const hours = Math.floor(sec / 3600);
  const minutes = Math.floor((sec % 3600) / 60);
  return `${hours}h ${minutes}m`;
}

function formatBytes(bytes: number | null | undefined): string {
  const b = bytes ?? 0;
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(2)} KB`;
  return `${(b / (1024 * 1024)).toFixed(2)} MB`;
}

function statusToken(status: FileWatcherStatus | null): {
  token: string;
  label: string;
  tone: 'green' | 'amber' | 'muted';
} {
  if (!status) return { token: 'warning', label: 'unknown', tone: 'muted' };
  if (status.running) return { token: 'healthy', label: 'running', tone: 'green' };
  if (status.enabled) return { token: 'warning', label: 'enabled · not running', tone: 'amber' };
  return { token: 'warning', label: 'disabled', tone: 'muted' };
}

function FileWatcherPage() {
  const { getStatus, getMetrics, updateConfig } = useFileWatcher();
  const toast = useToastContext();

  const [status, setStatus] = useState<FileWatcherStatus | null>(null);
  const [metrics, setMetrics] = useState<FileWatcherMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [configOpen, setConfigOpen] = useState(false);
  const [configForm, setConfigForm] = useState<Partial<FileWatcherStatus>>({});
  const [saving, setSaving] = useState(false);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [statusData, metricsData] = await Promise.all([
        getStatus(),
        getMetrics().catch(() => null),
      ]);

      setStatus(statusData);
      setMetrics(metricsData);

      // Determine if running based on metrics
      if (metricsData && metricsData.timing.uptime_seconds > 0) {
        setStatus((prev) => (prev ? { ...prev, running: true } : null));
      }
    } catch (err) {
      console.error('Error loading file watcher data:', err);
      setError(err instanceof Error ? err.message : 'Failed to load file watcher data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleStart = async () => {
    try {
      await updateConfig({ enabled: true });
      toast.success('File watcher enabled');
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to enable file watcher');
    }
  };

  const handleStop = async () => {
    try {
      await updateConfig({ enabled: false });
      toast.success('File watcher disabled');
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to disable file watcher');
    }
  };

  const handleSaveConfig = async () => {
    setSaving(true);
    try {
      await updateConfig(configForm);
      toast.success('Configuration updated successfully');
      setConfigOpen(false);
      await loadData();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to update configuration');
    } finally {
      setSaving(false);
    }
  };

  const isRunning = !!status?.running;
  const isEnabled = !!status?.enabled;
  const pathCount = status?.watch_paths.length ?? 0;
  const excludeCount = status?.exclude_patterns.length ?? 0;
  const st = statusToken(status);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">File Watcher</h1>
          <p className="page-sub">Monitor filesystem paths for changes and auto-reindex</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <Pill tone={st.tone}>
            <span className={`dot ${st.tone === 'muted' ? '' : st.tone}`} />
            {st.label}
          </Pill>
          <button
            className="btn"
            onClick={() => {
              setConfigForm(status || {});
              setConfigOpen(true);
            }}
          >
            <Icons.settings size={13} />
            Configure
          </button>
          <button className="btn" onClick={loadData} disabled={loading}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          {isEnabled && isRunning ? (
            <button className="btn" onClick={handleStop}>
              <Icons.x size={13} />
              Stop
            </button>
          ) : (
            <button className="btn primary" onClick={handleStart}>
              <Icons.bolt size={13} />
              Start
            </button>
          )}
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
          title="Status"
          sub={loading && !status ? 'loading…' : undefined}
          right={<StatusPill status={st.token} />}
        />
        <CardBody>
          <div
            className="row"
            style={{ gap: 24, flexWrap: 'wrap', marginBottom: 14 }}
          >
            <Kpi label="Watch paths" value={pathCount > 0 ? pathCount : 'auto'} />
            <Kpi
              label="Auto discovery"
              value={status?.auto_discovery ? 'on' : 'off'}
              accent={status?.auto_discovery ? 'teal' : 'none'}
            />
            <Kpi
              label="Auto update"
              value={status?.enable_auto_update ? 'on' : 'off'}
              accent={status?.enable_auto_update ? 'teal' : 'none'}
            />
            <Kpi
              label="Hot reload"
              value={status?.hot_reload ? 'on' : 'off'}
              accent={status?.hot_reload ? 'magenta' : 'none'}
            />
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <Card>
        <CardHead
          title="Watched paths"
          sub={pathCount > 0 ? `${pathCount} configured` : undefined}
        />
        <CardBody tight>
          {pathCount === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No watch paths configured · Auto-discover is{' '}
              {status?.auto_discovery ? 'on' : 'off'}.
            </div>
          ) : (
            <Tbl>
              <thead>
                <tr>
                  <Th>Path</Th>
                  <Th>Recursive</Th>
                  <Th>Status</Th>
                </tr>
              </thead>
              <tbody>
                {status!.watch_paths.map((p) => (
                  <tr key={p}>
                    <Td className="mono" style={{ fontSize: 12 }}>
                      {p}
                    </Td>
                    <Td>
                      <Pill tone="muted">
                        <Icons.check size={11} />
                        recursive
                      </Pill>
                    </Td>
                    <Td>
                      <StatusPill status={isRunning ? 'healthy' : 'warning'} />
                    </Td>
                  </tr>
                ))}
              </tbody>
            </Tbl>
          )}
        </CardBody>
      </Card>

      {excludeCount > 0 && (
        <>
          <div style={{ height: 14 }} />
          <Card>
            <CardHead
              title="Exclude patterns"
              sub={`${excludeCount} pattern${excludeCount === 1 ? '' : 's'}`}
            />
            <CardBody tight>
              <Tbl>
                <thead>
                  <tr>
                    <Th>Pattern</Th>
                  </tr>
                </thead>
                <tbody>
                  {status!.exclude_patterns.map((pat) => (
                    <tr key={pat}>
                      <Td className="mono" style={{ fontSize: 12 }}>
                        {pat}
                      </Td>
                    </tr>
                  ))}
                </tbody>
              </Tbl>
            </CardBody>
          </Card>
        </>
      )}

      {metrics && (
        <>
          <div style={{ height: 14 }} />
          <Card>
            <CardHead title="File metrics" />
            <CardBody>
              <div className="row" style={{ gap: 16, flexWrap: 'wrap' }}>
                <Kpi label="Processed" value={formatNumber(metrics.files.total_files_processed)} />
                <Kpi
                  label="Success"
                  value={formatNumber(metrics.files.files_processed_success)}
                  accent="teal"
                />
                <Kpi
                  label="Errors"
                  value={formatNumber(metrics.files.files_processed_error)}
                  accent={metrics.files.files_processed_error > 0 ? 'magenta' : 'none'}
                />
                <Kpi
                  label="Skipped"
                  value={formatNumber(metrics.files.files_skipped)}
                  accent="amber"
                />
                <Kpi label="Discovered" value={formatNumber(metrics.files.files_discovered)} />
                <Kpi label="Removed" value={formatNumber(metrics.files.files_removed)} />
                <Kpi label="In progress" value={formatNumber(metrics.files.files_in_progress)} />
                <Kpi
                  label="Realtime indexed"
                  value={formatNumber(metrics.files.files_indexed_realtime)}
                  accent="teal"
                />
              </div>
            </CardBody>
          </Card>

          <div style={{ height: 14 }} />
          <Card>
            <CardHead title="Performance" />
            <CardBody>
              <div className="row" style={{ gap: 16, flexWrap: 'wrap' }}>
                <Kpi label="Uptime" value={formatUptime(metrics.timing.uptime_seconds)} />
                <Kpi
                  label="Avg processing"
                  value={(metrics.timing.avg_file_processing_ms ?? 0).toFixed(2)}
                  unit="ms"
                />
                <Kpi
                  label="Avg discovery"
                  value={(metrics.timing.avg_discovery_ms ?? 0).toFixed(2)}
                  unit="ms"
                />
                <Kpi
                  label="Avg sync"
                  value={(metrics.timing.avg_sync_ms ?? 0).toFixed(2)}
                  unit="ms"
                />
                <Kpi
                  label="Peak processing"
                  value={(metrics.timing.peak_processing_ms ?? 0).toString()}
                  unit="ms"
                />
                {metrics.timing.last_activity && (
                  <Kpi
                    label="Last activity"
                    value={new Date(metrics.timing.last_activity).toLocaleString()}
                  />
                )}
              </div>
            </CardBody>
          </Card>

          <div style={{ height: 14 }} />
          <Card>
            <CardHead title="System" />
            <CardBody>
              <div className="row" style={{ gap: 16, flexWrap: 'wrap' }}>
                <Kpi label="Memory" value={formatBytes(metrics.system.memory_usage_bytes)} />
                <Kpi
                  label="CPU"
                  value={(metrics.system.cpu_usage_percent ?? 0).toFixed(2)}
                  unit="%"
                />
                <Kpi label="Threads" value={formatNumber(metrics.system.thread_count)} />
                <Kpi
                  label="File handles"
                  value={formatNumber(metrics.system.active_file_handles)}
                />
                <Kpi
                  label="Disk I/O"
                  value={formatNumber(metrics.system.disk_io_ops_per_sec)}
                  unit="ops/s"
                />
                <Kpi
                  label="Network"
                  value={formatBytes(metrics.system.network_io_bytes_per_sec)}
                  unit="/s"
                />
              </div>
            </CardBody>
          </Card>

          <div style={{ height: 14 }} />
          <Card>
            <CardHead
              title="Health"
              right={
                <StatusPill
                  status={metrics.status.is_healthy !== false ? 'healthy' : 'error'}
                />
              }
            />
            <CardBody>
              <div className="row" style={{ gap: 16, flexWrap: 'wrap' }}>
                <Kpi
                  label="Errors"
                  value={formatNumber(metrics.status.total_errors ?? 0)}
                  accent={(metrics.status.total_errors ?? 0) > 0 ? 'magenta' : 'none'}
                />
                <Kpi
                  label="Warnings"
                  value={formatNumber(metrics.status.total_warnings ?? 0)}
                  accent={(metrics.status.total_warnings ?? 0) > 0 ? 'amber' : 'none'}
                />
              </div>
              {metrics.status.last_error && (
                <div
                  className="mono"
                  style={{
                    marginTop: 14,
                    padding: 10,
                    fontSize: 12,
                    color: 'var(--magenta-hi)',
                    background: 'var(--surface-2)',
                    border: '1px solid var(--border)',
                    borderRadius: 4,
                    wordBreak: 'break-all',
                  }}
                >
                  {metrics.status.last_error}
                </div>
              )}
            </CardBody>
          </Card>
        </>
      )}

      {configOpen && status && (
        <>
          <div style={{ height: 14 }} />
          {/* TODO(actions): replace inline panel with a real modal once
              the console design ships a modal primitive. */}
          <Card>
            <CardHead
              title="Configuration"
              right={
                <button className="btn sm" onClick={() => setConfigOpen(false)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div className="col" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <label
                  className="row"
                  style={{ justifyContent: 'space-between', alignItems: 'center', gap: 8 }}
                >
                  <span style={{ color: 'var(--text-2)' }}>Auto discovery</span>
                  <input
                    type="checkbox"
                    checked={configForm.auto_discovery ?? status.auto_discovery}
                    onChange={(e) =>
                      setConfigForm({ ...configForm, auto_discovery: e.target.checked })
                    }
                  />
                </label>
                <label
                  className="row"
                  style={{ justifyContent: 'space-between', alignItems: 'center', gap: 8 }}
                >
                  <span style={{ color: 'var(--text-2)' }}>Auto update</span>
                  <input
                    type="checkbox"
                    checked={configForm.enable_auto_update ?? status.enable_auto_update}
                    onChange={(e) =>
                      setConfigForm({ ...configForm, enable_auto_update: e.target.checked })
                    }
                  />
                </label>
                <label
                  className="row"
                  style={{ justifyContent: 'space-between', alignItems: 'center', gap: 8 }}
                >
                  <span style={{ color: 'var(--text-2)' }}>Hot reload</span>
                  <input
                    type="checkbox"
                    checked={configForm.hot_reload ?? status.hot_reload}
                    onChange={(e) =>
                      setConfigForm({ ...configForm, hot_reload: e.target.checked })
                    }
                  />
                </label>
                <div className="row" style={{ gap: 8, marginTop: 4 }}>
                  <button
                    className="btn primary"
                    onClick={handleSaveConfig}
                    disabled={saving}
                  >
                    <Icons.check size={13} />
                    {saving ? 'Saving…' : 'Save changes'}
                  </button>
                  <button className="btn" onClick={() => setConfigOpen(false)}>
                    Cancel
                  </button>
                </div>
              </div>
            </CardBody>
          </Card>
        </>
      )}
    </div>
  );
}

export default FileWatcherPage;
