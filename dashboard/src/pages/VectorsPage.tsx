import { useEffect, useMemo, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import { useToastContext } from '@/providers/ToastProvider';
import FileUploadModal from '@/components/modals/FileUploadModal';
import {
  Icons,
  Pill,
  Card,
  CardHead,
  CardBody,
  KeyValue,
  KeyValueRow,
  Tbl,
  Th,
  Td,
} from '@/components/console';
import type { Collection } from '@/hooks/useCollections';

interface VectorRow {
  id: string;
  text?: string;
  dimension?: number;
  vector?: number[];
  norm?: number;
  payload?: Record<string, unknown>;
  inserted_at?: string;
}

const HIST_COUNT = 96;

function VectorsPage() {
  const api = useApiClient();
  const { listCollections } = useCollections();
  const { collections, setCollections } = useCollectionsStore();
  const toast = useToastContext();

  const [collection, setCollection] = useState<string>('');
  const [vectors, setVectors] = useState<VectorRow[]>([]);
  const [filter, setFilter] = useState('');
  const [selected, setSelected] = useState<VectorRow | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [insertOpen, setInsertOpen] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const loadVectors = async () => {
    if (!collection) return;
    setLoading(true);
    setError(null);
    try {
      const resp = await api.get<{ vectors?: VectorRow[] } | VectorRow[]>(
        `/collections/${encodeURIComponent(collection)}/vectors?limit=200`,
      );
      const payload = (resp as { data?: unknown })?.data ?? resp;
      const arr = Array.isArray(payload)
        ? (payload as VectorRow[])
        : ((payload as { vectors?: VectorRow[] })?.vectors ?? []);
      setVectors(arr);
      setSelected((prev) => (prev && arr.find((v) => v.id === prev.id)) || arr[0] || null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch vectors');
      setVectors([]);
      setSelected(null);
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = async () => {
    if (!selected) return;
    try {
      await navigator.clipboard.writeText(selected.id);
      toast.success('Vector ID copied');
    } catch {
      toast.error('Failed to copy to clipboard');
    }
  };

  const handleDeleteVector = async () => {
    if (!selected || deleting) return;
    if (!window.confirm(`Delete vector "${selected.id}" from "${collection}"?`)) return;
    setDeleting(true);
    try {
      await api.delete(
        `/collections/${encodeURIComponent(collection)}/vectors/${encodeURIComponent(selected.id)}`,
      );
      toast.success('Vector deleted');
      await loadVectors();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete vector');
    } finally {
      setDeleting(false);
    }
  };

  // Hydrate collection list once
  useEffect(() => {
    (async () => {
      try {
        const data = await listCollections();
        const arr = Array.isArray(data)
          ? data
          : ((data as unknown as { collections?: Collection[] })?.collections ?? []);
        setCollections(arr);
        if (arr.length && !collection) setCollection(arr[0].name);
      } catch {
        // dropdown stays empty
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Fetch vectors whenever collection changes
  useEffect(() => {
    loadVectors();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [collection]);

  const filtered = useMemo(() => {
    if (!filter.trim()) return vectors;
    const q = filter.toLowerCase();
    return vectors.filter(
      (v) =>
        v.id.toLowerCase().includes(q) ||
        (v.text?.toLowerCase().includes(q) ?? false),
    );
  }, [vectors, filter]);

  // Embedding histogram data: take the first 96 dims of selected.vector (or fewer if shorter)
  const dims = useMemo(() => {
    if (!selected?.vector?.length) return [] as number[];
    return selected.vector.slice(0, HIST_COUNT);
  }, [selected]);

  const collList = Array.isArray(collections) ? collections : [];
  const selectedDim = selected?.dimension ?? selected?.vector?.length ?? 0;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Vector Browser</h1>
          <p className="page-sub">
            Inspect raw embeddings, payloads and norms across collections
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <select
            id="vectors-collection"
            className="input"
            style={{ width: 220, fontSize: 12 }}
            value={collection}
            onChange={(e) => setCollection(e.target.value)}
            aria-label="Collection"
          >
            {collList.length === 0 && <option value="">No collections</option>}
            {collList.map((c) => (
              <option key={c.name} value={c.name}>{c.name}</option>
            ))}
          </select>
          <button className="btn" onClick={() => setInsertOpen(true)}>
            <Icons.plus size={13} />
            Insert vector
          </button>
        </div>
      </div>

      <div className="grid" style={{ gridTemplateColumns: '1.4fr 1fr', gap: 14 }}>
        <Card>
          <CardHead>
            <input
              className="input"
              placeholder="Filter by text or id…"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              style={{ height: 30, padding: '4px 10px', fontSize: 12 }}
              aria-label="Filter vectors"
            />
            <span className="pill muted mono right">
              {filtered.length} / {vectors.length}
            </span>
          </CardHead>
          <CardBody tight>
            {loading && (
              <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
                Loading vectors…
              </div>
            )}
            {error && (
              <div style={{ padding: 14 }}>
                <Pill tone="red">{error}</Pill>
              </div>
            )}
            {!loading && !error && filtered.length === 0 && (
              <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
                {vectors.length === 0 ? 'No vectors in this collection.' : 'No matches.'}
              </div>
            )}
            {filtered.length > 0 && (
              <Tbl>
                <thead>
                  <tr>
                    <Th style={{ width: 140 }}>ID</Th>
                    <Th>Text</Th>
                    <Th>Dim</Th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((v) => {
                    const isSel = selected?.id === v.id;
                    return (
                      <tr
                        key={v.id}
                        className={isSel ? 'active' : ''}
                        onClick={() => setSelected(v)}
                        style={{ cursor: 'pointer' }}
                      >
                        <Td className="id">{v.id}</Td>
                        <Td
                          style={{
                            maxWidth: 380,
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                          }}
                        >
                          {v.text ?? '—'}
                        </Td>
                        <Td className="num muted">{v.dimension ?? v.vector?.length ?? '—'}</Td>
                      </tr>
                    );
                  })}
                </tbody>
              </Tbl>
            )}
          </CardBody>
        </Card>

        <div className="col" style={{ gap: 14 }}>
          <Card>
            <CardHead>
              <div className="row" style={{ gap: 8 }}>
                <Icons.vectors size={14} className="muted" />
                <span className="mono" style={{ fontSize: 12 }}>
                  {selected?.id ?? 'no selection'}
                </span>
              </div>
              <div className="row" style={{ gap: 6 }}>
                <button className="btn sm" disabled={!selected} onClick={handleCopy}>
                  <Icons.copy size={11} />
                  Copy
                </button>
                <button
                  className="btn sm magenta"
                  disabled={!selected || deleting}
                  onClick={handleDeleteVector}
                >
                  <Icons.trash size={11} />
                  {deleting ? 'Deleting…' : 'Delete'}
                </button>
              </div>
            </CardHead>
            <CardBody>
              {selected ? (
                <>
                  <div
                    className="muted"
                    style={{
                      fontSize: 10,
                      textTransform: 'uppercase',
                      letterSpacing: '0.06em',
                      marginBottom: 6,
                    }}
                  >
                    Source text
                  </div>
                  <div style={{ fontSize: 13, lineHeight: 1.6, marginBottom: 14 }}>
                    {selected.text ?? <span className="muted">No text payload</span>}
                  </div>
                  <KeyValue>
                    <KeyValueRow term="Collection">{collection}</KeyValueRow>
                    <KeyValueRow term="Dimension">{selectedDim || '—'}</KeyValueRow>
                    <KeyValueRow term="L2 norm">{selected.norm?.toFixed(6) ?? '—'}</KeyValueRow>
                    <KeyValueRow term="Inserted">{selected.inserted_at ?? '—'}</KeyValueRow>
                    <KeyValueRow term="Payload">
                      <code className="mono" style={{ fontSize: 11 }}>
                        {selected.payload ? JSON.stringify(selected.payload) : '—'}
                      </code>
                    </KeyValueRow>
                  </KeyValue>
                </>
              ) : (
                <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
                  Select a vector to inspect its embedding.
                </div>
              )}
            </CardBody>
          </Card>

          <Card>
            <CardHead
              title={`Embedding · first ${dims.length} of ${selectedDim || '—'} dims`}
              right={<Pill tone="muted" className="mono">f32</Pill>}
            />
            <CardBody>
              {dims.length > 0 ? (
                <>
                  <svg
                    viewBox="0 0 480 100"
                    width="100%"
                    height="100"
                    preserveAspectRatio="none"
                    role="img"
                    aria-label={`Embedding histogram for ${selected?.id ?? 'selection'}`}
                  >
                    {dims.map((v, i) => {
                      const w = 480 / dims.length;
                      const h = Math.abs(v) * 46;
                      const y = v >= 0 ? 50 - h : 50;
                      const color = v >= 0 ? 'var(--teal)' : 'var(--magenta)';
                      return (
                        <rect
                          key={i}
                          x={i * w + 0.5}
                          y={y}
                          width={Math.max(0, w - 1)}
                          height={h}
                          fill={color}
                          opacity="0.85"
                        />
                      );
                    })}
                    <line
                      x1="0"
                      y1="50"
                      x2="480"
                      y2="50"
                      stroke="var(--border)"
                      strokeWidth="1"
                    />
                  </svg>
                  <div
                    className="row mono"
                    style={{
                      fontSize: 10,
                      color: 'var(--text-3)',
                      justifyContent: 'space-between',
                      marginTop: 6,
                    }}
                  >
                    <span>dim 0</span>
                    <span>dim {Math.floor(dims.length / 2)}</span>
                    <span>dim {dims.length}</span>
                  </div>
                </>
              ) : (
                <div
                  style={{
                    padding: 24,
                    color: 'var(--text-2)',
                    textAlign: 'center',
                    fontSize: 12,
                  }}
                >
                  {selected
                    ? 'No embedding data for this vector.'
                    : 'Select a vector to render its embedding.'}
                </div>
              )}
            </CardBody>
          </Card>
        </div>
      </div>

      <FileUploadModal
        isOpen={insertOpen}
        onClose={() => setInsertOpen(false)}
        onSuccess={() => {
          setInsertOpen(false);
          loadVectors();
        }}
      />
    </div>
  );
}

export default VectorsPage;
