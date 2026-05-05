import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import {
  Icons,
  Pill,
  Card,
  CardHead,
  CardBody,
} from '@/components/console';
import type { Collection } from '@/hooks/useCollections';

type SearchType = 'intelligent' | 'semantic' | 'contextual' | 'multi';

interface SearchTypeMeta {
  id: SearchType;
  label: string;
  desc: string;
  icon: keyof typeof Icons;
  endpoint: string;
}

const SEARCH_TYPES: readonly SearchTypeMeta[] = [
  {
    id: 'intelligent',
    label: 'Intelligent',
    desc: 'Multi-query generation + domain expansion + MMR diversification',
    icon: 'sparkles',
    endpoint: '/intelligent_search',
  },
  {
    id: 'semantic',
    label: 'Semantic',
    desc: 'High-precision search with semantic reranking and similarity threshold',
    icon: 'zap',
    endpoint: '/semantic_search',
  },
  {
    id: 'contextual',
    label: 'Contextual',
    desc: 'Context-aware search with metadata filtering and context reranking',
    icon: 'layers',
    endpoint: '/contextual_search',
  },
  {
    id: 'multi',
    label: 'Multi-collection',
    desc: 'Cross-collection search with intelligent reranking and deduplication',
    icon: 'globe',
    endpoint: '/multi_collection_search',
  },
];

interface ResultRow {
  score: number;
  title?: string;
  snippet?: string;
  text?: string;
  collection?: string;
  id?: string;
  path?: string;
}

// Highlight query terms inside `snippet` by emitting React nodes (no
// dangerouslySetInnerHTML — `text` is server-supplied content and the
// query is user-supplied, so escaping via DOM rather than HTML strings
// avoids the XSS surface entirely).
function renderHighlighted(text: string, query: string): ReactNode {
  if (!query.trim() || !text) return text;
  const escaped = query.trim().replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp(`(${escaped})`, 'gi');
  const parts = text.split(re);
  return parts.map((part, i) =>
    re.test(part) && i % 2 === 1 ? <mark key={i}>{part}</mark> : <span key={i}>{part}</span>,
  );
}

