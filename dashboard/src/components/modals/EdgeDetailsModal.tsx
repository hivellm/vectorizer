/**
 * Edge Details Modal - Display edge information and actions
 */

import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { GraphEdge } from '@/hooks/useGraph';

interface EdgeDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  edge: GraphEdge | null;
  onDelete: (edgeId: string) => Promise<void>;
}

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
          <Button variant="danger" onClick={handleDelete}>
            Delete Edge
          </Button>
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Edge ID
          </label>
          <p className="text-sm text-neutral-900 dark:text-white font-mono break-all">
            {edge.id}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Source Node
          </label>
          <p className="text-sm text-neutral-900 dark:text-white font-mono break-all">
            {edge.source}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Target Node
          </label>
          <p className="text-sm text-neutral-900 dark:text-white font-mono break-all">
            {edge.target}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Relationship Type
          </label>
          <p className="text-sm text-neutral-900 dark:text-white">
            {edge.relationship_type}
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Weight
          </label>
          <p className="text-sm text-neutral-900 dark:text-white">
            {edge.weight.toFixed(2)}
          </p>
        </div>

        {edge.metadata && Object.keys(edge.metadata).length > 0 && (
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
              Metadata
            </label>
            <pre className="p-3 bg-neutral-50 dark:bg-neutral-800 rounded text-xs overflow-auto">
              {JSON.stringify(edge.metadata, null, 2)}
            </pre>
          </div>
        )}

        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
            Created At
          </label>
          <p className="text-sm text-neutral-900 dark:text-white">
            {new Date(edge.created_at).toLocaleString()}
          </p>
        </div>
      </div>
    </Modal>
  );
}
