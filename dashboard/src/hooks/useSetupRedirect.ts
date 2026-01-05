/**
 * Hook for auto-redirecting to setup wizard when initial setup is needed
 */

import { useEffect, useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';

interface SetupStatus {
  needs_setup: boolean;
  version: string;
  deployment_type: string;
  has_workspace_config: boolean;
  project_count: number;
  collection_count: number;
}

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

  const { enabled = true, excludePaths = ['/setup', '/login'] } = options || {};

  useEffect(() => {
    if (!enabled) {
      setLoading(false);
      return;
    }

    const checkSetup = async () => {
      try {
        const response = await fetch('/setup/status');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data: SetupStatus = await response.json();
        setStatus(data);

        // Check if current path should be excluded from redirect
        const isExcludedPath = excludePaths.some(path => 
          location.pathname.startsWith(path)
        );

        // Redirect to setup if needed and not on an excluded path
        if (data.needs_setup && !isExcludedPath) {
          navigate('/setup', { replace: true });
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to check setup status');
        console.error('Failed to check setup status:', err);
      } finally {
        setLoading(false);
      }
    };

    checkSetup();
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

  useEffect(() => {
    const checkSetup = async () => {
      try {
        const response = await fetch('/setup/status');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data: SetupStatus = await response.json();
        setStatus(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to check setup status');
        console.error('Failed to check setup status:', err);
      } finally {
        setLoading(false);
      }
    };

    checkSetup();
  }, []);

  return {
    status,
    loading,
    error,
    needsSetup: status?.needs_setup ?? false,
  };
}

export default useSetupRedirect;
