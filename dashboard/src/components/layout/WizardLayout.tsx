/**
 * Wizard Layout — console-themed shell for the first-run setup flow.
 *
 * Activates `body[data-console="1"]` on mount so console.css variables
 * resolve correctly even though the wizard is rendered outside the
 * main `ConsoleLayout`.
 */

import { useEffect, type ReactNode } from 'react';
import { ToastProvider } from '@/providers/ToastProvider';
import { HexLogo } from '@/components/console';

interface WizardLayoutProps {
  children: ReactNode;
}

function WizardLayout({ children }: WizardLayoutProps) {
  useEffect(() => {
    document.body.dataset.console = '1';
    return () => {
      delete document.body.dataset.console;
    };
  }, []);

  return (
    <ToastProvider>
      <div
        style={{
          minHeight: '100vh',
          background: 'var(--bg)',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <header
          style={{
            height: 52,
            borderBottom: '1px solid var(--border)',
            background: 'var(--bg-1)',
            display: 'flex',
            alignItems: 'center',
            padding: '0 24px',
            gap: 12,
          }}
        >
          <HexLogo size={28} />
          <div style={{ fontSize: 14, fontWeight: 600, letterSpacing: '-0.01em' }}>
            Vectorizer Setup
          </div>
          <div className="muted" style={{ fontSize: 12, marginLeft: 8 }}>
            · first-run wizard
          </div>
        </header>
        <main style={{ flex: 1, overflowY: 'auto' }}>
          {children}
        </main>
      </div>
    </ToastProvider>
  );
}

export default WizardLayout;
