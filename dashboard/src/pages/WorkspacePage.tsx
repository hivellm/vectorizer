/**
 * Workspace page - Manage workspace configuration
 * Similar to GUI WorkspaceManager.vue
 */

import { useEffect, useState } from 'react';
import { useWorkspace, WorkspaceConfig, WorkspaceProject } from '@/hooks/useWorkspace';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import FileBrowser from '@/components/FileBrowser';
import { Plus, Trash01, RefreshCw01, FolderSearch } from '@untitledui/icons';

interface Collection {
  name: string;
  description: string;
  include_patterns: string[];
  exclude_patterns: string[];
  include_patterns_str?: string;
  exclude_patterns_str?: string;
}

function WorkspacePage() {
  const { getConfig, updateConfig } = useWorkspace();
  const toast = useToastContext();

  const [config, setConfig] = useState<WorkspaceConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchFilter, setSearchFilter] = useState('');
  const [saving, setSaving] = useState(false);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [expandedCollections, setExpandedCollections] = useState<Set<string>>(new Set());
  const [browsingProject, setBrowsingProject] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const configData = await getConfig();
      setConfig(configData);

      // Convert arrays to strings for textarea editing
      if (configData?.projects) {
        configData.projects.forEach((project) => {
          project.collections?.forEach((collection) => {
            (collection as any).include_patterns_str = collection.include_patterns?.join('\n') || '';
            (collection as any).exclude_patterns_str = collection.exclude_patterns?.join('\n') || '';
          });
        });
      }

      setHasUnsavedChanges(false);
    } catch (err) {
      console.error('Error loading workspace data:', err);
      setError(err instanceof Error ? err.message : 'Failed to load workspace data');
    } finally {
      setLoading(false);
    }
  };

  const filteredProjects = config?.projects?.filter((project) => {
    if (!searchFilter.trim()) return true;
    const search = searchFilter.toLowerCase();
    return (
      project.name?.toLowerCase().includes(search) ||
      project.description?.toLowerCase().includes(search) ||
      project.path?.toLowerCase().includes(search)
    );
  }) || [];

  const toggleCollection = (projectName: string, collectionIndex: number) => {
    const key = `${projectName}-${collectionIndex}`;
    const newExpanded = new Set(expandedCollections);
    if (newExpanded.has(key)) {
      newExpanded.delete(key);
    } else {
      newExpanded.add(key);
    }
    setExpandedCollections(newExpanded);
  };

  const isCollectionExpanded = (projectName: string, collectionIndex: number): boolean => {
    return expandedCollections.has(`${projectName}-${collectionIndex}`);
  };

  const addProject = () => {
    if (!config) return;

    const timestamp = Date.now();
    const newProject: WorkspaceProject = {
      name: `new-project-${timestamp}`,
      path: `../new-project-${timestamp}`,
      description: 'New Project Description',
      collections: [],
    };

    setConfig({
      ...config,
      projects: [...(config.projects || []), newProject],
    });

    setHasUnsavedChanges(true);
    setSearchFilter(`new-project-${timestamp}`);
  };

  const removeProject = async (projectName: string) => {
    if (!config) return;

    if (!window.confirm(`Are you sure you want to remove project "${projectName}"?`)) {
      return;
    }

    setConfig({
      ...config,
      projects: config.projects?.filter((p) => p.name !== projectName) || [],
    });

    setHasUnsavedChanges(true);
    setSearchFilter('');
  };

  const addCollection = (projectName: string) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    if (!project.collections) {
      project.collections = [];
    }

    const timestamp = Date.now();
    const newCollection: Collection = {
      name: `new-collection-${timestamp}`,
      description: 'New Collection Description',
      include_patterns: [],
      exclude_patterns: [],
      include_patterns_str: '',
      exclude_patterns_str: '',
    };

    project.collections.push(newCollection);
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const removeCollection = async (projectName: string, collectionIndex: number) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collectionName = project.collections?.[collectionIndex]?.name;
    if (!window.confirm(`Remove collection "${collectionName}"?`)) {
      return;
    }

    project.collections?.splice(collectionIndex, 1);
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const updateIncludePatterns = (projectName: string, collectionIndex: number, value: string) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collection = project.collections?.[collectionIndex] as any;
    if (!collection) return;

    collection.include_patterns_str = value;
    collection.include_patterns = value.split('\n').filter((p: string) => p.trim());
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const updateExcludePatterns = (projectName: string, collectionIndex: number, value: string) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collection = project.collections?.[collectionIndex] as any;
    if (!collection) return;

    collection.exclude_patterns_str = value;
    collection.exclude_patterns = value.split('\n').filter((p: string) => p.trim());
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const handleProjectFieldChange = (projectName: string, field: string, value: string) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    (project as any)[field] = value;
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const handleCollectionFieldChange = (
    projectName: string,
    collectionIndex: number,
    field: string,
    value: string
  ) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collection = project.collections?.[collectionIndex] as any;
    if (!collection) return;

    collection[field] = value;
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const saveWorkspaceConfig = async () => {
    if (!config) return;

    setSaving(true);
    try {
      // Clean up temporary string fields before saving
      const configToSave = JSON.parse(JSON.stringify(config));
      configToSave.projects?.forEach((project: any) => {
        project.collections?.forEach((collection: any) => {
          delete collection.include_patterns_str;
          delete collection.exclude_patterns_str;
        });
      });

      await updateConfig(configToSave);
      toast.success('Workspace configuration saved successfully!');
      setHasUnsavedChanges(false);
      await loadData();
    } catch (err) {
      console.error('Failed to save workspace configuration:', err);
      toast.error(`Failed to save configuration: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <LoadingState message="Loading workspace..." />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Workspace</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            Manage workspace projects and configuration
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={loadData}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button
            variant={hasUnsavedChanges ? 'danger' : 'secondary'}
            size="sm"
            onClick={saveWorkspaceConfig}
            disabled={saving || !hasUnsavedChanges}
          >
            {saving ? (
              <>
                <RefreshCw01 className="w-4 h-4 mr-2 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                {hasUnsavedChanges ? 'Save Changes *' : 'Save Configuration'}
              </>
            )}
          </Button>
          <Button variant="primary" size="sm" onClick={addProject}>
            <Plus className="w-4 h-4 mr-2" />
            Add Project
          </Button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Projects Configuration */}
      <Card>
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-4 flex-1">
            <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">Projects</h2>
            <div className="flex-1 max-w-md">
              <Input
                type="text"
                value={searchFilter}
                onChange={(e) => setSearchFilter(e.target.value)}
                placeholder="Search projects..."
                className="w-full"
              />
            </div>
          </div>
        </div>

        {filteredProjects.length === 0 && (!config?.projects || config.projects.length === 0) ? (
          <div className="flex flex-col items-center justify-center py-16 text-neutral-500 dark:text-neutral-400">
            <svg className="w-16 h-16 mb-4 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <h3 className="text-lg font-medium text-neutral-900 dark:text-white mb-2">No Projects</h3>
            <p className="text-sm">Add your first project to get started</p>
          </div>
        ) : filteredProjects.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-neutral-500 dark:text-neutral-400">
            <svg className="w-16 h-16 mb-4 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <h3 className="text-lg font-medium text-neutral-900 dark:text-white mb-2">No Results</h3>
            <p className="text-sm">No projects match your search</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredProjects.map((project) => (
              <div
                key={project.name}
                className="bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4"
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1 grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                        Project Name
                      </label>
                      <Input
                        type="text"
                        value={project.name || ''}
                        onChange={(e) => handleProjectFieldChange(project.name, 'name', e.target.value)}
                        placeholder="project-name"
                        className="text-sm"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                        Path
                      </label>
                      <div className="flex gap-2">
                        <Input
                          type="text"
                          value={project.path || ''}
                          onChange={(e) => handleProjectFieldChange(project.name, 'path', e.target.value)}
                          placeholder="../project-path"
                          className="text-sm font-mono flex-1"
                        />
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => setBrowsingProject(project.name)}
                          title="Browse folders"
                        >
                          <FolderSearch className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                        Description
                      </label>
                      <Input
                        type="text"
                        value={project.description || ''}
                        onChange={(e) => handleProjectFieldChange(project.name, 'description', e.target.value)}
                        placeholder="Project description"
                        className="text-sm"
                      />
                    </div>
                  </div>
                  <Button
                    variant="danger"
                    size="sm"
                    onClick={() => removeProject(project.name)}
                    className="ml-4"
                  >
                    <Trash01 className="w-4 h-4" />
                  </Button>
                </div>

                {/* Collections */}
                <div className="mt-4 pt-4 border-t border-neutral-200 dark:border-neutral-700">
                  <div className="flex items-center justify-between mb-3">
                    <h3 className="text-sm font-semibold text-neutral-900 dark:text-white">Collections</h3>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => addCollection(project.name)}
                    >
                      <Plus className="w-3 h-3 mr-1" />
                      Add Collection
                    </Button>
                  </div>

                  {!project.collections || project.collections.length === 0 ? (
                    <div className="text-sm text-neutral-500 dark:text-neutral-400 py-2">
                      No collections configured
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {project.collections.map((collection, collectionIndex) => (
                        <div
                          key={collectionIndex}
                          className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-700 rounded"
                        >
                          {/* Collection Header */}
                          <div
                            className="flex items-center justify-between p-3 cursor-pointer hover:bg-neutral-50 dark:hover:bg-neutral-800 transition-colors"
                            onClick={() => toggleCollection(project.name, collectionIndex)}
                          >
                            <div className="flex items-center gap-2 flex-1 min-w-0">
                              <svg
                                className={`w-4 h-4 text-neutral-400 transition-transform ${isCollectionExpanded(project.name, collectionIndex) ? 'rotate-90' : ''
                                  }`}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                              >
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                              </svg>
                              <span className="text-sm font-medium text-neutral-900 dark:text-white truncate">
                                {collection.name || 'Unnamed Collection'}
                              </span>
                              {collection.description && (
                                <span className="text-xs text-neutral-500 dark:text-neutral-400 truncate">
                                  {collection.description}
                                </span>
                              )}
                            </div>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                removeCollection(project.name, collectionIndex);
                              }}
                              className="text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300"
                            >
                              <Trash01 className="w-3 h-3" />
                            </Button>
                          </div>

                          {/* Collection Details (Collapsed by default) */}
                          {isCollectionExpanded(project.name, collectionIndex) && (
                            <div className="p-3 pt-0 space-y-3 border-t border-neutral-200 dark:border-neutral-700">
                              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                <div>
                                  <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                                    Collection Name
                                  </label>
                                  <Input
                                    type="text"
                                    value={collection.name || ''}
                                    onChange={(e) =>
                                      handleCollectionFieldChange(project.name, collectionIndex, 'name', e.target.value)
                                    }
                                    onClick={(e) => e.stopPropagation()}
                                    placeholder="collection-name"
                                    className="text-xs"
                                  />
                                </div>
                                <div>
                                  <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                                    Description
                                  </label>
                                  <Input
                                    type="text"
                                    value={collection.description || ''}
                                    onChange={(e) =>
                                      handleCollectionFieldChange(
                                        project.name,
                                        collectionIndex,
                                        'description',
                                        e.target.value
                                      )
                                    }
                                    onClick={(e) => e.stopPropagation()}
                                    placeholder="Collection description"
                                    className="text-xs"
                                  />
                                </div>
                              </div>

                              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                <div>
                                  <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                                    Include Patterns
                                  </label>
                                  <textarea
                                    value={(collection as any).include_patterns_str || ''}
                                    onChange={(e) => {
                                      e.stopPropagation();
                                      updateIncludePatterns(project.name, collectionIndex, e.target.value);
                                    }}
                                    onClick={(e) => e.stopPropagation()}
                                    rows={3}
                                    className="w-full px-2 py-1.5 text-xs font-mono bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                                    placeholder="**/*.md&#10;**/*.ts&#10;src/**/*"
                                  />
                                </div>
                                <div>
                                  <label className="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">
                                    Exclude Patterns
                                  </label>
                                  <textarea
                                    value={(collection as any).exclude_patterns_str || ''}
                                    onChange={(e) => {
                                      e.stopPropagation();
                                      updateExcludePatterns(project.name, collectionIndex, e.target.value);
                                    }}
                                    onClick={(e) => e.stopPropagation()}
                                    rows={3}
                                    className="w-full px-2 py-1.5 text-xs font-mono bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                                    placeholder="node_modules/**&#10;dist/**&#10;**/*.log"
                                  />
                                </div>
                              </div>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* File Browser Modal */}
      {browsingProject && (
        <FileBrowser
          initialPath={config?.projects?.find(p => p.name === browsingProject)?.path || ''}
          onSelect={(path) => {
            handleProjectFieldChange(browsingProject, 'path', path);
            setBrowsingProject(null);
          }}
          onCancel={() => setBrowsingProject(null)}
        />
      )}
    </div>
  );
}

export default WorkspacePage;
