/**
 * Edge Details Modal — console design.
 */

import Modal from '@/components/ui/Modal';
import { GraphEdge } from '@/hooks/useGraph';
import type { CSSProperties } from 'react';

interface EdgeDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  edge: GraphEdge | null;
  onDelete: (edgeId: string) => Promise<void>;
}

const FIELD_VALUE: CSSProperties = {
  fontSize: 13,
  color: 'var(--text)',
};

export default function EdgeDetailsModal({
  isOpen,
  onClose,
  edge,
  onDelete,
}: EdgeDetailsModalProps) {
  if (!edge) return null;

  const handleDelete = async () => {
    if (window.confirm('Are you sure you want to delete this edge?')) {
      await onDelete(edge.id);
      onClose();
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Edge Details"
      size="md"
      footer={
        <>
          <button type="button" className="btn magenta" onClick={handleDelete}>
            Delete Edge
          </button>
          <button type="button" className="btn" onClick={onClose}>
            Close
          </button>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        <div className="field">
          <label className="field-label">Edge ID</label>
          <p
            className="mono"
            style={{ ...FIELD_VALUE, margin: 0, wordBreak: 'break-all' }}
          >
            {edge.id}
          </p>
        </div>

        <div className="field">
          <label className="field-label">Source Node</label>
          <p
            className="mono"
            style={{ ...FIELD_VALUE, margin: 0, wordBreak: 'break-all' }}
          >
            {edge.source}
          </p>
        </div>

        <div className="field">
          <label className="field-label">Target Node</label>
          <p
            className="mono"
            style={{ ...FIELD_VALUE, margin: 0, wordBreak: 'break-all' }}
          >
            {edge.target}
          </p>
        </div>

        <div className="field">
          <label className="field-label">Relationship Type</label>
          <p style={{ ...FIELD_VALUE, margin: 0 }}>{edge.relationship_type}</p>
        </div>

        <div className="field">
          <label className="field-label">Weight</label>
          <p style={{ ...FIELD_VALUE, margin: 0, fontVariantNumeric: 'tabular-nums' }}>
            {edge.weight.toFixed(2)}
          </p>
        </div>

        {edge.metadata && Object.keys(edge.metadata).length > 0 && (
          <div className="field">
            <label className="field-label">Metadata</label>
            <pre
              style={{
                padding: 12,
                background: 'var(--bg-2)',
                border: '1px solid var(--border)',
                borderRadius: 6,
                fontSize: 11,
                overflow: 'auto',
                margin: 0,
                color: 'var(--text-1)',
                fontFamily: 'var(--font-mono)',
              }}
            >
              {JSON.stringify(edge.metadata, null, 2)}
            </pre>
          </div>
        )}

        <div className="field">
          <label className="field-label">Created At</label>
          <p style={{ ...FIELD_VALUE, margin: 0 }}>
            {new Date(edge.created_at).toLocaleString()}
          </p>
        </div>
      </div>
    </Modal>
  );
}
