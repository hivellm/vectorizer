/**
 * Zustand store for collections state management
 */

import { create } from 'zustand';
import type { Collection } from '@/hooks/useCollections';

interface CollectionsState {
  collections: Collection[];
  loading: boolean;
  error: string | null;
  
  // Actions
  setCollections: (collections: Collection[]) => void;
  addCollection: (collection: Collection) => void;
  removeCollection: (name: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useCollectionsStore = create<CollectionsState>((set) => ({
  collections: [],
  loading: false,
  error: null,

  setCollections: (collections) => set({ collections }),
  
  addCollection: (collection) => set((state) => ({
    collections: [...state.collections, collection],
  })),
  
  removeCollection: (name) => set((state) => ({
    collections: state.collections.filter((c) => c.name !== name),
  })),
  
  setLoading: (loading) => set({ loading }),
  
  setError: (error) => set({ error }),
  
  clearError: () => set({ error: null }),
}));

