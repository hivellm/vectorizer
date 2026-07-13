/**
 * Settings page — server configuration.
 *
 * Layout:
 *   1. General — editable form for server.{host,port,mcp_port}
 *   2. Logging — editable form for logging.{level,log_requests,log_responses,log_errors}
 *   3. Raw config — the Monaco YAML editor for everything else
 *
 * The structured cards only expose fields that actually exist in the live
 * `GET /config` payload, so a save never injects sections the backend's config
 * struct would reject. Structured edits live in local `form` state (smooth
 * typing) and are merged onto the parsed YAML at save time: the raw editor is
 * the source of truth for fields the forms don't cover; the form fields win for
 * the paths they own. The save/load endpoint (`GET/POST /config`) is unchanged.
 */

import { useEffect, useState, type CSSProperties, type ReactNode } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useToastContext } from '@/providers/ToastProvider';
import { Icons, Card, CardHead, CardBody, Pill } from '@/components/console';
import CodeEditor from '@/components/ui/CodeEditor';

interface ParsedConfig {
  server?: { host?: string; port?: number; mcp_port?: number };
  logging?: {
    level?: string;
    log_requests?: boolean;
    log_responses?: boolean;
    log_errors?: boolean;
  };
  [key: string]: unknown;
}

interface ConfigForm {
  host: string;
  port: string;
  mcpPort: string;
  logLevel: string;
  logRequests: boolean;
  logResponses: boolean;
  logErrors: boolean;
}

const EMPTY_FORM: ConfigForm = {
  host: '',
  port: '',
  mcpPort: '',
  logLevel: 'info',
  logRequests: false,
  logResponses: false,
  logErrors: false,
};

function formFromConfig(c: ParsedConfig): ConfigForm {
  return {
    host: c.server?.host ?? '',
    port: c.server?.port != null ? String(c.server.port) : '',
    mcpPort: c.server?.mcp_port != null ? String(c.server.mcp_port) : '',
    logLevel: c.logging?.level ?? 'info',
    logRequests: c.logging?.log_requests ?? false,
    logResponses: c.logging?.log_responses ?? false,
    logErrors: c.logging?.log_errors ?? false,
  };
}

/** Merge the form fields onto a base config (deep-cloned). Numeric text fields
 *  are applied only when non-empty and valid so clearing a box never writes a
 *  bad value; the log level and toggles always carry a value. */
function applyFormToConfig(base: ParsedConfig, f: ConfigForm): ParsedConfig {
  const c: ParsedConfig = JSON.parse(JSON.stringify(base ?? {}));
  c.server = { ...c.server };
  if (f.host) c.server.host = f.host;
  if (f.port !== '' && !Number.isNaN(Number(f.port))) c.server.port = Number(f.port);
  if (f.mcpPort !== '' && !Number.isNaN(Number(f.mcpPort))) c.server.mcp_port = Number(f.mcpPort);

  c.logging = {
    ...c.logging,
    level: f.logLevel,
    log_requests: f.logRequests,
    log_responses: f.logResponses,
    log_errors: f.logErrors,
  };
  return c;
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label style={{ display: 'block', marginBottom: 12 }}>
      <div
        className="muted"
        style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 4 }}
      >
        {label}
      </div>
      {children}
    </label>
  );
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="row" style={{ gap: 8, marginBottom: 10, cursor: 'pointer', fontSize: 12 }}>
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      {label}
    </label>
  );
}

const fieldStyle: CSSProperties = { height: 30, padding: '4px 10px', fontSize: 12, width: '100%' };

function ConfigurationPage() {
  const api = useApiClient();
  const toast = useToastContext();

  const [yamlContent, setYamlContent] = useState<string>('');
  const [form, setForm] = useState<ConfigForm>(EMPTY_FORM);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const configData = await api.get<ParsedConfig>('/config');
      const parsed = configData ?? {};
      setForm(formFromConfig(parsed));

      try {
        const yamlMod = await import('js-yaml');
        setYamlContent(yamlMod.dump(parsed, { indent: 2, lineWidth: 120 }));
      } catch {
        setYamlContent(JSON.stringify(parsed, null, 2));
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const update = (patch: Partial<ConfigForm>) => {
    setForm((f) => ({ ...f, ...patch }));
    setIsDirty(true);
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      let base: ParsedConfig;
      try {
        const yamlMod = await import('js-yaml');
        base = (yamlMod.load(yamlContent) as ParsedConfig) ?? {};
      } catch {
        toast.error('Failed to parse YAML');
        setError('Failed to parse YAML');
        return;
      }

      const payload = applyFormToConfig(base, form);
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
          <button className="btn" onClick={loadConfig} disabled={loading} type="button">
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
            <Field label="Host">
              <input
                className="input"
                style={fieldStyle}
                value={form.host}
                onChange={(e) => update({ host: e.target.value })}
              />
            </Field>
            <Field label="REST port">
              <input
                className="input"
                type="number"
                style={fieldStyle}
                value={form.port}
                onChange={(e) => update({ port: e.target.value })}
              />
            </Field>
            <Field label="MCP port">
              <input
                className="input"
                type="number"
                style={fieldStyle}
                value={form.mcpPort}
                onChange={(e) => update({ mcpPort: e.target.value })}
              />
            </Field>
          </CardBody>
        </Card>

        <Card>
          <CardHead title="Logging" />
          <CardBody>
            <Field label="Level">
              <select
                className="input"
                style={fieldStyle}
                value={form.logLevel}
                onChange={(e) => update({ logLevel: e.target.value })}
              >
                {['trace', 'debug', 'info', 'warn', 'error'].map((l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ))}
              </select>
            </Field>
            <div style={{ marginTop: 6 }}>
              <Toggle
                label="Log requests"
                checked={form.logRequests}
                onChange={(v) => update({ logRequests: v })}
              />
              <Toggle
                label="Log responses"
                checked={form.logResponses}
                onChange={(v) => update({ logResponses: v })}
              />
              <Toggle
                label="Log errors"
                checked={form.logErrors}
                onChange={(v) => update({ logErrors: v })}
              />
            </div>
          </CardBody>
        </Card>
      </div>

      <Card>
        <CardHead
          title="Raw config"
          sub="everything not covered by the forms above · YAML"
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
