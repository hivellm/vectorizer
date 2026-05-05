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
    // "SIMD Backend" appears in both the card title and the not-exposed
    // placeholder, so match on at least one occurrence.
    expect(screen.getAllByText(/SIMD Backend/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/Write-Ahead Log/i)).toBeTruthy();
    expect(screen.getAllByText(/Query Cache/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/File-ops Cache/i).length).toBeGreaterThan(0);
  });

  it('renders the throughput strip with the live total only', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    expect(screen.getByText(/HTTP \/ MCP throughput/i)).toBeTruthy();
    expect(screen.getByText(/^Total$/i)).toBeTruthy();
    // The page explains why the protocol split is missing — copy is stable.
    expect(
      screen.getByText(/REST\/MCP split, p99 latency and 5xx-rate are not yet exposed/i),
    ).toBeTruthy();
  });

  it('shows real /health-derived cache numbers without sparklines', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    // 4_210_000 hits formatted as "4,210,000".
    expect(screen.getByText('4,210,000')).toBeTruthy();
    expect(screen.getByText(/94\.2% hit rate/i)).toBeTruthy();
  });

  it('renders the SIMD-not-exposed placeholder', () => {
    render(<MemoryRouter><MonitoringPage /></MemoryRouter>);
    expect(
      screen.getByText(/SIMD backend introspection not yet exposed/i),
    ).toBeTruthy();
  });
});
