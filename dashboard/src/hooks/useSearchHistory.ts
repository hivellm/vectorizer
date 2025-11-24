/**
 * Hook for managing search history
 */

import { useState, useEffect, useCallback } from 'react';

export interface SearchHistoryItem {
  id: string;
  collection: string;
  query: string;
  type: 'text' | 'vector' | 'hybrid';
  limit: number;
  timestamp: number;
  resultCount?: number;
}

const STORAGE_KEY = 'vectorizer_search_history';
const MAX_HISTORY_ITEMS = 20;

export function useSearchHistory() {
  const [history, setHistory] = useState<SearchHistoryItem[]>([]);

  // Load history from localStorage on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored) as SearchHistoryItem[];
        setHistory(Array.isArray(parsed) ? parsed : []);
      }
    } catch (_error) {
      // Ignore localStorage errors (e.g., in private browsing mode)
    }
  }, []);

  // Save history to localStorage whenever it changes
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
    } catch (error) {
      console.error('Error saving search history:', error);
    }
  }, [history]);

  const addToHistory = useCallback((item: Omit<SearchHistoryItem, 'id' | 'timestamp'>) => {
    const newItem: SearchHistoryItem = {
      ...item,
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      timestamp: Date.now(),
    };

    setHistory((prev) => {
      // Remove duplicates (same collection + query + type)
      const filtered = prev.filter(
        (h) =>
          !(
            h.collection === newItem.collection &&
            h.query === newItem.query &&
            h.type === newItem.type
          )
      );
      
      // Add new item at the beginning
      const updated = [newItem, ...filtered];
      
      // Keep only MAX_HISTORY_ITEMS
      return updated.slice(0, MAX_HISTORY_ITEMS);
    });
  }, []);

  const updateHistoryItem = useCallback((id: string, updates: Partial<SearchHistoryItem>) => {
    setHistory((prev) =>
      prev.map((item) => (item.id === id ? { ...item, ...updates } : item))
    );
  }, []);

  const removeFromHistory = useCallback((id: string) => {
    setHistory((prev) => prev.filter((item) => item.id !== id));
  }, []);

  const clearHistory = useCallback(() => {
    setHistory([]);
  }, []);

  return {
    history,
    addToHistory,
    updateHistoryItem,
    removeFromHistory,
    clearHistory,
  };
}

