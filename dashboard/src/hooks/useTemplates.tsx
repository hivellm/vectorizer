/**
 * Hook for fetching and managing configuration templates
 */

import { useState, useEffect, useCallback } from 'react';
import { Box, Code01, BookOpen01, Settings02 } from '@untitledui/icons';

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
 * Get template icon component based on template ID
 */
export function getTemplateIcon(templateId: string): React.ReactElement {
  const iconClass = "w-6 h-6";
  switch (templateId) {
    case 'rag':
      return <Box className={ iconClass } />;
    case 'code_search':
      return <Code01 className={ iconClass } />;
    case 'documentation':
      return <BookOpen01 className={ iconClass } />;
    case 'custom':
    default:
      return <Settings02 className={ iconClass } />;
  }
}

/**
 * Get template color based on ID - using neutral color palette
 */
export function getTemplateColor(templateId: string): string {
  const colors: Record<string, string> = {
    rag: 'bg-neutral-700/50 text-neutral-300',
    code_search: 'bg-neutral-700/50 text-neutral-300',
    documentation: 'bg-neutral-700/50 text-neutral-300',
    custom: 'bg-neutral-800/50 text-neutral-400',
  };
  return colors[templateId] || colors.custom;
}

export default useTemplates;
