/**
 * Edit Vector Modal - Edit vector payload
 */

import { useState, useEffect } from 'react';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
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
          <Button variant="secondary" onClick={onClose} disabled={loading}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSave} disabled={loading} isLoading={loading}>
            {loading ? 'Saving...' : 'Save Changes'}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {/* Basic Info */}
        <div className="bg-neutral-50 dark:bg-neutral-900/50 rounded-lg p-4">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-neutral-500 dark:text-neutral-400">Vector ID:</span>
              <p className="text-neutral-900 dark:text-white font-mono mt-1 break-all">
                {vector.id}
              </p>
            </div>
            <div>
              <span className="text-neutral-500 dark:text-neutral-400">Collection:</span>
              <p className="text-neutral-900 dark:text-white mt-1">{collectionName}</p>
            </div>
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
            <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
          </div>
        )}

        {/* Payload Editor */}
        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
            Payload (JSON)
          </label>
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
          <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-2">
            Edit the JSON payload. Changes will be saved to the vector.
          </p>
        </div>
      </div>
    </Modal>
  );
}

