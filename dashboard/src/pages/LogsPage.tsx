/**
 * Logs page - View server logs
 */

import { useEffect, useState, useRef } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import { Select } from '@/components/ui/Select';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { RefreshCw01, Trash01 } from '@untitledui/icons';

type LogLevel = 'all' | 'error' | 'warn' | 'info' | 'debug' | 'trace';

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  target?: string;
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
  const logsContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadLogs();
  }, []);

  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(loadLogs, 2000); // Refresh every 2 seconds
      return () => clearInterval(interval);
    }
  }, [autoRefresh]);

  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  const loadLogs = async () => {
    try {
      const logsData = await api.get<any>('/api/logs');
      
      // Parse logs if they come as an array or string
      let parsedLogs: LogEntry[] = [];
      
      if (Array.isArray(logsData)) {
        parsedLogs = logsData;
      } else if (typeof logsData === 'string') {
        // Parse log lines if they come as a string
        const lines = logsData.split('\n').filter(line => line.trim());
        parsedLogs = lines.map((line) => {
          // Try to parse structured log
          try {
            const parsed = JSON.parse(line);
            return {
              timestamp: parsed.timestamp || new Date().toISOString(),
              level: parsed.level || 'info',
              message: parsed.message || parsed.msg || line,
              target: parsed.target,
            };
          } catch {
            // Plain text log
            return {
              timestamp: new Date().toISOString(),
              level: 'info',
              message: line,
            };
          }
        });
      } else if (logsData.logs && Array.isArray(logsData.logs)) {
        parsedLogs = logsData.logs;
      }

      setLogs(parsedLogs);
      setError(null);
    } catch (err) {
      console.error('Error loading logs:', err);
      // Don't show error toast on every refresh failure
      if (!autoRefresh) {
        setError(err instanceof Error ? err.message : 'Failed to load logs');
      }
    } finally {
      setLoading(false);
    }
  };

  const filteredLogs = logs.filter((log) => {
    if (logLevel === 'all') return true;
    return log.level.toLowerCase() === logLevel.toLowerCase();
  });

  const getLogLevelColor = (level: string) => {
    const levelLower = level.toLowerCase();
    switch (levelLower) {
      case 'error':
        return 'text-red-600 dark:text-red-400';
      case 'warn':
      case 'warning':
        return 'text-yellow-600 dark:text-yellow-400';
      case 'info':
        return 'text-blue-600 dark:text-blue-400';
      case 'debug':
        return 'text-purple-600 dark:text-purple-400';
      case 'trace':
        return 'text-neutral-600 dark:text-neutral-400';
      default:
        return 'text-neutral-900 dark:text-white';
    }
  };

  const getLogLevelBg = (level: string) => {
    const levelLower = level.toLowerCase();
    switch (levelLower) {
      case 'error':
        return 'bg-red-50 dark:bg-red-900/20';
      case 'warn':
      case 'warning':
        return 'bg-yellow-50 dark:bg-yellow-900/20';
      case 'info':
        return 'bg-blue-50 dark:bg-blue-900/20';
      case 'debug':
        return 'bg-purple-50 dark:bg-purple-900/20';
      case 'trace':
        return 'bg-neutral-50 dark:bg-neutral-900/20';
      default:
        return 'bg-white dark:bg-neutral-900';
    }
  };

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
    if (window.confirm('Are you sure you want to clear the logs? This action cannot be undone.')) {
      setLogs([]);
      toast.info('Logs cleared from view');
    }
  };

  if (loading && logs.length === 0) {
    return <LoadingState message="Loading logs..." />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Logs</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            View server logs and events
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="autoRefresh"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="w-4 h-4 text-primary-600 rounded focus:ring-primary-500"
            />
            <label htmlFor="autoRefresh" className="text-sm text-neutral-700 dark:text-neutral-300">
              Auto-refresh
            </label>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="autoScroll"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="w-4 h-4 text-primary-600 rounded focus:ring-primary-500"
            />
            <label htmlFor="autoScroll" className="text-sm text-neutral-700 dark:text-neutral-300">
              Auto-scroll
            </label>
          </div>
          <Button variant="secondary" size="sm" onClick={loadLogs}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button variant="secondary" size="sm" onClick={handleDownload}>
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Download
          </Button>
          <Button variant="secondary" size="sm" onClick={handleClear}>
            <Trash01 className="w-4 h-4 mr-2" />
            Clear
          </Button>
        </div>
      </div>

      {/* Filters */}
      <Card>
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-4">
          <div className="w-full sm:w-48">
            <Select
              label="Log Level"
              value={logLevel}
              onChange={(value) => setLogLevel(value as LogLevel)}
            >
              <Select.Option id="all" value="all">All Levels</Select.Option>
              <Select.Option id="error" value="error">Error</Select.Option>
              <Select.Option id="warn" value="warn">Warning</Select.Option>
              <Select.Option id="info" value="info">Info</Select.Option>
              <Select.Option id="debug" value="debug">Debug</Select.Option>
              <Select.Option id="trace" value="trace">Trace</Select.Option>
            </Select>
          </div>
          <div className="flex-1 text-sm text-neutral-500 dark:text-neutral-400">
            Showing {filteredLogs.length} of {logs.length} log entries
          </div>
        </div>
      </Card>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Logs Viewer */}
      <Card>
        <div
          ref={logsContainerRef}
          className="h-[600px] overflow-y-auto bg-neutral-950 dark:bg-neutral-950 rounded-lg p-4 font-mono text-sm"
        >
          {filteredLogs.length === 0 ? (
            <div className="text-center py-12 text-neutral-500 dark:text-neutral-400">
              No logs available
            </div>
          ) : (
            <div className="space-y-1">
              {filteredLogs.map((log, idx) => (
                <div
                  key={idx}
                  className={`p-2 rounded ${getLogLevelBg(log.level)} border-l-4 ${
                    log.level.toLowerCase() === 'error'
                      ? 'border-red-500'
                      : log.level.toLowerCase() === 'warn' || log.level.toLowerCase() === 'warning'
                      ? 'border-yellow-500'
                      : log.level.toLowerCase() === 'info'
                      ? 'border-blue-500'
                      : 'border-neutral-500'
                  }`}
                >
                  <div className="flex items-start gap-2">
                    <span className="text-xs text-neutral-400 dark:text-neutral-500 flex-shrink-0">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </span>
                    <span
                      className={`font-semibold uppercase text-xs flex-shrink-0 ${getLogLevelColor(log.level)}`}
                    >
                      {log.level}
                    </span>
                    <span className={`flex-1 ${getLogLevelColor(log.level)}`}>
                      {log.message}
                    </span>
                  </div>
                  {log.target && (
                    <div className="text-xs text-neutral-500 dark:text-neutral-400 mt-1 ml-20">
                      {log.target}
                    </div>
                  )}
                </div>
              ))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
      </Card>
    </div>
  );
}

export default LogsPage;
