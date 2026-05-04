/**
 * Loading spinner component — console design language.
 *
 * Uses the shared `.spinner` class defined in console.css.
 */

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const SIZE_PX: Record<'sm' | 'md' | 'lg', number> = {
  sm: 14,
  md: 20,
  lg: 32,
};

function LoadingSpinner({ size = 'md', className }: LoadingSpinnerProps) {
  const px = SIZE_PX[size];
  const cls = ['spinner', className].filter(Boolean).join(' ');
  return (
    <div
      className={cls}
      style={{
        width: px,
        height: px,
        borderWidth: size === 'sm' ? 1.5 : 2,
      }}
      aria-hidden
    />
  );
}

export default LoadingSpinner;
