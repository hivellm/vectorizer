/**
 * Vector Details Modal — console design.
 */

import Modal from '@/components/ui/Modal';
import CodeEditor from '@/components/ui/CodeEditor';
import { useToastContext } from '@/providers/ToastProvider';
import type { CSSProperties } from 'react';

interface Vector {
  id: string;
  payload?: Record<string, unknown>;
  metadata?: {
    source?: string;
    file_type?: string;
    chunk_index?: number;
    embedding_model?: string;
    dimension?: number;
    similarity_score?: number;
    created_at?: string;
  };
  vector?: number[];
}

interface VectorDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  vector: Vector | null;
  collectionName: string;
  onEdit?: () => void;
}

const SECTION_HEAD: CSSProperties = {
  fontSize: 12,
  fontWeight: 600,
  color: 'var(--text)',
  letterSpacing: '0.04em',
  textTransform: 'uppercase',
  marginBottom: 10,
};

const GRID: CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
  gap: 14,
};

const FIELD_LABEL: CSSProperties = {
  fontSize: 11,
  color: 'var(--text-2)',
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
};

const FIELD_VALUE: CSSProperties = {
  fontSize: 13,
  fontWeight: 500,
  color: 'var(--text)',
  marginTop: 4,
};

export default function VectorDetailsModal({
  isOpen,
  onClose,
  vector,
  collectionName,
  onEdit,
}: VectorDetailsModalProps) {
  const toast = useToastContext();

  if (!vector) return null;

  const formatJSON = (obj: unknown): string => {
    try {
      return JSON.stringify(obj, null, 2);
    } catch {
      return String(obj);
    }
  };

  const copyVectorId = async () => {
    try {
      await navigator.clipboard.writeText(vector.id);
      toast.success('Vector ID copied to clipboard');
    } catch {
      toast.error('Failed to copy Vector ID');
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Vector Details"
      size="xl"
      footer={
        <>
          <button type="button" className="btn" onClick={copyVectorId}>
            Copy ID
          </button>
          {onEdit && (
            <button type="button" className="btn primary" onClick={onEdit}>
              Edit
            </button>
          )}
          <button type="button" className="btn" onClick={onClose}>
            Close
          </button>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
        {/* Basic Information */}
        <div>
          <div style={SECTION_HEAD}>Basic Information</div>
          <div style={GRID}>
            <div>
              <div style={FIELD_LABEL}>Vector ID</div>
              <div
                className="mono"
                style={{ ...FIELD_VALUE, wordBreak: 'break-all' }}
              >
                {vector.id}
              </div>
            </div>
            <div>
              <div style={FIELD_LABEL}>Collection</div>
              <div style={FIELD_VALUE}>{collectionName}</div>
            </div>
            {vector.metadata?.dimension && (
              <div>
                <div style={FIELD_LABEL}>Dimension</div>
                <div style={FIELD_VALUE}>{vector.metadata.dimension}</div>
              </div>
            )}
            {vector.metadata?.embedding_model && (
              <div>
                <div style={FIELD_LABEL}>Embedding Model</div>
                <div style={FIELD_VALUE}>{vector.metadata.embedding_model}</div>
              </div>
            )}
            {vector.metadata?.similarity_score !== undefined && (
              <div>
                <div style={FIELD_LABEL}>Similarity Score</div>
                <div style={{ ...FIELD_VALUE, fontVariantNumeric: 'tabular-nums' }}>
                  {vector.metadata.similarity_score.toFixed(4)}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Payload */}
        {vector.payload && (
          <div>
            <div style={SECTION_HEAD}>Payload</div>
            <CodeEditor
              value={formatJSON(vector.payload)}
              language="json"
              height="300px"
              readOnly={true}
            />
          </div>
        )}

        {/* Metadata */}
        {vector.metadata && (
          <div>
            <div style={SECTION_HEAD}>Metadata</div>
            <div style={GRID}>
              {vector.metadata.source && (
                <div>
                  <div style={FIELD_LABEL}>Source</div>
                  <div style={FIELD_VALUE}>{vector.metadata.source}</div>
                </div>
              )}
              {vector.metadata.file_type && (
                <div>
                  <div style={FIELD_LABEL}>File Type</div>
                  <div style={FIELD_VALUE}>{vector.metadata.file_type}</div>
                </div>
              )}
              {vector.metadata.chunk_index !== undefined && (
                <div>
                  <div style={FIELD_LABEL}>Chunk Index</div>
                  <div style={FIELD_VALUE}>{vector.metadata.chunk_index}</div>
                </div>
              )}
              {vector.metadata.created_at && (
                <div>
                  <div style={FIELD_LABEL}>Created At</div>
                  <div style={FIELD_VALUE}>
                    {new Date(vector.metadata.created_at).toLocaleString()}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Vector Data */}
        {vector.vector && Array.isArray(vector.vector) && (
          <div>
            <div style={SECTION_HEAD}>
              Vector Data ({vector.vector.length} dimensions)
            </div>
            <CodeEditor
              value={JSON.stringify(vector.vector, null, 2)}
              language="json"
              height="300px"
              readOnly={true}
            />
          </div>
        )}
      </div>
    </Modal>
  );
}
