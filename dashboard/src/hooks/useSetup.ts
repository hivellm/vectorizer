/**
 * Setup Wizard Hook
 * Manages setup wizard API interactions
 */

import { useApiClient } from './useApiClient';

export interface SetupStatus {
  needs_setup: boolean;
  version: string;
  deployment_type: 'binary' | 'docker';
  has_workspace_config: boolean;
  project_count: number;
  collection_count: number;
}

export interface SuggestedCollection {
  name: string;
  description: string;
  include_patterns: string[];
  exclude_patterns: string[];
  content_type: string;
  estimated_file_count: number;
}

export interface DirectoryStats {
  total_files: number;
  total_directories: number;
  total_size_bytes: number;
  files_by_extension: Record<string, number>;
  has_git: boolean;
  has_docs: boolean;
}

export interface ProjectAnalysis {
  project_types: string[];
  languages: string[];
  frameworks: string[];
  suggested_collections: SuggestedCollection[];
  statistics: DirectoryStats;
  project_name: string;
  project_path: string;
}

export interface SetupProject {
  name: string;
  path: string;
  description: string;
  collections: Array<{
    name: string;
    description: string;
    include_patterns: string[];
    exclude_patterns: string[];
  }>;
}

export interface ApplyConfigRequest {
  projects: SetupProject[];
  global_settings?: {
    file_watcher?: {
      auto_discovery?: boolean;
      enable_auto_update?: boolean;
      hot_reload?: boolean;
      watch_paths?: string[];
    };
  };
}

export interface VerifyResult {
  setup_complete: boolean;
  health: {
    status: 'healthy' | 'unhealthy';
    version: string;
  };
  workspace: {
    valid: boolean;
    project_count?: number;
    error?: string;
  };
  collections: {
    count: number;
  };
  next_steps: string[];
}

export function useSetup() {
  const api = useApiClient();

  const getStatus = async (): Promise<SetupStatus> => {
    try {
      return await api.get<SetupStatus>('/setup/status');
    } catch (error) {
      console.error('Error fetching setup status:', error);
      throw error;
    }
  };

  const analyzeDirectory = async (path: string): Promise<ProjectAnalysis> => {
    try {
      return await api.post<ProjectAnalysis>('/setup/analyze', { path });
    } catch (error) {
      console.error('Error analyzing directory:', error);
      throw error;
    }
  };

  const applyConfig = async (config: ApplyConfigRequest): Promise<{ success: boolean; message: string }> => {
    try {
      return await api.post('/setup/apply', config);
    } catch (error) {
      console.error('Error applying setup config:', error);
      throw error;
    }
  };

  const verify = async (): Promise<VerifyResult> => {
    try {
      return await api.get<VerifyResult>('/setup/verify');
    } catch (error) {
      console.error('Error verifying setup:', error);
      throw error;
    }
  };

  return {
    getStatus,
    analyzeDirectory,
    applyConfig,
    verify,
  };
}
