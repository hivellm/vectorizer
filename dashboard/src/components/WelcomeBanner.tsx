/**
 * Welcome Banner — console design.
 *
 * Shows a welcome panel for first-time users that still need to
 * complete the setup wizard. Styled with the console palette and
 * primitives, no Tailwind utilities.
 */

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSetupStatus } from '@/hooks/useSetupRedirect';
import { Settings02, XClose, Rocket01 } from '@untitledui/icons';

interface WelcomeBannerProps {
  /** Whether the banner can be dismissed */
  dismissible?: boolean;
  /** Custom class name */
  className?: string;
}

function WelcomeBanner({ dismissible = true, className = '' }: WelcomeBannerProps) {
  const navigate = useNavigate();
  const { needsSetup, loading, status } = useSetupStatus();
  const [dismissed, setDismissed] = useState(false);

  // Don't show if loading, setup not needed, or dismissed
  if (loading || !needsSetup || dismissed) {
    return null;
  }

  return (
    <div
      className={['card', className].filter(Boolean).join(' ')}
      style={{
        position: 'relative',
        background: 'linear-gradient(135deg, var(--magenta), var(--teal))',
        color: 'var(--text)',
        border: '1px solid var(--border-hi)',
        padding: 24,
        overflow: 'hidden',
      }}
    >
      {/* Dismiss button */}
      {dismissible && (
        <button
          type="button"
          className="icon-btn"
          onClick={() => setDismissed(true)}
          aria-label="Dismiss banner"
          style={{
            position: 'absolute',
            top: 12,
            right: 12,
            background: 'rgba(255, 255, 255, 0.16)',
            borderColor: 'rgba(255, 255, 255, 0.28)',
            color: '#fff',
          }}
        >
          <XClose width={18} height={18} />
        </button>
      )}

      <div
        style={{
          display: 'flex',
          flexDirection: 'row',
          flexWrap: 'wrap',
          alignItems: 'center',
          gap: 16,
        }}
      >
        {/* Icon */}
        <div style={{ flexShrink: 0 }}>
          <div
            style={{
              width: 56,
              height: 56,
              background: 'rgba(255, 255, 255, 0.2)',
              borderRadius: 12,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: '#fff',
            }}
          >
            <Rocket01 width={32} height={32} />
          </div>
        </div>

        {/* Content */}
        <div style={{ flex: 1, minWidth: 240 }}>
          <h3
            style={{
              fontSize: 20,
              fontWeight: 600,
              color: '#fff',
              margin: '0 0 6px',
              letterSpacing: '-0.01em',
            }}
          >
            Welcome to Vectorizer!
          </h3>
          <p
            style={{
              color: 'rgba(255, 255, 255, 0.92)',
              fontSize: 13,
              lineHeight: 1.5,
              margin: '0 0 12px',
            }}
          >
            Get started by configuring your workspace. The Setup Wizard will help you
            detect your projects and create optimized collections automatically.
          </p>

          {/* Stats */}
          {status && (
            <div
              style={{
                display: 'flex',
                flexWrap: 'wrap',
                gap: 12,
                fontSize: 12,
                color: 'rgba(255, 255, 255, 0.85)',
                marginBottom: 12,
              }}
            >
              <span>
                Version:{' '}
                <strong style={{ color: '#fff' }}>{status.version}</strong>
              </span>
              <span aria-hidden>•</span>
              <span>
                Collections:{' '}
                <strong style={{ color: '#fff' }}>{status.collection_count}</strong>
              </span>
              <span aria-hidden>•</span>
              <span>
                Deployment:{' '}
                <strong style={{ color: '#fff', textTransform: 'capitalize' }}>
                  {status.deployment_type}
                </strong>
              </span>
            </div>
          )}
        </div>

        {/* CTA */}
        <div style={{ flexShrink: 0 }}>
          <button
            type="button"
            onClick={() => navigate('/setup')}
            className="btn"
            style={{
              background: '#fff',
              borderColor: '#fff',
              color: 'var(--bg-1)',
              gap: 8,
              padding: '10px 18px',
            }}
          >
            <Settings02 width={18} height={18} />
            Open Setup Wizard
          </button>
        </div>
      </div>

      {/* Quick tips */}
      <div
        style={{
          marginTop: 16,
          paddingTop: 16,
          borderTop: '1px solid rgba(255, 255, 255, 0.2)',
        }}
      >
        <p style={{ fontSize: 12, color: 'rgba(255, 255, 255, 0.85)', margin: 0 }}>
          <strong style={{ color: '#fff' }}>Quick tip:</strong> You can also run{' '}
          <code
            style={{
              background: 'rgba(255, 255, 255, 0.2)',
              padding: '2px 6px',
              borderRadius: 4,
              fontFamily: 'var(--font-mono)',
              fontSize: 11,
              color: '#fff',
            }}
          >
            vectorizer-cli setup /path/to/project
          </code>{' '}
          from the terminal.
        </p>
      </div>
    </div>
  );
}

export default WelcomeBanner;
