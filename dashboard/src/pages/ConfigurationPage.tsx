/**
 * Settings page — server configuration.
 *
 * Console-redesign port of the legacy ConfigurationPage. The new layout is
 * intentionally minimal:
 *
 *   1. General  — KeyValue card derived from the loaded YAML (server.*)
 *   2. Defaults — KeyValue card derived from collections.defaults.*
 *   3. Raw config — the legacy Monaco YAML editor + save/load wiring
 *
 * The legacy page exposed per-section forms for: server, embedding, collections,
 * performance, file_watcher, logging, transmutation, normalization, workspace,
 * storage, api. Those structured editors are intentionally collapsed into the
 * single Raw config editor for now — see TODO(config-features-deferred) below.
 *
 * The save/load endpoint (`GET/POST /config`) is preserved exactly as the
 * legacy page used it, so the runtime API contract is unchanged.
 */

import { useEffect, useMemo, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useToastContext } from '@/providers/ToastProvider';
import {
  Icons,
  Card,
  CardHead,
  CardBody,
  KeyValue,
  KeyValueRow,
  Pill,
} from '@/components/console';
import CodeEditor from '@/components/ui/CodeEditor';

// Subset of the legacy ConfigData shape that the new KeyValue cards read.
// The full YAML round-trips through the editor untouched, so the rest of the
// shape (performance, file_watcher, normalization, storage, api…) is preserved
// in `yamlContent` even though we don't surface it as structured fields here.
interface ParsedConfig {
  server?: {
    host?: string;
    port?: number;
    data_dir?: string;
  };
  collections?: {
    defaults?: {
      metric?: string;
      embedding?: { model?: string };
      index?: { type?: string };
      quantization?: { type?: string };
    };
  };
  logging?: {
    level?: string;
  };
  api?: {
    rest?: { enabled?: boolean };
    mcp?: { enabled?: boolean };
  };
  cache?: {
    ttl_seconds?: number;
  };
  [key: string]: unknown;
}

function muted(text: string) {
  return <span style={{ color: 'var(--text-3)' }}>{text}</span>;
}

