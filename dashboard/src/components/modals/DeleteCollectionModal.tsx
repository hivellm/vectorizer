/**
 * Delete Collection Confirmation Modal
 */

import { useState } from 'react';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
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
          <Button variant="secondary" onClick={onClose} disabled={loading}>
            Cancel
          </Button>
          <Button variant="danger" onClick={handleDelete} disabled={loading} isLoading={loading}>
            {loading ? 'Deleting...' : 'Delete Collection'}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <p className="text-neutral-700 dark:text-neutral-300">
          Are you sure you want to delete the collection <strong className="text-neutral-900 dark:text-white">{collectionName}</strong>?
        </p>
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <p className="text-sm text-yellow-800 dark:text-yellow-300">
            <strong>Warning:</strong> This action cannot be undone. All vectors and data in this collection will be permanently deleted.
          </p>
        </div>
      </div>
    </Modal>
  );
}

