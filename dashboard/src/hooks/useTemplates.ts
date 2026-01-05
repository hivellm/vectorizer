/**
 * Hook for fetching and managing configuration templates
 */

import { useState, useEffect, useCallback } from 'react';

export interface CollectionSettings {
  chunk_size: number;
  chunk_overlap: number;
  embedding_model: string;
}

export interface TemplateCollection {
  name_suffix: string;
  description: string;
  include_patterns: string[];
  exclude_patterns: string[];
  content_type: string;
  settings: CollectionSettings;
}

export interface ConfigTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  use_cases: string[];
  collections: TemplateCollection[];
}

/**
 * Hook for fetching all available templates
 */
export function useTemplates() {
  const [templates, setTemplates] = useState<ConfigTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTemplates = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await fetch('/setup/templates');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data: ConfigTemplate[] = await response.json();
      setTemplates(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch templates');
      console.error('Failed to fetch templates:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTemplates();
  }, [fetchTemplates]);

  return {
    templates,
    loading,
    error,
    refetch: fetchTemplates,
  };
}

/**
 * Hook for fetching a specific template by ID
 */
export function useTemplate(templateId: string | null) {
  const [template, setTemplate] = useState<ConfigTemplate | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTemplate = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await fetch(`/setup/templates/${id}`);
      if (!response.ok) {
        if (response.status === 404) {
          throw new Error(`Template '${id}' not found`);
        }
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data: ConfigTemplate = await response.json();
      setTemplate(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch template');
      console.error('Failed to fetch template:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (templateId) {
      fetchTemplate(templateId);
    } else {
      setTemplate(null);
    }
  }, [templateId, fetchTemplate]);

  return {
    template,
    loading,
    error,
    refetch: templateId ? () => fetchTemplate(templateId) : undefined,
  };
}

/**
 * Get template icon component or emoji
 */
export function getTemplateIcon(template: ConfigTemplate): string {
  return template.icon || '📦';
}

/**
 * Get template color based on ID
 */
export function getTemplateColor(templateId: string): string {
  const colors: Record<string, string> = {
    rag: 'bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400',
    code_search: 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400',
    documentation: 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400',
    custom: 'bg-gray-100 dark:bg-gray-900/30 text-gray-600 dark:text-gray-400',
  };
  return colors[templateId] || colors.custom;
}

export default useTemplates;
