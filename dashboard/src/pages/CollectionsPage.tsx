/**
 * Collections page - Dark mode support
 * Matches dashboard v1 layout with cards grid
 */

import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import LoadingState from '@/components/LoadingState';
import Button from '@/components/ui/Button';
import Card from '@/components/ui/Card';
import { Dropdown } from '@/components/ui/Dropdown';
import CreateCollectionModal from '@/components/modals/CreateCollectionModal';
import CollectionDetailsModal from '@/components/modals/CollectionDetailsModal';
import DeleteCollectionModal from '@/components/modals/DeleteCollectionModal';
import { formatNumber, formatDate } from '@/utils/formatters';

function CollectionsPage() {
  const navigate = useNavigate();
  const { listCollections } = useCollections();
  const { collections, loading, error, setCollections, setLoading, setError } = useCollectionsStore();
  const [filter, setFilter] = useState('');
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [detailsModalOpen, setDetailsModalOpen] = useState(false);
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [selectedCollection, setSelectedCollection] = useState<any>(null);
  const [collectionToDelete, setCollectionToDelete] = useState<string>('');

  useEffect(() => {
    const fetchCollections = async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await listCollections();
        console.log('[CollectionsPage] Received data:', data);
        // Ensure data is always an array
        const collectionsArray = Array.isArray(data) ? data : [];
        console.log('[CollectionsPage] Setting collections:', collectionsArray.length);
        setCollections(collectionsArray);
      } catch (err) {
        console.error('[CollectionsPage] Error fetching collections:', err);
        setError(err instanceof Error ? err.message : 'Failed to load collections');
        setCollections([]); // Ensure empty array on error
      } finally {
        setLoading(false);
      }
    };

    fetchCollections();
  }, [listCollections, setCollections, setLoading, setError]);

  // Ensure collections is always an array
  const collectionsArray = Array.isArray(collections) ? collections : [];
  
  // Filter collections
  const filteredCollections = filter
    ? collectionsArray.filter(col => 
        col.name.toLowerCase().includes(filter.toLowerCase()) ||
        col.metric?.toLowerCase().includes(filter.toLowerCase()) ||
        col.indexing_status?.status?.toLowerCase().includes(filter.toLowerCase())
      )
    : collectionsArray;

  // Calculate stats
  const totalVectors = collectionsArray.reduce((sum, col) => sum + (col.vector_count || 0), 0);
  const avgDimension = collectionsArray.length > 0
    ? Math.round(collectionsArray.reduce((sum, col) => sum + (col.dimension || 0), 0) / collectionsArray.length)
    : 0;

  const getNormalizationStatus = (collection: any) => {
    if (collection.normalization?.enabled) {
      return collection.normalization.level || 'Enabled';
    }
    return 'Disabled';
  };

  const getNormalizationClass = (collection: any) => {
    return collection.normalization?.enabled ? 'text-green-600 dark:text-green-400' : 'text-neutral-500 dark:text-neutral-400';
  };

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'completed':
      case 'cached':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400';
      case 'processing':
      case 'indexing':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400';
      case 'error':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      default:
        return 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-400';
    }
  };

  if (loading) {
    return <LoadingState message="Loading collections..." />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Collections</h1>
          <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">Manage your vector collections</p>
        </div>
        <Button variant="primary" onClick={() => setCreateModalOpen(true)} className="w-full sm:w-auto">
          Create Collection
        </Button>
      </div>

      {/* Stats */}
      {collectionsArray.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card>
            <div className="text-center">
              <div className="text-3xl font-bold text-neutral-900 dark:text-white">{collectionsArray.length}</div>
              <div className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">Collections</div>
            </div>
          </Card>
          <Card>
            <div className="text-center">
              <div className="text-3xl font-bold text-neutral-900 dark:text-white">{formatNumber(totalVectors)}</div>
              <div className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">Total Vectors</div>
            </div>
          </Card>
          <Card>
            <div className="text-center">
              <div className="text-3xl font-bold text-neutral-900 dark:text-white">{avgDimension}</div>
              <div className="text-sm text-neutral-500 dark:text-neutral-400 mt-1">Avg Dimension</div>
            </div>
          </Card>
        </div>
      )}

      {/* Filter */}
      {collectionsArray.length > 0 && (
        <Card>
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <input
                type="text"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                placeholder="Filter collections by name, metric, or status..."
                className="w-full px-4 py-2 pl-10 border border-neutral-300 dark:border-neutral-800 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
              <div className="absolute left-3 top-1/2 transform -translate-y-1/2 text-neutral-400">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              {filter && (
                <button
                  onClick={() => setFilter('')}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300"
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              )}
            </div>
            <div className="text-sm text-neutral-500 dark:text-neutral-400">
              {filter ? `Showing ${filteredCollections.length} of ${collectionsArray.length} collections` : `${collectionsArray.length} collections total`}
            </div>
          </div>
        </Card>
      )}

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Collections Grid */}
      {filteredCollections.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <p className="text-neutral-500 dark:text-neutral-400">
              {filter ? 'No collections match your filter' : 'No collections found'}
            </p>
            <p className="text-sm text-neutral-400 dark:text-neutral-500 mt-2">
              {filter ? 'Try adjusting your search' : 'Create your first collection to get started'}
            </p>
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
          {filteredCollections.map((collection) => (
            <Card key={collection.name} className="hover:shadow-lg transition-shadow">
              <div className="space-y-4">
                {/* Header */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900/20 flex items-center justify-center">
                      <svg className="w-6 h-6 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="font-semibold text-neutral-900 dark:text-white">{collection.name}</h3>
                      <span className={`text-xs px-2 py-1 rounded ${getStatusClass(collection.indexing_status?.status || 'completed')}`}>
                        {collection.indexing_status?.status === 'cached' ? 'Cache' : 'Indexed'}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Stats */}
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-neutral-500 dark:text-neutral-400">Vectors:</span>
                    <span className="font-medium text-neutral-900 dark:text-white">{formatNumber(collection.vector_count || 0)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-neutral-500 dark:text-neutral-400">Dimension:</span>
                    <span className="font-medium text-neutral-900 dark:text-white">{collection.dimension}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-neutral-500 dark:text-neutral-400">Metric:</span>
                    <span className="font-medium text-neutral-900 dark:text-white capitalize">{collection.metric}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-neutral-500 dark:text-neutral-400">Normalization:</span>
                    <span className={`font-medium ${getNormalizationClass(collection)}`}>
                      {getNormalizationStatus(collection)}
                    </span>
                  </div>
                  {collection.created_at && (
                    <div className="flex justify-between">
                      <span className="text-neutral-500 dark:text-neutral-400">Created:</span>
                      <span className="font-medium text-neutral-900 dark:text-white text-xs">
                        {formatDate(collection.created_at)}
                      </span>
                    </div>
                  )}
                </div>

                {/* Progress */}
                {collection.indexing_status && (collection.indexing_status.status === 'processing' || collection.indexing_status.status === 'indexing') && (
                  <div className="space-y-1">
                    <div className="flex justify-between text-xs text-neutral-500 dark:text-neutral-400">
                      <span>Indexing</span>
                      <span>{Math.round((collection.indexing_status.progress || 0) * 100)}%</span>
                    </div>
                    <div className="w-full bg-neutral-200 dark:bg-neutral-700 rounded-full h-2">
                      <div
                        className="bg-primary-600 h-2 rounded-full transition-all"
                        style={{ width: `${(collection.indexing_status.progress || 0) * 100}%` }}
                      />
                    </div>
                  </div>
                )}

                {/* Actions */}
                <div className="flex gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-700">
                  <Button
                    variant="secondary"
                    size="sm"
                    className="flex-1"
                    onClick={() => {
                      setSelectedCollection(collection);
                      setDetailsModalOpen(true);
                    }}
                  >
                    View
                  </Button>
                  <Button
                    variant="primary"
                    size="sm"
                    className="flex-1"
                    onClick={() => navigate('/vectors', { state: { collectionName: collection.name } })}
                  >
                    Browse
                  </Button>
                  <Dropdown
                    variant="icon"
                    placement="bottom end"
                    icon={
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                      </svg>
                    }
                  >
                    <Dropdown.Item
                      id="view-details"
                      onAction={() => {
                        setSelectedCollection(collection);
                        setDetailsModalOpen(true);
                      }}
                    >
                      View Details
                    </Dropdown.Item>
                    <Dropdown.Item
                      id="browse-vectors"
                      onAction={() => navigate('/vectors', { state: { collectionName: collection.name } })}
                    >
                      Browse Vectors
                    </Dropdown.Item>
                    <Dropdown.Separator />
                    <Dropdown.Item
                      id="delete"
                      onAction={() => {
                        setCollectionToDelete(collection.name);
                        setDeleteModalOpen(true);
                      }}
                    >
                      Delete Collection
                    </Dropdown.Item>
                  </Dropdown>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Modals */}
      <CreateCollectionModal
        isOpen={createModalOpen}
        onClose={() => {
          setCreateModalOpen(false);
          // Refresh collections after creation
          listCollections().then(data => {
            const collectionsArray = Array.isArray(data) ? data : [];
            setCollections(collectionsArray);
          });
        }}
      />
      <CollectionDetailsModal
        isOpen={detailsModalOpen}
        onClose={() => {
          setDetailsModalOpen(false);
          setSelectedCollection(null);
        }}
        collection={selectedCollection}
      />
      <DeleteCollectionModal
        isOpen={deleteModalOpen}
        onClose={() => {
          setDeleteModalOpen(false);
          setCollectionToDelete('');
        }}
        collectionName={collectionToDelete}
      />
    </div>
  );
}

export default CollectionsPage;
