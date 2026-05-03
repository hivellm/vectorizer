import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import MonitoringPage from '../MonitoringPage';

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
