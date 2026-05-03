type Tone = 'teal' | 'magenta' | 'amber';

interface BarProps {
  percent: number;
  tone?: Tone;
  ariaLabel?: string;
}

export function Bar({ percent, tone = 'teal', ariaLabel }: BarProps) {
  // .bar > span is teal by default in console.css; .magenta / .amber override.
  const cls = tone === 'teal' ? '' : tone;
  const pct = Math.max(0, Math.min(100, percent));
  return (
    <div
      className="bar"
      role="progressbar"
      aria-valuenow={pct}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={ariaLabel}
    >
      <span className={cls} style={{ width: `${pct}%` }} />
    </div>
  );
}
