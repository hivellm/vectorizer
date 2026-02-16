/**
 * Hook for auto-redirecting to setup wizard when initial setup is needed
 */

import { useEffect, useState, useMemo, useRef } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';

interface SetupStatus {
  needs_setup: boolean;
  version: string;
  deployment_type: string;
  has_workspace_config: boolean;
  project_count: number;
  collection_count: number;
}

// Default excluded paths - defined outside component to prevent recreating
const DEFAULT_EXCLUDE_PATHS = ['/setup', '/login'];

/**
 * Hook that checks if setup is needed and redirects to the setup wizard
 * 
 * @param options Configuration options
 * @param options.enabled Whether the redirect is enabled (default: true)
 * @param options.excludePaths Paths to exclude from redirect check
 * @returns Setup status and loading state
 */
export function useSetupRedirect(options?: {
  enabled?: boolean;
  excludePaths?: string[];
}) {
  const navigate = useNavigate();
  const location = useLocation();
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Track if we've already checked and redirected to prevent loops
  const hasChecked = useRef(false);

  const enabled = options?.enabled ?? true;
  // Memoize excludePaths to prevent dependency changes
  const excludePaths = useMemo(
    () => options?.excludePaths ?? DEFAULT_EXCLUDE_PATHS,
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [JSON.stringify(options?.excludePaths)]
  );

  useEffect(() => {
    if (!enabled || hasChecked.current) {
      setLoading(false);
      return;
    }

    let isMounted = true;

    const checkSetup = async () => {
      try {
        const response = await fetch('/setup/status');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data: SetupStatus = await response.json();

        if (!isMounted) return;

        setStatus(data);
        hasChecked.current = true;

        // Check if current path should be excluded from redirect
        const isExcludedPath = excludePaths.some(path =>
          location.pathname.startsWith(path)
        );

        // Redirect to setup if needed and not on an excluded path
        if (data.needs_setup && !isExcludedPath) {
          navigate('/setup', { replace: true });
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err.message : 'Failed to check setup status');
          console.error('Failed to check setup status:', err);
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    };

    checkSetup();

    return () => {
      isMounted = false;
    };
  }, [navigate, location.pathname, enabled, excludePaths]);

  return {
    status,
    loading,
    error,
    needsSetup: status?.needs_setup ?? false,
  };
}

/**
 * Hook for checking setup status without auto-redirect
 * 
 * @returns Setup status and loading state
 */
export function useSetupStatus() {
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Prevent multiple fetches
  const hasChecked = useRef(false);

  useEffect(() => {
    if (hasChecked.current) {
      return;
    }

    let isMounted = true;
    hasChecked.current = true;

    const checkSetup = async () => {
      try {
        const response = await fetch('/setup/status');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data: SetupStatus = await response.json();
        if (isMounted) {
          setStatus(data);
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err.message : 'Failed to check setup status');
          console.error('Failed to check setup status:', err);
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    };

    checkSetup();

    return () => {
      isMounted = false;
    };
  }, []);

  return {
    status,
    loading,
    error,
    needsSetup: status?.needs_setup ?? false,
  };
}

export default useSetupRedirect;
