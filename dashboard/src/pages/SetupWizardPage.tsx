/**
 * Setup Wizard Page — console-themed restyle.
 *
 * Multi-step wizard for initial workspace configuration. Visual
 * restyle only: behaviour (state shape, validation, API calls,
 * navigation between steps, persisted progress) is unchanged from the
 * pre-redesign version.
 *
 * The outer `WizardLayout` provides the dark console chrome (header
 * + dark background) and activates `body[data-console="1"]`. This
 * page renders inside that layout.
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSetup, SetupStatus, ProjectAnalysis, SetupProject, SuggestedCollection } from '@/hooks/useSetup';
import { useTemplates, ConfigTemplate, getTemplateIcon } from '@/hooks/useTemplates.tsx';
import { useApiKeys } from '@/hooks/useApiKeys';
import { useApiClient } from '@/hooks/useApiClient';
import { useWizardProgress } from '@/hooks/useWizardProgress';
import { Card, CardBody, Pill, Kpi, Icons } from '@/components/console';
import LoadingSpinner from '@/components/LoadingSpinner';
import FileBrowser from '@/components/FileBrowser';

// Validation types
interface PathValidation {
  isValidating: boolean;
  isValid: boolean | null;
  error: string | null;
  isProject: boolean;
  projectInfo?: {
    hasGit: boolean;
    hasPackageJson: boolean;
    hasCargoToml: boolean;
    hasPyProject: boolean;
  };
}

interface CollectionValidation {
  isValid: boolean;
  error: string | null;
}

// Debounce hook
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

type WizardStep = 'welcome' | 'template' | 'folder' | 'analysis' | 'review' | 'api-key' | 'complete';

interface AnalyzedProject {
  analysis: ProjectAnalysis;
  selected: boolean;
  collections: Array<{
    name: string;
    description: string;
    include_patterns: string[];
    exclude_patterns: string[];
    content_type: string;
    selected: boolean;
    enable_graph: boolean; // Enable automatic graph relationship discovery
  }>;
}

const STEP_LABELS: Record<Exclude<WizardStep, 'complete'>, string> = {
  welcome: 'Welcome',
  template: 'Template',
  folder: 'Folder',
  analysis: 'Analysis',
  review: 'Review',
  'api-key': 'API Key',
};

function SetupWizardPage() {
  const navigate = useNavigate();
  const api = useApiClient();
  const { getStatus, analyzeDirectory, applyConfig } = useSetup();
  const { templates, loading: templatesLoading } = useTemplates();
  const { createApiKey, loading: creatingKey } = useApiKeys();
  const progress = useWizardProgress<WizardStep, ConfigTemplate, AnalyzedProject>();
  const [showResumeBanner, setShowResumeBanner] = useState<boolean>(() => progress.snapshot !== null);

  // Wizard state
  const [currentStep, setCurrentStep] = useState<WizardStep>('welcome');
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState(false);
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [selectedTemplate, setSelectedTemplate] = useState<ConfigTemplate | null>(null);
  const [folderPath, setFolderPath] = useState('');
  const [analyzedProjects, setAnalyzedProjects] = useState<AnalyzedProject[]>([]);
  const [showYamlPreview, setShowYamlPreview] = useState(false);
  const [yamlCopied, setYamlCopied] = useState(false);
  const [showFileBrowser, setShowFileBrowser] = useState(false);

  // Real-time validation state
  const [pathValidation, setPathValidation] = useState<PathValidation>({
    isValidating: false,
    isValid: null,
    error: null,
    isProject: false,
  });
  const [collectionValidations, setCollectionValidations] = useState<Record<string, CollectionValidation>>({});

  // Skip wizard confirmation
  const [showSkipConfirm, setShowSkipConfirm] = useState(false);

  // API Key state
  const [apiKeyCreated, setApiKeyCreated] = useState(false);
  const [apiKey, setApiKey] = useState<string | null>(null);
  const [apiKeyName, setApiKeyName] = useState('Cursor MCP Integration');
  const [mcpConfigCopied, setMcpConfigCopied] = useState(false);
  const [mcpConfigType, setMcpConfigType] = useState<'npx' | 'streamablehttp'>('npx');

  // Debounced path for validation
  const debouncedPath = useDebounce(folderPath, 500);

  // Validate path in real-time
  const validatePath = useCallback(async (path: string) => {
    if (!path.trim()) {
      setPathValidation({
        isValidating: false,
        isValid: null,
        error: null,
        isProject: false,
      });
      return;
    }

    setPathValidation(prev => ({ ...prev, isValidating: true }));

    try {
      // Route through the API client so the request hits the configured
      // backend (localhost:15002 in dev) with the JWT bearer token. A
      // bare `fetch('/setup/browse')` was previously sent to the Vite
      // dev server, which 404'd silently and broke path validation.
      const data = await api.post<{
        valid: boolean;
        error?: string;
        entries?: Array<{ name: string }>;
      }>('/setup/browse', { path });

      if (!data.valid) {
        setPathValidation({
          isValidating: false,
          isValid: false,
          error: data.error || 'Invalid path',
          isProject: false,
        });
        return;
      }

      // Check for project indicators in the browsed directory
      const projectIndicators = {
        hasGit: data.entries?.some((e: { name: string }) => e.name === '.git') || false,
        hasPackageJson: data.entries?.some((e: { name: string }) => e.name === 'package.json') || false,
        hasCargoToml: data.entries?.some((e: { name: string }) => e.name === 'Cargo.toml') || false,
        hasPyProject: data.entries?.some((e: { name: string }) => e.name === 'pyproject.toml' || e.name === 'setup.py') || false,
      };

      const hasProjectFiles = Object.values(projectIndicators).some(Boolean);

      setPathValidation({
        isValidating: false,
        isValid: true,
        error: hasProjectFiles ? null : 'Path is valid but no project files detected',
        isProject: hasProjectFiles,
        projectInfo: projectIndicators,
      });
    } catch {
      setPathValidation({
        isValidating: false,
        isValid: false,
        error: 'Failed to validate path',
        isProject: false,
      });
    }
  }, [api]);

  // Effect to validate path when debounced value changes
  useEffect(() => {
    if (debouncedPath && currentStep === 'folder') {
      validatePath(debouncedPath);
    }
  }, [debouncedPath, currentStep, validatePath]);

  // Validate collection name
  const validateCollectionName = useCallback((name: string): CollectionValidation => {
    if (!name.trim()) {
      return { isValid: false, error: 'Collection name is required' };
    }
    if (name.length < 3) {
      return { isValid: false, error: 'Name must be at least 3 characters' };
    }
    if (name.length > 64) {
      return { isValid: false, error: 'Name must be 64 characters or less' };
    }
    if (!/^[a-z0-9][a-z0-9-_]*[a-z0-9]$/.test(name) && name.length > 1) {
      return { isValid: false, error: 'Use lowercase letters, numbers, hyphens, and underscores only' };
    }
    if (/--/.test(name) || /__/.test(name)) {
      return { isValid: false, error: 'Cannot have consecutive hyphens or underscores' };
    }
    return { isValid: true, error: null };
  }, []);

  // Update collection validation when collections change.
  // Runs cross-project duplicate detection so two selected collections
  // can never share the same final name — the server would refuse the
  // resulting workspace.yml later, so surface it at the wizard step.
  useEffect(() => {
    const validations: Record<string, CollectionValidation> = {};
    const nameCounts = new Map<string, number>();
    analyzedProjects.forEach((project) => {
      if (!project.selected) return;
      project.collections.forEach((col) => {
        if (!col.selected) return;
        const normalized = col.name.trim().toLowerCase();
        nameCounts.set(normalized, (nameCounts.get(normalized) ?? 0) + 1);
      });
    });

    analyzedProjects.forEach((project, pi) => {
      project.collections.forEach((col, ci) => {
        const key = `${pi}-${ci}`;
        const baseValidation = validateCollectionName(col.name);
        if (!baseValidation.isValid) {
          validations[key] = baseValidation;
          return;
        }
        const normalized = col.name.trim().toLowerCase();
        const count = nameCounts.get(normalized) ?? 0;
        if (project.selected && col.selected && count > 1) {
          validations[key] = {
            isValid: false,
            error: `Duplicate collection name — ${count} selected collections use "${col.name}"`,
          };
        } else {
          validations[key] = baseValidation;
        }
      });
    });
    setCollectionValidations(validations);
  }, [analyzedProjects, validateCollectionName]);

  // Persist wizard progress so a page reload or crash resumes at the
  // same step. Skipped for the welcome step (nothing to restore) and
  // for the api-key / complete steps (credentials must not persist).
  useEffect(() => {
    if (currentStep === 'welcome' || currentStep === 'api-key' || currentStep === 'complete') return;
    progress.save({
      step: currentStep,
      template: selectedTemplate,
      folderPath,
      projects: analyzedProjects,
    });
  }, [currentStep, selectedTemplate, folderPath, analyzedProjects, progress]);

  const resumeFromSnapshot = useCallback(() => {
    const snapshot = progress.snapshot;
    if (!snapshot) return;
    setCurrentStep(snapshot.step);
    setSelectedTemplate(snapshot.template);
    setFolderPath(snapshot.folderPath);
    setAnalyzedProjects(snapshot.projects);
    setShowResumeBanner(false);
  }, [progress.snapshot]);

  const discardSnapshot = useCallback(() => {
    progress.clear();
    setShowResumeBanner(false);
  }, [progress]);

  // Generate YAML preview
  const yamlPreview = useMemo(() => {
    const selectedProjects = analyzedProjects.filter(p => p.selected);
    if (selectedProjects.length === 0) return '';

    const config = {
      global_settings: {
        file_watcher: {
          auto_discovery: true,
          enable_auto_update: true,
          hot_reload: true,
        },
      },
      projects: selectedProjects.map(p => ({
        name: p.analysis.project_name,
        path: p.analysis.project_path,
        description: `${p.analysis.project_types[0] || 'Mixed'} project`,
        collections: p.collections
          .filter(c => c.selected)
          .map(c => ({
            name: c.name,
            description: c.description,
            include_patterns: c.include_patterns,
            exclude_patterns: c.exclude_patterns,
            ...(c.enable_graph && { enable_graph: true }),
          })),
      })),
    };

    // Simple YAML generation (can be improved with a library)
    const toYaml = (obj: unknown, level = 0): string => {
      const spaces = '  '.repeat(level);
      if (Array.isArray(obj)) {
        return obj.map(item => {
          if (typeof item === 'object' && item !== null) {
            const entries = Object.entries(item);
            const first = entries[0];
            const rest = entries.slice(1);
            let result = `${spaces}- ${first[0]}: ${typeof first[1] === 'object' ? '' : first[1]}`;
            if (typeof first[1] === 'object') {
              result += '\n' + toYaml(first[1], level + 2);
            }
            for (const [key, val] of rest) {
              if (typeof val === 'object') {
                result += `\n${spaces}  ${key}:`;
                result += '\n' + toYaml(val, level + 2);
              } else {
                result += `\n${spaces}  ${key}: ${val}`;
              }
            }
            return result;
          }
          return `${spaces}- ${item}`;
        }).join('\n');
      }
      if (typeof obj === 'object' && obj !== null) {
        return Object.entries(obj).map(([key, val]) => {
          if (typeof val === 'object') {
            return `${spaces}${key}:\n${toYaml(val, level + 1)}`;
          }
          return `${spaces}${key}: ${val}`;
        }).join('\n');
      }
      return String(obj);
    };

    return `# Vectorizer Workspace Configuration\n# Generated by Setup Wizard\n\n${toYaml(config)}`;
  }, [analyzedProjects]);

  const handleCopyYaml = async () => {
    try {
      await navigator.clipboard.writeText(yamlPreview);
      setYamlCopied(true);
      setTimeout(() => setYamlCopied(false), 2000);
    } catch {
      console.error('Failed to copy YAML');
    }
  };

  // Quick Setup - Apply template with current directory
  const [quickSetupLoading, setQuickSetupLoading] = useState(false);

  const handleQuickSetup = async (template: ConfigTemplate) => {
    setQuickSetupLoading(true);
    setError(null);

    try {
      // Use current working directory (from server) or a default. Route
      // through the API client so the request reaches the configured
      // backend in dev (Vite would otherwise serve the SPA HTML for
      // `/health`, breaking JSON parsing).
      const cwdData = await api.get<{ data_dir?: string }>('/health');
      const projectPath = cwdData.data_dir || '.';

      // Create a basic workspace config with the template
      const projects: SetupProject[] = [{
        name: 'my-project',
        path: projectPath,
        description: `${template.name} workspace`,
        collections: template.collections.map(c => ({
          name: `my-project-${c.name_suffix}`,
          description: c.description,
          include_patterns: c.include_patterns,
          exclude_patterns: c.exclude_patterns,
        })),
      }];

      await applyConfig({
        projects,
        global_settings: {
          file_watcher: {
            auto_discovery: true,
            enable_auto_update: true,
            hot_reload: true,
          },
        },
      });

      setCurrentStep('complete');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Quick setup failed');
    } finally {
      setQuickSetupLoading(false);
    }
  };

  // Load initial status - runs only once on mount
  useEffect(() => {
    let isMounted = true;

    const fetchStatus = async () => {
      setLoading(true);
      try {
        const s = await getStatus();
        if (isMounted) {
          setStatus(s);
          if (!s.needs_setup) {
            setCurrentStep('complete');
          }
        }
      } catch (_err) {
        if (isMounted) {
          setError('Failed to load setup status');
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    };
    fetchStatus();

    return () => {
      isMounted = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Run only once on mount

  const handleTemplateSelect = (template: ConfigTemplate) => {
    setSelectedTemplate(template);
    setCurrentStep('folder');
  };

  const handleAnalyze = async () => {
    if (!folderPath.trim()) {
      setError('Please enter a folder path');
      return;
    }

    setAnalyzing(true);
    setError(null);

    try {
      const analysis = await analyzeDirectory(folderPath.trim());

      // Merge template collections with detected collections if template selected
      let collections = analysis.suggested_collections.map((c: SuggestedCollection) => ({
        name: c.name,
        description: c.description,
        include_patterns: c.include_patterns,
        exclude_patterns: c.exclude_patterns,
        content_type: c.content_type,
        selected: true,
        enable_graph: false, // Graph relationships disabled by default
      }));

      // If template is selected and not custom, add template collections
      if (selectedTemplate && selectedTemplate.id !== 'custom') {
        const templateCollections = selectedTemplate.collections.map(tc => ({
          name: `${analysis.project_name}-${tc.name_suffix}`,
          description: tc.description,
          include_patterns: tc.include_patterns,
          exclude_patterns: tc.exclude_patterns,
          content_type: tc.content_type,
          selected: true,
          enable_graph: false, // Graph relationships disabled by default
        }));

        // Merge: prefer template collections, add unique detected ones
        const templateNames = new Set(templateCollections.map(c => c.name));
        const uniqueDetected = collections.filter(c => !templateNames.has(c.name));
        collections = [...templateCollections, ...uniqueDetected];
      }

      const project: AnalyzedProject = {
        analysis,
        selected: true,
        collections,
      };

      setAnalyzedProjects([...analyzedProjects, project]);
      setFolderPath('');
      setCurrentStep('analysis');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to analyze directory');
    } finally {
      setAnalyzing(false);
    }
  };

  const handleApplyConfig = async () => {
    setApplying(true);
    setError(null);

    try {
      const projects: SetupProject[] = analyzedProjects
        .filter(p => p.selected)
        .map(p => ({
          name: p.analysis.project_name,
          path: p.analysis.project_path,
          description: `${p.analysis.project_types[0] || 'Mixed'} project`,
          collections: p.collections
            .filter(c => c.selected)
            .map(c => ({
              name: c.name,
              description: c.description,
              include_patterns: c.include_patterns,
              exclude_patterns: c.exclude_patterns,
              enable_graph: c.enable_graph,
            })),
        }));

      if (projects.length === 0) {
        setError('Please select at least one project');
        setApplying(false);
        return;
      }

      await applyConfig({
        projects,
        global_settings: {
          file_watcher: {
            auto_discovery: true,
            enable_auto_update: true,
            hot_reload: true,
          },
        },
      });

      // Successful apply — drop any saved resume snapshot so the next
      // visit starts fresh instead of resurrecting the now-committed
      // workspace state.
      progress.clear();

      // Go to API key step instead of complete
      setCurrentStep('api-key');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to apply configuration');
    } finally {
      setApplying(false);
    }
  };

  const handleCreateApiKey = async () => {
    if (!apiKeyName.trim()) {
      setError('API key name is required');
      return;
    }

    try {
      const response = await createApiKey({
        name: apiKeyName,
        permissions: ['read', 'write', 'create_collection', 'delete_collection'],
      });
      setApiKey(response.api_key);
      setApiKeyCreated(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create API key');
    }
  };

  const generateMcpConfig = (): string => {
    if (!apiKey) return '';

    if (mcpConfigType === 'npx') {
      // Cursor MCP configuration with npx (recommended - uses official MCP package)
      const config = {
        mcpServers: {
          vectorizer: {
            command: 'npx',
            args: ['-y', '@hivellm/mcp-vectorizer'],
            env: {
              VECTORIZER_API_URL: 'http://localhost:15002',
              VECTORIZER_API_KEY: apiKey,
            },
          },
        },
      };
      return JSON.stringify(config, null, 2);
    } else {
      // Direct StreamableHTTP configuration (alternative - requires manual header setup)
      const config = {
        mcpServers: {
          vectorizer: {
            url: 'http://localhost:15002/mcp',
            type: 'streamablehttp',
            // Note: Headers may not be supported in Cursor MCP config
            // You may need to set Authorization header manually in your MCP client
          },
        },
      };
      return JSON.stringify(config, null, 2);
    }
  };

  const copyMcpConfig = async () => {
    const config = generateMcpConfig();
    try {
      await navigator.clipboard.writeText(config);
      setMcpConfigCopied(true);
      setTimeout(() => setMcpConfigCopied(false), 2000);
    } catch {
      setError('Failed to copy configuration');
    }
  };

  const toggleProjectSelection = (index: number) => {
    setAnalyzedProjects(prev => prev.map((p, i) =>
      i === index ? { ...p, selected: !p.selected } : p
    ));
  };

  const toggleCollectionSelection = (projectIndex: number, collectionIndex: number) => {
    setAnalyzedProjects(prev => prev.map((p, pi) =>
      pi === projectIndex ? {
        ...p,
        collections: p.collections.map((c, ci) =>
          ci === collectionIndex ? { ...c, selected: !c.selected } : c
        ),
      } : p
    ));
  };

  const toggleCollectionGraph = (projectIndex: number, collectionIndex: number) => {
    setAnalyzedProjects(prev => prev.map((p, pi) =>
      pi === projectIndex ? {
        ...p,
        collections: p.collections.map((c, ci) =>
          ci === collectionIndex ? { ...c, enable_graph: !c.enable_graph } : c
        ),
      } : p
    ));
  };

  const removeProject = (index: number) => {
    setAnalyzedProjects(prev => prev.filter((_, i) => i !== index));
  };

  // Handle skip wizard
  const handleSkipWizard = () => {
    setShowSkipConfirm(false);
    navigate('/overview');
  };

  // Steps for the progress indicator (complete is not shown as a step, it's the final state)
  const progressSteps: Array<Exclude<WizardStep, 'complete'>> = [
    'welcome', 'template', 'folder', 'analysis', 'review', 'api-key',
  ];
  const stepIndex = progressSteps.indexOf(currentStep as Exclude<WizardStep, 'complete'>);
  const isComplete = currentStep === 'complete';
  const totalSteps = progressSteps.length;
  const currentStepNumber = isComplete ? totalSteps : Math.max(0, stepIndex) + 1;
  const completed = isComplete ? totalSteps : Math.max(0, stepIndex);
  const remaining = Math.max(0, totalSteps - completed);

  if (loading) {
    return (
      <div className="page" style={{ display: 'grid', placeItems: 'center', minHeight: '60vh' }}>
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="page" style={{ maxWidth: 1100, margin: '0 auto' }}>
      {/* Skip Confirmation Modal */}
      {showSkipConfirm && (
        <div
          style={{
            position: 'fixed', inset: 0, zIndex: 50,
            display: 'grid', placeItems: 'center', padding: 16,
            background: 'rgba(0,0,0,0.65)', backdropFilter: 'blur(4px)',
          }}
          role="dialog"
          aria-modal="true"
        >
          <Card className="" >
            <CardBody>
              <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12, maxWidth: 460 }}>
                <div style={{
                  width: 40, height: 40, borderRadius: 999,
                  background: 'var(--bg-2)', border: '1px solid var(--border)',
                  display: 'grid', placeItems: 'center', flexShrink: 0,
                  color: 'var(--amber)',
                }}>
                  <Icons.bell size={18} />
                </div>
                <div>
                  <div style={{ fontSize: 14, fontWeight: 600 }}>Skip Setup Wizard?</div>
                  <p className="muted" style={{ fontSize: 12, marginTop: 6, lineHeight: 1.55 }}>
                    You can always access the setup wizard later from the settings.
                    Some features may not work correctly until the initial
                    configuration is complete.
                  </p>
                </div>
              </div>
              <div style={{ display: 'flex', gap: 8, marginTop: 16, justifyContent: 'flex-end' }}>
                <button className="btn" onClick={() => setShowSkipConfirm(false)}>
                  Continue Setup
                </button>
                <button className="btn primary" onClick={handleSkipWizard}>
                  Skip for Now
                </button>
              </div>
            </CardBody>
          </Card>
        </div>
      )}

      {/* Header */}
      <div className="page-head">
        <div>
          <h1 className="page-title">Setup Wizard</h1>
          <p className="page-sub">Configure your Vectorizer workspace</p>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {!isComplete && (
            <Pill tone="teal" className="mono">
              Step {currentStepNumber} of {totalSteps}
            </Pill>
          )}
          {!isComplete && (
            <button
              className="btn"
              onClick={() => setShowSkipConfirm(true)}
              title="Skip setup"
            >
              <Icons.x size={14} /> Skip
            </button>
          )}
        </div>
      </div>

      {/* KPI strip */}
      {!isComplete && (
        <div className="grid grid-4" style={{ marginBottom: 14 }}>
          <Kpi accent="teal" label="Step" value={`${currentStepNumber} / ${totalSteps}`} />
          <Kpi label="Completed" value={String(completed)} />
          <Kpi label="Remaining" value={String(remaining)} />
          <Kpi label="Phase" value={STEP_LABELS[currentStep as Exclude<WizardStep, 'complete'>] ?? '—'} />
        </div>
      )}

      {/* Progress dots */}
      <div
        style={{
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          gap: 8, marginBottom: 18, flexWrap: 'wrap',
        }}
      >
        {progressSteps.map((step, i) => {
          const isDone = isComplete || stepIndex > i;
          const isActive = currentStep === step;
          return (
            <div key={step} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div
                title={STEP_LABELS[step]}
                style={{
                  width: 28, height: 28, borderRadius: 999,
                  display: 'grid', placeItems: 'center',
                  fontSize: 11, fontFamily: 'var(--font-mono)',
                  background: isDone ? 'var(--teal-dim)' : isActive ? 'var(--bg-3)' : 'var(--bg-2)',
                  border: `1px solid ${isDone ? 'var(--teal)' : isActive ? 'var(--border-hi)' : 'var(--border)'}`,
                  color: isDone ? 'var(--teal-hi)' : isActive ? 'var(--text)' : 'var(--text-2)',
                  transition: 'all 0.15s',
                }}
              >
                {isDone ? <Icons.check size={14} /> : i + 1}
              </div>
              {i < progressSteps.length - 1 && (
                <div
                  style={{
                    width: 24, height: 1,
                    background: isDone ? 'var(--teal)' : 'var(--border)',
                  }}
                />
              )}
            </div>
          );
        })}
      </div>

      {/* Error Message */}
      {error && (
        <Card className="" >
          <CardBody>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <span style={{ color: 'var(--red)' }}><Icons.bell size={16} /></span>
              <p style={{ margin: 0, fontSize: 13, color: 'var(--text-1)', flex: 1 }}>{error}</p>
              <button className="btn sm" onClick={() => setError(null)} aria-label="Dismiss error">
                <Icons.x size={12} />
              </button>
            </div>
          </CardBody>
        </Card>
      )}
      {error && <div style={{ height: 12 }} />}

      {/* Welcome step */}
      {currentStep === 'welcome' && (
        <Card>
          <CardBody>
            {showResumeBanner && progress.snapshot && (
              <div
                role="region"
                aria-label="Resume previous setup"
                style={{
                  background: 'var(--teal-dim)',
                  border: '1px solid rgba(31,182,182,0.35)',
                  borderRadius: 8,
                  padding: 14,
                  marginBottom: 18,
                }}
              >
                <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--teal-hi)' }}>
                  Resume your previous setup?
                </div>
                <p className="muted" style={{ fontSize: 12, marginTop: 4 }}>
                  Saved {new Date(progress.snapshot.savedAt).toLocaleString()}
                  {' · '}
                  step: <span style={{ fontFamily: 'var(--font-mono)' }}>{progress.snapshot.step}</span>
                  {progress.snapshot.folderPath && (
                    <>
                      {' · '}
                      folder: <span style={{ fontFamily: 'var(--font-mono)' }}>{progress.snapshot.folderPath}</span>
                    </>
                  )}
                </p>
                <div style={{ display: 'flex', gap: 8, marginTop: 10 }}>
                  <button className="btn primary sm" onClick={resumeFromSnapshot}>
                    <Icons.chevron size={12} /> Resume
                  </button>
                  <button className="btn sm" onClick={discardSnapshot}>
                    <Icons.x size={12} /> Start fresh
                  </button>
                </div>
              </div>
            )}

            <div style={{ textAlign: 'center', padding: '24px 12px' }}>
              <div
                style={{
                  width: 64, height: 64, borderRadius: 999,
                  background: 'var(--bg-2)', border: '1px solid var(--border)',
                  display: 'grid', placeItems: 'center', margin: '0 auto 16px',
                  color: 'var(--teal)',
                }}
              >
                <Icons.settings size={28} />
              </div>
              <h2 style={{ fontSize: 18, fontWeight: 600, margin: 0 }}>Welcome to Vectorizer</h2>
              <p className="muted" style={{ fontSize: 13, marginTop: 8, maxWidth: 480, marginLeft: 'auto', marginRight: 'auto' }}>
                Let&apos;s set up your workspace. This wizard will help you configure your projects
                and create collection mappings for vector search.
              </p>

              {status && (
                <div
                  style={{
                    background: 'var(--bg-2)', border: '1px solid var(--border)',
                    borderRadius: 8, padding: 14, margin: '20px auto 0', maxWidth: 380,
                    textAlign: 'left',
                  }}
                >
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6, fontSize: 12 }}>
                    <span className="muted">Version:</span>
                    <span style={{ fontFamily: 'var(--font-mono)' }}>{status.version}</span>
                    <span className="muted">Deployment:</span>
                    <span style={{ textTransform: 'capitalize' }}>{status.deployment_type}</span>
                    <span className="muted">Collections:</span>
                    <span style={{ fontFamily: 'var(--font-mono)' }}>{status.collection_count}</span>
                  </div>
                </div>
              )}

              <div style={{ marginTop: 20 }}>
                <button className="btn primary" onClick={() => setCurrentStep('template')}>
                  Get Started <Icons.chevron size={14} />
                </button>
              </div>
            </div>
          </CardBody>
        </Card>
      )}

      {/* Template step */}
      {currentStep === 'template' && (
        <Card>
          <CardBody>
            <div style={{ marginBottom: 16 }}>
              <h2 style={{ fontSize: 16, fontWeight: 600, margin: 0 }}>Choose a Template</h2>
              <p className="muted" style={{ fontSize: 12, marginTop: 4 }}>
                Select a template that best matches your use case
              </p>
            </div>

            {templatesLoading ? (
              <div style={{ display: 'grid', placeItems: 'center', padding: 32 }}>
                <LoadingSpinner size="lg" />
              </div>
            ) : (
              <>
                {/* Quick Setup Section */}
                <div
                  style={{
                    background: 'var(--bg-2)', border: '1px solid var(--border)',
                    borderRadius: 8, padding: 14, marginBottom: 16,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12 }}>
                    <div style={{
                      width: 40, height: 40, borderRadius: 8,
                      background: 'var(--bg-3)', border: '1px solid var(--border)',
                      display: 'grid', placeItems: 'center', flexShrink: 0,
                      color: 'var(--amber)',
                    }}>
                      <Icons.zap size={18} />
                    </div>
                    <div style={{ flex: 1 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontWeight: 600, fontSize: 13 }}>
                        <span>Quick Setup</span>
                      </div>
                      <p className="muted" style={{ fontSize: 12, marginTop: 4 }}>
                        One-click setup with sensible defaults. Perfect for getting started quickly.
                      </p>
                      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginTop: 10 }}>
                        {templates.filter(t => t.id !== 'custom').slice(0, 3).map((template) => (
                          <button
                            key={template.id}
                            onClick={() => handleQuickSetup(template)}
                            disabled={quickSetupLoading}
                            className="btn sm"
                          >
                            {quickSetupLoading ? (
                              <LoadingSpinner size="sm" />
                            ) : (
                              <span style={{ display: 'inline-flex' }}>{getTemplateIcon(template.id)}</span>
                            )}
                            {template.name.split(' ')[0]}
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>

                <div style={{
                  display: 'flex', alignItems: 'center', gap: 8,
                  margin: '14px 0', color: 'var(--text-3)', fontSize: 11,
                }}>
                  <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
                  <span>Or customize your setup</span>
                  <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
                </div>

                <div className="grid grid-2">
                  {templates.map((template) => {
                    const isSelected = selectedTemplate?.id === template.id;
                    return (
                      <button
                        key={template.id}
                        onClick={() => handleTemplateSelect(template)}
                        style={{
                          textAlign: 'left',
                          padding: 14,
                          borderRadius: 8,
                          border: `1px solid ${isSelected ? 'var(--teal)' : 'var(--border)'}`,
                          background: isSelected ? 'var(--teal-dim)' : 'var(--bg-2)',
                          color: 'var(--text)',
                          cursor: 'pointer',
                          transition: 'all 0.1s',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12 }}>
                          <div style={{
                            width: 40, height: 40, borderRadius: 8,
                            background: 'var(--bg-3)', border: '1px solid var(--border)',
                            display: 'grid', placeItems: 'center', flexShrink: 0,
                            color: 'var(--teal)',
                          }}>
                            {getTemplateIcon(template.id)}
                          </div>
                          <div style={{ flex: 1 }}>
                            <div style={{ fontWeight: 600, fontSize: 13 }}>{template.name}</div>
                            <p className="muted" style={{ fontSize: 12, marginTop: 4 }}>
                              {template.description}
                            </p>
                            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4, marginTop: 8 }}>
                              {template.use_cases.slice(0, 2).map((useCase, i) => (
                                <Pill key={i} tone="muted">{useCase}</Pill>
                              ))}
                            </div>
                          </div>
                        </div>
                      </button>
                    );
                  })}
                </div>
              </>
            )}

            <div style={{
              display: 'flex', justifyContent: 'space-between',
              paddingTop: 14, marginTop: 16,
              borderTop: '1px solid var(--border)',
            }}>
              <button className="btn" onClick={() => setCurrentStep('welcome')}>
                <Icons.arrowDown size={12} style={{ transform: 'rotate(90deg)' }} /> Back
              </button>
            </div>
          </CardBody>
        </Card>
      )}

      {/* Folder step */}
      {currentStep === 'folder' && (
        <Card>
          <CardBody>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
              <div style={{
                width: 40, height: 40, borderRadius: 8,
                background: 'var(--bg-2)', border: '1px solid var(--border)',
                display: 'grid', placeItems: 'center', color: 'var(--teal)',
              }}>
                <Icons.layers size={18} />
              </div>
              <div>
                <h2 style={{ fontSize: 16, fontWeight: 600, margin: 0 }}>Add Project Folder</h2>
                <p className="muted" style={{ fontSize: 12, marginTop: 2 }}>
                  {selectedTemplate
                    ? `Using "${selectedTemplate.name}" template`
                    : 'Select or enter the path to your project folder'}
                </p>
              </div>
            </div>

            <div className="field">
              <label className="field-label" htmlFor="setup-folder-path">Folder Path</label>
              <div style={{ display: 'flex', gap: 8 }}>
                <div style={{ position: 'relative', flex: 1 }}>
                  <input
                    id="setup-folder-path"
                    type="text"
                    value={folderPath}
                    onChange={(e) => setFolderPath(e.target.value)}
                    placeholder="/path/to/your/project"
                    aria-invalid={pathValidation.isValid === false}
                    className="input"
                    style={{
                      paddingRight: 36,
                      borderColor:
                        pathValidation.isValid === true && pathValidation.isProject ? 'var(--green)'
                        : pathValidation.isValid === true ? 'var(--amber)'
                        : pathValidation.isValid === false ? 'var(--red)'
                        : undefined,
                    }}
                    onKeyDown={(e) => e.key === 'Enter' && handleAnalyze()}
                  />
                  <div style={{ position: 'absolute', right: 10, top: '50%', transform: 'translateY(-50%)' }}>
                    {pathValidation.isValidating ? (
                      <LoadingSpinner size="sm" />
                    ) : pathValidation.isValid === true ? (
                      pathValidation.isProject ? (
                        <span style={{ color: 'var(--green)' }}><Icons.check size={16} /></span>
                      ) : (
                        <span style={{ color: 'var(--amber)' }}><Icons.bell size={16} /></span>
                      )
                    ) : pathValidation.isValid === false ? (
                      <span style={{ color: 'var(--red)' }}><Icons.x size={16} /></span>
                    ) : null}
                  </div>
                </div>
                <button className="btn" onClick={() => setShowFileBrowser(true)} title="Browse folders">
                  <Icons.search size={14} />
                </button>
                <button
                  className="btn primary"
                  onClick={handleAnalyze}
                  disabled={analyzing || !folderPath.trim() || pathValidation.isValid === false}
                >
                  {analyzing ? <LoadingSpinner size="sm" /> : 'Analyze'}
                </button>
              </div>

              {pathValidation.isValid === true && pathValidation.isProject && pathValidation.projectInfo && (
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginTop: 6 }}>
                  {pathValidation.projectInfo.hasGit && (
                    <Pill tone="green"><Icons.check size={10} /> Git repository</Pill>
                  )}
                  {pathValidation.projectInfo.hasPackageJson && (
                    <Pill tone="green"><Icons.check size={10} /> Node.js project</Pill>
                  )}
                  {pathValidation.projectInfo.hasCargoToml && (
                    <Pill tone="green"><Icons.check size={10} /> Rust project</Pill>
                  )}
                  {pathValidation.projectInfo.hasPyProject && (
                    <Pill tone="green"><Icons.check size={10} /> Python project</Pill>
                  )}
                </div>
              )}

              {pathValidation.isValid === true && !pathValidation.isProject && (
                <p style={{ marginTop: 6, fontSize: 11, color: 'var(--amber)', display: 'flex', alignItems: 'center', gap: 4 }} role="status">
                  <Icons.bell size={12} />
                  Path is valid but no project files detected. You can still analyze it.
                </p>
              )}

              {pathValidation.isValid === false && pathValidation.error && (
                <p style={{ marginTop: 6, fontSize: 11, color: 'var(--red)', display: 'flex', alignItems: 'center', gap: 4 }} role="alert">
                  <Icons.x size={12} />
                  {pathValidation.error}
                </p>
              )}

              {!folderPath && (
                <p className="muted" style={{ fontSize: 11, marginTop: 4 }}>
                  Click the folder icon to browse, or type a path directly
                </p>
              )}
            </div>

            {analyzedProjects.length > 0 && (
              <div style={{ borderTop: '1px solid var(--border)', paddingTop: 14, marginTop: 18 }}>
                <div className="field-label" style={{ marginBottom: 8 }}>
                  Analyzed Projects ({analyzedProjects.length})
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  {analyzedProjects.map((p, i) => (
                    <div
                      key={i}
                      style={{
                        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                        padding: 10, background: 'var(--bg-2)',
                        border: '1px solid var(--border)', borderRadius: 6,
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                        <span style={{ color: 'var(--green)' }}><Icons.check size={16} /></span>
                        <div>
                          <div style={{ fontSize: 13, fontWeight: 500 }}>{p.analysis.project_name}</div>
                          <div className="muted" style={{ fontSize: 11 }}>
                            {p.analysis.project_types.join(', ')} · {p.collections.length} collections
                          </div>
                        </div>
                      </div>
                      <button className="btn sm" onClick={() => removeProject(i)}>Remove</button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div style={{
              display: 'flex', justifyContent: 'space-between',
              paddingTop: 14, marginTop: 18,
              borderTop: '1px solid var(--border)',
            }}>
              <button className="btn" onClick={() => setCurrentStep('template')}>
                <Icons.arrowDown size={12} style={{ transform: 'rotate(90deg)' }} /> Back
              </button>
              {analyzedProjects.length > 0 && (
                <button className="btn primary" onClick={() => setCurrentStep('analysis')}>
                  Continue <Icons.chevron size={14} />
                </button>
              )}
            </div>
          </CardBody>
        </Card>
      )}

      {/* Analysis step */}
      {currentStep === 'analysis' && (
        <Card>
          <CardBody>
            <h2 style={{ fontSize: 16, fontWeight: 600, margin: '0 0 14px' }}>Review Detected Projects</h2>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {analyzedProjects.map((project, pi) => (
                <div
                  key={pi}
                  style={{
                    border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden',
                  }}
                >
                  <div
                    style={{
                      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                      padding: 12, background: 'var(--bg-2)', cursor: 'pointer',
                    }}
                    onClick={() => toggleProjectSelection(pi)}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                      <input
                        type="checkbox"
                        checked={project.selected}
                        onChange={() => toggleProjectSelection(pi)}
                        style={{ accentColor: 'var(--teal)' }}
                      />
                      <div>
                        <div style={{ fontSize: 13, fontWeight: 600 }}>{project.analysis.project_name}</div>
                        <div className="muted" style={{ fontSize: 11, fontFamily: 'var(--font-mono)' }}>
                          {project.analysis.project_path}
                        </div>
                      </div>
                    </div>
                    <div style={{ textAlign: 'right' }}>
                      <div style={{ fontSize: 12 }}>{project.analysis.project_types.join(', ')}</div>
                      <div className="muted" style={{ fontSize: 11 }}>
                        {project.analysis.statistics.total_files} files
                      </div>
                    </div>
                  </div>

                  {project.selected && (
                    <div style={{ padding: 12, display: 'flex', flexDirection: 'column', gap: 8 }}>
                      <div className="field-label">Collections:</div>
                      {project.collections.map((col, ci) => {
                        const validationKey = `${pi}-${ci}`;
                        const validation = collectionValidations[validationKey];
                        const hasError = validation && !validation.isValid;

                        return (
                          <div
                            key={ci}
                            style={{
                              padding: 10, borderRadius: 6,
                              background: hasError ? 'rgba(229,72,77,0.08)' : 'var(--bg-2)',
                              border: `1px solid ${hasError ? 'var(--red)' : 'var(--border)'}`,
                            }}
                          >
                            <div
                              style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer' }}
                              onClick={() => toggleCollectionSelection(pi, ci)}
                            >
                              <input
                                type="checkbox"
                                checked={col.selected}
                                onChange={() => toggleCollectionSelection(pi, ci)}
                                style={{ accentColor: 'var(--teal)' }}
                              />
                              <div style={{ flex: 1 }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                  <span style={{ fontSize: 13, fontWeight: 500, fontFamily: 'var(--font-mono)' }}>
                                    {col.name}
                                  </span>
                                  {col.selected && validation && (
                                    validation.isValid ? (
                                      <span style={{ color: 'var(--green)' }}><Icons.check size={14} /></span>
                                    ) : (
                                      <span style={{ color: 'var(--red)' }}><Icons.x size={14} /></span>
                                    )
                                  )}
                                </div>
                                <div className="muted" style={{ fontSize: 11 }}>{col.description}</div>
                                {col.selected && hasError && validation.error && (
                                  <p style={{ fontSize: 11, color: 'var(--red)', marginTop: 4, display: 'flex', alignItems: 'center', gap: 4 }} role="alert">
                                    <Icons.bell size={10} />
                                    {validation.error}
                                  </p>
                                )}
                              </div>
                              <Pill tone="muted">{col.content_type}</Pill>
                            </div>

                            {/* Graph Relationship Toggle */}
                            {col.selected && (
                              <div style={{ marginTop: 10, paddingTop: 10, borderTop: '1px solid var(--border)' }}>
                                <label
                                  style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer' }}
                                  onClick={(e) => e.stopPropagation()}
                                >
                                  <input
                                    type="checkbox"
                                    checked={col.enable_graph}
                                    onChange={() => toggleCollectionGraph(pi, ci)}
                                    style={{ accentColor: 'var(--teal)' }}
                                  />
                                  <span style={{ color: col.enable_graph ? 'var(--teal)' : 'var(--text-2)' }}>
                                    <Icons.layers size={14} />
                                  </span>
                                  <div style={{ flex: 1 }}>
                                    <div style={{ fontSize: 12, fontWeight: 500, color: col.enable_graph ? 'var(--text)' : 'var(--text-2)' }}>
                                      Enable Graph Relationships
                                    </div>
                                    <div className="muted" style={{ fontSize: 11 }}>
                                      Automatically discover semantic relationships between documents (GraphRAG)
                                    </div>
                                  </div>
                                </label>
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              ))}
            </div>

            {/* Add Another Project Button */}
            <div
              style={{
                border: '1px dashed var(--border-hi)', borderRadius: 8,
                padding: 14, textAlign: 'center', marginTop: 14,
              }}
            >
              <button className="btn" onClick={() => setCurrentStep('folder')}>
                <Icons.plus size={14} /> Add Another Project
              </button>
              <p className="muted" style={{ fontSize: 11, marginTop: 6 }}>
                You can add multiple projects to your workspace
              </p>
            </div>

            <div style={{
              display: 'flex', justifyContent: 'space-between',
              paddingTop: 14, marginTop: 14,
              borderTop: '1px solid var(--border)',
            }}>
              <button className="btn" onClick={() => setCurrentStep('folder')}>
                <Icons.arrowDown size={12} style={{ transform: 'rotate(90deg)' }} /> Back
              </button>
              {(() => {
                const hasValidationErrors = analyzedProjects.some((project, pi) =>
                  project.selected && project.collections.some((col, ci) => {
                    if (!col.selected) return false;
                    const key = `${pi}-${ci}`;
                    const validation = collectionValidations[key];
                    return validation && !validation.isValid;
                  })
                );

                const selectedCount = analyzedProjects.reduce((sum, p) =>
                  sum + (p.selected ? p.collections.filter(c => c.selected).length : 0), 0
                );

                return (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    {hasValidationErrors && (
                      <span style={{ fontSize: 11, color: 'var(--red)', display: 'flex', alignItems: 'center', gap: 4 }} role="alert">
                        <Icons.bell size={12} />
                        Fix validation errors to continue
                      </span>
                    )}
                    <button
                      className="btn primary"
                      onClick={() => setCurrentStep('review')}
                      disabled={hasValidationErrors || selectedCount === 0}
                    >
                      Continue <Icons.chevron size={14} />
                    </button>
                  </div>
                );
              })()}
            </div>
          </CardBody>
        </Card>
      )}

      {/* Review step */}
      {currentStep === 'review' && (
        <Card>
          <CardBody>
            <h2 style={{ fontSize: 16, fontWeight: 600, margin: '0 0 14px' }}>Review &amp; Apply Configuration</h2>

            <div
              style={{
                background: 'var(--bg-2)', border: '1px solid var(--border)',
                borderRadius: 8, padding: 14, marginBottom: 14,
              }}
            >
              <div className="field-label" style={{ marginBottom: 10 }}>Configuration Summary</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6, fontSize: 12 }}>
                {selectedTemplate && (
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span className="muted">Template:</span>
                    <span style={{ fontWeight: 500 }}>{selectedTemplate.name}</span>
                  </div>
                )}
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span className="muted">Projects:</span>
                  <span style={{ fontWeight: 500, fontFamily: 'var(--font-mono)' }}>
                    {analyzedProjects.filter(p => p.selected).length}
                  </span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span className="muted">Collections:</span>
                  <span style={{ fontWeight: 500, fontFamily: 'var(--font-mono)' }}>
                    {analyzedProjects.reduce((sum, p) =>
                      sum + (p.selected ? p.collections.filter(c => c.selected).length : 0), 0
                    )}
                  </span>
                </div>
              </div>
            </div>

            {/* YAML Preview Toggle */}
            <div style={{ border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden', marginBottom: 14 }}>
              <button
                onClick={() => setShowYamlPreview(!showYamlPreview)}
                style={{
                  width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                  padding: 12, background: 'var(--bg-2)', border: 'none', color: 'var(--text)',
                  cursor: 'pointer', fontFamily: 'inherit',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  <span className="muted"><Icons.copy size={14} /></span>
                  <span style={{ fontWeight: 500, fontSize: 13 }}>Preview workspace.yml</span>
                </div>
                <span className="muted" style={{ fontSize: 11 }}>
                  {showYamlPreview ? 'Hide' : 'Show'}
                </span>
              </button>

              {showYamlPreview && (
                <div style={{ position: 'relative' }}>
                  <button
                    onClick={handleCopyYaml}
                    className="btn sm"
                    style={{ position: 'absolute', top: 8, right: 8, zIndex: 1 }}
                    title="Copy to clipboard"
                  >
                    <Icons.copy size={12} /> {yamlCopied ? 'Copied!' : 'Copy'}
                  </button>
                  <pre
                    style={{
                      padding: 14, background: 'var(--bg)', color: 'var(--text-1)',
                      fontSize: 12, fontFamily: 'var(--font-mono)',
                      overflowX: 'auto', maxHeight: 320, margin: 0,
                    }}
                  >
                    {yamlPreview}
                  </pre>
                </div>
              )}
            </div>

            <div
              style={{
                background: 'var(--amber-dim)', border: '1px solid rgba(240,168,58,0.35)',
                borderRadius: 8, padding: 12, marginBottom: 14,
              }}
            >
              <p style={{ margin: 0, fontSize: 12, color: 'var(--amber)' }}>
                <strong>Note:</strong> This will create a workspace.yml file in your Vectorizer directory.
                The server may need to be restarted to apply changes.
              </p>
            </div>

            <div style={{
              display: 'flex', justifyContent: 'space-between',
              paddingTop: 14, borderTop: '1px solid var(--border)',
            }}>
              <button className="btn" onClick={() => setCurrentStep('analysis')}>
                <Icons.arrowDown size={12} style={{ transform: 'rotate(90deg)' }} /> Back
              </button>
              <button className="btn primary" onClick={handleApplyConfig} disabled={applying}>
                {applying ? (
                  <>
                    <LoadingSpinner size="sm" />
                    <span>Applying...</span>
                  </>
                ) : (
                  <>Apply Configuration</>
                )}
              </button>
            </div>
          </CardBody>
        </Card>
      )}

      {/* API Key step */}
      {currentStep === 'api-key' && (
        <Card>
          <CardBody>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
              <div style={{
                width: 40, height: 40, borderRadius: 999,
                background: 'var(--bg-2)', border: '1px solid var(--border)',
                display: 'grid', placeItems: 'center', color: 'var(--teal)',
              }}>
                <Icons.keys size={18} />
              </div>
              <div>
                <h2 style={{ fontSize: 16, fontWeight: 600, margin: 0 }}>Create API Key for Cursor MCP</h2>
                <p className="muted" style={{ fontSize: 12, marginTop: 2 }}>
                  Generate an API key to integrate Vectorizer with Cursor IDE
                </p>
              </div>
            </div>

            {!apiKeyCreated ? (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div className="field">
                  <label className="field-label" htmlFor="setup-api-key-name">API Key Name</label>
                  <input
                    id="setup-api-key-name"
                    type="text"
                    value={apiKeyName}
                    onChange={(e) => setApiKeyName(e.target.value)}
                    placeholder="e.g., Cursor MCP Integration"
                    className="input"
                    disabled={creatingKey}
                  />
                </div>

                <div
                  style={{
                    background: 'var(--teal-dim)', border: '1px solid rgba(31,182,182,0.35)',
                    borderRadius: 8, padding: 12,
                  }}
                >
                  <p style={{ margin: 0, fontSize: 12, color: 'var(--teal-hi)' }}>
                    <strong>Permissions:</strong> This API key will have read, write, and collection management permissions to enable full MCP functionality.
                  </p>
                </div>

                <div style={{ display: 'flex', gap: 8 }}>
                  <button
                    className="btn primary"
                    onClick={handleCreateApiKey}
                    disabled={creatingKey || !apiKeyName.trim()}
                    style={{ flex: 1 }}
                  >
                    {creatingKey ? (
                      <>
                        <LoadingSpinner size="sm" />
                        <span>Creating...</span>
                      </>
                    ) : (
                      <>Create API Key</>
                    )}
                  </button>
                  <button
                    className="btn"
                    onClick={() => setCurrentStep('complete')}
                    disabled={creatingKey}
                  >
                    Skip
                  </button>
                </div>
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div
                  style={{
                    background: 'rgba(76,195,138,0.10)', border: '1px solid rgba(76,195,138,0.35)',
                    borderRadius: 8, padding: 12,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
                    <span style={{ color: 'var(--green)' }}><Icons.check size={16} /></span>
                    <p style={{ margin: 0, fontSize: 13, fontWeight: 600, color: 'var(--green)' }}>
                      API Key Created Successfully!
                    </p>
                  </div>
                  <p style={{ margin: 0, fontSize: 11, color: 'var(--text-2)' }}>
                    Save this API key now — it will not be shown again.
                  </p>
                </div>

                <div className="field">
                  <label className="field-label" htmlFor="setup-api-key-output">Your API Key</label>
                  <div style={{ display: 'flex', gap: 8 }}>
                    <input
                      id="setup-api-key-output"
                      type="text"
                      value={apiKey || ''}
                      readOnly
                      className="input mono"
                    />
                    <button
                      className="btn"
                      onClick={async () => {
                        if (apiKey) {
                          await navigator.clipboard.writeText(apiKey);
                          setMcpConfigCopied(true);
                          setTimeout(() => setMcpConfigCopied(false), 2000);
                        }
                      }}
                    >
                      <Icons.copy size={14} />
                    </button>
                  </div>
                </div>

                <div className="field">
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <label className="field-label" htmlFor="setup-mcp-config-type">
                      Cursor MCP Configuration
                    </label>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <select
                        id="setup-mcp-config-type"
                        value={mcpConfigType}
                        onChange={(e) => setMcpConfigType(e.target.value as 'npx' | 'streamablehttp')}
                        className="input"
                        style={{ width: 'auto', fontSize: 12, padding: '4px 8px' }}
                      >
                        <option value="npx">NPX (Recommended)</option>
                        <option value="streamablehttp">StreamableHTTP (Direct)</option>
                      </select>
                      <button className="btn sm" onClick={copyMcpConfig}>
                        {mcpConfigCopied ? (
                          <><Icons.check size={12} /> Copied!</>
                        ) : (
                          <><Icons.copy size={12} /> Copy Config</>
                        )}
                      </button>
                    </div>
                  </div>
                  <pre
                    style={{
                      background: 'var(--bg)', color: 'var(--text-1)',
                      border: '1px solid var(--border)', borderRadius: 6,
                      padding: 12, fontSize: 11, fontFamily: 'var(--font-mono)',
                      overflowX: 'auto', margin: 0,
                    }}
                  >
                    {generateMcpConfig()}
                  </pre>
                  {mcpConfigType === 'streamablehttp' && (
                    <div
                      style={{
                        background: 'var(--amber-dim)', border: '1px solid rgba(240,168,58,0.35)',
                        borderRadius: 6, padding: 10, marginTop: 8,
                      }}
                    >
                      <p style={{ margin: 0, fontSize: 11, color: 'var(--amber)' }}>
                        <strong>Note:</strong> StreamableHTTP configuration may require manual header setup.
                        Use the NPX option for automatic authentication.
                      </p>
                    </div>
                  )}
                  <div
                    style={{
                      background: 'var(--teal-dim)', border: '1px solid rgba(31,182,182,0.35)',
                      borderRadius: 6, padding: 10, marginTop: 8,
                    }}
                  >
                    <p style={{ margin: '0 0 6px', fontSize: 11, color: 'var(--teal-hi)' }}>
                      <strong>Instructions:</strong>
                    </p>
                    <ol style={{ margin: 0, paddingLeft: 16, fontSize: 11, color: 'var(--text-1)', display: 'flex', flexDirection: 'column', gap: 4 }}>
                      <li>Copy the configuration above</li>
                      <li>
                        Open or create the MCP configuration file:{' '}
                        <code style={{ background: 'var(--bg)', padding: '1px 5px', borderRadius: 3, fontFamily: 'var(--font-mono)' }}>
                          ~/.cursor/mcp.json
                        </code>
                        {' '}or{' '}
                        <code style={{ background: 'var(--bg)', padding: '1px 5px', borderRadius: 3, fontFamily: 'var(--font-mono)' }}>
                          .cursor/mcp.json
                        </code>
                        {' '}(project root)
                      </li>
                      <li>
                        Merge the configuration into your existing{' '}
                        <code style={{ background: 'var(--bg)', padding: '1px 5px', borderRadius: 3, fontFamily: 'var(--font-mono)' }}>mcpServers</code>{' '}
                        object
                      </li>
                      <li>Restart Cursor IDE to apply the changes</li>
                    </ol>
                  </div>
                </div>

                <div style={{ display: 'flex', gap: 8 }}>
                  <button
                    className="btn primary"
                    onClick={() => setCurrentStep('complete')}
                    style={{ flex: 1 }}
                  >
                    Continue to Dashboard <Icons.chevron size={14} />
                  </button>
                </div>
              </div>
            )}
          </CardBody>
        </Card>
      )}

      {/* Complete step */}
      {currentStep === 'complete' && (
        <Card>
          <CardBody>
            <div style={{ textAlign: 'center', padding: '24px 12px' }}>
              <div
                style={{
                  width: 72, height: 72, borderRadius: 999,
                  background: 'var(--teal-dim)', border: '1px solid var(--teal)',
                  display: 'grid', placeItems: 'center', margin: '0 auto 16px',
                  color: 'var(--teal-hi)',
                }}
              >
                <Icons.check size={32} />
              </div>

              <h2 style={{ fontSize: 20, fontWeight: 600, margin: 0 }}>Setup Complete!</h2>
              <p className="muted" style={{ fontSize: 13, marginTop: 6 }}>
                Your workspace has been configured successfully.
              </p>

              <div className="grid grid-3" style={{ maxWidth: 420, margin: '20px auto 0' }}>
                <Kpi
                  accent="teal"
                  label="Projects"
                  value={String(analyzedProjects.filter(p => p.selected).length)}
                />
                <Kpi
                  label="Collections"
                  value={String(
                    analyzedProjects.reduce((sum, p) =>
                      sum + (p.selected ? p.collections.filter(c => c.selected).length : 0), 0
                    )
                  )}
                />
                <Kpi
                  label="Template"
                  value={selectedTemplate?.name.split(' ')[0] || 'Custom'}
                />
              </div>

              <div
                style={{
                  background: 'var(--bg-2)', border: '1px solid var(--border)',
                  borderRadius: 8, padding: 14, margin: '20px auto 0', maxWidth: 460,
                  textAlign: 'left',
                }}
              >
                <div className="field-label" style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 8 }}>
                  <span style={{ color: 'var(--teal)' }}><Icons.chevron size={12} /></span>
                  What&apos;s Next:
                </div>
                <ul style={{ margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {[
                    'Restart the server to apply workspace configuration',
                    'Visit the Workspace page to manage projects',
                    'Use the Search page to query your data',
                  ].map((item, i) => (
                    <li key={i} style={{ display: 'flex', alignItems: 'center', gap: 10, fontSize: 12, color: 'var(--text-1)' }}>
                      <span
                        style={{
                          width: 20, height: 20, borderRadius: 999,
                          background: 'var(--bg-3)', border: '1px solid var(--border)',
                          display: 'inline-grid', placeItems: 'center',
                          fontSize: 10, color: 'var(--teal)', fontFamily: 'var(--font-mono)',
                        }}
                      >
                        {i + 1}
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>

              <div style={{ display: 'flex', gap: 8, justifyContent: 'center', marginTop: 20 }}>
                <button className="btn" onClick={() => navigate('/workspace')}>Go to Workspace</button>
                <button className="btn primary" onClick={() => navigate('/overview')}>
                  Go to Dashboard
                </button>
              </div>
            </div>
          </CardBody>
        </Card>
      )}

      {/* File Browser Modal */}
      {showFileBrowser && (
        <FileBrowser
          initialPath={folderPath || ''}
          onSelect={(path) => {
            setFolderPath(path);
            setShowFileBrowser(false);
          }}
          onCancel={() => setShowFileBrowser(false)}
        />
      )}
    </div>
  );
}

export default SetupWizardPage;
