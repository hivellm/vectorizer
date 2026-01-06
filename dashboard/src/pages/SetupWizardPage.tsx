/**
 * Setup Wizard Page
 * Multi-step wizard for initial project setup with template selection
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSetup, SetupStatus, ProjectAnalysis, SetupProject, SuggestedCollection } from '@/hooks/useSetup';
import { useTemplates, ConfigTemplate, getTemplateIcon, getTemplateColor } from '@/hooks/useTemplates.tsx';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import LoadingSpinner from '@/components/LoadingSpinner';
import { CheckCircle, Folder, Settings02, AlertCircle, ArrowRight, ArrowLeft, File06, Copy01, Zap, FolderSearch, XCircle, AlertTriangle, Plus, XClose, Share07 } from '@untitledui/icons';
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

type WizardStep = 'welcome' | 'template' | 'folder' | 'analysis' | 'review' | 'complete';

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

function SetupWizardPage() {
  const navigate = useNavigate();
  const { getStatus, analyzeDirectory, applyConfig } = useSetup();
  const { templates, loading: templatesLoading } = useTemplates();

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
      const response = await fetch('/setup/browse', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
      });

      const data = await response.json();

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
  }, []);

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

  // Update collection validation when collections change
  useEffect(() => {
    const validations: Record<string, CollectionValidation> = {};
    analyzedProjects.forEach((project, pi) => {
      project.collections.forEach((col, ci) => {
        const key = `${pi}-${ci}`;
        validations[key] = validateCollectionName(col.name);
      });
    });
    setCollectionValidations(validations);
  }, [analyzedProjects, validateCollectionName]);

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
      // Use current working directory (from server) or a default
      const cwdResponse = await fetch('/health');
      const cwdData = await cwdResponse.json();
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

      setCurrentStep('complete');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to apply configuration');
    } finally {
      setApplying(false);
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
  const progressSteps = ['welcome', 'template', 'folder', 'analysis', 'review'];
  const stepIndex = progressSteps.indexOf(currentStep);
  const isComplete = currentStep === 'complete';

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Skip Confirmation Modal */}
      {showSkipConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
          <div className="bg-white dark:bg-neutral-900 rounded-xl shadow-2xl max-w-md w-full p-6 animate-fade-in-up">
            <div className="flex items-start gap-4">
              <div className="w-12 h-12 bg-yellow-100 dark:bg-yellow-900/30 rounded-full flex items-center justify-center flex-shrink-0">
                <AlertTriangle className="w-6 h-6 text-yellow-600 dark:text-yellow-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-lg font-semibold text-white">
                  Skip Setup Wizard?
                </h3>
                <p className="text-sm text-white/60 mt-2">
                  You can always access the setup wizard later from the settings. However, some features
                  may not work correctly until you complete the initial configuration.
                </p>
              </div>
            </div>
            <div className="flex gap-3 mt-6 justify-end">
              <Button
                variant="secondary"
                onClick={() => setShowSkipConfirm(false)}
              >
                Continue Setup
              </Button>
              <Button
                variant="primary"
                onClick={handleSkipWizard}
                className="!bg-yellow-600 hover:!bg-yellow-700"
              >
                Skip for Now
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Header with Skip Button */}
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <h1 className="text-2xl sm:text-3xl font-bold text-white">
            Setup Wizard
          </h1>
          <p className="text-white/60 mt-2">
            Configure your Vectorizer workspace
          </p>
        </div>
        {!isComplete && (
          <button
            onClick={() => setShowSkipConfirm(true)}
            className="flex items-center gap-2 px-3 py-2 text-sm text-white/50 hover:text-white hover:bg-white/10 rounded-lg transition-colors"
          >
            <XClose className="w-4 h-4" />
            <span className="hidden sm:inline">Skip Setup</span>
          </button>
        )}
      </div>

      {/* Progress Steps */}
      <div className="flex items-center justify-center gap-2 sm:gap-4">
        {progressSteps.map((step, i) => (
          <div key={step} className="flex items-center">
            <div className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold transition-all duration-300 ${isComplete || stepIndex > i
              ? 'bg-green-500 text-white shadow-lg shadow-green-500/30'
              : currentStep === step
                ? 'bg-gradient-to-br from-primary-500 to-indigo-600 text-white shadow-lg shadow-primary-500/30 scale-110'
                : 'bg-white/10 text-white/40 border border-white/10'
              }`}>
              {isComplete || stepIndex > i ? (
                <CheckCircle className="w-5 h-5" />
              ) : i + 1}
            </div>
            {i < progressSteps.length - 1 && (
              <div className={`w-8 sm:w-12 h-0.5 transition-all duration-300 ${isComplete || stepIndex > i
                ? 'bg-green-500'
                : 'bg-white/10'
                }`} />
            )}
          </div>
        ))}
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
          <button onClick={() => setError(null)} className="ml-auto text-red-500 hover:text-red-700">×</button>
        </div>
      )}

      {/* Step Content */}
      {currentStep === 'welcome' && (
        <Card className="bg-white/10 backdrop-blur-xl border border-white/10 shadow-2xl shadow-black/20 p-6">
          <div className="text-center space-y-6">
            <div className="w-20 h-20 bg-primary-100 dark:bg-primary-900/30 rounded-full flex items-center justify-center mx-auto">
              <Settings02 className="w-10 h-10 text-primary-600 dark:text-primary-400" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-white">
                Welcome to Vectorizer
              </h2>
              <p className="text-white/60 mt-2 max-w-md mx-auto">
                Let&apos;s set up your workspace. This wizard will help you configure your projects
                and create collection mappings for vector search.
              </p>
            </div>

            {status && (
              <div className="bg-white/5 rounded-lg p-4 text-left max-w-md mx-auto">
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <span className="text-white/50">Version:</span>
                  <span className="text-white font-medium">{status.version}</span>
                  <span className="text-white/50">Deployment:</span>
                  <span className="text-white font-medium capitalize">{status.deployment_type}</span>
                  <span className="text-white/50">Collections:</span>
                  <span className="text-white font-medium">{status.collection_count}</span>
                </div>
              </div>
            )}

            <Button variant="primary" size="lg" onClick={() => setCurrentStep('template')}>
              Get Started <ArrowRight className="w-4 h-4 ml-2" />
            </Button>
          </div>
        </Card>
      )}

      {currentStep === 'template' && (
        <Card className="bg-white/10 backdrop-blur-xl border border-white/10 shadow-2xl shadow-black/20 p-6">
          <div className="space-y-6">
            <div className="text-center">
              <h2 className="text-xl font-semibold text-white">
                Choose a Template
              </h2>
              <p className="text-white/60 text-sm mt-1">
                Select a template that best matches your use case
              </p>
            </div>

            {templatesLoading ? (
              <div className="flex justify-center py-8">
                <LoadingSpinner size="lg" />
              </div>
            ) : (
              <>
                {/* Quick Setup Section */}
                <div className="bg-gradient-to-r from-primary-50 to-indigo-50 dark:from-primary-900/20 dark:to-indigo-900/20 rounded-xl p-4 border border-primary-200 dark:border-primary-800">
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 bg-primary-600 rounded-lg flex items-center justify-center flex-shrink-0">
                      <Zap className="w-6 h-6 text-white" />
                    </div>
                    <div className="flex-1">
                      <h3 className="font-semibold text-white flex items-center gap-2">
                        <Zap className="w-4 h-4 text-primary-500" />
                        Quick Setup
                      </h3>
                      <p className="text-sm text-white/60 mt-1">
                        One-click setup with sensible defaults. Perfect for getting started quickly.
                      </p>
                      <div className="flex flex-wrap gap-2 mt-3">
                        {templates.filter(t => t.id !== 'custom').slice(0, 3).map((template) => (
                          <button
                            key={template.id}
                            onClick={() => handleQuickSetup(template)}
                            disabled={quickSetupLoading}
                            className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium bg-white dark:bg-neutral-800 border border-white/10 rounded-lg hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors disabled:opacity-50"
                          >
                            {quickSetupLoading ? (
                              <LoadingSpinner size="sm" />
                            ) : (
                              <span className="flex items-center">{getTemplateIcon(template.id)}</span>
                            )}
                            {template.name.split(' ')[0]}
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>

                <div className="relative">
                  <div className="absolute inset-0 flex items-center">
                    <div className="w-full border-t border-white/10"></div>
                  </div>
                  <div className="relative flex justify-center">
                    <span className="px-3 bg-white dark:bg-neutral-900 text-sm text-white/50">
                      Or customize your setup
                    </span>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {templates.map((template) => (
                    <button
                      key={template.id}
                      onClick={() => handleTemplateSelect(template)}
                      className={`text-left p-4 rounded-lg border-2 transition-all hover:shadow-md ${selectedTemplate?.id === template.id
                        ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                        : 'border-white/10 hover:border-primary-300'
                        }`}
                    >
                      <div className="flex items-start gap-4">
                        <div className={`w-12 h-12 rounded-lg flex items-center justify-center ${getTemplateColor(template.id)}`}>
                          {getTemplateIcon(template.id)}
                        </div>
                        <div className="flex-1">
                          <h3 className="font-semibold text-white">
                            {template.name}
                          </h3>
                          <p className="text-sm text-white/60 mt-1">
                            {template.description}
                          </p>
                          <div className="flex flex-wrap gap-1 mt-2">
                            {template.use_cases.slice(0, 2).map((useCase, i) => (
                              <span
                                key={i}
                                className="text-xs px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 rounded-full text-white/60"
                              >
                                {useCase}
                              </span>
                            ))}
                          </div>
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              </>
            )}

            <div className="flex justify-between pt-4 border-t border-white/10">
              <Button variant="secondary" onClick={() => setCurrentStep('welcome')}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
            </div>
          </div>
        </Card>
      )}

      {currentStep === 'folder' && (
        <Card className="bg-white/10 backdrop-blur-xl border border-white/10 shadow-2xl shadow-black/20 p-6">
          <div className="space-y-6">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 bg-primary-100 dark:bg-primary-900/30 rounded-lg flex items-center justify-center">
                <Folder className="w-6 h-6 text-primary-600 dark:text-primary-400" />
              </div>
              <div>
                <h2 className="text-xl font-semibold text-white">
                  Add Project Folder
                </h2>
                <p className="text-white/60 text-sm">
                  {selectedTemplate
                    ? `Using "${selectedTemplate.name}" template`
                    : 'Select or enter the path to your project folder'}
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-white/70 mb-2">
                  Folder Path
                </label>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <input
                      type="text"
                      value={folderPath}
                      onChange={(e) => setFolderPath(e.target.value)}
                      placeholder="/path/to/your/project"
                      className={`w-full px-4 py-2 pr-10 border rounded-lg 
                               bg-white dark:bg-neutral-800 text-white
                               focus:ring-2 focus:ring-primary-500 focus:border-primary-500
                               ${pathValidation.isValid === true
                          ? 'border-green-500 dark:border-green-500'
                          : pathValidation.isValid === false
                            ? 'border-red-500 dark:border-red-500'
                            : 'border-neutral-300 dark:border-neutral-700'}`}
                      onKeyDown={(e) => e.key === 'Enter' && handleAnalyze()}
                    />
                    {/* Validation indicator */}
                    <div className="absolute right-3 top-1/2 -translate-y-1/2">
                      {pathValidation.isValidating ? (
                        <LoadingSpinner size="sm" />
                      ) : pathValidation.isValid === true ? (
                        pathValidation.isProject ? (
                          <CheckCircle className="w-5 h-5 text-green-500" />
                        ) : (
                          <AlertTriangle className="w-5 h-5 text-yellow-500" />
                        )
                      ) : pathValidation.isValid === false ? (
                        <XCircle className="w-5 h-5 text-red-500" />
                      ) : null}
                    </div>
                  </div>
                  <Button
                    variant="secondary"
                    onClick={() => setShowFileBrowser(true)}
                    title="Browse folders"
                  >
                    <FolderSearch className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="primary"
                    onClick={handleAnalyze}
                    disabled={analyzing || !folderPath.trim() || pathValidation.isValid === false}
                  >
                    {analyzing ? <LoadingSpinner size="sm" /> : 'Analyze'}
                  </Button>
                </div>

                {/* Validation feedback */}
                {pathValidation.isValid === true && pathValidation.isProject && pathValidation.projectInfo && (
                  <div className="mt-2 flex flex-wrap gap-2">
                    {pathValidation.projectInfo.hasGit && (
                      <span className="inline-flex items-center gap-1 px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 text-xs rounded-full">
                        <CheckCircle className="w-3 h-3" /> Git repository
                      </span>
                    )}
                    {pathValidation.projectInfo.hasPackageJson && (
                      <span className="inline-flex items-center gap-1 px-2 py-1 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 text-xs rounded-full">
                        <CheckCircle className="w-3 h-3" /> Node.js project
                      </span>
                    )}
                    {pathValidation.projectInfo.hasCargoToml && (
                      <span className="inline-flex items-center gap-1 px-2 py-1 bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400 text-xs rounded-full">
                        <CheckCircle className="w-3 h-3" /> Rust project
                      </span>
                    )}
                    {pathValidation.projectInfo.hasPyProject && (
                      <span className="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 text-xs rounded-full">
                        <CheckCircle className="w-3 h-3" /> Python project
                      </span>
                    )}
                  </div>
                )}

                {pathValidation.isValid === true && !pathValidation.isProject && (
                  <p className="mt-2 text-xs text-yellow-600 dark:text-yellow-400 flex items-center gap-1">
                    <AlertTriangle className="w-3 h-3" />
                    Path is valid but no project files detected. You can still analyze it.
                  </p>
                )}

                {pathValidation.isValid === false && pathValidation.error && (
                  <p className="mt-2 text-xs text-red-600 dark:text-red-400 flex items-center gap-1">
                    <XCircle className="w-3 h-3" />
                    {pathValidation.error}
                  </p>
                )}

                {!folderPath && (
                  <p className="text-xs text-white/50 mt-1">
                    Click the folder icon to browse, or type a path directly
                  </p>
                )}
              </div>

              {analyzedProjects.length > 0 && (
                <div className="border-t border-white/10 pt-4">
                  <h3 className="text-sm font-medium text-white/70 mb-2">
                    Analyzed Projects ({analyzedProjects.length})
                  </h3>
                  <div className="space-y-2">
                    {analyzedProjects.map((p, i) => (
                      <div key={i} className="flex items-center justify-between p-3 bg-white/5 rounded-lg">
                        <div className="flex items-center gap-3">
                          <CheckCircle className="w-5 h-5 text-green-500" />
                          <div>
                            <p className="text-sm font-medium text-white">
                              {p.analysis.project_name}
                            </p>
                            <p className="text-xs text-white/50">
                              {p.analysis.project_types.join(', ')} • {p.collections.length} collections
                            </p>
                          </div>
                        </div>
                        <button
                          onClick={() => removeProject(i)}
                          className="text-red-500 hover:text-red-700 text-sm"
                        >
                          Remove
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="flex justify-between pt-4 border-t border-white/10">
              <Button variant="secondary" onClick={() => setCurrentStep('template')}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              {analyzedProjects.length > 0 && (
                <Button variant="primary" onClick={() => setCurrentStep('analysis')}>
                  Continue <ArrowRight className="w-4 h-4 ml-2" />
                </Button>
              )}
            </div>
          </div>
        </Card>
      )}

      {currentStep === 'analysis' && (
        <Card className="bg-white/10 backdrop-blur-xl border border-white/10 shadow-2xl shadow-black/20 p-6">
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-white">
              Review Detected Projects
            </h2>

            {analyzedProjects.map((project, pi) => (
              <div key={pi} className="border border-white/10 rounded-lg overflow-hidden">
                <div
                  className="flex items-center justify-between p-4 bg-white/5 cursor-pointer"
                  onClick={() => toggleProjectSelection(pi)}
                >
                  <div className="flex items-center gap-3">
                    <input
                      type="checkbox"
                      checked={project.selected}
                      onChange={() => toggleProjectSelection(pi)}
                      className="w-4 h-4 text-primary-600 rounded"
                    />
                    <div>
                      <p className="font-medium text-white">
                        {project.analysis.project_name}
                      </p>
                      <p className="text-sm text-white/50">
                        {project.analysis.project_path}
                      </p>
                    </div>
                  </div>
                  <div className="text-right text-sm">
                    <p className="text-white/60">
                      {project.analysis.project_types.join(', ')}
                    </p>
                    <p className="text-neutral-500 dark:text-neutral-500">
                      {project.analysis.statistics.total_files} files
                    </p>
                  </div>
                </div>

                {project.selected && (
                  <div className="p-4 space-y-3">
                    <p className="text-sm font-medium text-white/70">
                      Collections:
                    </p>
                    {project.collections.map((col, ci) => {
                      const validationKey = `${pi}-${ci}`;
                      const validation = collectionValidations[validationKey];
                      const hasError = validation && !validation.isValid;

                      return (
                        <div
                          key={ci}
                          className={`p-3 rounded-lg transition-colors ${hasError
                            ? 'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800'
                            : 'bg-neutral-50 dark:bg-neutral-800/30'
                            }`}
                        >
                          <div 
                            className="flex items-center gap-3 cursor-pointer"
                            onClick={() => toggleCollectionSelection(pi, ci)}
                          >
                            <input
                              type="checkbox"
                              checked={col.selected}
                              onChange={() => toggleCollectionSelection(pi, ci)}
                              className="w-4 h-4 text-primary-600 rounded"
                            />
                            <div className="flex-1">
                              <div className="flex items-center gap-2">
                                <p className="text-sm font-medium text-white">
                                  {col.name}
                                </p>
                                {col.selected && validation && (
                                  validation.isValid ? (
                                    <CheckCircle className="w-4 h-4 text-green-500" />
                                  ) : (
                                    <XCircle className="w-4 h-4 text-red-500" />
                                  )
                                )}
                              </div>
                              <p className="text-xs text-white/50">
                                {col.description}
                              </p>
                              {col.selected && hasError && validation.error && (
                                <p className="text-xs text-red-600 dark:text-red-400 mt-1 flex items-center gap-1">
                                  <AlertCircle className="w-3 h-3" />
                                  {validation.error}
                                </p>
                              )}
                            </div>
                            <span className="text-xs text-neutral-400 dark:text-neutral-500">
                              {col.content_type}
                            </span>
                          </div>
                          
                          {/* Graph Relationship Toggle */}
                          {col.selected && (
                            <div className="mt-3 pt-3 border-t border-white/10">
                              <label 
                                className="flex items-center gap-3 cursor-pointer group"
                                onClick={(e) => e.stopPropagation()}
                              >
                                <input
                                  type="checkbox"
                                  checked={col.enable_graph}
                                  onChange={() => toggleCollectionGraph(pi, ci)}
                                  className="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500"
                                />
                                <Share07 className={`w-4 h-4 transition-colors ${col.enable_graph ? 'text-indigo-600 dark:text-indigo-400' : 'text-neutral-400 dark:text-neutral-500'}`} />
                                <div className="flex-1">
                                  <p className={`text-sm font-medium transition-colors ${col.enable_graph ? 'text-indigo-700 dark:text-indigo-300' : 'text-white/60'}`}>
                                    Enable Graph Relationships
                                  </p>
                                  <p className="text-xs text-white/50">
                                    Automatically discover semantic relationships between documents (GraphRAG)
                                  </p>
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

            {/* Add Another Project Button */}
            <div className="border border-dashed border-neutral-300 dark:border-neutral-600 rounded-lg p-4 text-center">
              <Button
                variant="secondary"
                onClick={() => setCurrentStep('folder')}
                className="w-full sm:w-auto"
              >
                <Plus className="w-4 h-4 mr-2" />
                Add Another Project
              </Button>
              <p className="text-xs text-white/50 mt-2">
                You can add multiple projects to your workspace
              </p>
            </div>

            <div className="flex justify-between pt-4 border-t border-white/10">
              <Button variant="secondary" onClick={() => setCurrentStep('folder')}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              {(() => {
                // Check if all selected collections have valid names
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
                  <div className="flex items-center gap-3">
                    {hasValidationErrors && (
                      <span className="text-xs text-red-600 dark:text-red-400 flex items-center gap-1">
                        <AlertCircle className="w-4 h-4" />
                        Fix validation errors to continue
                      </span>
                    )}
                    <Button
                      variant="primary"
                      onClick={() => setCurrentStep('review')}
                      disabled={hasValidationErrors || selectedCount === 0}
                    >
                      Continue <ArrowRight className="w-4 h-4 ml-2" />
                    </Button>
                  </div>
                );
              })()}
            </div>
          </div>
        </Card>
      )}

      {currentStep === 'review' && (
        <Card className="bg-white/10 backdrop-blur-xl border border-white/10 shadow-2xl shadow-black/20 p-6">
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-white">
              Review & Apply Configuration
            </h2>

            <div className="bg-white/5 rounded-lg p-4">
              <h3 className="text-sm font-medium text-white/70 mb-3">
                Configuration Summary
              </h3>
              <div className="space-y-2 text-sm">
                {selectedTemplate && (
                  <div className="flex justify-between">
                    <span className="text-white/50">Template:</span>
                    <span className="font-medium text-white">
                      {selectedTemplate.name}
                    </span>
                  </div>
                )}
                <div className="flex justify-between">
                  <span className="text-white/50">Projects:</span>
                  <span className="font-medium text-white">
                    {analyzedProjects.filter(p => p.selected).length}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-white/50">Collections:</span>
                  <span className="font-medium text-white">
                    {analyzedProjects.reduce((sum, p) =>
                      sum + (p.selected ? p.collections.filter(c => c.selected).length : 0), 0
                    )}
                  </span>
                </div>
              </div>
            </div>

            {/* YAML Preview Toggle */}
            <div className="border border-white/10 rounded-lg overflow-hidden">
              <button
                onClick={() => setShowYamlPreview(!showYamlPreview)}
                className="w-full flex items-center justify-between p-4 bg-white/5 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <File06 className="w-5 h-5 text-white/50" />
                  <span className="font-medium text-white">
                    Preview workspace.yml
                  </span>
                </div>
                <span className="text-sm text-white/50">
                  {showYamlPreview ? '▲ Hide' : '▼ Show'}
                </span>
              </button>

              {showYamlPreview && (
                <div className="relative">
                  <button
                    onClick={handleCopyYaml}
                    className="absolute top-2 right-2 p-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg transition-colors z-10"
                    title="Copy to clipboard"
                  >
                    <Copy01 className="w-4 h-4 text-white" />
                    {yamlCopied && (
                      <span className="absolute -top-8 right-0 text-xs text-green-400 whitespace-nowrap">
                        Copied!
                      </span>
                    )}
                  </button>
                  <pre className="p-4 bg-neutral-900 text-neutral-100 text-sm overflow-x-auto max-h-80 font-mono">
                    {yamlPreview}
                  </pre>
                </div>
              )}
            </div>

            <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
              <p className="text-sm text-yellow-800 dark:text-yellow-300">
                <strong>Note:</strong> This will create a workspace.yml file in your Vectorizer directory.
                The server may need to be restarted to apply changes.
              </p>
            </div>

            <div className="flex justify-between pt-4 border-t border-white/10">
              <Button variant="secondary" onClick={() => setCurrentStep('analysis')}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              <Button
                variant="primary"
                onClick={handleApplyConfig}
                disabled={applying}
              >
                {applying ? (
                  <>
                    <LoadingSpinner size="sm" />
                    <span className="ml-2">Applying...</span>
                  </>
                ) : (
                  <>Apply Configuration</>
                )}
              </Button>
            </div>
          </div>
        </Card>
      )}

      {currentStep === 'complete' && (
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6 overflow-hidden">
          {/* Success confetti animation */}
          <div className="absolute inset-0 pointer-events-none overflow-hidden">
            {[...Array(20)].map((_, i) => (
              <div
                key={i}
                className="absolute w-2 h-2 rounded-full animate-confetti"
                style={{
                  backgroundColor: ['#10B981', '#6366F1', '#F59E0B', '#EF4444', '#8B5CF6'][i % 5],
                  left: `${Math.random() * 100}%`,
                  animationDelay: `${Math.random() * 0.5}s`,
                  animationDuration: `${1 + Math.random() * 2}s`,
                }}
              />
            ))}
          </div>

          <div className="relative text-center space-y-6">
            {/* Animated checkmark */}
            <div className="relative w-24 h-24 mx-auto">
              <div className="absolute inset-0 bg-green-100 dark:bg-green-900/30 rounded-full animate-pulse-slow" />
              <div className="absolute inset-2 bg-green-200 dark:bg-green-800/40 rounded-full animate-bounce-gentle" />
              <div className="absolute inset-0 flex items-center justify-center">
                <CheckCircle className="w-12 h-12 text-green-600 dark:text-green-400 animate-scale-in" />
              </div>
            </div>

            <div className="animate-fade-in-up" style={{ animationDelay: '0.3s' }}>
              <h2 className="text-2xl font-bold text-white">
                Setup Complete!
              </h2>
              <p className="text-white/60 mt-2">
                Your workspace has been configured successfully.
              </p>
            </div>

            {/* Success stats */}
            <div className="grid grid-cols-3 gap-4 max-w-md mx-auto animate-fade-in-up" style={{ animationDelay: '0.5s' }}>
              <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-3">
                <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                  {analyzedProjects.filter(p => p.selected).length}
                </div>
                <div className="text-xs text-white/50">Projects</div>
              </div>
              <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3">
                <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                  {analyzedProjects.reduce((sum, p) =>
                    sum + (p.selected ? p.collections.filter(c => c.selected).length : 0), 0
                  )}
                </div>
                <div className="text-xs text-white/50">Collections</div>
              </div>
              <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-3">
                <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                  {selectedTemplate?.name.split(' ')[0] || 'Custom'}
                </div>
                <div className="text-xs text-white/50">Template</div>
              </div>
            </div>

            <div className="bg-white/5 rounded-lg p-4 text-left max-w-md mx-auto animate-fade-in-up" style={{ animationDelay: '0.7s' }}>
              <h3 className="text-sm font-medium text-white/70 mb-2 flex items-center gap-2">
                <ArrowRight className="w-4 h-4 text-primary-500" />
                What&apos;s Next:
              </h3>
              <ul className="text-sm text-white/60 space-y-2">
                <li className="flex items-center gap-2">
                  <span className="w-5 h-5 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center text-green-600 text-xs">1</span>
                  Restart the server to apply workspace configuration
                </li>
                <li className="flex items-center gap-2">
                  <span className="w-5 h-5 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center text-green-600 text-xs">2</span>
                  Visit the Workspace page to manage projects
                </li>
                <li className="flex items-center gap-2">
                  <span className="w-5 h-5 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center text-green-600 text-xs">3</span>
                  Use the Search page to query your data
                </li>
              </ul>
            </div>

            <div className="flex gap-3 justify-center animate-fade-in-up" style={{ animationDelay: '0.9s' }}>
              <Button variant="secondary" onClick={() => navigate('/workspace')}>
                Go to Workspace
              </Button>
              <Button variant="primary" onClick={() => navigate('/overview')}>
                Go to Dashboard
              </Button>
            </div>
          </div>

          {/* CSS for animations */}
          <style>{`
            @keyframes confetti {
              0% { transform: translateY(-100%) rotate(0deg); opacity: 1; }
              100% { transform: translateY(1000%) rotate(720deg); opacity: 0; }
            }
            @keyframes pulse-slow {
              0%, 100% { transform: scale(1); opacity: 0.5; }
              50% { transform: scale(1.1); opacity: 0.3; }
            }
            @keyframes bounce-gentle {
              0%, 100% { transform: scale(1); }
              50% { transform: scale(1.05); }
            }
            @keyframes scale-in {
              0% { transform: scale(0); opacity: 0; }
              50% { transform: scale(1.2); }
              100% { transform: scale(1); opacity: 1; }
            }
            @keyframes fade-in-up {
              0% { transform: translateY(20px); opacity: 0; }
              100% { transform: translateY(0); opacity: 1; }
            }
            .animate-confetti { animation: confetti 3s ease-in-out forwards; }
            .animate-pulse-slow { animation: pulse-slow 2s ease-in-out infinite; }
            .animate-bounce-gentle { animation: bounce-gentle 1s ease-in-out infinite; }
            .animate-scale-in { animation: scale-in 0.5s ease-out forwards; }
            .animate-fade-in-up { animation: fade-in-up 0.5s ease-out forwards; opacity: 0; }
          `}</style>
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
