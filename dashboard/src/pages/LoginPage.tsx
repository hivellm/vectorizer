/**
 * Login Page Component
 *
 * Console-styled sign-in for the dashboard. Rendered standalone outside
 * `ConsoleLayout`, so it activates `body[data-console="1"]` itself to pick
 * up the dark theme variables defined in `console.css`.
 */

import { useEffect, useState, type FormEvent } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
import { HexLogo, Card, CardBody, Pill, Icons } from '@/components/console';
import Checkbox from '@/components/ui/Checkbox';

function LoginPage() {
  const { login, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const from = (location.state as { from?: { pathname?: string } } | null)?.from?.pathname ?? '/overview';

  const [username, setUsername] = useState(
    () => localStorage.getItem('vectorizer_remembered_username') || ''
  );
  const [password, setPassword] = useState('');
  const [rememberMe, setRememberMe] = useState(
    () => localStorage.getItem('vectorizer_remember_me') === 'true'
  );
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isRateLimited, setIsRateLimited] = useState(false);
  const [sessionExpired, setSessionExpired] = useState<boolean>(() => {
    // Phase24 §4.2 — surface a "session expired" notice when the
    // api-middleware's unauthorizedMiddleware redirected us here. The
    // page sits outside ConsoleLayout's ToastProvider so we render an
    // inline Pill instead of a toast.
    if (typeof window === 'undefined') return false;
    const flag = window.sessionStorage.getItem('vectorizer_session_expired');
    if (flag === '1') {
      window.sessionStorage.removeItem('vectorizer_session_expired');
      return true;
    }
    return false;
  });

  // Activate console body styling — login is rendered OUTSIDE ConsoleLayout.
  useEffect(() => {
    document.body.dataset.console = '1';
    return () => {
      delete document.body.dataset.console;
    };
  }, []);

  // If already authenticated (e.g. from a stale session), bounce to the redirect target.
  useEffect(() => {
    if (isAuthenticated) navigate(from, { replace: true });
  }, [isAuthenticated, from, navigate]);

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setIsRateLimited(false);
    setSessionExpired(false);
    setSubmitting(true);
    try {
      await login(username, password);
      if (rememberMe) {
        localStorage.setItem('vectorizer_remember_me', 'true');
        localStorage.setItem('vectorizer_remembered_username', username);
      } else {
        localStorage.removeItem('vectorizer_remember_me');
        localStorage.removeItem('vectorizer_remembered_username');
      }
      navigate(from, { replace: true });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign-in failed';
      const lower = message.toLowerCase();
      if (lower.includes('too many') || lower.includes('try again')) {
        setIsRateLimited(true);
      }
      setError(message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'grid',
        placeItems: 'center',
        background: 'var(--bg)',
        padding: 24,
      }}
    >
      <div style={{ width: '100%', maxWidth: 400 }}>
        <Card>
          <CardBody>
            <div
              style={{
                display: 'grid',
                placeItems: 'center',
                textAlign: 'center',
                marginBottom: 18,
              }}
            >
              <HexLogo size={48} />
              <div
                style={{
                  marginTop: 12,
                  fontSize: 20,
                  fontWeight: 600,
                  letterSpacing: '-0.02em',
                }}
              >
                Vectorizer
              </div>
              <div className="muted" style={{ fontSize: 12, marginTop: 4 }}>
                Console · sign in to continue
              </div>
            </div>

            <form
              onSubmit={onSubmit}
              style={{ display: 'flex', flexDirection: 'column', gap: 12 }}
            >
              <div className="field">
                <label className="field-label" htmlFor="login-username">
                  Username
                </label>
                <input
                  id="login-username"
                  name="username"
                  className="input"
                  autoComplete="username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                  autoFocus
                />
              </div>
              <div className="field">
                <label className="field-label" htmlFor="login-password">
                  Password
                </label>
                <input
                  id="login-password"
                  name="password"
                  className="input"
                  type="password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                />
              </div>

              <div style={{ marginTop: 2 }}>
                <Checkbox
                  id="remember-me"
                  checked={rememberMe}
                  onChange={setRememberMe}
                  label="Remember me"
                />
              </div>

              {sessionExpired && !error && (
                <div role="status" style={{ marginTop: 4 }}>
                  <Pill tone="amber">
                    Your session expired — please sign in again.
                  </Pill>
                </div>
              )}

              {error && (
                <div role="alert" style={{ marginTop: 4 }}>
                  <Pill tone={isRateLimited ? 'amber' : 'red'}>{error}</Pill>
                </div>
              )}

              <button
                type="submit"
                className="btn primary"
                disabled={submitting}
                style={{ justifyContent: 'center', marginTop: 6 }}
              >
                {submitting ? (
                  <>
                    <Icons.refresh size={13} />
                    Signing in…
                  </>
                ) : (
                  'Sign in'
                )}
              </button>
            </form>
          </CardBody>
        </Card>

        <div
          className="muted"
          style={{ textAlign: 'center', fontSize: 11, marginTop: 16 }}
        >
          Vectorizer · High-performance vector database
        </div>
      </div>
    </div>
  );
}

export default LoginPage;
