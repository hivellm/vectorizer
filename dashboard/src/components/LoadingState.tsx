/**
 * Loading state component with message — console design language.
 */

interface LoadingStateProps {
  message?: string;
  /** Retained for back-compat; the console spinner has a fixed size. */
  size?: 'sm' | 'md' | 'lg';
}

function LoadingState({ message = 'Loading...' }: LoadingStateProps) {
  return (
    <div
      style={{
        display: 'grid',
        placeItems: 'center',
        minHeight: '60vh',
        gap: 12,
      }}
    >
      <div className="spinner" aria-hidden />
      {message && (
        <div className="muted" style={{ fontSize: 13 }}>
          {message}
        </div>
      )}
    </div>
  );
}

export default LoadingState;
