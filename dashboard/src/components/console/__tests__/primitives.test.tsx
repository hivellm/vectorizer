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

  it('Sparkline emits role=img with aria-label', () => {
    const { container } = render(<Sparkline data={[1, 2, 3]} ariaLabel="qps last 60s" />);
    const svg = container.querySelector('svg')!;
    expect(svg.getAttribute('role')).toBe('img');
    expect(svg.getAttribute('aria-label')).toBe('qps last 60s');
  });

  it('Sparkline omits polygon when fill=false', () => {
    const { container } = render(<Sparkline data={[1, 2]} fill={false} />);
    expect(container.querySelector('polygon')).toBeNull();
    expect(container.querySelector('polyline')).toBeTruthy();
  });

  it('Ring exposes progressbar semantics with value/max/label', () => {
    const { container } = render(<Ring value={42} max={100} label="42%" sub="CPU" />);
    const wrapper = container.querySelector('[role="progressbar"]')!;
    expect(wrapper.getAttribute('aria-valuenow')).toBe('42');
    expect(wrapper.getAttribute('aria-valuemax')).toBe('100');
    expect(wrapper.getAttribute('aria-label')).toBe('CPU');
  });

  it('Ring clamps overflowing value to 100% of arc', () => {
    const { container } = render(<Ring value={200} max={100} label="OVR" />);
    // strokeDashoffset should be 0 when pct === 1 (clamped)
    const arc = container.querySelectorAll('circle')[1] as SVGCircleElement;
    expect(parseFloat(arc.getAttribute('stroke-dashoffset')!)).toBeCloseTo(0, 5);
  });

  it('Bar exposes progressbar semantics', () => {
    const { container } = render(<Bar percent={73} ariaLabel="MAP score" />);
    const bar = container.querySelector('[role="progressbar"]')!;
    expect(bar.getAttribute('aria-valuenow')).toBe('73');
    expect(bar.getAttribute('aria-label')).toBe('MAP score');
  });

  it('Card emits .card / .card-head / .card-body classes', () => {
    const { container } = render(
      <Card>
        <CardHead title="t" sub="s" />
        <CardBody tight>x</CardBody>
      </Card>,
    );
    expect(container.querySelector('.card')).toBeTruthy();
    expect(container.querySelector('.card-head .title')?.textContent).toBe('t');
    expect(container.querySelector('.card-head .sub')?.textContent).toBe('s');
    expect(container.querySelector('.card-body.tight')).toBeTruthy();
  });

  it('Tbl merges consumer-supplied className with .tbl', () => {
    const { container } = render(
      <Tbl className="my-table">
        <tbody><tr><Td>x</Td></tr></tbody>
      </Tbl>,
    );
    const table = container.querySelector('table')!;
    expect(table.className).toContain('tbl');
    expect(table.className).toContain('my-table');
  });
});
