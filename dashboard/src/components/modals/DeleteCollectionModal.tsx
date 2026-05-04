/**
 * Delete Collection Confirmation Modal — console design.
 */

import { useState } from 'react';
import Modal from '@/components/ui/Modal';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import { useToastContext } from '@/providers/ToastProvider';

interface DeleteCollectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  collectionName: string;
}

export default function DeleteCollectionModal({
  isOpen,
  onClose,
  collectionName,
}: DeleteCollectionModalProps) {
  const { deleteCollection, listCollections } = useCollections();
  const { setCollections, removeCollection } = useCollectionsStore();
  const toast = useToastContext();
  const [loading, setLoading] = useState(false);

  const handleDelete = async () => {
    setLoading(true);
    try {
      await deleteCollection(collectionName);

      // Remove from store
      removeCollection(collectionName);

      // Refresh collections list
      const updated = await listCollections();
      setCollections(Array.isArray(updated) ? updated : []);

      toast.success(`Collection "${collectionName}" deleted successfully`);
      onClose();
    } catch (error) {
      console.error('Error deleting collection:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to delete collection');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Delete Collection"
      size="md"
      footer={
        <>
          <button type="button" className="btn" onClick={onClose} disabled={loading}>
            Cancel
          </button>
          <button
            type="button"
            className="btn magenta"
            onClick={handleDelete}
            disabled={loading}
          >
            {loading ? 'Deleting...' : 'Delete Collection'}
          </button>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        <p style={{ color: 'var(--text-1)', margin: 0, fontSize: 13 }}>
          Are you sure you want to delete the collection{' '}
          <strong style={{ color: 'var(--text)' }}>{collectionName}</strong>?
        </p>
        <div
          style={{
            background: 'var(--amber-dim)',
            border: '1px solid rgba(240,168,58,0.35)',
            borderRadius: 6,
            padding: 12,
          }}
        >
          <p style={{ color: 'var(--amber)', fontSize: 12, margin: 0 }}>
            <strong>Warning:</strong> This action cannot be undone. All vectors and data
            in this collection will be permanently deleted.
          </p>
        </div>
      </div>
    </Modal>
  );
}
