import { useEffect, useRef, useState } from 'react';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import LoadingState from '@/components/LoadingState';
import {
  Icons,
  Sparkline,
  StatusPill,
  Pill,
  Card,
  CardHead,
  CardBody,
  KeyValue,
  KeyValueRow,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';
import type { Collection } from '@/hooks/useCollections';

const SPARK = (n: number, base: number, amp: number): number[] =>
  Array.from({ length: n }, (_, i) => base + Math.sin(i / 2) * amp + Math.random() * amp * 0.3);

function CollectionsPage() {
  const { listCollections } = useCollections();
  const { collections, loading, setCollections, setLoading, setError } = useCollectionsStore();
  const ref = useRef<NodeJS.Timeout | null>(null);
  const [filter, setFilter] = useState('');
  const [selectedName, setSelectedName] = useState<string | null>(null);

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

  const list = Array.isArray(collections) ? collections : [];
  const filtered = filter
    ? list.filter((c) => c.name.toLowerCase().includes(filter.toLowerCase()))
    : list;

  // Default selection = first filtered item; user click overrides
  const selected =
    (selectedName && list.find((c) => c.name === selectedName)) || filtered[0] || null;

  const totalVectors = list.reduce((s, c) => s + (c.vector_count ?? 0), 0);

  if (loading && !list.length) return <LoadingState message="Loading collections..." />;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Collections</h1>
          <p className="page-sub">
            {list.length} collections · {formatNumber(totalVectors)} total vectors
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn" onClick={fetchCollections}>
            <Icons.refresh size={13} />
            Refresh
          </button>
          <button className="btn primary">
            <Icons.plus size={13} />
            Create collection
          </button>
        </div>
      </div>

      <div className="grid grid-1-2" style={{ gap: 14 }}>
        {/* List card */}
        <Card>
          <CardHead>
            <input
              className="input"
              placeholder="Filter collections…"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              style={{ height: 30, padding: '4px 10px', fontSize: 12 }}
              aria-label="Filter collections"
            />
          </CardHead>
          <CardBody tight>
            <div style={{ maxHeight: 560, overflowY: 'auto' }}>
              {filtered.map((c) => {
                const isSel = selected?.name === c.name;
                return (
                  <div
                    key={c.name}
                    onClick={() => setSelectedName(c.name)}
                    style={{
                      padding: '12px 14px',
                      borderBottom: '1px solid var(--border)',
                      cursor: 'pointer',
                      background: isSel ? 'var(--panel-hi)' : 'transparent',
                      borderLeft: isSel ? '2px solid var(--teal)' : '2px solid transparent',
                    }}
                  >
                    <div className="row" style={{ marginBottom: 4 }}>
                      <Icons.database size={13} className="muted" />
                      <span style={{ fontSize: 13, fontWeight: 500 }}>{c.name}</span>
                      <span className="right">
                        <StatusPill status={(c as { status?: string }).status ?? 'healthy'} />
                      </span>
                    </div>
                    <div
                      className="row mono"
                      style={{ fontSize: 11, color: 'var(--text-2)', gap: 14, marginLeft: 21 }}
                    >
                      <span>{formatNumber(c.vector_count ?? 0)} vec</span>
                      <span>{c.dimension ?? '—'}d</span>
                      <span>{(c as { metric?: string }).metric ?? 'cosine'}</span>
                    </div>
                  </div>
                );
              })}
              {!filtered.length && (
                <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
                  No collections match "{filter}".
                </div>
              )}
            </div>
          </CardBody>
        </Card>

        {/* Detail column */}
        <div className="col" style={{ gap: 14 }}>
          {selected ? (
            <>
              <Card>
                <CardHead>
                  <div className="row" style={{ gap: 10 }}>
                    <Icons.database size={16} className="muted" />
                    <div>
                      <div style={{ fontSize: 15, fontWeight: 600 }}>{selected.name}</div>
                      <div className="mono muted-2" style={{ fontSize: 11 }}>
                        collection · {formatNumber(selected.vector_count ?? 0)} vectors
                      </div>
                    </div>
                  </div>
                  <div className="row" style={{ gap: 6 }}>
                    {/* TODO(actions): wire reindex/copy/delete to API */}
                    <button className="btn sm">
                      <Icons.refresh size={11} />
                      Reindex
                    </button>
                    <button className="btn sm">
                      <Icons.copy size={11} />
                      Copy ID
                    </button>
                    <button className="btn sm magenta">
                      <Icons.trash size={11} />
                      Delete
                    </button>
                  </div>
                </CardHead>
                <CardBody>
                  <div className="grid grid-4" style={{ gap: 14, marginBottom: 14 }}>
                    <div>
                      <div
                        className="muted"
                        style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
                      >
                        Vectors
                      </div>
                      <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                        {formatNumber(selected.vector_count ?? 0)}
                      </div>
                    </div>
                    <div>
                      <div
                        className="muted"
                        style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
                      >
                        Dimension
                      </div>
                      <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                        {selected.dimension ?? '—'}
                      </div>
                    </div>
                    <div>
                      <div
                        className="muted"
                        style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
                      >
                        Metric
                      </div>
                      <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                        {(selected as { metric?: string }).metric ?? 'cosine'}
                      </div>
                    </div>
                    <div>
                      <div
                        className="muted"
                        style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
                      >
                        Status
                      </div>
                      <div style={{ marginTop: 4 }}>
                        <StatusPill status={(selected as { status?: string }).status ?? 'healthy'} />
                      </div>
                    </div>
                  </div>
                  <KeyValue>
                    <KeyValueRow term="Index type">HNSW · M=16, ef=200</KeyValueRow>
                    <KeyValueRow term="Distance">
                      {(selected as { metric?: string }).metric ?? 'cosine'} (pre-normalised)
                    </KeyValueRow>
                    <KeyValueRow term="Quantization">
                      <Pill tone="teal">SQ-8bit</Pill>
                    </KeyValueRow>
                    <KeyValueRow term="Embedding">
                      BM25 <span className="muted">· dim {selected.dimension ?? '—'}</span>
                    </KeyValueRow>
                  </KeyValue>
                </CardBody>
              </Card>

              <div className="grid grid-2" style={{ gap: 14 }}>
                <Card>
                  <CardHead title="Vector growth · 7d" />
                  <CardBody>
                    <Sparkline
                      data={SPARK(40, (selected.vector_count ?? 100) / 1000, 8)}
                      width={420}
                      height={100}
                      color="var(--magenta)"
                      ariaLabel={`Vector growth for ${selected.name} over 7 days`}
                    />
                    <div
                      className="row mono"
                      style={{
                        fontSize: 11,
                        color: 'var(--text-2)',
                        justifyContent: 'space-between',
                        marginTop: 6,
                      }}
                    >
                      <span>−7d</span>
                      <span>−3d</span>
                      <span>now</span>
                    </div>
                  </CardBody>
                </Card>
                <Card>
                  <CardHead title="Query throughput · 24h" sub="qpm" />
                  <CardBody>
                    <Sparkline
                      data={SPARK(40, 60, 20)}
                      width={420}
                      height={100}
                      color="var(--teal)"
                      ariaLabel={`Query throughput for ${selected.name} over 24 hours`}
                    />
                    <div
                      className="row mono"
                      style={{
                        fontSize: 11,
                        color: 'var(--text-2)',
                        justifyContent: 'space-between',
                        marginTop: 6,
                      }}
                    >
                      <span>−24h</span>
                      <span>−12h</span>
                      <span>now</span>
                    </div>
                  </CardBody>
                </Card>
              </div>
            </>
          ) : (
            <Card>
              <CardBody>
                <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
                  Select a collection to view its details.
                </div>
              </CardBody>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}

export default CollectionsPage;
