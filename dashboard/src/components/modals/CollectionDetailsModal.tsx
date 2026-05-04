/**
 * Collection Details Modal — console design.
 */

import Modal from '@/components/ui/Modal';
import { formatNumber, formatDate } from '@/utils/formatters';
import { useNavigate } from 'react-router-dom';
import type { CSSProperties } from 'react';

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

const SECTION_HEAD: CSSProperties = {
  fontSize: 12,
  fontWeight: 600,
  color: 'var(--text)',
  letterSpacing: '0.04em',
  textTransform: 'uppercase',
  marginBottom: 10,
};

const GRID: CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
  gap: 14,
};

const FIELD_LABEL: CSSProperties = {
  fontSize: 11,
  color: 'var(--text-2)',
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
};

const FIELD_VALUE: CSSProperties = {
  fontSize: 13,
  fontWeight: 500,
  color: 'var(--text)',
  marginTop: 4,
};

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

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'completed':
      case 'cached':
        return 'var(--green)';
      case 'processing':
      case 'indexing':
        return 'var(--teal)';
      case 'error':
        return 'var(--red)';
      default:
        return 'var(--text-2)';
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
          <button type="button" className="btn primary" onClick={handleBrowseVectors}>
            Browse Vectors
          </button>
          <button type="button" className="btn" onClick={onClose}>
            Close
          </button>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
        {/* Basic Information */}
        <div>
          <div style={SECTION_HEAD}>Basic Information</div>
          <div style={GRID}>
            <div>
              <div style={FIELD_LABEL}>Name</div>
              <div style={FIELD_VALUE}>{collection.name}</div>
            </div>
            <div>
              <div style={FIELD_LABEL}>Vector Count</div>
              <div style={{ ...FIELD_VALUE, fontVariantNumeric: 'tabular-nums' }}>
                {formatNumber(collection.vector_count || 0)}
              </div>
            </div>
            <div>
              <div style={FIELD_LABEL}>Dimension</div>
              <div style={FIELD_VALUE}>{collection.dimension}</div>
            </div>
            <div>
              <div style={FIELD_LABEL}>Distance Metric</div>
              <div style={{ ...FIELD_VALUE, textTransform: 'capitalize' }}>
                {collection.metric}
              </div>
            </div>
          </div>
        </div>

        {/* Status Information */}
        {collection.indexing_status && (
          <div>
            <div style={SECTION_HEAD}>Status Information</div>
            <div style={GRID}>
              <div>
                <div style={FIELD_LABEL}>Status</div>
                <div
                  style={{
                    ...FIELD_VALUE,
                    color: getStatusColor(collection.indexing_status.status),
                  }}
                >
                  {collection.indexing_status.status}
                </div>
              </div>
              {collection.indexing_status.progress !== undefined && (
                <div>
                  <div style={FIELD_LABEL}>Progress</div>
                  <div style={FIELD_VALUE}>
                    {Math.round(collection.indexing_status.progress * 100)}%
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Timestamps */}
        {(collection.created_at || collection.updated_at) && (
          <div>
            <div style={SECTION_HEAD}>Timestamps</div>
            <div style={GRID}>
              {collection.created_at && (
                <div>
                  <div style={FIELD_LABEL}>Created</div>
                  <div style={FIELD_VALUE}>{formatDate(collection.created_at)}</div>
                </div>
              )}
              {collection.updated_at && (
                <div>
                  <div style={FIELD_LABEL}>Updated</div>
                  <div style={FIELD_VALUE}>{formatDate(collection.updated_at)}</div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Normalization */}
        {collection.normalization && (
          <div>
            <div style={SECTION_HEAD}>Text Normalization</div>
            <div style={GRID}>
              <div>
                <div style={FIELD_LABEL}>Enabled</div>
                <div
                  style={{
                    ...FIELD_VALUE,
                    color: collection.normalization.enabled
                      ? 'var(--green)'
                      : 'var(--text-2)',
                  }}
                >
                  {collection.normalization.enabled ? 'Yes' : 'No'}
                </div>
              </div>
              {collection.normalization.level && (
                <div>
                  <div style={FIELD_LABEL}>Level</div>
                  <div style={FIELD_VALUE}>{collection.normalization.level}</div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Quantization */}
        {collection.quantization && (
          <div>
            <div style={SECTION_HEAD}>Quantization</div>
            <div style={GRID}>
              <div>
                <div style={FIELD_LABEL}>Enabled</div>
                <div
                  style={{
                    ...FIELD_VALUE,
                    color: collection.quantization.enabled
                      ? 'var(--green)'
                      : 'var(--text-2)',
                  }}
                >
                  {collection.quantization.enabled ? 'Yes' : 'No'}
                </div>
              </div>
              {collection.quantization.bits && (
                <div>
                  <div style={FIELD_LABEL}>Bits</div>
                  <div style={FIELD_VALUE}>{collection.quantization.bits}</div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Size */}
        {collection.size && collection.size.total && (
          <div>
            <div style={SECTION_HEAD}>Storage</div>
            <div style={GRID}>
              <div>
                <div style={FIELD_LABEL}>Total Size</div>
                <div style={FIELD_VALUE}>{collection.size.total}</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </Modal>
  );
}
