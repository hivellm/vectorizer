/**
 * Setup Wizard Page
 * Multi-step wizard for initial project setup with template selection
 */

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSetup, SetupStatus, ProjectAnalysis, SetupProject, SuggestedCollection } from '@/hooks/useSetup';
import { useTemplates, ConfigTemplate, getTemplateIcon, getTemplateColor } from '@/hooks/useTemplates';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import LoadingSpinner from '@/components/LoadingSpinner';
import { CheckCircle, Folder, Settings02, AlertCircle, ArrowRight, ArrowLeft } from '@untitledui/icons';

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

  // Load initial status
  useEffect(() => {
    const fetchStatus = async () => {
      setLoading(true);
      try {
        const s = await getStatus();
        setStatus(s);
        if (!s.needs_setup) {
          setCurrentStep('complete');
        }
      } catch (_err) {
        setError('Failed to load setup status');
      } finally {
        setLoading(false);
      }
    };
    fetchStatus();
  }, [getStatus]);

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

  const removeProject = (index: number) => {
    setAnalyzedProjects(prev => prev.filter((_, i) => i !== index));
  };

  const steps = ['welcome', 'template', 'folder', 'analysis', 'review', 'complete'];
  const stepIndex = steps.indexOf(currentStep);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-2xl sm:text-3xl font-bold text-neutral-900 dark:text-white">
          Setup Wizard
        </h1>
        <p className="text-neutral-600 dark:text-neutral-400 mt-2">
          Configure your Vectorizer workspace
        </p>
      </div>

      {/* Progress Steps */}
      <div className="flex items-center justify-center gap-2 sm:gap-4">
        {steps.map((step, i) => (
          <div key={step} className="flex items-center">
            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors ${
              currentStep === step
                ? 'bg-primary-600 text-white'
                : stepIndex > i
                  ? 'bg-green-500 text-white'
                  : 'bg-neutral-200 dark:bg-neutral-700 text-neutral-500 dark:text-neutral-400'
            }`}>
              {stepIndex > i ? (
                <CheckCircle className="w-4 h-4" />
              ) : i + 1}
            </div>
            {i < steps.length - 1 && (
              <div className={`w-8 sm:w-12 h-0.5 transition-colors ${
                stepIndex > i
                  ? 'bg-green-500'
                  : 'bg-neutral-200 dark:bg-neutral-700'
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
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6">
          <div className="text-center space-y-6">
            <div className="w-20 h-20 bg-primary-100 dark:bg-primary-900/30 rounded-full flex items-center justify-center mx-auto">
              <Settings02 className="w-10 h-10 text-primary-600 dark:text-primary-400" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
                Welcome to Vectorizer
              </h2>
              <p className="text-neutral-600 dark:text-neutral-400 mt-2 max-w-md mx-auto">
                Let&apos;s set up your workspace. This wizard will help you configure your projects
                and create collection mappings for vector search.
              </p>
            </div>

            {status && (
              <div className="bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-4 text-left max-w-md mx-auto">
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <span className="text-neutral-500 dark:text-neutral-400">Version:</span>
                  <span className="text-neutral-900 dark:text-white font-medium">{status.version}</span>
                  <span className="text-neutral-500 dark:text-neutral-400">Deployment:</span>
                  <span className="text-neutral-900 dark:text-white font-medium capitalize">{status.deployment_type}</span>
                  <span className="text-neutral-500 dark:text-neutral-400">Collections:</span>
                  <span className="text-neutral-900 dark:text-white font-medium">{status.collection_count}</span>
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
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6">
          <div className="space-y-6">
            <div className="text-center">
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
                Choose a Template
              </h2>
              <p className="text-neutral-600 dark:text-neutral-400 text-sm mt-1">
                Select a template that best matches your use case
              </p>
            </div>

            {templatesLoading ? (
              <div className="flex justify-center py-8">
                <LoadingSpinner size="lg" />
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {templates.map((template) => (
                  <button
                    key={template.id}
                    onClick={() => handleTemplateSelect(template)}
                    className={`text-left p-4 rounded-lg border-2 transition-all hover:shadow-md ${
                      selectedTemplate?.id === template.id
                        ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                        : 'border-neutral-200 dark:border-neutral-700 hover:border-primary-300'
                    }`}
                  >
                    <div className="flex items-start gap-4">
                      <div className={`w-12 h-12 rounded-lg flex items-center justify-center text-2xl ${getTemplateColor(template.id)}`}>
                        {getTemplateIcon(template)}
                      </div>
                      <div className="flex-1">
                        <h3 className="font-semibold text-neutral-900 dark:text-white">
                          {template.name}
                        </h3>
                        <p className="text-sm text-neutral-600 dark:text-neutral-400 mt-1">
                          {template.description}
                        </p>
                        <div className="flex flex-wrap gap-1 mt-2">
                          {template.use_cases.slice(0, 2).map((useCase, i) => (
                            <span
                              key={i}
                              className="text-xs px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 rounded-full text-neutral-600 dark:text-neutral-400"
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
            )}

            <div className="flex justify-between pt-4 border-t border-neutral-200 dark:border-neutral-700">
              <Button variant="secondary" onClick={() => setCurrentStep('welcome')}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
            </div>
          </div>
        </Card>
      )}

      {currentStep === 'folder' && (
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6">
          <div className="space-y-6">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 bg-primary-100 dark:bg-primary-900/30 rounded-lg flex items-center justify-center">
                <Folder className="w-6 h-6 text-primary-600 dark:text-primary-400" />
              </div>
              <div>
                <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
                  Add Project Folder
                </h2>
                <p className="text-neutral-600 dark:text-neutral-400 text-sm">
                  {selectedTemplate 
                    ? `Using "${selectedTemplate.name}" template`
                    : 'Enter the path to your project folder'}
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
                  Folder Path
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={folderPath}
                    onChange={(e) => setFolderPath(e.target.value)}
                    placeholder="/path/to/your/project"
                    className="flex-1 px-4 py-2 border border-neutral-300 dark:border-neutral-700 rounded-lg 
                             bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white
                             focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    onKeyDown={(e) => e.key === 'Enter' && handleAnalyze()}
                  />
                  <Button
                    variant="primary"
                    onClick={handleAnalyze}
                    disabled={analyzing || !folderPath.trim()}
                  >
                    {analyzing ? <LoadingSpinner size="sm" /> : 'Analyze'}
                  </Button>
                </div>
              </div>

              {analyzedProjects.length > 0 && (
                <div className="border-t border-neutral-200 dark:border-neutral-700 pt-4">
                  <h3 className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
                    Analyzed Projects ({analyzedProjects.length})
                  </h3>
                  <div className="space-y-2">
                    {analyzedProjects.map((p, i) => (
                      <div key={i} className="flex items-center justify-between p-3 bg-neutral-50 dark:bg-neutral-800/50 rounded-lg">
                        <div className="flex items-center gap-3">
                          <CheckCircle className="w-5 h-5 text-green-500" />
                          <div>
                            <p className="text-sm font-medium text-neutral-900 dark:text-white">
                              {p.analysis.project_name}
                            </p>
                            <p className="text-xs text-neutral-500 dark:text-neutral-400">
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

            <div className="flex justify-between pt-4 border-t border-neutral-200 dark:border-neutral-700">
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
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6">
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
              Review Detected Projects
            </h2>

            {analyzedProjects.map((project, pi) => (
              <div key={pi} className="border border-neutral-200 dark:border-neutral-700 rounded-lg overflow-hidden">
                <div
                  className="flex items-center justify-between p-4 bg-neutral-50 dark:bg-neutral-800/50 cursor-pointer"
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
                      <p className="font-medium text-neutral-900 dark:text-white">
                        {project.analysis.project_name}
                      </p>
                      <p className="text-sm text-neutral-500 dark:text-neutral-400">
                        {project.analysis.project_path}
                      </p>
                    </div>
                  </div>
                  <div className="text-right text-sm">
                    <p className="text-neutral-600 dark:text-neutral-400">
                      {project.analysis.project_types.join(', ')}
                    </p>
                    <p className="text-neutral-500 dark:text-neutral-500">
                      {project.analysis.statistics.total_files} files
                    </p>
                  </div>
                </div>

                {project.selected && (
                  <div className="p-4 space-y-3">
                    <p className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                      Collections:
                    </p>
                    {project.collections.map((col, ci) => (
                      <div
                        key={ci}
                        className="flex items-center gap-3 p-3 bg-neutral-50 dark:bg-neutral-800/30 rounded-lg cursor-pointer"
                        onClick={() => toggleCollectionSelection(pi, ci)}
                      >
                        <input
                          type="checkbox"
                          checked={col.selected}
                          onChange={() => toggleCollectionSelection(pi, ci)}
                          className="w-4 h-4 text-primary-600 rounded"
                        />
                        <div className="flex-1">
                          <p className="text-sm font-medium text-neutral-900 dark:text-white">
                            {col.name}
                          </p>
                          <p className="text-xs text-neutral-500 dark:text-neutral-400">
                            {col.description}
                          </p>
                        </div>
                        <span className="text-xs text-neutral-400 dark:text-neutral-500">
                          {col.content_type}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}

            <div className="flex justify-between pt-4 border-t border-neutral-200 dark:border-neutral-700">
              <Button variant="secondary" onClick={() => setCurrentStep('folder')}>
                <ArrowLeft className="w-4 h-4 mr-2" /> Back
              </Button>
              <Button variant="primary" onClick={() => setCurrentStep('review')}>
                Continue <ArrowRight className="w-4 h-4 ml-2" />
              </Button>
            </div>
          </div>
        </Card>
      )}

      {currentStep === 'review' && (
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6">
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
              Review & Apply Configuration
            </h2>

            <div className="bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-4">
              <h3 className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-3">
                Configuration Summary
              </h3>
              <div className="space-y-2 text-sm">
                {selectedTemplate && (
                  <div className="flex justify-between">
                    <span className="text-neutral-500 dark:text-neutral-400">Template:</span>
                    <span className="font-medium text-neutral-900 dark:text-white">
                      {selectedTemplate.name}
                    </span>
                  </div>
                )}
                <div className="flex justify-between">
                  <span className="text-neutral-500 dark:text-neutral-400">Projects:</span>
                  <span className="font-medium text-neutral-900 dark:text-white">
                    {analyzedProjects.filter(p => p.selected).length}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-neutral-500 dark:text-neutral-400">Collections:</span>
                  <span className="font-medium text-neutral-900 dark:text-white">
                    {analyzedProjects.reduce((sum, p) =>
                      sum + (p.selected ? p.collections.filter(c => c.selected).length : 0), 0
                    )}
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
              <p className="text-sm text-yellow-800 dark:text-yellow-300">
                <strong>Note:</strong> This will create a workspace.yml file in your Vectorizer directory.
                The server may need to be restarted to apply changes.
              </p>
            </div>

            <div className="flex justify-between pt-4 border-t border-neutral-200 dark:border-neutral-700">
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
        <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-6">
          <div className="text-center space-y-6">
            <div className="w-20 h-20 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto">
              <CheckCircle className="w-10 h-10 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
                Setup Complete!
              </h2>
              <p className="text-neutral-600 dark:text-neutral-400 mt-2">
                Your workspace has been configured successfully.
              </p>
            </div>

            <div className="bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-4 text-left max-w-md mx-auto">
              <h3 className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
                Next Steps:
              </h3>
              <ul className="text-sm text-neutral-600 dark:text-neutral-400 space-y-1">
                <li>• Restart the server to apply workspace configuration</li>
                <li>• Visit the Workspace page to manage projects</li>
                <li>• Use the Search page to query your data</li>
              </ul>
            </div>

            <div className="flex gap-3 justify-center">
              <Button variant="secondary" onClick={() => navigate('/workspace')}>
                Go to Workspace
              </Button>
              <Button variant="primary" onClick={() => navigate('/overview')}>
                Go to Dashboard
              </Button>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}

export default SetupWizardPage;
