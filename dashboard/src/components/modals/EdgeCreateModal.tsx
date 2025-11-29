/**
 * Edge Create Modal - Create edge between two nodes
 */

import { useState, useEffect } from 'react';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Select, SelectOption } from '@/components/ui/Select';
import { useToastContext } from '@/providers/ToastProvider';

interface EdgeCreateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreateEdge: (source: string, target: string, relationshipType: string, weight: number) => Promise<void>;
  nodes: Array<{ id: string; node_type: string }>;
  preselectedSource?: string;
  preselectedTarget?: string;
}

const RELATIONSHIP_TYPES = [
  'SIMILAR_TO',
  'REFERENCES',
  'CONTAINS',
  'DERIVED_FROM',
  'RELATES_TO',
  'DEPENDS_ON',
  'PART_OF',
] as const;

export default function EdgeCreateModal({
  isOpen,
  onClose,
  onCreateEdge,
  nodes,
  preselectedSource,
  preselectedTarget,
}: EdgeCreateModalProps) {
  const toast = useToastContext();
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    source: preselectedSource || '',
    target: preselectedTarget || '',
    relationshipType: 'SIMILAR_TO',
    weight: 1.0,
  });

  // Update form when preselected values change
  useEffect(() => {
    if (preselectedSource) {
      setFormData((prev) => ({ ...prev, source: preselectedSource }));
    }
    if (preselectedTarget) {
      setFormData((prev) => ({ ...prev, target: preselectedTarget }));
    }
  }, [preselectedSource, preselectedTarget]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!formData.source || !formData.target) {
      toast.error('Please select both source and target nodes');
      return;
    }

    if (formData.source === formData.target) {
      toast.error('Source and target nodes must be different');
      return;
    }

    setLoading(true);
    try {
      await onCreateEdge(
        formData.source,
        formData.target,
        formData.relationshipType,
        formData.weight
      );
      toast.success('Edge created successfully');
      // Reset form
      setFormData({
        source: '',
        target: '',
        relationshipType: 'SIMILAR_TO',
        weight: 1.0,
      });
      onClose();
    } catch (error) {
      console.error('Error creating edge:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to create edge');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setFormData({
        source: '',
        target: '',
        relationshipType: 'SIMILAR_TO',
        weight: 1.0,
      });
      onClose();
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title="Create Edge"
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={handleClose} disabled={loading}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSubmit} disabled={loading}>
            {loading ? 'Creating...' : 'Create Edge'}
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <Select
          label="Source Node"
          value={formData.source}
          onChange={(value) => setFormData({ ...formData, source: value })}
          placeholder="Select source node"
          required
          disabled={loading}
        >
          {nodes.map((node) => (
            <SelectOption key={node.id} id={node.id} value={node.id}>
              {node.id.length > 50 ? `${node.id.substring(0, 50)}...` : node.id} ({node.node_type})
            </SelectOption>
          ))}
        </Select>

        <Select
          label="Target Node"
          value={formData.target}
          onChange={(value) => setFormData({ ...formData, target: value })}
          placeholder="Select target node"
          required
          disabled={loading}
        >
          {nodes.map((node) => (
            <SelectOption key={node.id} id={node.id} value={node.id}>
              {node.id.length > 50 ? `${node.id.substring(0, 50)}...` : node.id} ({node.node_type})
            </SelectOption>
          ))}
        </Select>

        <Select
          label="Relationship Type"
          value={formData.relationshipType}
          onChange={(value) => setFormData({ ...formData, relationshipType: value })}
          required
          disabled={loading}
        >
          {RELATIONSHIP_TYPES.map((type) => (
            <SelectOption key={type} id={type} value={type}>
              {type}
            </SelectOption>
          ))}
        </Select>

        <Input
          label="Weight"
          type="number"
          value={formData.weight.toString()}
          onChange={(e) => setFormData({ ...formData, weight: parseFloat(e.target.value) || 1.0 })}
          min="0"
          max="10"
          step="0.1"
          required
          disabled={loading}
          helpText="Weight for the edge (0-10)"
        />
      </form>
    </Modal>
  );
}
