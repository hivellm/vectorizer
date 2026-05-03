import { useEffect, useRef } from 'react';
import { useCollections } from '@/hooks/useCollections';
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
  Bar,
  Tbl,
  Th,
  Td,
  KeyValue,
  KeyValueRow,
  useTick,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';
import type { Collection } from '@/hooks/useCollections';

const SPARK = (n: number, base: number, amp: number): number[] =>
  Array.from({ length: n }, (_, i) => base + Math.sin(i / 2) * amp + Math.random() * amp * 0.3);

function OverviewPage() {
  const { listCollections } = useCollections();
  const { collections, loading, setCollections, setLoading, setError } = useCollectionsStore();
  const ref = useRef<NodeJS.Timeout | null>(null);
  const tick = useTick(2000);

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

  useEffect(() => {
    fetchCollections();
    ref.current = setInterval(fetchCollections, 30000);
    return () => {
      if (ref.current) clearInterval(ref.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (loading && !collections.length) return <LoadingState message="Loading dashboard..." />;

  const list = Array.isArray(collections) ? collections : [];
  const totalVectors = list.reduce((s, c) => s + (c.vector_count ?? 0), 0);
  const top = list.slice(0, 6);

  // TODO(metrics-endpoint): synthetic until Task 4.1 wires /metrics
  const qps = 2480 + Math.round(Math.sin(tick / 2) * 120);
  const cpu = 38 + Math.sin(tick / 2.5) * 6;
  const mem = 62.4 + Math.sin(tick / 3) * 1.2;
  const conns = 184 + Math.round(Math.cos(tick) * 14);

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
          delta={{ tone: 'up', text: '+12.4% vs 24h' }}
          spark={{ data: SPARK(20, 2400, 200), color: 'var(--teal)' }}
        />
        <Kpi
          label={
            <>
              <Icons.cpu size={12} />
              Search latency p99
            </>
          }
          value="2.8"
          unit="ms"
          delta={{ tone: 'up', text: '−0.4ms vs 24h' }}
          spark={{ data: SPARK(20, 2.8, 0.4), color: 'var(--text-2)' }}
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
          spark={{ data: SPARK(20, 580, 8), color: 'var(--magenta)' }}
        />
        <Kpi
          label={
            <>
              <Icons.flame size={12} />
              Cache hit rate
            </>
          }
          value="94.2"
          unit="%"
          delta={{ tone: 'up', text: '+1.8% vs 24h' }}
          spark={{ data: SPARK(20, 94, 2), color: 'var(--green)' }}
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
                <Ring value={cpu} max={100} label={`${cpu.toFixed(0)}%`} sub="CPU" color="var(--teal)" />
              </div>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring value={mem} max={100} label={`${mem.toFixed(1)}%`} sub="MEMORY" color="var(--magenta)" />
              </div>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring value={conns} max={500} label={String(conns)} sub="CONNECTIONS" color="var(--amber)" />
              </div>
            </div>
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Server binary">vectorizer 3.0.0</KeyValueRow>
              <KeyValueRow term="Bind">127.0.0.1:15002 (REST) · /mcp (StreamableHTTP)</KeyValueRow>
              <KeyValueRow term="Workspace">{`${list.length} collections`}</KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>

        <Card>
          <CardHead title="Quantization" sub="SQ-8bit · default" />
          <CardBody>
            <div style={{ textAlign: 'center', marginBottom: 14 }}>
              <div style={{ fontSize: 36, fontWeight: 600, letterSpacing: '-0.02em' }}>4.0×</div>
              <div className="muted" style={{ fontSize: 11, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                compression ratio
              </div>
            </div>
            <div className="col" style={{ gap: 10 }}>
              <div>
                <div className="row" style={{ fontSize: 11, marginBottom: 4 }}>
                  <span className="muted">MAP score</span>
                  <span className="right mono">+8.9%</span>
                </div>
                <Bar percent={82} ariaLabel="MAP score relative gain" />
              </div>
              <div>
                <div className="row" style={{ fontSize: 11, marginBottom: 4 }}>
                  <span className="muted">Recall@10</span>
                  <span className="right mono">98.4%</span>
                </div>
                <Bar percent={98} tone="magenta" ariaLabel="Recall at 10" />
              </div>
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
              <div style={{ padding: 24, color: 'var(--text-2)' }}>
                Wire events feed in Task 4.2.
              </div>
            </div>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

export default OverviewPage;
