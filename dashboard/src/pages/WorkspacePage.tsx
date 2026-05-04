/**
 * Workspace page — console-themed restyle.
 *
 * Visual restyle only: behaviour (loading workspace config, editing
 * projects + collections inline, browsing folders via FileBrowser, and
 * persisting changes through `updateConfig`) is preserved from the
 * pre-redesign version. The redesign brief has no dedicated mockup for
 * Workspace, so this page applies the established Phase 3 recipe:
 *   - `.page` + `.page-head` shell with title/sub + toolbar buttons
 *   - console `Card` / `CardHead` / `CardBody`
 *   - `Kpi` cards for the headline metrics
 *   - `StatusPill` / `Pill` for unsaved/error chips
 *   - `.btn` + `.input` for actions and form controls
 *   - no Tailwind utility classes, no `dark:` variants
 *
 * The FileBrowser modal is kept as-is — its internals still use the
 * legacy styling. Marked with `// TODO(workspace-modal)` until the
 * console design ships a modal primitive.
 */

import { useCallback, useEffect, useState } from 'react';
import { useWorkspace, WorkspaceConfig, WorkspaceProject } from '@/hooks/useWorkspace';
import { useToastContext } from '@/providers/ToastProvider';
import FileBrowser from '@/components/FileBrowser';
import {
  Icons,
  Pill,
  Card,
  CardHead,
  CardBody,
  Kpi,
} from '@/components/console';

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

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const configData = await getConfig();
      setConfig(configData);

      // Convert arrays to strings for textarea editing
      if (configData?.projects) {
        configData.projects.forEach((project) => {
          project.collections?.forEach((collection) => {
            (collection as Collection).include_patterns_str =
              collection.include_patterns?.join('\n') || '';
            (collection as Collection).exclude_patterns_str =
              collection.exclude_patterns?.join('\n') || '';
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
  }, [getConfig]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const allProjects = config?.projects ?? [];
  const filteredProjects = allProjects.filter((project) => {
    if (!searchFilter.trim()) return true;
    const search = searchFilter.toLowerCase();
    return (
      project.name?.toLowerCase().includes(search) ||
      project.description?.toLowerCase().includes(search) ||
      project.path?.toLowerCase().includes(search)
    );
  });

  const totalCollections = allProjects.reduce(
    (sum, p) => sum + (p.collections?.length ?? 0),
    0,
  );

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

  const removeProject = (projectName: string) => {
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

  const removeCollection = (projectName: string, collectionIndex: number) => {
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

  const updateIncludePatterns = (
    projectName: string,
    collectionIndex: number,
    value: string,
  ) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collection = project.collections?.[collectionIndex] as Collection | undefined;
    if (!collection) return;

    collection.include_patterns_str = value;
    collection.include_patterns = value.split('\n').filter((p: string) => p.trim());
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const updateExcludePatterns = (
    projectName: string,
    collectionIndex: number,
    value: string,
  ) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collection = project.collections?.[collectionIndex] as Collection | undefined;
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

    (project as unknown as Record<string, unknown>)[field] = value;
    setConfig({ ...config });
    setHasUnsavedChanges(true);
  };

  const handleCollectionFieldChange = (
    projectName: string,
    collectionIndex: number,
    field: string,
    value: string,
  ) => {
    if (!config) return;

    const project = config.projects?.find((p) => p.name === projectName);
    if (!project) return;

    const collection = project.collections?.[collectionIndex] as
      | Record<string, unknown>
      | undefined;
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
      configToSave.projects?.forEach((project: WorkspaceProject) => {
        project.collections?.forEach((collection) => {
          delete (collection as Collection).include_patterns_str;
          delete (collection as Collection).exclude_patterns_str;
        });
      });

      await updateConfig(configToSave);
      toast.success('Workspace configuration saved successfully!');
      setHasUnsavedChanges(false);
      await loadData();
    } catch (err) {
      console.error('Failed to save workspace configuration:', err);
      toast.error(
        `Failed to save configuration: ${err instanceof Error ? err.message : 'Unknown error'}`,
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Workspace</h1>
          <p className="page-sub">Manage workspace projects and configuration</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          {hasUnsavedChanges && (
            <Pill tone="amber">
              <span className="dot amber" />
              unsaved changes
            </Pill>
          )}
          <button className="btn" onClick={loadData} disabled={loading}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button
            className="btn"
            onClick={saveWorkspaceConfig}
            disabled={saving || !hasUnsavedChanges}
          >
            <Icons.check size={13} />
            {saving ? 'Saving…' : hasUnsavedChanges ? 'Save changes' : 'Saved'}
          </button>
          <button className="btn primary" onClick={addProject} disabled={!config}>
            <Icons.plus size={13} />
            Add project
          </button>
        </div>
      </div>

      {error && (
        <div style={{ marginBottom: 14 }}>
          <Card>
            <CardBody>
              <div className="row" style={{ gap: 8 }}>
                <Pill tone="red">error</Pill>
                <span style={{ color: 'var(--text-2)' }}>{error}</span>
              </div>
            </CardBody>
          </Card>
        </div>
      )}

      <Card>
        <CardHead
          title="Overview"
          sub={loading && !config ? 'loading…' : undefined}
        />
        <CardBody>
          <div className="row" style={{ gap: 24, flexWrap: 'wrap' }}>
            <Kpi label="Projects" value={allProjects.length} />
            <Kpi
              label="Collections"
              value={totalCollections}
              accent={totalCollections > 0 ? 'teal' : 'none'}
            />
            <Kpi
              label="Filter"
              value={searchFilter ? `${filteredProjects.length}/${allProjects.length}` : 'all'}
            />
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <Card>
        <CardHead
          title="Projects"
          sub={allProjects.length > 0 ? `${allProjects.length} configured` : undefined}
          right={
            <input
              className="input"
              type="text"
              value={searchFilter}
              onChange={(e) => setSearchFilter(e.target.value)}
              placeholder="Search projects…"
              style={{ width: 220 }}
            />
          }
        />
        <CardBody>
          {allProjects.length === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No projects · Add your first project to get started.
            </div>
          ) : filteredProjects.length === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No projects match your search.
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {filteredProjects.map((project) => (
                <div
                  key={project.name}
                  style={{
                    border: '1px solid var(--border)',
                    borderRadius: 4,
                    background: 'var(--surface-2)',
                    padding: 14,
                  }}
                >
                  <div className="row" style={{ gap: 12, alignItems: 'flex-start' }}>
                    <div
                      style={{
                        flex: 1,
                        display: 'grid',
                        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
                        gap: 12,
                      }}
                    >
                      <label
                        className="col"
                        style={{ display: 'flex', flexDirection: 'column', gap: 4 }}
                      >
                        <span style={{ color: 'var(--text-2)', fontSize: 11 }}>
                          Project name
                        </span>
                        <input
                          className="input"
                          type="text"
                          value={project.name || ''}
                          onChange={(e) =>
                            handleProjectFieldChange(project.name, 'name', e.target.value)
                          }
                          placeholder="project-name"
                        />
                      </label>
                      <label
                        className="col"
                        style={{ display: 'flex', flexDirection: 'column', gap: 4 }}
                      >
                        <span style={{ color: 'var(--text-2)', fontSize: 11 }}>Path</span>
                        <div className="row" style={{ gap: 6 }}>
                          <input
                            className="input mono"
                            type="text"
                            value={project.path || ''}
                            onChange={(e) =>
                              handleProjectFieldChange(project.name, 'path', e.target.value)
                            }
                            placeholder="../project-path"
                            style={{ flex: 1 }}
                          />
                          <button
                            className="btn sm"
                            onClick={() => setBrowsingProject(project.name)}
                            title="Browse folders"
                            aria-label={`Browse folders for ${project.name}`}
                          >
                            <Icons.search size={11} />
                          </button>
                        </div>
                      </label>
                      <label
                        className="col"
                        style={{ display: 'flex', flexDirection: 'column', gap: 4 }}
                      >
                        <span style={{ color: 'var(--text-2)', fontSize: 11 }}>
                          Description
                        </span>
                        <input
                          className="input"
                          type="text"
                          value={project.description || ''}
                          onChange={(e) =>
                            handleProjectFieldChange(
                              project.name,
                              'description',
                              e.target.value,
                            )
                          }
                          placeholder="Project description"
                        />
                      </label>
                    </div>
                    <button
                      className="btn sm"
                      onClick={() => removeProject(project.name)}
                      aria-label={`Remove project ${project.name}`}
                    >
                      <Icons.trash size={11} />
                    </button>
                  </div>

                  <div
                    style={{
                      marginTop: 14,
                      paddingTop: 12,
                      borderTop: '1px solid var(--border)',
                    }}
                  >
                    <div
                      className="row"
                      style={{ justifyContent: 'space-between', marginBottom: 10 }}
                    >
                      <div className="row" style={{ gap: 8 }}>
                        <Icons.collections size={12} className="muted" />
                        <span style={{ fontSize: 12, fontWeight: 500 }}>Collections</span>
                        <Pill tone="muted">{project.collections?.length ?? 0}</Pill>
                      </div>
                      <button
                        className="btn sm"
                        onClick={() => addCollection(project.name)}
                      >
                        <Icons.plus size={11} />
                        Add collection
                      </button>
                    </div>

                    {!project.collections || project.collections.length === 0 ? (
                      <div style={{ color: 'var(--text-3)', fontSize: 12, padding: 6 }}>
                        No collections configured
                      </div>
                    ) : (
                      <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                        {project.collections.map((collection, collectionIndex) => {
                          const expanded = isCollectionExpanded(
                            project.name,
                            collectionIndex,
                          );
                          return (
                            <div
                              key={collectionIndex}
                              style={{
                                border: '1px solid var(--border)',
                                borderRadius: 4,
                                background: 'var(--surface-1)',
                              }}
                            >
                              <div
                                className="row"
                                style={{
                                  justifyContent: 'space-between',
                                  padding: '8px 10px',
                                  cursor: 'pointer',
                                }}
                                onClick={() =>
                                  toggleCollection(project.name, collectionIndex)
                                }
                              >
                                <div
                                  className="row"
                                  style={{
                                    gap: 8,
                                    flex: 1,
                                    minWidth: 0,
                                    alignItems: 'center',
                                  }}
                                >
                                  <span
                                    style={{
                                      display: 'inline-flex',
                                      transition: 'transform 120ms ease',
                                      transform: expanded
                                        ? 'rotate(90deg)'
                                        : 'rotate(0deg)',
                                      color: 'var(--text-3)',
                                    }}
                                  >
                                    <Icons.chevron size={11} />
                                  </span>
                                  <span
                                    style={{
                                      fontSize: 12,
                                      fontWeight: 500,
                                      whiteSpace: 'nowrap',
                                      overflow: 'hidden',
                                      textOverflow: 'ellipsis',
                                    }}
                                  >
                                    {collection.name || 'Unnamed Collection'}
                                  </span>
                                  {collection.description && (
                                    <span
                                      style={{
                                        fontSize: 11,
                                        color: 'var(--text-3)',
                                        whiteSpace: 'nowrap',
                                        overflow: 'hidden',
                                        textOverflow: 'ellipsis',
                                      }}
                                    >
                                      {collection.description}
                                    </span>
                                  )}
                                </div>
                                <button
                                  className="btn sm"
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    removeCollection(project.name, collectionIndex);
                                  }}
                                  aria-label={`Remove collection ${collection.name}`}
                                >
                                  <Icons.trash size={11} />
                                </button>
                              </div>

                              {expanded && (
                                <div
                                  style={{
                                    padding: 10,
                                    borderTop: '1px solid var(--border)',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    gap: 10,
                                  }}
                                  onClick={(e) => e.stopPropagation()}
                                >
                                  <div
                                    style={{
                                      display: 'grid',
                                      gridTemplateColumns:
                                        'repeat(auto-fit, minmax(180px, 1fr))',
                                      gap: 10,
                                    }}
                                  >
                                    <label
                                      className="col"
                                      style={{
                                        display: 'flex',
                                        flexDirection: 'column',
                                        gap: 4,
                                      }}
                                    >
                                      <span
                                        style={{
                                          color: 'var(--text-2)',
                                          fontSize: 11,
                                        }}
                                      >
                                        Collection name
                                      </span>
                                      <input
                                        className="input"
                                        type="text"
                                        value={collection.name || ''}
                                        onChange={(e) =>
                                          handleCollectionFieldChange(
                                            project.name,
                                            collectionIndex,
                                            'name',
                                            e.target.value,
                                          )
                                        }
                                        placeholder="collection-name"
                                      />
                                    </label>
                                    <label
                                      className="col"
                                      style={{
                                        display: 'flex',
                                        flexDirection: 'column',
                                        gap: 4,
                                      }}
                                    >
                                      <span
                                        style={{
                                          color: 'var(--text-2)',
                                          fontSize: 11,
                                        }}
                                      >
                                        Description
                                      </span>
                                      <input
                                        className="input"
                                        type="text"
                                        value={collection.description || ''}
                                        onChange={(e) =>
                                          handleCollectionFieldChange(
                                            project.name,
                                            collectionIndex,
                                            'description',
                                            e.target.value,
                                          )
                                        }
                                        placeholder="Collection description"
                                      />
                                    </label>
                                  </div>

                                  <div
                                    style={{
                                      display: 'grid',
                                      gridTemplateColumns:
                                        'repeat(auto-fit, minmax(220px, 1fr))',
                                      gap: 10,
                                    }}
                                  >
                                    <label
                                      className="col"
                                      style={{
                                        display: 'flex',
                                        flexDirection: 'column',
                                        gap: 4,
                                      }}
                                    >
                                      <span
                                        style={{
                                          color: 'var(--text-2)',
                                          fontSize: 11,
                                        }}
                                      >
                                        Include patterns
                                      </span>
                                      <textarea
                                        className="input mono"
                                        value={
                                          (collection as Collection)
                                            .include_patterns_str || ''
                                        }
                                        onChange={(e) =>
                                          updateIncludePatterns(
                                            project.name,
                                            collectionIndex,
                                            e.target.value,
                                          )
                                        }
                                        rows={3}
                                        placeholder={'**/*.md\n**/*.ts\nsrc/**/*'}
                                        style={{ resize: 'vertical' }}
                                      />
                                    </label>
                                    <label
                                      className="col"
                                      style={{
                                        display: 'flex',
                                        flexDirection: 'column',
                                        gap: 4,
                                      }}
                                    >
                                      <span
                                        style={{
                                          color: 'var(--text-2)',
                                          fontSize: 11,
                                        }}
                                      >
                                        Exclude patterns
                                      </span>
                                      <textarea
                                        className="input mono"
                                        value={
                                          (collection as Collection)
                                            .exclude_patterns_str || ''
                                        }
                                        onChange={(e) =>
                                          updateExcludePatterns(
                                            project.name,
                                            collectionIndex,
                                            e.target.value,
                                          )
                                        }
                                        rows={3}
                                        placeholder={
                                          'node_modules/**\ndist/**\n**/*.log'
                                        }
                                        style={{ resize: 'vertical' }}
                                      />
                                    </label>
                                  </div>
                                </div>
                              )}
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardBody>
      </Card>

      {/* TODO(workspace-modal): FileBrowser still uses legacy
          Tailwind/UntitledUI styling. Replace once the console design
          ships a folder-picker primitive. */}
      {browsingProject && (
        <FileBrowser
          initialPath={
            config?.projects?.find((p) => p.name === browsingProject)?.path || ''
          }
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
