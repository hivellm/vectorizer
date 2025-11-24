/**
 * Configuration page - Server configuration
 */

import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import CodeEditor from '@/components/ui/CodeEditor';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import { RefreshCw01 } from '@untitledui/icons';

function ConfigurationPage() {
  const api = useApiClient();
  const toast = useToastContext();

  const [config, setConfig] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const configData = await api.get<any>('/api/config');
      setConfig(JSON.stringify(configData, null, 2));
    } catch (err) {
      console.error('Error loading configuration:', err);
      setError(err instanceof Error ? err.message : 'Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!config.trim()) {
      toast.error('Configuration cannot be empty');
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const parsedConfig = JSON.parse(config);
      await api.post('/api/config', parsedConfig);
      toast.success('Configuration saved successfully. Restart server for changes to take effect.');
    } catch (err) {
      if (err instanceof SyntaxError) {
        toast.error('Invalid JSON format');
        setError('Invalid JSON format');
      } else {
        toast.error(err instanceof Error ? err.message : 'Failed to save configuration');
        setError(err instanceof Error ? err.message : 'Failed to save configuration');
      }
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <LoadingState message="Loading configuration..." />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Configuration</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">
            Manage server configuration settings
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={loadConfig}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Reload
          </Button>
          <Button variant="primary" size="sm" onClick={handleSave} disabled={saving} isLoading={saving}>
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            Save Configuration
          </Button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Warning */}
      <Card>
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <svg className="w-5 h-5 text-yellow-600 dark:text-yellow-400 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div>
              <h3 className="text-sm font-semibold text-yellow-800 dark:text-yellow-300 mb-1">
                Configuration Warning
              </h3>
              <p className="text-sm text-yellow-700 dark:text-yellow-400">
                Changes to the configuration will require a server restart to take effect. Make sure you understand the impact of your changes before saving.
              </p>
            </div>
          </div>
        </div>
      </Card>

      {/* Configuration Editor */}
      <Card>
        <div className="space-y-4">
          <div>
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">
              Server Configuration (JSON)
            </h2>
            <p className="text-sm text-neutral-500 dark:text-neutral-400">
              Edit the server configuration in JSON format. The configuration will be saved as YAML.
            </p>
          </div>
          <CodeEditor
            value={config}
            onChange={(value) => {
              setConfig(value || '');
              setError(null);
            }}
            language="json"
            height="600px"
            readOnly={false}
          />
        </div>
      </Card>
    </div>
  );
}

export default ConfigurationPage;
