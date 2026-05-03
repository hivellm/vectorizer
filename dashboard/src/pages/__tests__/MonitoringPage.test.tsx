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

describe('MonitoringPage', () => {
  it('renders the page heading and the four metric cards', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Monitoring/i })).toBeTruthy();
    expect(screen.getByText(/SIMD Backend/i)).toBeTruthy();
    expect(screen.getByText(/Write-Ahead Log/i)).toBeTruthy();
    // "Query Cache" appears in the page subtitle and as the card title;
    // assert at least one match (the card title is one of them).
    expect(screen.getAllByText(/Query Cache/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/File-ops Cache/i)).toBeTruthy();
  });

  it('renders the throughput strip with REST and MCP breakdown', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    expect(screen.getByText(/HTTP \/ MCP throughput/i)).toBeTruthy();
    expect(screen.getByText(/REST/i)).toBeTruthy();
    // "MCP" appears in the throughput card title and as the per-protocol label;
    // assert at least one match (the per-protocol label is one of them).
    expect(screen.getAllByText(/MCP/i).length).toBeGreaterThan(0);
  });
});
