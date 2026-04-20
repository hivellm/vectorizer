/**
 * useSandboxHistory — persists the API-sandbox request history and
 * user-curated favorites in localStorage so reloading the docs page
 * keeps the workbench populated.
 *
 * History is capped at HISTORY_LIMIT entries and pruned LIFO; favorites
 * are unbounded but deduplicated by (method + path + body hash). Both
 * lists survive across sessions under the well-known keys below — they
 * are intentionally unversioned because the shape is append-only.
 */

import { useCallback, useEffect, useState } from 'react';

export interface SandboxRequestRecord {
  id: string;
  method: string;
  path: string;
  pathParams: Record<string, string>;
  body: string;
  status?: number;
  timingMs?: number;
  /** ISO-8601 timestamp. */
  ranAt: string;
}

export interface SandboxFavoriteRecord extends SandboxRequestRecord {
  label?: string;
}

const HISTORY_KEY = 'vectorizer.sandbox.history.v1';
const FAVORITES_KEY = 'vectorizer.sandbox.favorites.v1';
const HISTORY_LIMIT = 25;

function readJson<T>(key: string, fallback: T): T {
  if (typeof window === 'undefined') return fallback;
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as T;
    return parsed;
  } catch {
    return fallback;
  }
}

function writeJson(key: string, value: unknown): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Quota errors: silently drop — the sandbox is a convenience, not a guarantee.
  }
}

function fingerprint(method: string, path: string, body: string): string {
  return `${method.toUpperCase()} ${path} ${body.trim()}`;
}

export interface SandboxHistoryApi {
  history: SandboxRequestRecord[];
  favorites: SandboxFavoriteRecord[];
  /** Append the record to history; prunes to HISTORY_LIMIT. */
  recordRequest: (entry: Omit<SandboxRequestRecord, 'id' | 'ranAt'>) => void;
  /** Toggle favorite by id. If the request is in history but not favorites, it's added; otherwise removed. */
  toggleFavorite: (entry: Omit<SandboxFavoriteRecord, 'id' | 'ranAt'>) => void;
  /** Remove one history entry. */
  removeHistory: (id: string) => void;
  /** Remove one favorite by its fingerprint. */
  removeFavorite: (id: string) => void;
  /** Wipe history. Favorites are preserved. */
  clearHistory: () => void;
  /** Returns true if the (method, path, body) triple is currently starred. */
  isFavorited: (method: string, path: string, body: string) => boolean;
}

export function useSandboxHistory(): SandboxHistoryApi {
  const [history, setHistory] = useState<SandboxRequestRecord[]>(() =>
    readJson<SandboxRequestRecord[]>(HISTORY_KEY, [])
  );
  const [favorites, setFavorites] = useState<SandboxFavoriteRecord[]>(() =>
    readJson<SandboxFavoriteRecord[]>(FAVORITES_KEY, [])
  );

  useEffect(() => writeJson(HISTORY_KEY, history), [history]);
  useEffect(() => writeJson(FAVORITES_KEY, favorites), [favorites]);

  const recordRequest = useCallback<SandboxHistoryApi['recordRequest']>((entry) => {
    setHistory((prev) => {
      const record: SandboxRequestRecord = {
        ...entry,
        id: `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
        ranAt: new Date().toISOString(),
      };
      const next = [record, ...prev];
      if (next.length > HISTORY_LIMIT) next.length = HISTORY_LIMIT;
      return next;
    });
  }, []);

  const toggleFavorite = useCallback<SandboxHistoryApi['toggleFavorite']>((entry) => {
    const fp = fingerprint(entry.method, entry.path, entry.body);
    setFavorites((prev) => {
      const existing = prev.find((f) => fingerprint(f.method, f.path, f.body) === fp);
      if (existing) {
        return prev.filter((f) => f.id !== existing.id);
      }
      const record: SandboxFavoriteRecord = {
        ...entry,
        id: fp,
        ranAt: new Date().toISOString(),
      };
      return [record, ...prev];
    });
  }, []);

  const removeHistory = useCallback<SandboxHistoryApi['removeHistory']>((id) => {
    setHistory((prev) => prev.filter((h) => h.id !== id));
  }, []);

  const removeFavorite = useCallback<SandboxHistoryApi['removeFavorite']>((id) => {
    setFavorites((prev) => prev.filter((f) => f.id !== id));
  }, []);

  const clearHistory = useCallback<SandboxHistoryApi['clearHistory']>(() => {
    setHistory([]);
  }, []);

  const isFavorited = useCallback<SandboxHistoryApi['isFavorited']>(
    (method, path, body) => {
      const fp = fingerprint(method, path, body);
      return favorites.some((f) => fingerprint(f.method, f.path, f.body) === fp);
    },
    [favorites]
  );

  return {
    history,
    favorites,
    recordRequest,
    toggleFavorite,
    removeHistory,
    removeFavorite,
    clearHistory,
    isFavorited,
  };
}

// Re-exported for tests.
export const __SANDBOX_HISTORY_INTERNALS = {
  fingerprint,
  HISTORY_KEY,
  FAVORITES_KEY,
  HISTORY_LIMIT,
};
