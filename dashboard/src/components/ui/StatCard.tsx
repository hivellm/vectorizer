/**
 * Stat Card component — console design language.
 *
 * Public API preserved: `{ title, value, subtitle, trend, icon }`. The
 * markup uses the same styling tokens as the console `Kpi` primitive
 * (`var(--panel)`, `var(--border)`, `var(--text)`, `var(--text-2)`).
 */

import type { ReactNode } from 'react';

interface StatCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  icon?: ReactNode;
}

function StatCard({ title, value, subtitle, trend, icon }: StatCardProps) {
  return (
    <div
      style={{
        background: 'var(--panel)',
        border: '1px solid var(--border)',
        borderRadius: 'var(--radius)',
        padding: 16,
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'space-between',
      }}
    >
      <div style={{ flex: 1 }}>
        <div
          style={{
            fontSize: 11,
            fontWeight: 500,
            color: 'var(--text-2)',
            letterSpacing: '0.02em',
            textTransform: 'uppercase',
          }}
        >
          {title}
        </div>
        <div
          style={{
            fontSize: 24,
            fontWeight: 600,
            color: 'var(--text)',
            marginTop: 8,
            fontFeatureSettings: '"tnum" 1',
          }}
        >
          {typeof value === 'number' ? value.toLocaleString() : value}
        </div>
        {subtitle && (
          <div style={{ fontSize: 13, color: 'var(--text-2)', marginTop: 4 }}>{subtitle}</div>
        )}
        {trend && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 4,
              marginTop: 8,
              fontSize: 13,
              color: trend.isPositive ? 'var(--teal)' : 'var(--red)',
            }}
          >
            <span>{trend.isPositive ? '↑' : '↓'}</span>
            <span>{Math.abs(trend.value)}%</span>
          </div>
        )}
      </div>
      {icon && (
        <div style={{ flexShrink: 0, color: 'var(--text-3)' }}>{icon}</div>
      )}
    </div>
  );
}

export default StatCard;
