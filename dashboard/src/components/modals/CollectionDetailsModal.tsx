/**
 * Collection Details Modal
 */

import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { formatNumber, formatDate } from '@/utils/formatters';
import { useNavigate } from 'react-router';

interface Collection {
  name: string;
  vector_count?: number;
  dimension?: number;
  metric?: string;
  normalization?: {
    enabled: boolean;
    level?: string;
  };
  created_at?: string;
  updated_at?: string;
  indexing_status?: {
    status: string;
    progress?: number;
  };
  size?: {
    total?: string;
    total_bytes?: number;
  };
  quantization?: {
    enabled: boolean;
    type?: string;
    bits?: number;
  };
}

interface CollectionDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  collection: Collection | null;
}

export default function CollectionDetailsModal({
  isOpen,
  onClose,
  collection,
}: CollectionDetailsModalProps) {
  const navigate = useNavigate();

  if (!collection) return null;

  const handleBrowseVectors = () => {
    onClose();
    navigate('/vectors', { state: { collectionName: collection.name } });
  };

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'completed':
      case 'cached':
        return 'text-green-600 dark:text-green-400';
      case 'processing':
      case 'indexing':
        return 'text-blue-600 dark:text-blue-400';
      case 'error':
        return 'text-red-600 dark:text-red-400';
      default:
        return 'text-neutral-600 dark:text-neutral-400';
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={`Collection: ${collection.name}`}
      size="lg"
      footer={
        <>
          <Button variant="primary" onClick={handleBrowseVectors}>
            Browse Vectors
          </Button>
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
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Name:</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white">{collection.name}</p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Vector Count:</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white">
                {formatNumber(collection.vector_count || 0)}
              </p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Dimension:</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white">{collection.dimension}</p>
            </div>
            <div>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">Distance Metric:</span>
              <p className="text-sm font-medium text-neutral-900 dark:text-white capitalize">
                {collection.metric}
              </p>
            </div>
          </div>
        </div>

        {/* Status Information */}
        {collection.indexing_status && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Status Information
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Status:</span>
                <p className={`text-sm font-medium ${getStatusClass(collection.indexing_status.status)}`}>
                  {collection.indexing_status.status}
                </p>
              </div>
              {collection.indexing_status.progress !== undefined && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Progress:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    {Math.round(collection.indexing_status.progress * 100)}%
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Timestamps */}
        {(collection.created_at || collection.updated_at) && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Timestamps
            </h3>
            <div className="grid grid-cols-2 gap-4">
              {collection.created_at && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Created:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    {formatDate(collection.created_at)}
                  </p>
                </div>
              )}
              {collection.updated_at && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Updated:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    {formatDate(collection.updated_at)}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Normalization */}
        {collection.normalization && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Text Normalization
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Enabled:</span>
                <p className={`text-sm font-medium ${collection.normalization.enabled ? 'text-green-600 dark:text-green-400' : 'text-neutral-600 dark:text-neutral-400'}`}>
                  {collection.normalization.enabled ? 'Yes' : 'No'}
                </p>
              </div>
              {collection.normalization.level && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Level:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    {collection.normalization.level}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Quantization */}
        {collection.quantization && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Quantization
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">Enabled:</span>
                <p className={`text-sm font-medium ${collection.quantization.enabled ? 'text-green-600 dark:text-green-400' : 'text-neutral-600 dark:text-neutral-400'}`}>
                  {collection.quantization.enabled ? 'Yes' : 'No'}
                </p>
              </div>
              {collection.quantization.bits && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Bits:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    {collection.quantization.bits}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Size */}
        {collection.size && (
          <div>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
              Storage
            </h3>
            <div className="grid grid-cols-2 gap-4">
              {collection.size.total && (
                <div>
                  <span className="text-sm text-neutral-500 dark:text-neutral-400">Total Size:</span>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    {collection.size.total}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </Modal>
  );
}

