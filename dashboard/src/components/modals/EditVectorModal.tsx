/**
 * Edit Vector Modal — console design.
 */

import { useState, useEffect } from 'react';
import Modal from '@/components/ui/Modal';
import CodeEditor from '@/components/ui/CodeEditor';
import { useApiClient } from '@/hooks/useApiClient';
import { useToastContext } from '@/providers/ToastProvider';

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

interface EditVectorModalProps {
  isOpen: boolean;
  onClose: () => void;
  vector: Vector | null;
  collectionName: string;
  onUpdate?: () => void;
}

export default function EditVectorModal({
  isOpen,
  onClose,
  vector,
  collectionName,
  onUpdate,
}: EditVectorModalProps) {
  const api = useApiClient();
  const toast = useToastContext();
  const [payloadJson, setPayloadJson] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (vector && isOpen) {
      try {
        setPayloadJson(JSON.stringify(vector.payload || {}, null, 2));
        setError(null);
      } catch {
        setError('Failed to parse payload');
        setPayloadJson('');
      }
    }
  }, [vector, isOpen]);

  const handleSave = async () => {
    if (!vector || !collectionName) return;

    // Validate JSON
    let parsedPayload: Record<string, unknown>;
    try {
      parsedPayload = JSON.parse(payloadJson) as Record<string, unknown>;
    } catch {
      setError('Invalid JSON format');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await api.put(
        `/collections/${encodeURIComponent(collectionName)}/vectors/${encodeURIComponent(vector.id)}`,
        {
          payload: parsedPayload,
        }
      );

      toast.success('Vector updated successfully');
      onUpdate?.();
      onClose();
    } catch (err) {
      console.error('Error updating vector:', err);
      setError(err instanceof Error ? err.message : 'Failed to update vector');
    } finally {
      setLoading(false);
    }
  };

  if (!vector) return null;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Edit Vector"
      size="xl"
      footer={
        <>
          <button type="button" className="btn" onClick={onClose} disabled={loading}>
            Cancel
          </button>
          <button
            type="button"
            className="btn primary"
            onClick={handleSave}
            disabled={loading}
          >
            {loading ? 'Saving...' : 'Save Changes'}
          </button>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        {/* Basic Info */}
        <div
          style={{
            background: 'var(--bg-2)',
            border: '1px solid var(--border)',
            borderRadius: 6,
            padding: 12,
          }}
        >
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
              gap: 14,
              fontSize: 12,
            }}
          >
            <div>
              <div style={{ color: 'var(--text-2)' }}>Vector ID:</div>
              <div
                className="mono"
                style={{
                  color: 'var(--text)',
                  marginTop: 4,
                  wordBreak: 'break-all',
                }}
              >
                {vector.id}
              </div>
            </div>
            <div>
              <div style={{ color: 'var(--text-2)' }}>Collection:</div>
              <div style={{ color: 'var(--text)', marginTop: 4 }}>{collectionName}</div>
            </div>
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div
            style={{
              background: 'rgba(229,72,77,0.10)',
              border: '1px solid rgba(229,72,77,0.35)',
              borderRadius: 6,
              padding: 10,
            }}
          >
            <p style={{ color: 'var(--red)', fontSize: 12, margin: 0 }}>{error}</p>
          </div>
        )}

        {/* Payload Editor */}
        <div className="field">
          <label className="field-label">Payload (JSON)</label>
          <CodeEditor
            value={payloadJson}
            onChange={(value) => {
              setPayloadJson(value || '');
              setError(null);
            }}
            language="json"
            height="400px"
            readOnly={false}
          />
          <div style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 6 }}>
            Edit the JSON payload. Changes will be saved to the vector.
          </div>
        </div>
      </div>
    </Modal>
  );
}