function ConfigurationPage() {
  const api = useApiClient();
  const toast = useToastContext();

  const [config, setConfig] = useState<ParsedConfig>({});
  const [yamlContent, setYamlContent] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const configData = await api.get<ParsedConfig>('/config');
      setConfig(configData ?? {});

      // Round-trip through js-yaml when available; fall back to JSON so the
      // editor still has something useful to display in restricted environments.
      try {
        const yamlMod = await import('js-yaml');
        setYamlContent(yamlMod.dump(configData, { indent: 2, lineWidth: 120 }));
      } catch {
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

  useEffect(() => {
    loadConfig();
    // loadConfig is stable enough for the page lifecycle; no deps to watch.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      let payload: ParsedConfig;
      try {
        const yamlMod = await import('js-yaml');
        payload = (yamlMod.load(yamlContent) as ParsedConfig) ?? {};
      } catch {
        toast.error('Failed to parse YAML');
        setError('Failed to parse YAML');
        return;
      }

      await api.post('/config', payload);
      setIsDirty(false);
      toast.success('Configuration saved. Restart the server for changes to take effect.');
      await loadConfig();
    } catch (err) {
      console.error('Error saving configuration:', err);
      const message = err instanceof Error ? err.message : 'Failed to save configuration';
      toast.error(message);
      setError(message);
    } finally {
      setSaving(false);
    }
  };

  // Derive General + Defaults rows from the loaded structured config. We don't
  // re-parse the editor text on every keystroke — the cards reflect the last
  // saved/loaded state until the user reloads.
  const general = config.server ?? {};
  const defaults = config.collections?.defaults ?? {};

  const bindAddress = useMemo(() => {
    if (general.host && general.port) return `${general.host}:${general.port}`;
    if (general.host) return general.host;
    if (general.port) return `${general.port}`;
    return null;
  }, [general.host, general.port]);

  const cacheTtl = config.cache?.ttl_seconds;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-sub">
            Server configuration ·{' '}
            <span className="mono" style={{ color: 'var(--text)' }}>
              /etc/vectorizer/config.yaml
            </span>
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button
            className="btn"
            onClick={loadConfig}
            disabled={loading}
            type="button"
          >
            <Icons.refresh size={13} />
            Reload
          </button>
          <button
            className="btn primary"
            onClick={handleSave}
            disabled={saving || loading || !isDirty}
            type="button"
          >
            {saving ? 'Saving…' : 'Save changes'}
          </button>
        </div>
      </div>

      <div className="grid grid-2" style={{ gap: 14, marginBottom: 14 }}>
        <Card>
          <CardHead title="General" />
          <CardBody>
            <KeyValue>
              {/* TODO(config-derive): server.name is not exposed by the
                  current /config payload. Surface it once the backend ships
                  the field; until then we show the host as a stand-in. */}
              <KeyValueRow term="Server name">
                {general.host ?? muted('hivellm-prod')}
              </KeyValueRow>
              <KeyValueRow term="Bind address">
                {bindAddress ? (
                  <span className="mono">{bindAddress}</span>
                ) : (
                  muted('127.0.0.1:15002')
                )}
              </KeyValueRow>
              <KeyValueRow term="Workspace">
                {general.data_dir ? (
                  <span className="mono">{general.data_dir}</span>
                ) : (
                  muted('/var/lib/vectorizer')
                )}
              </KeyValueRow>
              <KeyValueRow term="Log level">
                {config.logging?.level ?? muted('info')}
              </KeyValueRow>
              <KeyValueRow term="Telemetry">
                {/* TODO(config-derive): telemetry isn't represented as a single
                    field in the current YAML; show a sensible static label. */}
                {muted('Prometheus + tracing')}
              </KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>
        <Card>
          <CardHead title="Defaults" />
          <CardBody>
            <KeyValue>
              <KeyValueRow term="Embedding">
                {defaults.embedding?.model
                  ? <span className="mono">{defaults.embedding.model}</span>
                  : muted('BM25')}
              </KeyValueRow>
              <KeyValueRow term="Index">
                {defaults.index?.type
                  ? <span className="mono">{defaults.index.type}</span>
                  : muted('HNSW (M=16, ef=200)')}
              </KeyValueRow>
              <KeyValueRow term="Quantization">
                {defaults.quantization?.type
                  ? <span className="mono">{defaults.quantization.type}</span>
                  : muted('SQ-8bit (auto)')}
              </KeyValueRow>
              <KeyValueRow term="Distance">
                {defaults.metric ?? muted('cosine')}
              </KeyValueRow>
              <KeyValueRow term="Cache TTL">
                {typeof cacheTtl === 'number' ? `${cacheTtl}s` : muted('300s')}
              </KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>
      </div>

      <Card>
        <CardHead
          title="Raw config"
          right={
            error ? (
              <Pill tone="red">{error}</Pill>
            ) : (
              <Pill tone="muted" className="mono">
                {loading ? 'loading…' : isDirty ? 'unsaved' : 'YAML'}
              </Pill>
            )
          }
        />
        <CardBody>
          {/*
            TODO(config-features-deferred): the legacy ConfigurationPage
            exposed dedicated tabs with structured form controls for every
            section of config.yaml — server, embedding, collections,
            performance, file_watcher, logging, transmutation, normalization,
            workspace, storage, api (REST + MCP). The console redesign
            collapses all of those into this single Monaco editor. Bring
            them back as discrete cards once we agree on the visual language.
          */}
          <CodeEditor
            value={yamlContent}
            onChange={(next) => {
              setYamlContent(next ?? '');
              setIsDirty(true);
            }}
            language="yaml"
            height="500px"
            readOnly={false}
          />
        </CardBody>
      </Card>
    </div>
  );
}

export default ConfigurationPage;
