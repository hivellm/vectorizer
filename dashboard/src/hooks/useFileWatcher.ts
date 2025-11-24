/**
 * File Watcher hooks
 */

import { useApiClient } from './useApiClient';

export interface FileWatcherStatus {
  enabled: boolean;
  running: boolean;
  watch_paths: string[];
  auto_discovery: boolean;
  enable_auto_update: boolean;
  hot_reload: boolean;
  exclude_patterns: string[];
}

export interface FileWatcherMetrics {
  timing: {
    avg_file_processing_ms: number;
    avg_discovery_ms: number;
    avg_sync_ms: number;
    uptime_seconds: number;
    last_activity?: string;
    peak_processing_ms: number;
  };
  files: {
    total_files_processed: number;
    files_processed_success: number;
    files_processed_error: number;
    files_skipped: number;
    files_in_progress: number;
    files_discovered: number;
    files_removed: number;
    files_indexed_realtime: number;
  };
  system: {
    memory_usage_bytes: number;
    cpu_usage_percent: number;
    thread_count: number;
    active_file_handles: number;
    disk_io_ops_per_sec: number;
    network_io_bytes_per_sec: number;
  };
  network: {
    total_api_requests: number;
    successful_api_requests: number;
    failed_api_requests: number;
    avg_api_response_ms: number;
    peak_api_response_ms: number;
    active_connections: number;
  };
  status: {
    total_errors: number;
    total_warnings: number;
    is_healthy: boolean;
    last_error?: string;
  };
}

export function useFileWatcher() {
  const api = useApiClient();

  const getStatus = async (): Promise<FileWatcherStatus> => {
    try {
      const config = await api.get<any>('/api/workspace/config');
      const fileWatcher = config.global_settings?.file_watcher || {};
      
      return {
        enabled: fileWatcher.enabled !== false,
        running: false, // Will be determined by metrics
        watch_paths: fileWatcher.watch_paths || [],
        auto_discovery: fileWatcher.auto_discovery !== false,
        enable_auto_update: fileWatcher.enable_auto_update !== false,
        hot_reload: fileWatcher.hot_reload !== false,
        exclude_patterns: fileWatcher.exclude_patterns || [],
      };
    } catch (error) {
      console.error('Error fetching file watcher status:', error);
      throw error;
    }
  };

  const getMetrics = async (): Promise<FileWatcherMetrics | null> => {
    try {
      const metrics = await api.get<FileWatcherMetrics>('/metrics');
      return metrics;
    } catch (error) {
      console.warn('File watcher metrics not available:', error);
      return null; // Return null if metrics endpoint is not available
    }
  };

  const updateConfig = async (config: Partial<FileWatcherStatus>): Promise<void> => {
    try {
      const currentConfig = await api.get<any>('/api/workspace/config');
      const updatedConfig = {
        ...currentConfig,
        global_settings: {
          ...currentConfig.global_settings,
          file_watcher: {
            ...currentConfig.global_settings?.file_watcher,
            ...config,
          },
        },
      };
      await api.post('/api/workspace/config', updatedConfig);
    } catch (error) {
      console.error('Error updating file watcher config:', error);
      throw error;
    }
  };

  return {
    getStatus,
    getMetrics,
    updateConfig,
  };
}

