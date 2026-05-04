/**
 * Path Finder Modal — console design.
 */

import { useState } from 'react';
import Modal from '@/components/ui/Modal';
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
          <button type="button" className="btn" onClick={handleClose}>
            Close
          </button>
          <button
            type="button"
            className="btn primary"
            onClick={handleFindPath}
            disabled={loading || !source || !target}
          >
            {loading ? 'Finding...' : 'Find Path'}
          </button>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        <Select
          label="Source Node"
          value={source}
          onChange={setSource}
          placeholder="Select source node"
        >
          {nodes.map((node) => (
            <SelectOption key={node.id} id={node.id} value={node.id}>
              {node.id.length > 50 ? `${node.id.substring(0, 50)}...` : node.id} (
              {node.node_type})
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
              {node.id.length > 50 ? `${node.id.substring(0, 50)}...` : node.id} (
              {node.node_type})
            </SelectOption>
          ))}
        </Select>

        {path && path.length > 0 && (
          <div
            style={{
              marginTop: 6,
              padding: 14,
              background: 'var(--bg-2)',
              border: '1px solid var(--border)',
              borderRadius: 6,
            }}
          >
            <h4
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: 'var(--text)',
                margin: 0,
                marginBottom: 10,
                letterSpacing: '0.04em',
                textTransform: 'uppercase',
              }}
            >
              Path Found ({path.length} nodes)
            </h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              {path.map((node, idx) => (
                <div
                  key={node.id}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    fontSize: 12,
                  }}
                >
                  <span
                    className="mono"
                    style={{ color: 'var(--text-2)', minWidth: 24 }}
                  >
                    {idx + 1}.
                  </span>
                  <span style={{ color: 'var(--text)' }}>
                    {node.id.length > 60 ? `${node.id.substring(0, 60)}...` : node.id}
                  </span>
                  <span style={{ color: 'var(--text-2)' }}>({node.node_type})</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {path && path.length === 0 && (
          <div
            style={{
              marginTop: 6,
              padding: 14,
              background: 'var(--amber-dim)',
              border: '1px solid rgba(240,168,58,0.35)',
              borderRadius: 6,
            }}
          >
            <p style={{ color: 'var(--amber)', margin: 0, fontSize: 12 }}>
              No path exists between the selected nodes.
            </p>
          </div>
        )}
      </div>
    </Modal>
  );
}
