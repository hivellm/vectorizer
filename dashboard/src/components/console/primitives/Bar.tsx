type Tone = 'teal' | 'magenta' | 'amber';

interface BarProps {
  percent: number;
  tone?: Tone;
}

export function Bar({ percent, tone = 'teal' }: BarProps) {
  const cls = tone === 'teal' ? '' : tone;
  const pct = Math.max(0, Math.min(100, percent));
  return (
    <div className="bar">
      <span className={cls} style={{ width: `${pct}%` }} />
    </div>
  );
}
