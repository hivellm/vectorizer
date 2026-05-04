import type { ReactNode } from 'react';

interface RingProps {
  value: number;
  max?: number;
  size?: number;
  label: ReactNode;
  sub?: ReactNode;
  color?: string;
  ariaLabel?: string;
}

export function Ring({
  value,
  max = 100,
  size = 110,
  label,
  sub,
  color = 'var(--teal)',
  ariaLabel,
}: RingProps) {
  const r = size / 2 - 8;
  const c = 2 * Math.PI * r;
  const pct = Math.min(value / max, 1);
  const computedLabel = ariaLabel ?? (typeof sub === 'string' ? sub : undefined);
  return (
    <div
      style={{ position: 'relative', width: size, height: size }}
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-label={computedLabel}
    >
      <svg width={size} height={size}>
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--bg-3)" strokeWidth="6" />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke={color}
          strokeWidth="6"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - pct)}
          strokeLinecap="round"
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          style={{ transition: 'stroke-dashoffset 0.6s' }}
        />
      </svg>
      <div style={{ position: 'absolute', inset: 0, display: 'grid', placeItems: 'center', textAlign: 'center' }}>
        <div>
          <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.02em', fontFeatureSettings: '"tnum" 1' }}>
            {label}
          </div>
          {sub && (
            <div style={{ fontSize: 10, color: 'var(--text-2)', textTransform: 'uppercase', letterSpacing: '0.06em', marginTop: 2 }}>
              {sub}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
