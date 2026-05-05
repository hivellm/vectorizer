/**
 * Protected Route Component — console design.
 *
 * Wraps routes that require authentication. Redirects to login if
 * the user isn't authenticated. Loading state and admin-denied
 * fallback are styled with the console palette (no Tailwind).
 */

import { ReactNode } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
import LoadingSpinner from '@/components/LoadingSpinner';

interface ProtectedRouteProps {
  children: ReactNode;
  /** Require admin role */
  requireAdmin?: boolean;
}

function ProtectedRoute({ children, requireAdmin = false }: ProtectedRouteProps) {
  const { isAuthenticated, isLoading, user } = useAuth();
  const location = useLocation();

  // Show loading while checking auth status
  if (isLoading) {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '100vh',
          background: 'var(--bg-1)',
        }}
      >
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  // Redirect to login if not authenticated
  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // Check admin requirement
  if (requireAdmin && user && !user.roles.includes('Admin')) {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '100vh',
          background: 'var(--bg-1)',
          padding: 16,
        }}
      >
        <div style={{ textAlign: 'center' }}>
          <h2
            style={{
              fontSize: 22,
              fontWeight: 600,
              color: 'var(--text)',
              margin: '0 0 12px',
              letterSpacing: '-0.01em',
            }}
          >
            Access Denied
          </h2>
          <p style={{ color: 'var(--text-2)', fontSize: 13, margin: 0 }}>
            You need admin privileges to access this page.
          </p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}

export default ProtectedRoute;
