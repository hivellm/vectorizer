/**
 * Inline SVG sparkline. Renders a polyline across the supplied numeric
 * series and a tooltip-quality readout of the most recent bucket.
 *
 * Intentionally dependency-free — no chart library — so it stays cheap
 * to render inside table rows / side panels.
 */

import type { CSSProperties } from 'react';

export interface SparklineDatum {
  /** ISO-8601 date string, e.g. "2026-05-03". */
  date: string;
  /** Bucket value. */
  count: number;
}

export interface SparklineProps {
  data: SparklineDatum[];
  width?: number;
  height?: number;
  strokeColor?: string;
  fillColor?: string;
  className?: string;
  style?: CSSProperties;
  ariaLabel?: string;
}

export default function Sparkline({
  data,
  width = 120,
  height = 32,
  strokeColor = 'currentColor',
  fillColor = 'rgba(99,102,241,0.12)',
  className,
  style,
  ariaLabel = 'Usage sparkline',
}: SparklineProps) {
  if (data.length === 0) {
    return (
      <span
        className={className}
        style={{ display: 'inline-block', width, height, ...style }}
        aria-label={`${ariaLabel} (no data)`}
        role="img"
      />
    );
  }

  const max = Math.max(1, ...data.map((d) => d.count));
  const stepX = data.length > 1 ? width / (data.length - 1) : 0;
  const points = data
    .map((d, i) => {
      const x = i * stepX;
      const y = height - (d.count / max) * height;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(' ');

  const areaPath = `M0,${height} L ${points
    .split(' ')
    .map((p) => p.replace(',', ' '))
    .join(' L ')} L ${width},${height} Z`;

  return (
    <svg
      className={className}
      style={style}
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label={ariaLabel}
    >
      <path d={areaPath} fill={fillColor} stroke="none" />
      <polyline points={points} fill="none" stroke={strokeColor} strokeWidth={1.5} />
    </svg>
  );
}
