/**
 * Authentication Context for Dashboard
 *
 * Provides authentication state and methods throughout the application.
 * Supports JWT token-based authentication with automatic session refresh.
 *
 * IMPORTANT: Authentication is only required when server has auth.enabled: true
 * The context checks a protected endpoint to determine if auth is required.
 */

import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';

// User information from the server
export interface User {
  user_id: string;
  username: string;
  roles: string[];
}

// Login response from the server
interface LoginResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

// Auth context state
interface AuthContextType {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  authRequired: boolean; // Whether the server requires authentication
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  verifySession: () => Promise<boolean>;
}

// Create the context
const AuthContext = createContext<AuthContextType | undefined>(undefined);

// Storage keys
const TOKEN_KEY = 'vectorizer_dashboard_token';
const USER_KEY = 'vectorizer_dashboard_user';
const AUTH_REQUIRED_KEY = 'vectorizer_auth_required';

// Get API base URL - in production, use relative path
const getApiBaseUrl = () => {
  // In development, we might be on a different port
  if (import.meta.env.DEV) {
    return 'http://localhost:15002';
  }
  // In production, use relative path (same origin)
  return '';
};

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [authRequired, setAuthRequired] = useState<boolean>(() => {
    // Default to cached value or false
    const cached = localStorage.getItem(AUTH_REQUIRED_KEY);
    return cached === 'true';
  });

  // Check if server requires authentication
  const checkAuthRequired = useCallback(async (): Promise<boolean> => {
    const baseUrl = getApiBaseUrl();

    try {
      // Try to access a protected endpoint without auth
      // If we get 401, auth is required. If we get 200/other, auth is not required.
      const response = await fetch(`${baseUrl}/collections`, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
      });

      // If we get 401 Unauthorized, auth is required
      const required = response.status === 401;

      setAuthRequired(required);
      localStorage.setItem(AUTH_REQUIRED_KEY, String(required));

      return required;
    } catch {
      // On network error, assume auth might be required (safer default)
      return true;
    }
  }, []);

  // Load stored auth on mount and check if auth is required
  useEffect(() => {
    const initialize = async () => {
      // Check if auth is required
      const required = await checkAuthRequired();

      if (!required) {
        // Auth not required, skip login
        setIsLoading(false);
        return;
      }

      // Auth is required, load stored credentials
      const storedToken = localStorage.getItem(TOKEN_KEY);
      const storedUser = localStorage.getItem(USER_KEY);

      if (storedToken && storedUser) {
        try {
          setToken(storedToken);
          setUser(JSON.parse(storedUser));
        } catch {
          // Clear invalid data
          localStorage.removeItem(TOKEN_KEY);
          localStorage.removeItem(USER_KEY);
        }
      }
      setIsLoading(false);
    };

    initialize();
  }, [checkAuthRequired]);

  // Login function
  const login = useCallback(async (username: string, password: string) => {
    const baseUrl = getApiBaseUrl();

    const response = await fetch(`${baseUrl}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Login failed' }));
      throw new Error(error.message || 'Invalid username or password');
    }

    const data: LoginResponse = await response.json();

    // Store auth data
    localStorage.setItem(TOKEN_KEY, data.access_token);
    localStorage.setItem(USER_KEY, JSON.stringify(data.user));

    setToken(data.access_token);
    setUser(data.user);
  }, []);

  // Logout function
  const logout = useCallback(async () => {
    // Clear local storage
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);

    // Clear state
    setToken(null);
    setUser(null);
  }, []);

  // Verify current session
  const verifySession = useCallback(async (): Promise<boolean> => {
    if (!token) {
      return false;
    }

    const baseUrl = getApiBaseUrl();

    try {
      const response = await fetch(`${baseUrl}/auth/me`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        // Session invalid, clear auth
        await logout();
        return false;
      }

      // Session valid, update user info
      const userData = await response.json();
      setUser(userData);
      localStorage.setItem(USER_KEY, JSON.stringify(userData));

      return true;
    } catch {
      // Network error or other issue
      return false;
    }
  }, [token, logout]);

  // Auto-verify session on mount if we have a token
  useEffect(() => {
    if (token && !isLoading && authRequired) {
      verifySession();
    }
  }, [token, isLoading, authRequired, verifySession]);

  const value: AuthContextType = {
    user,
    token,
    // If auth is not required, consider user as authenticated
    isAuthenticated: !authRequired || (!!token && !!user),
    isLoading,
    authRequired,
    login,
    logout,
    verifySession,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}

// Hook to use auth context
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

export default AuthContext;
