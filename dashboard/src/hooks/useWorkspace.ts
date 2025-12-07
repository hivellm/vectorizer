/**
 * Workspace hooks
 */

import { useApiClient } from './useApiClient';

export interface WorkspaceProject {
  name: string;
  path: string;
  description?: string;
  collections?: Array<{
    name: string;
    description?: string;
    include_patterns?: string[];
    exclude_patterns?: string[];
  }>;
}

export interface WorkspaceConfig {
  global_settings?: {
    file_watcher?: {
      watch_paths?: string[];
      auto_discovery?: boolean;
      enable_auto_update?: boolean;
      hot_reload?: boolean;
      exclude_patterns?: string[];
    };
  };
  projects?: WorkspaceProject[];
}

export function useWorkspace() {
  const api = useApiClient();

  const getConfig = async (): Promise<WorkspaceConfig> => {
    try {
      const config = await api.get<WorkspaceConfig>('/workspace/config');
      return config;
    } catch (error) {
      console.error('Error fetching workspace config:', error);
      throw error;
    }
  };

  const updateConfig = async (config: WorkspaceConfig): Promise<void> => {
    try {
      await api.post('/workspace/config', config);
    } catch (error) {
      console.error('Error updating workspace config:', error);
      throw error;
    }
  };

  const addWorkspace = async (path: string, collectionName: string): Promise<void> => {
    try {
      await api.post('/workspace/add', {
        path,
        collection_name: collectionName,
      });
    } catch (error) {
      console.error('Error adding workspace:', error);
      throw error;
    }
  };

  const removeWorkspace = async (path: string): Promise<void> => {
    try {
      await api.post('/workspace/remove', { path });
    } catch (error) {
      console.error('Error removing workspace:', error);
      throw error;
    }
  };

  const listWorkspaces = async (): Promise<any[]> => {
    try {
      const workspaces = await api.get<any[]>('/workspace/list');
      return Array.isArray(workspaces) ? workspaces : [];
    } catch (error) {
      console.error('Error listing workspaces:', error);
      return [];
    }
  };

  return {
    getConfig,
    updateConfig,
    addWorkspace,
    removeWorkspace,
    listWorkspaces,
  };
}

