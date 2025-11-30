/**
 * Configuration page - Server configuration
 * Similar to GUI ConfigEditor.vue
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import Button from '@/components/ui/Button';
import CodeEditor from '@/components/ui/CodeEditor';
import Checkbox from '@/components/ui/Checkbox';
import { Input } from '@/components/ui/Input';
import { Select, SelectOption } from '@/components/ui/Select';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { 
  RefreshCw01, 
  Settings01, 
  Code01, 
  Folder, 
  Zap, 
  Eye, 
  AlignLeft, 
  Database01
} from '@untitledui/icons';

// Custom icon components for icons not available in @untitledui/icons
const FileTextIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
  </svg>
);

const ArrowSwapIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
  </svg>
);

const PlugIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
  </svg>
);

const CpuIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
  </svg>
);

const HardDriveIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
  </svg>
);

interface ConfigData {
  server?: {
    host?: string;
    port?: number;
    data_dir?: string;
  };
  collections?: {
    defaults?: {
      dimension?: number;
      metric?: string;
      embedding?: {
        model?: string;
      };
      index?: {
        type?: string;
      };
      quantization?: {
        type?: string;
      };
      sharding?: {
        enabled?: boolean;
        target_max_size?: number;
        soft_limit_size?: number;
      };
    };
  };
  performance?: {
    cpu?: {
      max_threads?: number;
      enable_simd?: boolean;
      memory_pool_size_mb?: number;
    };
    batch?: {
      default_size?: number;
      max_size?: number;
      parallel_processing?: boolean;
    };
  };
  file_watcher?: {
    enabled?: boolean;
    debounce_delay_ms?: number;
    collection_name?: string;
    min_file_size_bytes?: number;
    max_file_size_bytes?: number;
    hash_validation_enabled?: boolean;
  };
  logging?: {
    level?: string;
    format?: string;
    log_requests?: boolean;
    log_responses?: boolean;
    log_errors?: boolean;
  };
  transmutation?: {
    enabled?: boolean;
    max_file_size_mb?: number;
    conversion_timeout_secs?: number;
    preserve_images?: boolean;
  };
  normalization?: {
    enabled?: boolean;
    level?: string;
    line_endings?: {
      normalize_crlf?: boolean;
      normalize_cr?: boolean;
      collapse_multiple_newlines?: boolean;
      trim_trailing_whitespace?: boolean;
    };
    content_detection?: {
      enabled?: boolean;
      preserve_code_structure?: boolean;
      preserve_markdown_format?: boolean;
    };
  };
  workspace?: {
    enabled?: boolean;
    auto_load_collections?: boolean;
    default_workspace_file?: string;
  };
  storage?: {
    compression?: {
      enabled?: boolean;
      format?: string;
      level?: number;
    };
    snapshots?: {
      enabled?: boolean;
      interval_hours?: number;
      retention_days?: number;
      max_snapshots?: number;
      path?: string;
    };
  };
  api?: {
    rest?: {
      enabled?: boolean;
      cors_enabled?: boolean;
      max_request_size_mb?: number;
      timeout_seconds?: number;
    };
    mcp?: {
      enabled?: boolean;
      port?: number;
      max_connections?: number;
    };
  };
  [key: string]: any;
}

const tabs = [
  { id: 'general', label: 'General', icon: Settings01 },
  { id: 'embedding', label: 'Embedding', icon: CpuIcon },
  { id: 'collections', label: 'Collections', icon: Database01 },
  { id: 'performance', label: 'Performance', icon: Zap },
  { id: 'file_watcher', label: 'File Watcher', icon: Eye },
  { id: 'logging', label: 'Logging', icon: FileTextIcon },
  { id: 'transmutation', label: 'Transmutation', icon: ArrowSwapIcon },
  { id: 'normalization', label: 'Normalization', icon: AlignLeft },
  { id: 'workspace', label: 'Workspace', icon: Folder },
  { id: 'storage', label: 'Storage', icon: HardDriveIcon },
  { id: 'api', label: 'API', icon: PlugIcon },
  { id: 'yaml', label: 'YAML', icon: Code01 },
];

function ConfigurationPage() {
  const api = useApiClient();
  const toast = useToastContext();

  const [config, setConfig] = useState<ConfigData>({});
  const [yamlContent, setYamlContent] = useState<string>('');
  const [activeTab, setActiveTab] = useState<string>('general');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [, setError] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const markDirty = () => {
    setIsDirty(true);
  };

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const configData = await api.get<ConfigData>('/api/config');
      setConfig(configData);
      
      // Convert to YAML for the YAML editor
      try {
        // Try to use js-yaml if available, otherwise use JSON
        const yaml = await import('js-yaml');
        setYamlContent(yaml.dump(configData, { indent: 2, lineWidth: 120 }));
      } catch (e) {
        // Fallback to JSON if js-yaml is not available
        setYamlContent(JSON.stringify(configData, null, 2));
      }
      
      setIsDirty(false);
    } catch (err) {
      console.error('Error loading configuration:', err);
      setError(err instanceof Error ? err.message : 'Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      let payload: ConfigData;

      if (activeTab === 'yaml') {
        // Convert YAML to JSON
        try {
          const yaml = await import('js-yaml');
          payload = yaml.load(yamlContent) as ConfigData;
        } catch (e) {
          toast.error('Failed to parse YAML');
          setError('Failed to parse YAML');
          return;
        }
      } else {
        payload = config;
      }

      await api.post('/api/config', payload);
      setIsDirty(false);
      toast.success('Configuration saved successfully! Restart server for changes to take effect.');
      // Reload to sync all tabs
      await loadConfig();
    } catch (err) {
      console.error('Error saving configuration:', err);
      toast.error(err instanceof Error ? err.message : 'Failed to save configuration');
      setError(err instanceof Error ? err.message : 'Failed to save configuration');
    } finally {
      setSaving(false);
    }
  };

  const updateConfig = (path: string[], value: any) => {
    const newConfig = { ...config };
    let current: any = newConfig;
    
    for (let i = 0; i < path.length - 1; i++) {
      if (!current[path[i]]) {
        current[path[i]] = {};
      }
      current = current[path[i]];
    }
    
    current[path[path.length - 1]] = value;
    setConfig(newConfig);
    markDirty();
  };

  if (loading) {
    return <LoadingState message="Loading configuration..." />;
  }

  return (
    <div className="p-8">
      <div className="flex gap-6 h-full">
        {/* Sidebar Navigation */}
        <div className="w-48 flex-shrink-0">
          <nav className="space-y-1">
            {tabs.map((tab) => {
              const IconComponent = tab.icon;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`w-full flex items-center gap-3 px-3 py-2 text-sm rounded transition-colors ${
                    activeTab === tab.id
                      ? 'bg-neutral-100 dark:bg-neutral-800 text-neutral-900 dark:text-white'
                      : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-50 dark:hover:bg-neutral-900'
                  }`}
                >
                  <IconComponent className="w-4 h-4" />
                  <span>{tab.label}</span>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Main Content */}
        <div className="flex-1 bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-6 overflow-y-auto">
          {/* General Tab */}
          {activeTab === 'general' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">General Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Server</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Host</label>
                      <Input
                        value={config.server?.host || ''}
                        onChange={(e) => updateConfig(['server', 'host'], e.target.value)}
                        placeholder="127.0.0.1"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Port</label>
                      <Input
                        type="number"
                        value={config.server?.port || ''}
                        onChange={(e) => updateConfig(['server', 'port'], parseInt(e.target.value) || 0)}
                        placeholder="15002"
                      />
                    </div>
                  </div>
                  <div className="mt-4">
                    <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Data Directory</label>
                    <Input
                      value={config.server?.data_dir || ''}
                      onChange={(e) => updateConfig(['server', 'data_dir'], e.target.value)}
                      placeholder="./data"
                    />
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Embedding Tab */}
          {activeTab === 'embedding' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Embedding Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Model Configuration</h3>
                  <div>
                    <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Default Model</label>
                    <Select
                      value={config.collections?.defaults?.embedding?.model || 'bm25'}
                      onChange={(value) => updateConfig(['collections', 'defaults', 'embedding', 'model'], value)}
                    >
                      <SelectOption id="bm25" value="bm25">BM25</SelectOption>
                      <SelectOption id="bow" value="bow">Bag of Words</SelectOption>
                      <SelectOption id="hash" value="hash">Hash</SelectOption>
                      <SelectOption id="ngram" value="ngram">N-gram</SelectOption>
                      <SelectOption id="hnsw" value="hnsw">HNSW</SelectOption>
                      <SelectOption id="optimized_hnsw" value="optimized_hnsw">Optimized HNSW</SelectOption>
                    </Select>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Collections Tab */}
          {activeTab === 'collections' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Collection Defaults</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Vector Settings</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Dimension</label>
                      <Input
                        type="number"
                        value={config.collections?.defaults?.dimension || ''}
                        onChange={(e) => updateConfig(['collections', 'defaults', 'dimension'], parseInt(e.target.value) || 0)}
                        placeholder="512"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Metric</label>
                      <Select
                        value={config.collections?.defaults?.metric || 'cosine'}
                        onChange={(value) => updateConfig(['collections', 'defaults', 'metric'], value)}
                      >
                        <SelectOption id="cosine" value="cosine">Cosine</SelectOption>
                        <SelectOption id="euclidean" value="euclidean">Euclidean</SelectOption>
                        <SelectOption id="dot_product" value="dot_product">Dot Product</SelectOption>
                      </Select>
                    </div>
                  </div>
                </div>

                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Index Settings</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Index Type</label>
                      <Select
                        value={config.collections?.defaults?.index?.type || 'hnsw'}
                        onChange={(value) => updateConfig(['collections', 'defaults', 'index', 'type'], value)}
                      >
                        <SelectOption id="hnsw" value="hnsw">HNSW</SelectOption>
                        <SelectOption id="optimized_hnsw" value="optimized_hnsw">Optimized HNSW</SelectOption>
                      </Select>
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Quantization Type</label>
                      <Select
                        value={config.collections?.defaults?.quantization?.type || 'sq'}
                        onChange={(value) => updateConfig(['collections', 'defaults', 'quantization', 'type'], value)}
                      >
                        <SelectOption id="sq" value="sq">Scalar Quantization</SelectOption>
                      </Select>
                    </div>
                  </div>
                </div>

                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Sharding</h3>
                  <div className="space-y-4">
                    <div>
                      <Checkbox
                        checked={config.collections?.defaults?.sharding?.enabled || false}
                        onChange={(checked) => updateConfig(['collections', 'defaults', 'sharding', 'enabled'], checked)}
                        label="Enable Sharding"
                      />
                    </div>
                    {config.collections?.defaults?.sharding?.enabled && (
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Target Max Size</label>
                          <Input
                            type="number"
                            value={config.collections?.defaults?.sharding?.target_max_size || ''}
                            onChange={(e) => updateConfig(['collections', 'defaults', 'sharding', 'target_max_size'], parseInt(e.target.value) || 0)}
                            placeholder="10000"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Soft Limit Size</label>
                          <Input
                            type="number"
                            value={config.collections?.defaults?.sharding?.soft_limit_size || ''}
                            onChange={(e) => updateConfig(['collections', 'defaults', 'sharding', 'soft_limit_size'], parseInt(e.target.value) || 0)}
                            placeholder="8000"
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Performance Tab */}
          {activeTab === 'performance' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Performance Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">CPU</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max Threads</label>
                      <Input
                        type="number"
                        value={config.performance?.cpu?.max_threads || ''}
                        onChange={(e) => updateConfig(['performance', 'cpu', 'max_threads'], parseInt(e.target.value) || 0)}
                        placeholder="8"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Memory Pool Size (MB)</label>
                      <Input
                        type="number"
                        value={config.performance?.cpu?.memory_pool_size_mb || ''}
                        onChange={(e) => updateConfig(['performance', 'cpu', 'memory_pool_size_mb'], parseInt(e.target.value) || 0)}
                        placeholder="1024"
                      />
                    </div>
                  </div>
                  <div className="mt-4">
                    <Checkbox
                      checked={config.performance?.cpu?.enable_simd || false}
                      onChange={(checked) => updateConfig(['performance', 'cpu', 'enable_simd'], checked)}
                      label="Enable SIMD"
                    />
                  </div>
                </div>

                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Batch Processing</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Default Batch Size</label>
                      <Input
                        type="number"
                        value={config.performance?.batch?.default_size || ''}
                        onChange={(e) => updateConfig(['performance', 'batch', 'default_size'], parseInt(e.target.value) || 0)}
                        placeholder="100"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max Batch Size</label>
                      <Input
                        type="number"
                        value={config.performance?.batch?.max_size || ''}
                        onChange={(e) => updateConfig(['performance', 'batch', 'max_size'], parseInt(e.target.value) || 0)}
                        placeholder="1000"
                      />
                    </div>
                  </div>
                  <div className="mt-4">
                    <Checkbox
                      checked={config.performance?.batch?.parallel_processing || false}
                      onChange={(checked) => updateConfig(['performance', 'batch', 'parallel_processing'], checked)}
                      label="Parallel Processing"
                    />
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* File Watcher Tab */}
          {activeTab === 'file_watcher' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">File Watcher Settings</h2>
              <div className="space-y-6">
                <div>
                  <Checkbox
                    checked={config.file_watcher?.enabled || false}
                    onChange={(checked) => updateConfig(['file_watcher', 'enabled'], checked)}
                    label="Enable File Watcher"
                  />
                </div>
                
                {config.file_watcher?.enabled && (
                  <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Debounce Delay (ms)</label>
                        <Input
                          type="number"
                          value={config.file_watcher?.debounce_delay_ms || ''}
                          onChange={(e) => updateConfig(['file_watcher', 'debounce_delay_ms'], parseInt(e.target.value) || 0)}
                          placeholder="1000"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Collection Name</label>
                        <Input
                          value={config.file_watcher?.collection_name || ''}
                          onChange={(e) => updateConfig(['file_watcher', 'collection_name'], e.target.value)}
                          placeholder="workspace-files"
                        />
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Min File Size (bytes)</label>
                        <Input
                          type="number"
                          value={config.file_watcher?.min_file_size_bytes || ''}
                          onChange={(e) => updateConfig(['file_watcher', 'min_file_size_bytes'], parseInt(e.target.value) || 0)}
                          placeholder="1"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max File Size (bytes)</label>
                        <Input
                          type="number"
                          value={config.file_watcher?.max_file_size_bytes || ''}
                          onChange={(e) => updateConfig(['file_watcher', 'max_file_size_bytes'], parseInt(e.target.value) || 0)}
                          placeholder="10485760"
                        />
                      </div>
                    </div>
                    <div>
                      <Checkbox
                        checked={config.file_watcher?.hash_validation_enabled || false}
                        onChange={(checked) => updateConfig(['file_watcher', 'hash_validation_enabled'], checked)}
                        label="Hash Validation Enabled"
                      />
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Logging Tab */}
          {activeTab === 'logging' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Logging Settings</h2>
              <div className="space-y-6">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Log Level</label>
                    <Select
                      value={config.logging?.level || 'info'}
                      onChange={(value) => updateConfig(['logging', 'level'], value)}
                    >
                      <SelectOption id="error" value="error">Error</SelectOption>
                      <SelectOption id="warn" value="warn">Warning</SelectOption>
                      <SelectOption id="info" value="info">Info</SelectOption>
                      <SelectOption id="debug" value="debug">Debug</SelectOption>
                      <SelectOption id="trace" value="trace">Trace</SelectOption>
                    </Select>
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Log Format</label>
                    <Select
                      value={config.logging?.format || 'json'}
                      onChange={(value) => updateConfig(['logging', 'format'], value)}
                    >
                      <SelectOption id="json" value="json">JSON</SelectOption>
                      <SelectOption id="text" value="text">Text</SelectOption>
                    </Select>
                  </div>
                </div>

                <div className="space-y-4">
                  <div>
                    <Checkbox
                      checked={config.logging?.log_requests || false}
                      onChange={(checked) => updateConfig(['logging', 'log_requests'], checked)}
                      label="Log Requests"
                    />
                  </div>
                  <div>
                    <Checkbox
                      checked={config.logging?.log_responses || false}
                      onChange={(checked) => updateConfig(['logging', 'log_responses'], checked)}
                      label="Log Responses"
                    />
                  </div>
                  <div>
                    <Checkbox
                      checked={config.logging?.log_errors || false}
                      onChange={(checked) => updateConfig(['logging', 'log_errors'], checked)}
                      label="Log Errors"
                    />
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Transmutation Tab */}
          {activeTab === 'transmutation' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Transmutation Settings</h2>
              <div className="space-y-6">
                <div>
                  <Checkbox
                    checked={config.transmutation?.enabled || false}
                    onChange={(checked) => updateConfig(['transmutation', 'enabled'], checked)}
                    label="Enable Transmutation"
                  />
                </div>
                
                {config.transmutation?.enabled && (
                  <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max File Size (MB)</label>
                        <Input
                          type="number"
                          value={config.transmutation?.max_file_size_mb || ''}
                          onChange={(e) => updateConfig(['transmutation', 'max_file_size_mb'], parseInt(e.target.value) || 0)}
                          placeholder="50"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Conversion Timeout (seconds)</label>
                        <Input
                          type="number"
                          value={config.transmutation?.conversion_timeout_secs || ''}
                          onChange={(e) => updateConfig(['transmutation', 'conversion_timeout_secs'], parseInt(e.target.value) || 0)}
                          placeholder="300"
                        />
                      </div>
                    </div>
                    <div>
                      <Checkbox
                        checked={config.transmutation?.preserve_images || false}
                        onChange={(checked) => updateConfig(['transmutation', 'preserve_images'], checked)}
                        label="Preserve Images"
                      />
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Normalization Tab */}
          {activeTab === 'normalization' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Normalization Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">General</h3>
                  <div className="space-y-4">
                    <div>
                      <Checkbox
                        checked={config.normalization?.enabled || false}
                        onChange={(checked) => updateConfig(['normalization', 'enabled'], checked)}
                        label="Enable Normalization"
                      />
                    </div>
                    {config.normalization?.enabled && (
                      <div>
                        <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Normalization Level</label>
                        <Select
                          value={config.normalization?.level || 'conservative'}
                          onChange={(value) => updateConfig(['normalization', 'level'], value)}
                        >
                          <SelectOption id="conservative" value="conservative">Conservative</SelectOption>
                          <SelectOption id="moderate" value="moderate">Moderate</SelectOption>
                          <SelectOption id="aggressive" value="aggressive">Aggressive</SelectOption>
                        </Select>
                      </div>
                    )}
                  </div>
                </div>

                {config.normalization?.enabled && (
                  <>
                    <div>
                      <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Line Endings</h3>
                      <div className="space-y-4">
                        <div>
                          <Checkbox
                            checked={config.normalization?.line_endings?.normalize_crlf || false}
                            onChange={(checked) => updateConfig(['normalization', 'line_endings', 'normalize_crlf'], checked)}
                            label="Normalize CRLF to LF"
                          />
                        </div>
                        <div>
                          <Checkbox
                            checked={config.normalization?.line_endings?.normalize_cr || false}
                            onChange={(checked) => updateConfig(['normalization', 'line_endings', 'normalize_cr'], checked)}
                            label="Normalize CR to LF"
                          />
                        </div>
                        <div>
                          <Checkbox
                            checked={config.normalization?.line_endings?.collapse_multiple_newlines || false}
                            onChange={(checked) => updateConfig(['normalization', 'line_endings', 'collapse_multiple_newlines'], checked)}
                            label="Collapse Multiple Newlines"
                          />
                        </div>
                        <div>
                          <Checkbox
                            checked={config.normalization?.line_endings?.trim_trailing_whitespace || false}
                            onChange={(checked) => updateConfig(['normalization', 'line_endings', 'trim_trailing_whitespace'], checked)}
                            label="Trim Trailing Whitespace"
                          />
                        </div>
                      </div>
                    </div>

                    <div>
                      <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Content Detection</h3>
                      <div className="space-y-4">
                        <div>
                          <Checkbox
                            checked={config.normalization?.content_detection?.enabled || false}
                            onChange={(checked) => updateConfig(['normalization', 'content_detection', 'enabled'], checked)}
                            label="Enable Content Detection"
                          />
                        </div>
                        <div>
                          <Checkbox
                            checked={config.normalization?.content_detection?.preserve_code_structure || false}
                            onChange={(checked) => updateConfig(['normalization', 'content_detection', 'preserve_code_structure'], checked)}
                            label="Preserve Code Structure"
                          />
                        </div>
                        <div>
                          <Checkbox
                            checked={config.normalization?.content_detection?.preserve_markdown_format || false}
                            onChange={(checked) => updateConfig(['normalization', 'content_detection', 'preserve_markdown_format'], checked)}
                            label="Preserve Markdown Format"
                          />
                        </div>
                      </div>
                    </div>
                  </>
                )}
              </div>
            </div>
          )}

          {/* Workspace Tab */}
          {activeTab === 'workspace' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Workspace Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">General</h3>
                  <div className="space-y-4">
                    <div>
                      <Checkbox
                        checked={config.workspace?.enabled || false}
                        onChange={(checked) => updateConfig(['workspace', 'enabled'], checked)}
                        label="Enable Workspace"
                      />
                    </div>
                    <div>
                      <Checkbox
                        checked={config.workspace?.auto_load_collections || false}
                        onChange={(checked) => updateConfig(['workspace', 'auto_load_collections'], checked)}
                        label="Auto Load Collections"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Default Workspace File</label>
                      <Input
                        value={config.workspace?.default_workspace_file || ''}
                        onChange={(e) => updateConfig(['workspace', 'default_workspace_file'], e.target.value)}
                        placeholder="./workspace.yml"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Storage Tab */}
          {activeTab === 'storage' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">Storage Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Compression</h3>
                  <div className="space-y-4">
                    <div>
                      <Checkbox
                        checked={config.storage?.compression?.enabled || false}
                        onChange={(checked) => updateConfig(['storage', 'compression', 'enabled'], checked)}
                        label="Enable Compression"
                      />
                    </div>
                    {config.storage?.compression?.enabled && (
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Format</label>
                          <Select
                            value={config.storage?.compression?.format || 'zstd'}
                            onChange={(value) => updateConfig(['storage', 'compression', 'format'], value)}
                          >
                            <SelectOption id="zstd" value="zstd">Zstandard</SelectOption>
                          </Select>
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Level (1-22)</label>
                          <Input
                            type="number"
                            min="1"
                            max="22"
                            value={config.storage?.compression?.level || ''}
                            onChange={(e) => updateConfig(['storage', 'compression', 'level'], parseInt(e.target.value) || 0)}
                            placeholder="3"
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">Snapshots</h3>
                  <div className="space-y-4">
                    <div>
                      <Checkbox
                        checked={config.storage?.snapshots?.enabled || false}
                        onChange={(checked) => updateConfig(['storage', 'snapshots', 'enabled'], checked)}
                        label="Enable Snapshots"
                      />
                    </div>
                    {config.storage?.snapshots?.enabled && (
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Interval (hours)</label>
                          <Input
                            type="number"
                            value={config.storage?.snapshots?.interval_hours || ''}
                            onChange={(e) => updateConfig(['storage', 'snapshots', 'interval_hours'], parseInt(e.target.value) || 0)}
                            placeholder="1"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Retention (days)</label>
                          <Input
                            type="number"
                            value={config.storage?.snapshots?.retention_days || ''}
                            onChange={(e) => updateConfig(['storage', 'snapshots', 'retention_days'], parseInt(e.target.value) || 0)}
                            placeholder="2"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max Snapshots</label>
                          <Input
                            type="number"
                            value={config.storage?.snapshots?.max_snapshots || ''}
                            onChange={(e) => updateConfig(['storage', 'snapshots', 'max_snapshots'], parseInt(e.target.value) || 0)}
                            placeholder="48"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Snapshots Path</label>
                          <Input
                            value={config.storage?.snapshots?.path || ''}
                            onChange={(e) => updateConfig(['storage', 'snapshots', 'path'], e.target.value)}
                            placeholder="./data/snapshots"
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* API Tab */}
          {activeTab === 'api' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">API Settings</h2>
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">REST API</h3>
                  <div className="space-y-4">
                    <div>
                      <Checkbox
                        checked={config.api?.rest?.enabled || false}
                        onChange={(checked) => updateConfig(['api', 'rest', 'enabled'], checked)}
                        label="Enable REST API"
                      />
                    </div>
                    {config.api?.rest?.enabled && (
                      <div className="space-y-4">
                        <div>
                          <Checkbox
                            checked={config.api?.rest?.cors_enabled || false}
                            onChange={(checked) => updateConfig(['api', 'rest', 'cors_enabled'], checked)}
                            label="Enable CORS"
                          />
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                          <div>
                            <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max Request Size (MB)</label>
                            <Input
                              type="number"
                              value={config.api?.rest?.max_request_size_mb || ''}
                              onChange={(e) => updateConfig(['api', 'rest', 'max_request_size_mb'], parseInt(e.target.value) || 0)}
                              placeholder="10"
                            />
                          </div>
                          <div>
                            <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Timeout (seconds)</label>
                            <Input
                              type="number"
                              value={config.api?.rest?.timeout_seconds || ''}
                              onChange={(e) => updateConfig(['api', 'rest', 'timeout_seconds'], parseInt(e.target.value) || 0)}
                              placeholder="30"
                            />
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                <div>
                  <h3 className="text-sm font-semibold text-neutral-900 dark:text-white mb-3">MCP (Model Context Protocol)</h3>
                  <div className="space-y-4">
                    <Checkbox
                      checked={config.api?.mcp?.enabled || false}
                      onChange={(checked) => updateConfig(['api', 'mcp', 'enabled'], checked)}
                      label="Enable MCP"
                    />
                    {config.api?.mcp?.enabled && (
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Port</label>
                          <Input
                            type="number"
                            value={config.api?.mcp?.port || ''}
                            onChange={(e) => updateConfig(['api', 'mcp', 'port'], parseInt(e.target.value) || 0)}
                            placeholder="15002"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-neutral-600 dark:text-neutral-400 mb-2">Max Connections</label>
                          <Input
                            type="number"
                            value={config.api?.mcp?.max_connections || ''}
                            onChange={(e) => updateConfig(['api', 'mcp', 'max_connections'], parseInt(e.target.value) || 0)}
                            placeholder="100"
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* YAML Editor Tab */}
          {activeTab === 'yaml' && (
            <div>
              <h2 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">YAML Editor</h2>
              <CodeEditor
                value={yamlContent}
                onChange={(value) => {
                  setYamlContent(value || '');
                  markDirty();
                }}
                language="yaml"
                height="500px"
                readOnly={false}
              />
            </div>
          )}

          {/* Help Text */}
          <div className="mt-6 p-4 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded">
            <p className="text-xs text-neutral-600 dark:text-neutral-400">
              Configure your Vectorizer settings using the form above or edit the YAML directly. Changes are automatically saved when you click 'Save & Restart'.
            </p>
          </div>
        </div>
      </div>

      {/* Unsaved Changes Indicator */}
      {isDirty && (
        <div className="fixed bottom-6 right-6 px-4 py-3 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg shadow-lg flex items-center gap-3 z-50">
          <svg className="w-5 h-5 text-yellow-600 dark:text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <span className="text-sm text-neutral-600 dark:text-neutral-400">Unsaved changes</span>
        </div>
      )}

      {/* Action Buttons */}
      <div className="fixed bottom-6 left-6 flex items-center gap-2 z-50">
        <Button variant="secondary" size="sm" onClick={loadConfig}>
          <RefreshCw01 className="w-4 h-4 mr-2" />
          Reload
        </Button>
        <Button variant="primary" size="sm" onClick={handleSave} disabled={saving || !isDirty} isLoading={saving}>
          Save & Restart
        </Button>
      </div>
    </div>
  );
}

export default ConfigurationPage;
