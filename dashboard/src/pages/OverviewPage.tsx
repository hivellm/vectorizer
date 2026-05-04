import { useEffect } from 'react';
import { useCollections } from '@/hooks/useCollections';
import { useWsTopic } from '@/providers/WsDashboardProvider';
import { useMetrics } from '@/hooks/useMetrics';
import { useStats } from '@/hooks/useStats';
import { useStatus } from '@/hooks/useStatus';
import { useEvents } from '@/hooks/useEvents';
import { useRuntimeMetrics } from '@/hooks/useRuntimeMetrics';
import { useCollectionsStore } from '@/stores/collections';
import LoadingState from '@/components/LoadingState';
import {
  Icons,
  Ring,
  StatusPill,
  Pill,
  Card,
  CardHead,
  CardBody,
  Kpi,
  Tbl,
  Th,
  Td,
  KeyValue,
  KeyValueRow,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';
import type { Collection } from '@/hooks/useCollections';

// Map a server event level to the console.css dot tone. Levels follow the
// reference design: ok→green, warn→amber, info→teal, error→red.
function eventDotTone(level: string): 'green' | 'amber' | 'teal' | 'red' {
  const l = level.toLowerCase();
  if (l === 'ok') return 'green';
  if (l === 'warn' || l === 'warning') return 'amber';
  if (l === 'error' || l === 'err' || l === 'fail') return 'red';
  return 'teal';
}

function OverviewPage() {
  const { listCollections } = useCollections();
  const { collections, loading, setCollections, setLoading, setError } = useCollectionsStore();
  const { metrics } = useMetrics();
  const { stats } = useStats();
  const { status: serverStatus } = useStatus();
  const events = useEvents();
  const { metrics: runtimeMetrics, loading: runtimeLoading, error: runtimeError } = useRuntimeMetrics();

  const fetchCollections = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listCollections();
      const arr = Array.isArray(data)
        ? data
        : ((data as unknown as { collections?: Collection[] })?.collections ?? []);
      setCollections(arr);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load collections');
      setCollections([]);
    } finally {
      setLoading(false);
    }
  };

  // Phase30 — initial paint via REST one-shot, then refetch on every
  // WS `collections` snapshot. The slim snapshot is just a trigger;
  // the rich `Collection[]` shape the overview rendering wants comes
  // from `GET /collections` as before.
  useEffect(() => {
    fetchCollections();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const wsSnap = useWsTopic<{ collections: unknown[] }>('collections');
  useEffect(() => {
    if (!wsSnap) return;
    fetchCollections();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wsSnap]);

  if (loading && !collections.length) return <LoadingState message="Loading dashboard..." />;

  const list = Array.isArray(collections) ? collections : [];
  const totalVectors = list.reduce((s, c) => s + (c.vector_count ?? 0), 0);
  const top = list.slice(0, 6);

  // Real KPIs come from /stats (totals) + /health (cache) via useMetrics +
  // useStats. CPU / memory / connections now come from GET /metrics/runtime.
  const qps = metrics.qps;
  // /health emits hit_rate as 0..1; clamp + scale for the percentage KPI.
  const cacheHitPct = Math.max(0, Math.min(100, stats.cache.hitRate * 100));

  // Ring gauge values from /metrics/runtime. While the hook is still on its
  // first fetch or has errored, runtimeUnavailable is true and the JSX renders
  // '--' instead of a zero value.
  const runtimeUnavailable = runtimeLoading || runtimeError !== null;
  const cpu = runtimeMetrics.cpuPercent;
  const mem = runtimeMetrics.memoryPercent;
  const conns = runtimeMetrics.activeConnections;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Overview</h1>
          <p className="page-sub">Real-time health of the Vectorizer node</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn" onClick={fetchCollections}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button className="btn primary">
            <Icons.plus size={13} />
            New Collection
          </button>
        </div>
      </div>

      <div className="grid grid-4" style={{ marginBottom: 14 }}>
        <Kpi
          accent="teal"
          label={
            <>
              <Icons.zap size={12} />
              Queries / sec
            </>
          }
          value={qps.toLocaleString()}
          unit="qps"
        />
        <Kpi
          label={
            <>
              <Icons.cpu size={12} />
              Search latency p99
            </>
          }
          value={runtimeUnavailable ? '--' : runtimeMetrics.throughputByRoute.reduce((max, r) => Math.max(max, r.p99Ms), 0).toFixed(1)}
          unit="ms"
        />
        <Kpi
          accent="magenta"
          label={
            <>
              <Icons.layers size={12} />
              Total vectors
            </>
          }
          value={formatNumber(totalVectors)}
          delta={{ tone: 'neutral', text: `${list.length} collections` }}
        />
        <Kpi
          label={
            <>
              <Icons.flame size={12} />
              Cache hit rate
            </>
          }
          value={cacheHitPct.toFixed(1)}
          unit="%"
        />
      </div>

      <div className="grid grid-2-1" style={{ marginBottom: 14 }}>
        <Card>
          <CardHead
            title="System Health"
            right={
              <Pill tone="green" live>
                <span className="dot green" />
                healthy
              </Pill>
            }
          />
          <CardBody>
            <div className="grid grid-3" style={{ gap: 18, alignItems: 'center' }}>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring
                  value={runtimeUnavailable ? 0 : cpu}
                  max={100}
                  label={runtimeUnavailable ? '--' : `${cpu.toFixed(0)}%`}
                  sub="CPU"
                  color="var(--teal)"
                />
              </div>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring
                  value={runtimeUnavailable ? 0 : mem}
                  max={100}
                  label={runtimeUnavailable ? '--' : `${mem.toFixed(1)}%`}
                  sub="MEMORY"
                  color="var(--magenta)"
                />
              </div>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring
                  value={runtimeUnavailable ? 0 : conns}
                  max={500}
                  label={runtimeUnavailable ? '--' : String(conns)}
                  sub="CONNECTIONS"
                  color="var(--amber)"
                />
              </div>
            </div>
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Server binary">
                {`vectorizer ${serverStatus.version || '—'}`}
              </KeyValueRow>
              <KeyValueRow term="Bind">
                {`${window.location.origin} (REST)`}
              </KeyValueRow>
              <KeyValueRow term="Workspace">{`${list.length} collections`}</KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>

        <Card>
          <CardHead title="Quantization" />
          <CardBody>
            {/* compression_ratio and default_quantization are not yet exposed
                by GET /stats (phase25 §5 is still open). Per-collection
                quantization detail is reported on each collection's detail
                page once that endpoint ships. */}
            <div
              className="muted"
              style={{ fontSize: 12, lineHeight: 1.6 }}
            >
              Per-collection quantization is reported on each collection&apos;s
              detail page. Aggregate compression ratio and recall metrics
              will appear here once the backend exposes them via{' '}
              <span className="mono">GET /stats</span>.
            </div>
          </CardBody>
        </Card>
      </div>

      <div className="grid grid-2-1">
        <Card>
          <CardHead title="Top Collections" />
          <CardBody tight>
            <Tbl>
              <thead>
                <tr>
                  <Th>Name</Th>
                  <Th>Vectors</Th>
                  <Th>Dim</Th>
                  <Th>Status</Th>
                </tr>
              </thead>
              <tbody>
                {top.map((c) => (
                  <tr key={c.name}>
                    <Td>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <Icons.database size={13} className="muted" />
                        <span style={{ fontWeight: 500 }}>{c.name}</span>
                      </div>
                    </Td>
                    <Td className="num">{formatNumber(c.vector_count ?? 0)}</Td>
                    <Td className="num">{c.dimension ?? '—'}</Td>
                    <Td>
                      <StatusPill status={(c as { status?: string }).status ?? 'healthy'} />
                    </Td>
                  </tr>
                ))}
              </tbody>
            </Tbl>
          </CardBody>
        </Card>

        <Card>
          <CardHead
            title="Recent Events"
            right={
              <Pill tone="green" live>
                <span className="dot green" />
                live
              </Pill>
            }
          />
          <CardBody tight>
            <div className="scroll-body">
              {events.available && events.events.length > 0 ? (
                <ul style={{ listStyle: 'none', margin: 0, padding: 0 }}>
                  {events.events.map((e, i) => (
                    <li
                      key={`${e.ts}-${i}`}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 10,
                        padding: '10px 14px',
                        borderBottom: '1px solid var(--line)',
                        fontSize: 12,
                      }}
                    >
                      <span className={`dot ${eventDotTone(e.level)}`} aria-hidden="true" />
                      <span className="muted mono" style={{ fontSize: 11, minWidth: 92 }}>
                        {e.ts}
                      </span>
                      <span style={{ color: 'var(--text-1)' }}>{e.msg}</span>
                    </li>
                  ))}
                </ul>
              ) : (
                <div style={{ padding: 24, color: 'var(--text-2)' }}>No recent events</div>
              )}
            </div>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

export default OverviewPage;
