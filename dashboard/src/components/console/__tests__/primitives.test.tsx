import { render, screen, act } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import {
  HexLogo, Sparkline, Ring, StatusPill, Pill, Card, CardHead, CardBody,
  Kpi, Bar, Tbl, Th, Td, KeyValue, KeyValueRow, useTick,
} from '../';
import { useEffect } from 'react';

describe('console primitives', () => {
  it('HexLogo renders /logo.png', () => {
    render(<HexLogo size={32} />);
    const img = screen.getByAltText('Vectorizer');
    expect(img.getAttribute('src')).toBe('/logo.png');
    expect(img.getAttribute('width')).toBe('32');
  });

  it('Sparkline returns null on empty data', () => {
    const { container } = render(<Sparkline data={[]} />);
    expect(container.querySelector('svg')).toBeNull();
  });

  it('Sparkline draws a polyline for non-empty data', () => {
    const { container } = render(<Sparkline data={[1, 4, 2, 8, 5]} width={100} height={20} />);
    expect(container.querySelector('polyline')).toBeTruthy();
    expect(container.querySelector('polygon')).toBeTruthy();
  });

  it('Ring renders centered label and sub', () => {
    render(<Ring value={42} max={100} label="42%" sub="CPU" />);
    expect(screen.getByText('42%')).toBeTruthy();
    expect(screen.getByText('CPU')).toBeTruthy();
  });

  it('StatusPill maps healthy to green class', () => {
    const { container } = render(<StatusPill status="healthy" />);
    const pill = container.querySelector('.pill');
    expect(pill?.className).toContain('green');
    expect(pill?.textContent).toContain('healthy');
  });

  it('Pill applies tone class', () => {
    const { container } = render(<Pill tone="magenta">Admin</Pill>);
    expect(container.querySelector('.pill.magenta')).toBeTruthy();
  });

  it('Card composes head + body', () => {
    render(
      <Card>
        <CardHead title="Top Collections" />
        <CardBody>content</CardBody>
      </Card>,
    );
    expect(screen.getByText('Top Collections')).toBeTruthy();
    expect(screen.getByText('content')).toBeTruthy();
  });

  it('Kpi renders label, value and delta', () => {
    render(<Kpi label="qps" value="2,480" unit="qps" delta={{ tone: 'up', text: '+12.4%' }} />);
    expect(screen.getAllByText('qps').length).toBeGreaterThan(0);
    expect(screen.getByText('2,480')).toBeTruthy();
    expect(screen.getByText('+12.4%')).toBeTruthy();
  });

  it('Bar fills to percent', () => {
    const { container } = render(<Bar percent={73} />);
    const fill = container.querySelector('.bar > span') as HTMLElement;
    expect(fill.style.width).toBe('73%');
  });

  it('Tbl renders thead + tbody', () => {
    render(
      <Tbl>
        <thead><tr><Th>Name</Th></tr></thead>
        <tbody><tr><Td>foo</Td></tr></tbody>
      </Tbl>,
    );
    expect(screen.getByText('Name')).toBeTruthy();
    expect(screen.getByText('foo')).toBeTruthy();
  });

  it('KeyValue renders dt/dd pairs', () => {
    render(
      <KeyValue>
        <KeyValueRow term="Index">HNSW</KeyValueRow>
      </KeyValue>,
    );
    expect(screen.getByText('Index')).toBeTruthy();
    expect(screen.getByText('HNSW')).toBeTruthy();
  });

  it('useTick increments at the given interval', () => {
    vi.useFakeTimers();
    let observed = -1;
    const Probe = () => {
      const t = useTick(100);
      useEffect(() => { observed = t; }, [t]);
      return null;
    };
    render(<Probe />);
    expect(observed).toBe(0);
    act(() => { vi.advanceTimersByTime(350); });
    expect(observed).toBe(3);
    vi.useRealTimers();
  });
});
