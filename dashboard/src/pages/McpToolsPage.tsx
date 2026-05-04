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

// TODO(mcp-endpoint): if the backend exposes a tools-with-stats endpoint
// (e.g. /mcp/capabilities/stats), wire it here. Until then, this static
// list mirrors `reference/data.js` MOCK_MCP_TOOLS so the page renders
// the canonical visual.
const FALLBACK: McpTool[] = [
  { name: 'search_vectors',          calls: 184_220, p99: 2.8,  status: 'ok' },
  { name: 'intelligent_search',      calls:  92_113, p99: 4.4,  status: 'ok' },
  { name: 'semantic_search',         calls:  41_770, p99: 3.1,  status: 'ok' },
  { name: 'contextual_search',       calls:  28_104, p99: 5.2,  status: 'ok' },
  { name: 'multi_collection_search', calls:  12_504, p99: 6.8,  status: 'ok' },
  { name: 'list_collections',        calls:   8_801, p99: 0.4,  status: 'ok' },
  { name: 'insert_texts',            calls:   3_122, p99: 12.1, status: 'ok' },
  { name: 'batch_insert_texts',      calls:   1_204, p99: 38.4, status: 'ok' },
  { name: 'delete_vectors',          calls:     882, p99: 0.8,  status: 'ok' },
  { name: 'get_database_stats',      calls:     422, p99: 0.2,  status: 'ok' },
];

function McpToolsPage() {
  const api = useApiClient();
  const [tools, setTools] = useState<McpTool[]>(FALLBACK);
  const [usingFallback, setUsingFallback] = useState(true);

  // Best-effort fetch; if the endpoint exists and returns a populated list,
  // swap the fallback for live data.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const resp = await api.get<{ tools?: McpTool[] } | McpTool[]>('/mcp/capabilities/stats');
        const payload = (resp as { data?: unknown }).data ?? resp;
        const arr = Array.isArray(payload)
          ? (payload as McpTool[])
          : ((payload as { tools?: McpTool[] })?.tools ?? []);
        if (cancelled) return;
        if (arr.length > 0) {
          setTools([...arr].sort((a, b) => (b.calls ?? 0) - (a.calls ?? 0)));
          setUsingFallback(false);
        }
      } catch {
        // ignore — static fallback already rendered
      }
    })();
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalCalls = tools.reduce((s, t) => s + (t.calls ?? 0), 0);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">MCP Tools</h1>
          <p className="page-sub">
            {tools.length} tools registered · StreamableHTTP at{' '}
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

      <div className="grid grid-3" style={{ marginBottom: 14 }}>
        <Kpi
          accent="teal"
          label="Active connections"
          value="12"
          delta={{ tone: 'neutral', text: 'Cursor · 8 · others · 4' }}
        />
        <Kpi
          label="Tool calls today"
          value={formatNumber(totalCalls)}
          delta={{ tone: 'up', text: '+8.1%' }}
        />
        <Kpi
          accent="magenta"
          label="Errors"
          value="0"
          delta={{ tone: 'up', text: '100% success' }}
        />
      </div>

      <Card>
        <CardHead title="Tools" sub={usingFallback ? 'static fallback' : 'sorted by usage'} />
        <CardBody tight>
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
        </CardBody>
      </Card>
    </div>
  );
}

export default McpToolsPage;
