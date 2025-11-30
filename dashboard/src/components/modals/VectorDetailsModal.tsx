/**
 * Vector Details Modal
 */

import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import CodeEditor from '@/components/ui/CodeEditor';
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

interface VectorDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  vector: Vector | null;
  collectionName: string;
  onEdit?: () => void;
}

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
          <Button variant="secondary" onClick={copyVectorId}>
            Copy ID
          </Button>
          {onEdit && (
            <Button variant="primary" onClick={onEdit}>
              Edit
            </Button>
          )}
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </>
      }
    >
      <div className="space-y-6">
        {/* Basic Information */}
        <div>
          <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
            Basic Information
          </h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Vector ID:</span>
              <p className="text-sm font-mono text-neutral-900 dark:text-white mt-1 break-all">
                {vector.id}
              </p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Collection:</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                {collectionName}
              </p>
            </div>
            {vector.metadata?.dimension && (
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Dimension:</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {vector.metadata.dimension}
                </p>
              </div>
            )}
            {vector.metadata?.embedding_model && (
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Embedding Model:</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {vector.metadata.embedding_model}
                </p>
              </div>
            )}
            {vector.metadata?.similarity_score !== undefined && (
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Similarity Score:</span>
                <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                  {vector.metadata.similarity_score.toFixed(4)}
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Payload */}
        {vector.payload && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Payload
            </h3>
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
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Metadata
            </h3>
            <div className="grid grid-cols-2 gap-4">
              {vector.metadata.source && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Source:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                    {vector.metadata.source}
                  </p>
                </div>
              )}
              {vector.metadata.file_type && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">File Type:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                    {vector.metadata.file_type}
                  </p>
                </div>
              )}
              {vector.metadata.chunk_index !== undefined && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Chunk Index:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                    {vector.metadata.chunk_index}
                  </p>
                </div>
              )}
              {vector.metadata.created_at && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Created At:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white mt-1">
                    {new Date(vector.metadata.created_at).toLocaleString()}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Vector Data */}
        {vector.vector && Array.isArray(vector.vector) && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Vector Data ({vector.vector.length} dimensions)
            </h3>
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

