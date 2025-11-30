/**
 * Environment configuration
 * Loads environment variables with defaults
 */

export const env = {
    /** API base URL - defaults to localhost:15002 in dev, same origin in prod */
    apiBaseUrl: import.meta.env.VITE_API_BASE_URL ||
        (import.meta.env.DEV 
            ? 'http://localhost:15002'
            : (typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:15002')),

    /** API version */
    apiVersion: import.meta.env.VITE_API_VERSION || 'v1',

    /** Development mode */
    isDev: import.meta.env.DEV,

    /** Production mode */
    isProd: import.meta.env.PROD,
} as const;

/**
 * Get full API URL for a given endpoint
 */
export function getApiUrl(endpoint: string): string {
    const base = env.apiBaseUrl.replace(/\/$/, '');
    const path = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
    // Server routes are directly at root level (e.g., /collections, /health)
    // No /api/v1 prefix needed
    return `${base}${path}`;
}

