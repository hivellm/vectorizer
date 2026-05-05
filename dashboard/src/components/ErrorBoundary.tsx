/**
 * Error Boundary — console design.
 *
 * Catches uncaught render-tree errors and shows a fallback panel
 * styled with the console palette (no Tailwind utilities).
 */

import { Component, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div
          style={{
            minHeight: '100vh',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'var(--bg-1)',
            padding: 16,
          }}
        >
          <div className="card" style={{ width: '100%', maxWidth: 480 }}>
            <div
              className="card-head"
              style={{ display: 'flex', alignItems: 'center', gap: 12 }}
            >
              <div style={{ flexShrink: 0, color: 'var(--red)' }} aria-hidden>
                <svg
                  width={28}
                  height={28}
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
              </div>
              <div className="title">Something went wrong</div>
            </div>
            <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
              <p style={{ margin: 0, color: 'var(--text-2)', fontSize: 13, lineHeight: 1.5 }}>
                {this.state.error?.message || 'An unexpected error occurred'}
              </p>
              <button
                type="button"
                className="btn primary"
                style={{ width: '100%', justifyContent: 'center' }}
                onClick={() => {
                  this.setState({ hasError: false, error: null });
                  window.location.reload();
                }}
              >
                Reload Page
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