function SearchPage() {
  const api = useApiClient();
  const { listCollections } = useCollections();
  const { collections, setCollections } = useCollectionsStore();

  const [type, setType] = useState<SearchType>('intelligent');
  const [query, setQuery] = useState('How does REST/MCP parity work in the capability registry?');
  const [collection, setCollection] = useState<string>('all');
  const [limit, setLimit] = useState(10);
  const [threshold, setThreshold] = useState(0.7);
  const [running, setRunning] = useState(false);
  const [results, setResults] = useState<ResultRow[]>([]);
  const [latencyMs, setLatencyMs] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  // First-load: hydrate collection list (read-only, doesn't poll)
  useEffect(() => {
    (async () => {
      try {
        const data = await listCollections();
        const arr = Array.isArray(data)
          ? data
          : ((data as unknown as { collections?: Collection[] })?.collections ?? []);
        setCollections(arr);
      } catch {
        // swallow — page still works without dropdown
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const cur = useMemo(() => SEARCH_TYPES.find((s) => s.id === type)!, [type]);
  const CurIc = Icons[cur.icon];

  const runSearch = async () => {
    if (!query.trim()) return;
    setRunning(true);
    setError(null);
    const t0 = performance.now();
    try {
      const body: Record<string, unknown> = {
        query,
        limit,
        threshold,
      };
      if (collection !== 'all') body.collection = collection;
      // useApiClient.post() returns the unwrapped data via middleware. The
      // defensive checks below also handle the raw `{ data: ... }` shape so
      // the page works against both client variants.
      const resp = await api.post<{ results?: ResultRow[] } | ResultRow[]>(cur.endpoint, body);
      const payload = (resp as { data?: unknown })?.data ?? resp;
      const list = Array.isArray(payload)
        ? (payload as ResultRow[])
        : ((payload as { results?: ResultRow[] })?.results ?? []);
      setResults(list);
      setLatencyMs(Math.round(performance.now() - t0));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
      setResults([]);
      setLatencyMs(null);
    } finally {
      setRunning(false);
    }
  };

  const collList = Array.isArray(collections) ? collections : [];

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Search Playground</h1>
          <p className="page-sub">
            Test the four intelligent search modes against any collection · results streamed
            from <span className="mono" style={{ color: 'var(--text)' }}>POST {cur.endpoint}</span>
          </p>
        </div>
      </div>

      {/* Tabs + form */}
      <Card>
        <div className="tabs" role="tablist" aria-label="Search type">
          {SEARCH_TYPES.map((s) => {
            const Ic = Icons[s.icon];
            const isActive = type === s.id;
            return (
              <button
                key={s.id}
                role="tab"
                type="button"
                aria-selected={isActive}
                className={`tab ${isActive ? 'active' : ''}`}
                onClick={() => setType(s.id)}
                style={{ background: 'none', border: 0, font: 'inherit', cursor: 'pointer' }}
              >
                <Ic size={13} />
                {s.label}
              </button>
            );
          })}
        </div>
        <div
          style={{
            padding: '12px 18px',
            borderBottom: '1px solid var(--border)',
            display: 'flex',
            alignItems: 'center',
            gap: 10,
          }}
        >
          <CurIc size={14} style={{ color: 'var(--teal-hi)' }} />
          <span style={{ fontSize: 12, color: 'var(--text-1)' }}>{cur.desc}</span>
        </div>

        <CardBody className="search-form">
          <div
            className="grid"
            style={{
              gridTemplateColumns: '1fr 200px 110px 110px auto',
              gap: 10,
              alignItems: 'end',
            }}
          >
            <div className="field">
              <label className="field-label" htmlFor="search-query">Query</label>
              <input
                id="search-query"
                className="input"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Type a natural-language query…"
                onKeyDown={(e) => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) runSearch(); }}
              />
            </div>
            <div className="field">
              <label className="field-label" htmlFor="search-coll">Collection</label>
              <select
                id="search-coll"
                className="input"
                value={collection}
                onChange={(e) => setCollection(e.target.value)}
              >
                <option value="all">all collections</option>
                {collList.map((c) => (
                  <option key={c.name} value={c.name}>{c.name}</option>
                ))}
              </select>
            </div>
            <div className="field">
              <label className="field-label" htmlFor="search-limit">Limit</label>
              <input
                id="search-limit"
                className="input mono"
                type="number"
                min={1}
                max={100}
                value={limit}
                onChange={(e) => setLimit(Number(e.target.value))}
              />
            </div>
            <div className="field">
              <label className="field-label" htmlFor="search-thr">Threshold</label>
              <input
                id="search-thr"
                className="input mono"
                type="number"
                step="0.05"
                min={0}
                max={1}
                value={threshold}
                onChange={(e) => setThreshold(Number(e.target.value))}
              />
            </div>
            <button className="btn primary" onClick={runSearch} disabled={running}>
              {running ? (
                <>
                  <Icons.refresh size={13} />
                  Running…
                </>
              ) : (
                <>
                  <Icons.search size={13} />
                  Run search
                </>
              )}
            </button>
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      {/* Results + Request */}
      <div className="grid" style={{ gridTemplateColumns: '1.7fr 1fr', gap: 14 }}>
        <Card>
          <CardHead
            title="Results"
            right={
              <div className="row" style={{ gap: 8 }}>
                <Pill tone="muted" className="mono">
                  {results.length ? `${results.length} hits` : '—'}
                </Pill>
                {latencyMs !== null && (
                  <Pill tone="teal" className="mono">{latencyMs} ms</Pill>
                )}
              </div>
            }
          />
          <CardBody tight>
            {error && (
              <div style={{ padding: 14 }}>
                <Pill tone="red">{error}</Pill>
              </div>
            )}
            {!error && results.length === 0 && (
              <div style={{ padding: 40, textAlign: 'center', color: 'var(--text-2)' }}>
                {running ? 'Running…' : 'Run a query to see results'}
              </div>
            )}
            {results.map((r, i) => {
              const text = r.snippet ?? r.text ?? '';
              return (
                <div key={`${r.id ?? i}`} className="result">
                  <div className="rank">#{i + 1}</div>
                  <div className="body">
                    <div className="title">{r.title ?? r.id ?? `Result ${i + 1}`}</div>
                    <div className="snippet">{renderHighlighted(text, query)}</div>
                    <div className="meta">
                      {r.collection && (
                        <span>
                          <Icons.database size={10} /> {r.collection}
                        </span>
                      )}
                      {r.id && <span>{r.id}</span>}
                      {r.path && <span>{r.path}</span>}
                    </div>
                  </div>
                  <div className="score">
                    <div className="v">{r.score?.toFixed(3) ?? '—'}</div>
                    <div className="l">cosine</div>
                  </div>
                </div>
              );
            })}
          </CardBody>
        </Card>

        <div className="col" style={{ gap: 14 }}>
          <Card>
            <CardHead
              title="Request"
              right={<Pill tone="muted" className="mono">POST</Pill>}
            />
            <CardBody>
              <pre className="code">
                <span className="c"># curl</span>{'\n'}
                curl -X <span className="k">POST</span>{' '}
                http://localhost:15002<span className="s">{cur.endpoint}</span> \{'\n'}
                {'  '}-H <span className="s">"Authorization: Bearer $VK"</span> \{'\n'}
                {'  '}-d <span className="s">'{`{`}{'\n'}
                {'    '}"query": "{query.length > 38 ? query.slice(0, 38) + '…' : query}",{'\n'}
                {'    '}"collection": "{collection}",{'\n'}
                {'    '}"limit": {limit},{'\n'}
                {'    '}"threshold": {threshold}{'\n'}
                {'  '}{`}`}'</span>
              </pre>
            </CardBody>
          </Card>

        </div>
      </div>
    </div>
  );
}

export default SearchPage;
