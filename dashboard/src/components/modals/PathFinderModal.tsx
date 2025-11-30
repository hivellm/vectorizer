/**
 * Path Finder Modal - Find shortest path between two nodes
 */

import { useState } from 'react';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { Select, SelectOption } from '@/components/ui/Select';
import { useToastContext } from '@/providers/ToastProvider';
import { GraphNode } from '@/hooks/useGraph';

interface PathFinderModalProps {
  isOpen: boolean;
  onClose: () => void;
  onFindPath: (source: string, target: string) => Promise<GraphNode[]>;
  nodes: Array<{ id: string; node_type: string }>;
}

export default function PathFinderModal({
  isOpen,
  onClose,
  onFindPath,
  nodes,
}: PathFinderModalProps) {
  const toast = useToastContext();
  const [loading, setLoading] = useState(false);
  const [source, setSource] = useState('');
  const [target, setTarget] = useState('');
  const [path, setPath] = useState<GraphNode[] | null>(null);

  const handleFindPath = async () => {
    if (!source || !target) {
      toast.error('Please select both source and target nodes');
      return;
    }

    if (source === target) {
      toast.error('Source and target must be different');
      return;
    }

    setLoading(true);
    try {
      const result = await onFindPath(source, target);
      setPath(result);
      if (result.length === 0) {
        toast.info('No path found between selected nodes');
      } else {
        toast.success(`Path found with ${result.length} nodes`);
      }
    } catch (error) {
      console.error('Error finding path:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to find path');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setSource('');
    setTarget('');
    setPath(null);
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title="Find Path Between Nodes"
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={handleClose}>
            Close
          </Button>
          <Button variant="primary" onClick={handleFindPath} disabled={loading || !source || !target}>
            {loading ? 'Finding...' : 'Find Path'}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <Select
          label="Source Node"
          value={source}
          onChange={setSource}
          placeholder="Select source node"
        >
          {nodes.map((node) => (
            <SelectOption key={node.id} id={node.id} value={node.id}>
              {node.id.length > 50 ? `${node.id.substring(0, 50)}...` : node.id} ({node.node_type})
            </SelectOption>
          ))}
        </Select>

        <Select
          label="Target Node"
          value={target}
          onChange={setTarget}
          placeholder="Select target node"
        >
          {nodes.map((node) => (
            <SelectOption key={node.id} id={node.id} value={node.id}>
              {node.id.length > 50 ? `${node.id.substring(0, 50)}...` : node.id} ({node.node_type})
            </SelectOption>
          ))}
        </Select>

        {path && path.length > 0 && (
          <div className="mt-4 p-4 bg-neutral-50 dark:bg-neutral-800 rounded-lg">
            <h4 className="font-semibold text-neutral-900 dark:text-white mb-2">
              Path Found ({path.length} nodes)
            </h4>
            <div className="space-y-2">
              {path.map((node, idx) => (
                <div key={node.id} className="flex items-center gap-2 text-sm">
                  <span className="font-mono text-neutral-600 dark:text-neutral-400">
                    {idx + 1}.
                  </span>
                  <span className="text-neutral-900 dark:text-white">
                    {node.id.length > 60 ? `${node.id.substring(0, 60)}...` : node.id}
                  </span>
                  <span className="text-neutral-500 dark:text-neutral-400">
                    ({node.node_type})
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {path && path.length === 0 && (
          <div className="mt-4 p-4 bg-amber-50 dark:bg-amber-900/20 rounded-lg">
            <p className="text-amber-900 dark:text-amber-200">
              No path exists between the selected nodes.
            </p>
          </div>
        )}
      </div>
    </Modal>
  );
}
