import { useEffect, useRef, useState } from 'react';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import { useToastContext } from '@/providers/ToastProvider';
import LoadingState from '@/components/LoadingState';
import CreateCollectionModal from '@/components/modals/CreateCollectionModal';
import FileUploadModal from '@/components/modals/FileUploadModal';
import DeleteCollectionModal from '@/components/modals/DeleteCollectionModal';
import {
  Icons,
  StatusPill,
  Card,
  CardHead,
  CardBody,
  KeyValue,
  KeyValueRow,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';
import type { Collection } from '@/hooks/useCollections';

function CollectionsPage() {
  const { listCollections } = useCollections();
  const { collections, loading, setCollections, setLoading, setError } = useCollectionsStore();
  const toast = useToastContext();
  const ref = useRef<NodeJS.Timeout | null>(null);
  const [filter, setFilter] = useState('');
  const [selectedName, setSelectedName] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [uploadOpen, setUploadOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const handleCopyName = async (name: string) => {
    try {
      await navigator.clipboard.writeText(name);
      toast.success('Collection name copied');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to copy');
    }
  };

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
          <button className="btn" onClick={() => setUploadOpen(true)}>
            <Icons.arrowUp size={13} />
            Upload file
          </button>
          <button className="btn primary" onClick={() => setCreateOpen(true)}>
            <Icons.plus size={13} />
            Create collection
          </button>
        </div>
      </div>

      <CreateCollectionModal
        isOpen={createOpen}
        onClose={() => setCreateOpen(false)}
      />
      <FileUploadModal
        isOpen={uploadOpen}
        onClose={() => setUploadOpen(false)}
        onSuccess={fetchCollections}
      />
      {selected && (
        <DeleteCollectionModal
          isOpen={deleteOpen}
          onClose={() => {
            setDeleteOpen(false);
            // The modal removes the deleted collection from the store and
            // refetches the list itself; reset local selection so the
            // detail pane falls back to the next available collection.
            setSelectedName(null);
          }}
          collectionName={selected.name}
        />
      )}

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
                      <span>{c.metric ?? '—'}</span>
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
                    <button
                      className="btn sm"
                      disabled
                      title="Reindex endpoint not yet exposed by the server"
                    >
                      <Icons.refresh size={11} />
                      Reindex
                    </button>
                    <button
                      className="btn sm"
                      onClick={() => handleCopyName(selected.name)}
                    >
                      <Icons.copy size={11} />
                      Copy ID
                    </button>
                    <button
                      className="btn sm magenta"
                      onClick={() => setDeleteOpen(true)}
                    >
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
                        {selected.metric ?? '—'}
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
                    <KeyValueRow term="Distance">
                      {selected.metric ?? <span className="muted">—</span>}
                      {selected.normalization?.enabled ? ' (pre-normalised)' : ''}
                    </KeyValueRow>
                    <KeyValueRow term="Quantization">
                      {selected.quantization?.enabled
                        ? `${selected.quantization.type ?? 'enabled'}${
                            selected.quantization.bits ? ` · ${selected.quantization.bits}-bit` : ''
                          }`
                        : selected.quantization?.enabled === false
                          ? 'disabled'
                          : <span className="muted">—</span>}
                    </KeyValueRow>
                    <KeyValueRow term="Embedding">
                      {selected.embedding_provider ?? <span className="muted">—</span>}
                      {selected.dimension ? (
                        <span className="muted"> · dim {selected.dimension}</span>
                      ) : null}
                    </KeyValueRow>
                    {selected.size?.total && (
                      <KeyValueRow term="Size on disk">{selected.size.total}</KeyValueRow>
                    )}
                    {selected.created_at && (
                      <KeyValueRow term="Created">{selected.created_at}</KeyValueRow>
                    )}
                  </KeyValue>
                </CardBody>
              </Card>

              <div className="grid grid-2" style={{ gap: 14 }}>
                <Card>
                  <CardHead title="Vector growth · 7d" />
                  <CardBody>
                    <div
                      className="muted"
                      style={{ padding: 24, textAlign: 'center', fontSize: 12 }}
                    >
                      Time-series endpoint not yet exposed by the server.
                    </div>
                  </CardBody>
                </Card>
                <Card>
                  <CardHead title="Query throughput · 24h" />
                  <CardBody>
                    <div
                      className="muted"
                      style={{ padding: 24, textAlign: 'center', fontSize: 12 }}
                    >
                      Time-series endpoint not yet exposed by the server.
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
