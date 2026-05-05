import { useEffect, useMemo, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import { useToastContext } from '@/providers/ToastProvider';
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

  // Insert-vector inline form
  const [insertOpen, setInsertOpen] = useState(false);
  const [insertId, setInsertId] = useState('');
  const [insertText, setInsertText] = useState('');
  const [insertPayload, setInsertPayload] = useState('');
  const [inserting, setInserting] = useState(false);

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

  // Fetch vectors for the active collection. Returned promise resolves
  // after state has been updated so callers (insert/delete) can chain.
  const fetchVectors = async (preserveSelectionId?: string | null) => {
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
      if (preserveSelectionId) {
        const match = arr.find((v) => v.id === preserveSelectionId);
        setSelected(match ?? arr[0] ?? null);
      } else {
        setSelected(arr[0] ?? null);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch vectors');
      setVectors([]);
      setSelected(null);
    } finally {
      setLoading(false);
    }
  };

  // Fetch vectors whenever collection changes
  useEffect(() => {
    if (!collection) return;
    let cancelled = false;
    (async () => {
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
        if (cancelled) return;
        setVectors(arr);
        setSelected(arr[0] ?? null);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to fetch vectors');
        setVectors([]);
        setSelected(null);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [collection]);

  const handleCopyId = async () => {
    if (!selected) return;
    try {
      await navigator.clipboard.writeText(selected.id);
      toast.success('Vector ID copied');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to copy');
    }
  };

  const handleDelete = async () => {
    if (!selected || !collection) return;
    if (!window.confirm(`Delete vector "${selected.id}"? This cannot be undone.`)) return;
    const id = selected.id;
    try {
      await api.delete(
        `/collections/${encodeURIComponent(collection)}/vectors/${encodeURIComponent(id)}`,
      );
      toast.success(`Vector "${id}" deleted`);
      await fetchVectors();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete vector');
    }
  };

  const resetInsertForm = () => {
    setInsertId('');
    setInsertText('');
    setInsertPayload('');
  };

  const handleInsert = async () => {
    if (!collection) {
      toast.error('Select a collection first');
      return;
    }
    const id = insertId.trim();
    const text = insertText.trim();
    if (!id) {
      toast.error('Vector ID is required');
      return;
    }
    if (!text) {
      toast.error('Vector text is required');
      return;
    }
    let parsedPayload: Record<string, unknown> | undefined;
    if (insertPayload.trim().length > 0) {
      try {
        const parsed = JSON.parse(insertPayload);
        if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
          parsedPayload = parsed as Record<string, unknown>;
        } else {
          toast.error('Payload must be a JSON object');
          return;
        }
      } catch {
        toast.error('Invalid JSON payload');
        return;
      }
    }

    setInserting(true);
    try {
      await api.post('/insert_texts', {
        collection,
        texts: [
          parsedPayload
            ? { id, text, metadata: parsedPayload }
            : { id, text },
        ],
      });
      toast.success(`Vector "${id}" inserted`);
      resetInsertForm();
      setInsertOpen(false);
      await fetchVectors(id);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to insert vector');
    } finally {
      setInserting(false);
    }
  };

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
          <button
            className="btn"
            onClick={() => setInsertOpen((v) => !v)}
            disabled={!collection}
            aria-expanded={insertOpen}
          >
            <Icons.plus size={13} />
            Insert vector
          </button>
        </div>
      </div>

      {insertOpen && (
        <Card>
          <CardHead title="Insert vector" />
          <CardBody>
            <div className="col" style={{ gap: 10 }}>
              <div className="grid" style={{ gridTemplateColumns: '1fr 1fr', gap: 10 }}>
                <div className="field">
                  <label className="field-label" htmlFor="insert-vec-id">ID</label>
                  <input
                    id="insert-vec-id"
                    className="input"
                    placeholder="vector-id"
                    value={insertId}
                    onChange={(e) => setInsertId(e.target.value)}
                    disabled={inserting}
                  />
                </div>
                <div className="field">
                  <label className="field-label" htmlFor="insert-vec-collection">Collection</label>
                  <input
                    id="insert-vec-collection"
                    className="input"
                    value={collection}
                    readOnly
                    aria-readonly
                  />
                </div>
              </div>
              <div className="field">
                <label className="field-label" htmlFor="insert-vec-text">Text</label>
                <textarea
                  id="insert-vec-text"
                  className="input"
                  rows={3}
                  placeholder="Source text to embed"
                  value={insertText}
                  onChange={(e) => setInsertText(e.target.value)}
                  disabled={inserting}
                  style={{ minHeight: 70, resize: 'vertical' }}
                />
              </div>
              <div className="field">
                <label className="field-label" htmlFor="insert-vec-payload">
                  Payload (JSON, optional)
                </label>
                <textarea
                  id="insert-vec-payload"
                  className="input mono"
                  rows={3}
                  placeholder='{"source": "manual"}'
                  value={insertPayload}
                  onChange={(e) => setInsertPayload(e.target.value)}
                  disabled={inserting}
                  style={{ minHeight: 70, resize: 'vertical', fontSize: 12 }}
                />
              </div>
              <div className="row" style={{ gap: 8, justifyContent: 'flex-end' }}>
                <button
                  type="button"
                  className="btn"
                  onClick={() => {
                    setInsertOpen(false);
                    resetInsertForm();
                  }}
                  disabled={inserting}
                >
                  Cancel
                </button>
                <button
                  type="button"
                  className="btn primary"
                  onClick={handleInsert}
                  disabled={inserting}
                >
                  {inserting ? 'Inserting…' : 'Insert vector'}
                </button>
              </div>
            </div>
          </CardBody>
        </Card>
      )}

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
                <button
                  className="btn sm"
                  disabled={!selected}
                  onClick={handleCopyId}
                >
                  <Icons.copy size={11} />
                  Copy
                </button>
                <button
                  className="btn sm magenta"
                  disabled={!selected}
                  onClick={handleDelete}
                >
                  <Icons.trash size={11} />
                  Delete
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
    </div>
  );
}

export default VectorsPage;
