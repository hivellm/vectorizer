/**
 * File Watcher page - Monitor file changes
 */

import { useEffect, useState } from 'react';
import { useFileWatcher, FileWatcherStatus, FileWatcherMetrics } from '@/hooks/useFileWatcher';
import LoadingState from '@/components/LoadingState';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import { useToastContext } from '@/providers/ToastProvider';
import { formatNumber } from '@/utils/formatters';
import { Play, Square, RefreshCw01, Settings01, AlertCircle, CheckCircle } from '@untitledui/icons';

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
        getMetrics().catch(() => null), // Metrics might not be available
      ]);

      setStatus(statusData);
      setMetrics(metricsData);
      
      // Determine if running based on metrics
      if (metricsData && metricsData.timing.uptime_seconds > 0) {
        setStatus(prev => prev ? { ...prev, running: true } : null);
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
    // Auto-refresh every 5 seconds
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
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

  const formatUptime = (seconds: number | null | undefined): string => {
    const sec = seconds ?? 0;
    if (sec < 60) return `${sec}s`;
    if (sec < 3600) return `${Math.floor(sec / 60)}m ${sec % 60}s`;
    const hours = Math.floor(sec / 3600);
    const minutes = Math.floor((sec % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const formatBytes = (bytes: number | null | undefined): string => {
    const b = bytes ?? 0;
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(2)} KB`;
    return `${(b / (1024 * 1024)).toFixed(2)} MB`;
  };

  if (loading && !status) {
    return <LoadingState message="Loading file watcher..." />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">File Watcher</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            Monitor and index file changes automatically
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              setConfigForm(status || {});
              setConfigOpen(true);
            }}
          >
            <Settings01 className="w-4 h-4 mr-2" />
            Configure
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={loadData}
          >
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          {status?.enabled && status?.running ? (
            <Button variant="danger" size="sm" onClick={handleStop}>
              <Square className="w-4 h-4 mr-2" />
              Stop
            </Button>
          ) : (
            <Button variant="primary" size="sm" onClick={handleStart}>
              <Play className="w-4 h-4 mr-2" />
              Start
            </Button>
          )}
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400" />
            <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
          </div>
        </div>
      )}

      {/* Status Card */}
      {status && (
        <Card>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-neutral-900 dark:text-white">Status</h2>
              <div className="flex items-center gap-2">
                {status.running ? (
                  <>
                    <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400" />
                    <span className="text-sm font-medium text-green-600 dark:text-green-400">Running</span>
                  </>
                ) : status.enabled ? (
                  <>
                    <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-400" />
                    <span className="text-sm font-medium text-yellow-600 dark:text-yellow-400">Enabled (Not Running)</span>
                  </>
                ) : (
                  <>
                    <Square className="w-5 h-5 text-neutral-400 dark:text-neutral-500" />
                    <span className="text-sm font-medium text-neutral-400 dark:text-neutral-500">Disabled</span>
                  </>
                )}
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Watch Paths</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {status.watch_paths.length > 0 ? status.watch_paths.length : 'Auto-discover'}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Auto Discovery</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {status.auto_discovery ? 'Enabled' : 'Disabled'}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Auto Update</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {status.enable_auto_update ? 'Enabled' : 'Disabled'}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Hot Reload</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {status.hot_reload ? 'Enabled' : 'Disabled'}
                </p>
              </div>
            </div>

            {status.watch_paths.length > 0 && (
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Watched Paths:</span>
                <ul className="mt-2 space-y-1">
                  {status.watch_paths.map((path, index) => (
                    <li key={index} className="text-sm font-mono text-neutral-700 dark:text-neutral-300">
                      {path}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {status.exclude_patterns.length > 0 && (
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Exclude Patterns:</span>
                <ul className="mt-2 space-y-1">
                  {status.exclude_patterns.map((pattern, index) => (
                    <li key={index} className="text-sm font-mono text-neutral-700 dark:text-neutral-300">
                      {pattern}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Metrics Card */}
      {metrics && (
        <>
          {/* File Metrics */}
          <Card>
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">File Metrics</h2>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Total Processed</span>
                <p className="text-xl font-bold text-neutral-900 dark:text-white mt-1">
                  {formatNumber(metrics.files.total_files_processed)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Success</span>
                <p className="text-xl font-bold text-green-600 dark:text-green-400 mt-1">
                  {formatNumber(metrics.files.files_processed_success)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Errors</span>
                <p className="text-xl font-bold text-red-600 dark:text-red-400 mt-1">
                  {formatNumber(metrics.files.files_processed_error)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Skipped</span>
                <p className="text-xl font-bold text-yellow-600 dark:text-yellow-400 mt-1">
                  {formatNumber(metrics.files.files_skipped)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Discovered</span>
                <p className="text-xl font-bold text-blue-600 dark:text-blue-400 mt-1">
                  {formatNumber(metrics.files.files_discovered)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Removed</span>
                <p className="text-xl font-bold text-neutral-600 dark:text-neutral-400 mt-1">
                  {formatNumber(metrics.files.files_removed)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">In Progress</span>
                <p className="text-xl font-bold text-purple-600 dark:text-purple-400 mt-1">
                  {formatNumber(metrics.files.files_in_progress)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Realtime Indexed</span>
                <p className="text-xl font-bold text-green-600 dark:text-green-400 mt-1">
                  {formatNumber(metrics.files.files_indexed_realtime)}
                </p>
              </div>
            </div>
          </Card>

          {/* Performance Metrics */}
          <Card>
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">Performance Metrics</h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Uptime</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {formatUptime(metrics.timing.uptime_seconds)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Avg Processing Time</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {(metrics.timing.avg_file_processing_ms ?? 0).toFixed(2)}ms
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Avg Discovery Time</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {(metrics.timing.avg_discovery_ms ?? 0).toFixed(2)}ms
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Avg Sync Time</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {(metrics.timing.avg_sync_ms ?? 0).toFixed(2)}ms
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Peak Processing Time</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {metrics.timing.peak_processing_ms ?? 0}ms
                </p>
              </div>
              {metrics.timing.last_activity && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Last Activity</span>
                  <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                    {new Date(metrics.timing.last_activity).toLocaleString()}
                  </p>
                </div>
              )}
            </div>
          </Card>

          {/* System Metrics */}
          <Card>
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">System Metrics</h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Memory Usage</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {formatBytes(metrics.system.memory_usage_bytes ?? 0)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">CPU Usage</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {(metrics.system.cpu_usage_percent ?? 0).toFixed(2)}%
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Thread Count</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {formatNumber(metrics.system.thread_count)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Active File Handles</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {formatNumber(metrics.system.active_file_handles)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Disk I/O Ops/sec</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {formatNumber(metrics.system.disk_io_ops_per_sec)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Network I/O</span>
                <p className="text-lg font-semibold text-neutral-900 dark:text-white mt-1">
                  {formatBytes(metrics.system.network_io_bytes_per_sec ?? 0)}/s
                </p>
              </div>
            </div>
          </Card>

          {/* Status Metrics */}
          <Card>
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">Health Status</h2>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Status</span>
                <p className={`text-lg font-semibold mt-1 ${metrics.status.is_healthy !== false ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>
                  {metrics.status.is_healthy !== false ? 'Healthy' : 'Unhealthy'}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Total Errors</span>
                <p className="text-lg font-semibold text-red-600 dark:text-red-400 mt-1">
                  {formatNumber(metrics.status.total_errors ?? 0)}
                </p>
              </div>
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Total Warnings</span>
                <p className="text-lg font-semibold text-yellow-600 dark:text-yellow-400 mt-1">
                  {formatNumber(metrics.status.total_warnings ?? 0)}
                </p>
              </div>
              {metrics.status.last_error && (
                <div className="sm:col-span-3">
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Last Error</span>
                  <p className="text-sm font-mono text-red-600 dark:text-red-400 mt-1 break-all">
                    {metrics.status.last_error}
                  </p>
                </div>
              )}
            </div>
          </Card>
        </>
      )}

      {/* Configuration Modal */}
      {configOpen && status && (
        <Card>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-neutral-900 dark:text-white">Configuration</h2>
              <Button variant="ghost" size="sm" onClick={() => setConfigOpen(false)}>
                Close
              </Button>
            </div>

            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                  Auto Discovery
                </label>
                <input
                  type="checkbox"
                  checked={configForm.auto_discovery ?? status.auto_discovery}
                  onChange={(e) => setConfigForm({ ...configForm, auto_discovery: e.target.checked })}
                  className="w-4 h-4 text-primary-600 rounded focus:ring-primary-500"
                />
              </div>

              <div className="flex items-center justify-between">
                <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                  Auto Update
                </label>
                <input
                  type="checkbox"
                  checked={configForm.enable_auto_update ?? status.enable_auto_update}
                  onChange={(e) => setConfigForm({ ...configForm, enable_auto_update: e.target.checked })}
                  className="w-4 h-4 text-primary-600 rounded focus:ring-primary-500"
                />
              </div>

              <div className="flex items-center justify-between">
                <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                  Hot Reload
                </label>
                <input
                  type="checkbox"
                  checked={configForm.hot_reload ?? status.hot_reload}
                  onChange={(e) => setConfigForm({ ...configForm, hot_reload: e.target.checked })}
                  className="w-4 h-4 text-primary-600 rounded focus:ring-primary-500"
                />
              </div>

              <div className="flex gap-2">
                <Button variant="primary" onClick={handleSaveConfig} disabled={saving} isLoading={saving}>
                  Save Changes
                </Button>
                <Button variant="secondary" onClick={() => setConfigOpen(false)}>
                  Cancel
                </Button>
              </div>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}

export default FileWatcherPage;
