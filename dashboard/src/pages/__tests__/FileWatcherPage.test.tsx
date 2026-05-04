import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import FileWatcherPage from '../FileWatcherPage';

// The real hook (src/hooks/useFileWatcher.ts) exposes
// { getStatus, getMetrics, updateConfig } — the legacy
// FileWatcherPage drives off `status.watch_paths` and the metrics
// timing/files/system/status sub-objects. Mock the same surface here so
// the restyled page can mount without hitting the network.
vi.mock('@/hooks/useFileWatcher', () => ({
  useFileWatcher: () => ({
    getStatus: vi.fn(async () => ({
      enabled: true,
      running: true,
      watch_paths: ['/var/lib/vectorizer/docs', '/srv/code'],
      auto_discovery: true,
      enable_auto_update: true,
      hot_reload: false,
      exclude_patterns: ['*.tmp', 'node_modules/**'],
    })),
    getMetrics: vi.fn(async () => ({
      timing: {
        avg_file_processing_ms: 4.21,
        avg_discovery_ms: 12.3,
        avg_sync_ms: 0.9,
        uptime_seconds: 4321,
        peak_processing_ms: 18,
      },
      files: {
        total_files_processed: 4812,
        files_processed_success: 4799,
        files_processed_error: 7,
        files_skipped: 6,
        files_in_progress: 0,
        files_discovered: 4823,
        files_removed: 11,
        files_indexed_realtime: 4799,
      },
      system: {
        memory_usage_bytes: 134_217_728,
        cpu_usage_percent: 1.7,
        thread_count: 4,
        active_file_handles: 12,
        disk_io_ops_per_sec: 88,
        network_io_bytes_per_sec: 0,
      },
      network: {
        total_api_requests: 0,
        successful_api_requests: 0,
        failed_api_requests: 0,
        avg_api_response_ms: 0,
        peak_api_response_ms: 0,
        active_connections: 0,
      },
      status: { total_errors: 0, total_warnings: 0, is_healthy: true },
    })),
    updateConfig: vi.fn(async () => undefined),
  }),
}));

vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

describe('FileWatcherPage', () => {
  it('renders the page heading and the watched-paths table', async () => {
    render(<MemoryRouter><FileWatcherPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /File Watcher/i })).toBeTruthy();
    // Either a watched path or the "no paths" empty state should appear
    // once the async status fetch resolves.
    const matches = await screen.findAllByText(
      /var\/lib\/vectorizer|srv\/code|No watch paths|Auto-discover/i,
      undefined,
      { timeout: 3000 },
    );
    expect(matches.length).toBeGreaterThan(0);
  });
});
