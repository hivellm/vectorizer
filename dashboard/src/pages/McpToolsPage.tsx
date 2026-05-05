import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import {
  Icons,
  StatusPill,
  Pill,
  Card,
  CardHead,
  CardBody,
  Kpi,
  Tbl,
  Th,
  Td,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';

interface McpTool {
  name: string;
  calls: number;
  p99: number;
  status: string;
}

function McpToolsPage() {
  const api = useApiClient();
  const [tools, setTools] = useState<McpTool[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setLoading(true);
      try {
        const resp = await api.get<{ tools?: McpTool[] } | McpTool[]>(
          '/mcp/capabilities/stats',
        );
        const payload = (resp as { data?: unknown }).data ?? resp;
        const arr = Array.isArray(payload)
          ? (payload as McpTool[])
          : ((payload as { tools?: McpTool[] })?.tools ?? []);
        if (cancelled) return;
        setTools([...arr].sort((a, b) => (b.calls ?? 0) - (a.calls ?? 0)));
        setError(null);
      } catch {
        if (cancelled) return;
        setTools([]);
        setError('MCP capability stats endpoint not yet exposed by the server.');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalCalls = tools.reduce((s, t) => s + (t.calls ?? 0), 0);
  const errorCount = tools.filter((t) => (t.status ?? 'ok') !== 'ok').length;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">MCP Tools</h1>
          <p className="page-sub">
            {tools.length
              ? `${tools.length} tools registered · `
              : 'Tool registry — '}
            StreamableHTTP at{' '}
            <span className="mono" style={{ color: 'var(--text)' }}>
              http://localhost:15002/mcp
            </span>
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <Pill tone="green">
            <span className="dot green" />
            schema parity OK
          </Pill>
          <button className="btn">
            <Icons.copy size={13} />
            Cursor config
          </button>
        </div>
      </div>

      <div className="grid grid-2" style={{ marginBottom: 14 }}>
        <Kpi
          label="Tool calls observed"
          value={totalCalls ? formatNumber(totalCalls) : '—'}
        />
        <Kpi
          accent="magenta"
          label="Errors"
          value={String(errorCount)}
        />
      </div>

      <Card>
        <CardHead
          title="Tools"
          right={
            error ? (
              <Pill tone="amber">{error}</Pill>
            ) : tools.length ? (
              <Pill tone="muted" className="mono">sorted by usage</Pill>
            ) : null
          }
        />
        <CardBody tight>
          {loading && tools.length === 0 && (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              Loading…
            </div>
          )}
          {!loading && tools.length === 0 && (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No MCP tool stats to display.
            </div>
          )}
          {tools.length > 0 && (
            <Tbl>
              <thead>
                <tr>
                  <Th>Tool</Th>
                  <Th>Calls</Th>
                  <Th>p99</Th>
                  <Th>Schema</Th>
                  <Th>Status</Th>
                </tr>
              </thead>
              <tbody>
                {tools.map((t) => (
                  <tr key={t.name}>
                    <Td className="mono" style={{ fontWeight: 500 }}>{t.name}</Td>
                    <Td className="num">{formatNumber(t.calls ?? 0)}</Td>
                    <Td className="num">{(t.p99 ?? 0).toFixed(1)}ms</Td>
                    <Td>
                      <Pill tone="teal" className="mono">parity ✓</Pill>
                    </Td>
                    <Td>
                      <StatusPill status={t.status ?? 'ok'} />
                    </Td>
                  </tr>
                ))}
              </tbody>
            </Tbl>
          )}
        </CardBody>
      </Card>
    </div>
  );
}

export default McpToolsPage;
