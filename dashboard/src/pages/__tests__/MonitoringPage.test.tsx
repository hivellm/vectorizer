import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import MonitoringPage from '../MonitoringPage';

vi.mock('@/hooks/useMetrics', () => ({
  useMetrics: () => ({
    metrics: {
      qps: 0,
      p99Ms: 0,
      cpuPercent: 0,
      memPercent: 0,
      connections: 0,
      cacheHitRate: 0,
      totalVectors: 0,
    },
    loading: false,
    error: null,
  }),
}));

vi.mock('@/hooks/useStats', () => ({
  useStats: () => ({
    stats: {
      status: 'healthy',
      cache: {
        size: 7000,
        capacity: 10000,
        hits: 4_210_000,
        misses: 258_000,
        evictions: 1204,
        hitRate: 0.942,
      },
    },
    loading: false,
    error: null,
  }),
}));

vi.mock('@/hooks/useRuntimeMetrics', () => ({
  useRuntimeMetrics: () => ({
    metrics: {
      cpuPercent: 19.4,
      memoryPercent: 0.6,
      memoryRssBytes: 188_000_000,
      memoryTotalBytes: 33_000_000_000,
      activeConnections: 3,
      uptimeSeconds: 6947,
      qpsWindow60s: 0,
      errorRate5xx60s: 0,
      throughputByRoute: [],
      wal: { currentSeq: 0, sizeBytes: 0, lastCheckpointAt: 0, lastCheckpointSeq: 0 },
    },
    qpsHistory: [],
    loading: false,
    error: null,
  }),
}));

describe('MonitoringPage', () => {
  it('renders the page heading and the metric cards', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Monitoring/i })).toBeTruthy();
    // "HTTP throughput" appears in the subtitle and the card title.
    expect(screen.getAllByText(/HTTP throughput/i).length).toBeGreaterThan(0);
    expect(screen.getByText('System')).toBeTruthy();
    expect(screen.getByText(/Write-Ahead Log/i)).toBeTruthy();
    // "Query Cache" appears in the page subtitle and as the card title.
    expect(screen.getAllByText(/Query Cache/i).length).toBeGreaterThan(0);
  });

  it('renders the System card with live resource metrics from /metrics/runtime', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    expect(screen.getByText(/^CPU$/i)).toBeTruthy();
    expect(screen.getByText(/Memory \(RSS\)/i)).toBeTruthy();
    expect(screen.getByText(/Uptime/i)).toBeTruthy();
    expect(screen.getByText(/Active connections/i)).toBeTruthy();
  });
});
