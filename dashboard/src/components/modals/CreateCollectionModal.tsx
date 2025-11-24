/**
 * Create Collection Modal
 */

import { useState } from 'react';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Select, SelectOption } from '@/components/ui/Select';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import { useToastContext } from '@/providers/ToastProvider';

interface CreateCollectionModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function CreateCollectionModal({
  isOpen,
  onClose,
}: CreateCollectionModalProps) {
  const { createCollection, listCollections } = useCollections();
  const { setCollections } = useCollectionsStore();
  const toast = useToastContext();
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    dimension: 512,
    metric: 'cosine' as 'cosine' | 'euclidean' | 'dot',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.name.trim()) {
      return;
    }

    setLoading(true);
    try {
      await createCollection({
        name: formData.name.trim(),
        dimension: formData.dimension,
        metric: formData.metric, // Already typed correctly
      });
      
      // Refresh collections
      const updated = await listCollections();
      setCollections(Array.isArray(updated) ? updated : []);
      
      // Reset form and close
      setFormData({ name: '', dimension: 512, metric: 'cosine' });
      toast.success('Collection created successfully');
      onClose();
    } catch (error) {
      console.error('Error creating collection:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to create collection');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Create Collection"
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} disabled={loading}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSubmit} disabled={loading}>
            {loading ? 'Creating...' : 'Create Collection'}
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          label="Collection Name"
          type="text"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          placeholder="my-collection"
          required
          disabled={loading}
        />

        <Input
          label="Dimension"
          type="number"
          value={formData.dimension.toString()}
          onChange={(e) => setFormData({ ...formData, dimension: parseInt(e.target.value) || 512 })}
          min="1"
          max="4096"
          required
          disabled={loading}
        />

        <Select
          label="Distance Metric"
          value={formData.metric}
          onChange={(value) => setFormData({ ...formData, metric: value as 'cosine' | 'euclidean' | 'dot' })}
          isDisabled={loading}
        >
          <SelectOption id="cosine" value="cosine">Cosine</SelectOption>
          <SelectOption id="euclidean" value="euclidean">Euclidean</SelectOption>
          <SelectOption id="dot" value="dot">Dot Product</SelectOption>
        </Select>
      </form>
    </Modal>
  );
}

