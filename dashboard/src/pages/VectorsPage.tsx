/**
 * Vectors page - Browse vectors in collections
 */

import { useEffect, useState } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { RefreshCw01 } from '@untitledui/icons';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import LoadingState from '@/components/LoadingState';
import Button from '@/components/ui/Button';
import Card from '@/components/ui/Card';
import { Select } from '@/components/ui/Select';
import CodeEditor from '@/components/ui/CodeEditor';
import VectorDetailsModal from '@/components/modals/VectorDetailsModal';
import EditVectorModal from '@/components/modals/EditVectorModal';
import { formatNumber } from '@/utils/formatters';
import { useApiClient } from '@/hooks/useApiClient';
import { useToastContext } from '@/providers/ToastProvider';

interface Vector {
  id: string;
  payload?: any;
  metadata?: {
    source?: string;
    file_type?: string;
    chunk_index?: number;
    embedding_model?: string;
    dimension?: number;
    similarity_score?: number;
    created_at?: string;
  };
}

function VectorsPage() {
  const navigate = useNavigate();
  const location = useLocation();
  const { listCollections } = useCollections();
  const { collections } = useCollectionsStore();
  const api = useApiClient();
  const toast = useToastContext();
  
  const [selectedCollection, setSelectedCollection] = useState<string>(
    location.state?.collectionName || ''
  );
  const [vectors, setVectors] = useState<Vector[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [totalVectors, setTotalVectors] = useState(0);
  const [detailsModalOpen, setDetailsModalOpen] = useState(false);
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [selectedVector, setSelectedVector] = useState<Vector | null>(null);

  const { setCollections } = useCollectionsStore();

  // Load collections on mount
  useEffect(() => {
    const loadCollections = async () => {
      try {
        const data = await listCollections();
        const collectionsArray = Array.isArray(data) ? data : [];
        setCollections(collectionsArray);
      } catch (err) {
        console.error('Error loading collections:', err);
      }
    };
    loadCollections();
  }, [listCollections, setCollections]);

  // Load vectors when collection or page changes
  useEffect(() => {
    if (selectedCollection) {
      loadVectors();
    } else {
      setVectors([]);
      setTotalVectors(0);
    }
  }, [selectedCollection, currentPage, pageSize]);

  const loadVectors = async () => {
    if (!selectedCollection) return;

    setLoading(true);
    setError(null);
    try {
      const offset = (currentPage - 1) * pageSize;
      const response = await api.get<any>(
        `/collections/${encodeURIComponent(selectedCollection)}/vectors?limit=${pageSize}&offset=${offset}`
      );
      
      // Handle response format - API returns { vectors: [...], total: N }
      const vectorsData = Array.isArray(response.vectors) ? response.vectors : [];
      const total = typeof response.total === 'number' ? response.total : vectorsData.length;
      
      setVectors(vectorsData);
      setTotalVectors(total);
    } catch (err) {
      console.error('Error loading vectors:', err);
      setError(err instanceof Error ? err.message : 'Failed to load vectors');
      setVectors([]);
      setTotalVectors(0);
    } finally {
      setLoading(false);
    }
  };

  const handlePageChange = (newPage: number) => {
    const totalPages = Math.ceil(totalVectors / pageSize);
    if (newPage >= 1 && newPage <= totalPages) {
      setCurrentPage(newPage);
    }
  };

  const truncateText = (text: string, maxLength: number) => {
    if (text.length <= maxLength) return text;
    return text.slice(0, maxLength) + '...';
  };

  const formatJSON = (obj: any): string => {
    try {
      return JSON.stringify(obj, null, 2);
    } catch {
      return String(obj);
    }
  };

  const getVectorFileType = (vector: Vector): string => {
    // Try metadata first
    if (vector.metadata?.file_type) {
      return vector.metadata.file_type;
    }
    // Try payload metadata
    if (vector.payload?.metadata?.file_type) {
      return vector.payload.metadata.file_type;
    }
    // Try direct payload file_type
    if (vector.payload?.file_type) {
      return vector.payload.file_type;
    }
    // Try to infer from file_path
    if (vector.payload?.file_path) {
      const ext = vector.payload.file_path.split('.').pop()?.toLowerCase();
      if (ext) {
        return ext.toUpperCase();
      }
    }
    if (vector.payload?.content) {
      return 'Text';
    }
    return 'Unknown';
  };

  const getVectorSource = (vector: Vector): string => {
    // Try metadata first
    if (vector.metadata?.source) {
      return vector.metadata.source;
    }
    // Try payload metadata
    if (vector.payload?.metadata?.source) {
      return vector.payload.metadata.source;
    }
    // Try payload file_path
    if (vector.payload?.file_path) {
      return vector.payload.file_path;
    }
    // Try payload source
    if (vector.payload?.source) {
      return vector.payload.source;
    }
    return 'Unknown';
  };

  const getVectorChunkIndex = (vector: Vector): number => {
    // Try metadata first
    if (vector.metadata?.chunk_index !== undefined) {
      return vector.metadata.chunk_index;
    }
    // Try payload metadata
    if (vector.payload?.metadata?.chunk_index !== undefined) {
      return vector.payload.metadata.chunk_index;
    }
    // Try direct payload chunk_index
    if (vector.payload?.chunk_index !== undefined) {
      return typeof vector.payload.chunk_index === 'number' 
        ? vector.payload.chunk_index 
        : parseInt(vector.payload.chunk_index) || 0;
    }
    return 0;
  };

  const getPayloadContent = (vector: Vector): string => {
    if (vector.payload?.content) {
      return vector.payload.content;
    }
    if (vector.payload) {
      return formatJSON(vector.payload);
    }
    return '';
  };

  const collectionsArray = Array.isArray(collections) ? collections : [];
  const totalPages = Math.ceil(totalVectors / pageSize);
  const startIndex = (currentPage - 1) * pageSize + 1;
  const endIndex = Math.min(currentPage * pageSize, totalVectors);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Vector Browser</h1>
        <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">Browse and manage vectors in your collections</p>
      </div>

      {/* Controls */}
      <Card>
        <div className="flex flex-col sm:flex-row gap-4 items-stretch sm:items-end">
          <div className="flex-1 flex gap-4">
            <div className="flex-1">
              <Select
                label="Collection"
                value={selectedCollection}
                onChange={(value) => {
                  setSelectedCollection(value);
                  setCurrentPage(1);
                }}
                placeholder="Select a collection..."
              >
                <Select.Option id="" value="">
                  Select a collection...
                </Select.Option>
                {collectionsArray.map((col) => (
                  <Select.Option key={col.name} id={col.name} value={col.name}>
                    {col.name} ({formatNumber(col.vector_count || 0)} vectors)
                  </Select.Option>
                ))}
              </Select>
            </div>
            <div className="w-32">
              <Select
                label="Page Size"
                value={String(pageSize)}
                onChange={(value) => {
                  setPageSize(Number(value));
                  setCurrentPage(1);
                }}
              >
                <Select.Option id="10" value="10">10</Select.Option>
                <Select.Option id="25" value="25">25</Select.Option>
                <Select.Option id="50" value="50">50</Select.Option>
                <Select.Option id="100" value="100">100</Select.Option>
              </Select>
            </div>
          </div>
          <div className="flex items-end">
            <Button
              variant="primary"
              onClick={loadVectors}
              disabled={!selectedCollection || loading}
              isLoading={loading}
            >
              <RefreshCw01 className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </div>
      </Card>

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {loading ? (
        <LoadingState message="Loading vectors..." />
      ) : !selectedCollection ? (
        <Card>
          <div className="text-center py-12">
            <svg className="w-16 h-16 mx-auto text-neutral-400 dark:text-neutral-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">Select a Collection</h3>
            <p className="text-neutral-500 dark:text-neutral-400">Choose a collection to browse its vectors</p>
          </div>
        </Card>
      ) : vectors.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <svg className="w-16 h-16 mx-auto text-neutral-400 dark:text-neutral-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">No Vectors Found</h3>
            <p className="text-neutral-500 dark:text-neutral-400">This collection appears to be empty</p>
          </div>
        </Card>
      ) : (
        <>
          {/* Header with collection name and total */}
          <Card>
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-xl font-semibold text-neutral-900 dark:text-white">
                  Vectors in {selectedCollection}
                </h2>
                <p className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">
                  {formatNumber(totalVectors)} total vectors
                </p>
              </div>
            </div>
          </Card>

          {/* Pagination Info */}
          <div className="flex items-center justify-between">
            <div className="text-sm text-neutral-600 dark:text-neutral-400">
              Showing {startIndex}-{endIndex} of {formatNumber(totalVectors)} vectors
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handlePageChange(currentPage - 1)}
                disabled={currentPage <= 1}
              >
                Previous
              </Button>
              <span className="text-sm text-neutral-600 dark:text-neutral-400 px-3">
                Page {currentPage} of {totalPages}
              </span>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handlePageChange(currentPage + 1)}
                disabled={currentPage >= totalPages}
              >
                Next
              </Button>
            </div>
          </div>

          {/* Vectors Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {vectors.map((vector) => (
              <Card key={vector.id} className="hover:shadow-lg transition-shadow">
                <div className="space-y-4">
                  {/* Header with icon */}
                  <div className="flex items-center justify-center">
                    <div className="w-12 h-12 rounded-lg bg-primary-100 dark:bg-primary-900/20 flex items-center justify-center">
                      <svg className="w-6 h-6 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                      </svg>
                    </div>
                  </div>

                  {/* Vector ID and File Type */}
                  <div className="text-center">
                    <div className="text-lg font-semibold text-neutral-900 dark:text-white truncate" title={vector.id}>
                      {vector.id}
                    </div>
                    <div className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">
                      {getVectorFileType(vector)}
                    </div>
                  </div>

                  {/* Payload Preview */}
                  <div className="min-h-[120px]">
                    {getPayloadContent(vector) ? (
                      <CodeEditor
                        value={getPayloadContent(vector)}
                        language="json"
                        height="120px"
                        readOnly={true}
                      />
                    ) : (
                      <div className="flex flex-col items-center justify-center text-neutral-400 dark:text-neutral-500 py-4 border border-neutral-200 dark:border-neutral-800 rounded-lg">
                        <svg className="w-6 h-6 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                        <span className="text-xs">No content</span>
                      </div>
                    )}
                  </div>

                  {/* Footer with meta info and actions */}
                  <div className="flex items-center justify-between pt-2 border-t border-neutral-200 dark:border-neutral-800/50">
                    <div className="flex items-center gap-3 text-xs text-neutral-500 dark:text-neutral-400">
                      {getVectorSource(vector) !== 'Unknown' && (
                        <span className="flex items-center gap-1" title={getVectorSource(vector)}>
                          <svg className="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                          </svg>
                          <span className="truncate max-w-[100px]">{getVectorSource(vector)}</span>
                        </span>
                      )}
                      {getVectorChunkIndex(vector) > 0 && (
                        <span className="flex items-center gap-1">
                          <svg className="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14" />
                          </svg>
                          Chunk {getVectorChunkIndex(vector)}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={async () => {
                          try {
                            await navigator.clipboard.writeText(vector.id);
                            toast.success('Vector ID copied to clipboard');
                          } catch (err) {
                            toast.error('Failed to copy Vector ID');
                          }
                        }}
                        className="p-1.5 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 transition-colors rounded"
                        title="Copy ID"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                        </svg>
                      </button>
                      <button
                        onClick={() => {
                          setSelectedVector(vector);
                          setDetailsModalOpen(true);
                        }}
                        className="p-1.5 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 transition-colors rounded"
                        title="View Details"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              </Card>
            ))}
          </div>

          {/* Bottom Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2">
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handlePageChange(currentPage - 1)}
                disabled={currentPage <= 1}
              >
                Previous
              </Button>
              <span className="text-sm text-neutral-600 dark:text-neutral-400 px-3">
                Page {currentPage} of {totalPages}
              </span>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handlePageChange(currentPage + 1)}
                disabled={currentPage >= totalPages}
              >
                Next
              </Button>
            </div>
          )}
        </>
      )}

      {/* Vector Details Modal */}
      <VectorDetailsModal
        isOpen={detailsModalOpen}
        onClose={() => {
          setDetailsModalOpen(false);
          setSelectedVector(null);
        }}
        vector={selectedVector}
        collectionName={selectedCollection}
        onEdit={() => {
          setDetailsModalOpen(false);
          setEditModalOpen(true);
        }}
      />

      {/* Edit Vector Modal */}
      <EditVectorModal
        isOpen={editModalOpen}
        onClose={() => {
          setEditModalOpen(false);
          setSelectedVector(null);
        }}
        vector={selectedVector}
        collectionName={selectedCollection}
        onUpdate={() => {
          // Reload vectors after update
          if (selectedCollection) {
            loadVectors();
          }
        }}
      />
    </div>
  );
}

export default VectorsPage;

