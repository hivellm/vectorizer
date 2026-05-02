interface SparklineProps {
  data: number[];
  color?: string;
  width?: number;
  height?: number;
  fill?: boolean;
}

export function Sparkline({
  data,
  color = 'var(--teal)',
  width = 80,
  height = 28,
  fill = true,
}: SparklineProps) {
  if (!data.length) return null;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const denom = data.length === 1 ? 1 : data.length - 1;
  const points = data
    .map((d, i) => `${(i / denom) * width},${height - ((d - min) / range) * (height - 4) - 2}`)
    .join(' ');
  const area = `0,${height} ${points} ${width},${height}`;
  return (
    <svg width={width} height={height} style={{ display: 'block' }}>
      {fill && <polygon points={area} fill={color} opacity="0.12" />}
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
