/**
 * Discovery Config Modal - Configure edge discovery parameters
 */

import { useState } from 'react';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { useToastContext } from '@/providers/ToastProvider';

interface DiscoveryConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  onDiscover: (threshold: number, maxPerNode: number) => Promise<void>;
  nodeId?: string; // If provided, discovery is for this specific node
}

export default function DiscoveryConfigModal({
  isOpen,
  onClose,
  onDiscover,
  nodeId,
}: DiscoveryConfigModalProps) {
  const toast = useToastContext();
  const [loading, setLoading] = useState(false);
  const [threshold, setThreshold] = useState(0.7);
  const [maxPerNode, setMaxPerNode] = useState(10);

  const handleDiscover = async () => {
    setLoading(true);
    try {
      await onDiscover(threshold, maxPerNode);
      onClose();
    } catch (error) {
      console.error('Error discovering edges:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to discover edges');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={nodeId ? 'Discover Edges for Node' : 'Discover Edges'}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} disabled={loading}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleDiscover} disabled={loading}>
            {loading ? 'Discovering...' : 'Start Discovery'}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {nodeId && (
          <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <p className="text-sm text-blue-900 dark:text-blue-200">
              Discovering edges for node: <span className="font-mono">{nodeId.substring(0, 40)}...</span>
            </p>
          </div>
        )}

        <Input
          label="Similarity Threshold"
          type="number"
          value={threshold.toString()}
          onChange={(e) => setThreshold(parseFloat(e.target.value) || 0.7)}
          min="0"
          max="1"
          step="0.05"
          disabled={loading}
          helpText="Minimum similarity score (0-1)"
        />

        <Input
          label="Max Edges Per Node"
          type="number"
          value={maxPerNode.toString()}
          onChange={(e) => setMaxPerNode(parseInt(e.target.value) || 10)}
          min="1"
          max="100"
          disabled={loading}
          helpText="Maximum number of edges to create per node"
        />
      </div>
    </Modal>
  );
}
