/**
 * Search page - Search vectors across collections
 */

import { useState, useEffect } from 'react';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import { useApiClient } from '@/hooks/useApiClient';
import { useSearchHistory } from '@/hooks/useSearchHistory';
// LoadingState imported but not used - using inline loading instead
import Button from '@/components/ui/Button';
import Card from '@/components/ui/Card';
import { Select } from '@/components/ui/Select';
import CodeEditor from '@/components/ui/CodeEditor';
import { formatNumber, formatDate } from '@/utils/formatters';
import { SearchLg } from '@untitledui/icons';

// Clock icon component
const ClockIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

// XMark icon component
const XMarkIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
  </svg>
);

interface SearchResult {
  id: string;
  score?: number;
  payload?: any;
  vector?: number[];
}

function SearchPage() {
  const { listCollections } = useCollections();
  const { collections, setCollections } = useCollectionsStore();
  const api = useApiClient();
  const { history, addToHistory, removeFromHistory, clearHistory } = useSearchHistory();

  const [selectedCollection, setSelectedCollection] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [searchLimit, setSearchLimit] = useState<number>(10);
  const [searchType, setSearchType] = useState<'text' | 'vector' | 'hybrid'>('text');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchTime, setSearchTime] = useState<number>(0);
  const [vectorInput, setVectorInput] = useState<string>('');
  const [showHistory, setShowHistory] = useState(false);

  const collectionsArray = Array.isArray(collections) ? collections : [];

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

  const performSearch = async () => {
    if (!selectedCollection) {
      setError('Please select a collection');
      return;
    }

    if (searchType === 'text' && !searchQuery.trim()) {
      setError('Please enter a search query');
      return;
    }

    if (searchType === 'vector' && !vectorInput.trim()) {
      setError('Please enter a vector array');
      return;
    }

    setLoading(true);
    setError(null);
    setResults([]);
    const startTime = Date.now();

    try {
      let response: any;

      if (searchType === 'text') {
        response = await api.post<any>(
          `/collections/${encodeURIComponent(selectedCollection)}/search/text`,
          {
            query: searchQuery,
            limit: searchLimit,
          }
        );
      } else if (searchType === 'vector') {
        try {
          const vector = JSON.parse(vectorInput);
          if (!Array.isArray(vector)) {
            throw new Error('Vector must be an array');
          }
          response = await api.post<any>(
            `/collections/${encodeURIComponent(selectedCollection)}/search`,
            {
              vector,
              limit: searchLimit,
            }
          );
        } catch (parseError) {
          throw new Error('Invalid vector format. Must be a JSON array.');
        }
      } else {
        // Hybrid search
        response = await api.post<any>(
          `/collections/${encodeURIComponent(selectedCollection)}/hybrid_search`,
          {
            query: searchQuery,
            limit: searchLimit,
          }
        );
      }

      const searchResults = Array.isArray(response.results) ? response.results : [];
      setResults(searchResults);
      setSearchTime(Date.now() - startTime);

      // Add to search history
      addToHistory({
        collection: selectedCollection,
        query: searchType === 'vector' ? vectorInput : searchQuery,
        type: searchType,
        limit: searchLimit,
        resultCount: searchResults.length,
      });
    } catch (err) {
      console.error('Search error:', err);
      setError(err instanceof Error ? err.message : 'Search failed');
      setResults([]);
    } finally {
      setLoading(false);
    }
  };

  const formatJSON = (obj: any): string => {
    try {
      return JSON.stringify(obj, null, 2);
    } catch {
      return String(obj);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Search Vectors</h1>
        <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">Search across your vector collections</p>
      </div>

      {/* Search History */}
      {history.length > 0 && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <ClockIcon className="w-5 h-5 text-neutral-500 dark:text-neutral-400" />
              <h2 className="text-lg font-semibold text-neutral-900 dark:text-white">
                Search History
              </h2>
              <span className="text-sm text-neutral-500 dark:text-neutral-400">
                ({history.length})
              </span>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowHistory(!showHistory)}
              >
                {showHistory ? 'Hide' : 'Show'}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={clearHistory}
              >
                Clear
              </Button>
            </div>
          </div>
          {showHistory && (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {history.map((item) => (
                <div
                  key={item.id}
                  className="flex items-center justify-between p-3 border border-neutral-200 dark:border-neutral-800 rounded-lg hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors group"
                >
                  <div
                    className="flex-1 cursor-pointer"
                    onClick={() => {
                      setSelectedCollection(item.collection);
                      setSearchType(item.type);
                      setSearchLimit(item.limit);
                      if (item.type === 'vector') {
                        setVectorInput(item.query);
                      } else {
                        setSearchQuery(item.query);
                      }
                      setShowHistory(false);
                    }}
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-xs font-medium px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 rounded text-neutral-600 dark:text-neutral-300">
                        {item.type}
                      </span>
                      <span className="text-sm font-medium text-neutral-900 dark:text-white">
                        {item.collection}
                      </span>
                      {item.resultCount !== undefined && (
                        <span className="text-xs text-neutral-500 dark:text-neutral-400">
                          ({item.resultCount} results)
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-neutral-600 dark:text-neutral-400 truncate">
                      {item.query}
                    </p>
                    <p className="text-xs text-neutral-500 dark:text-neutral-500 mt-1">
                      {formatDate(new Date(item.timestamp).toISOString())}
                    </p>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      removeFromHistory(item.id);
                    }}
                    className="opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded"
                  >
                    <XMarkIcon className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* Search Controls */}
      <Card>
        <div className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <Select
                label="Collection"
                value={selectedCollection}
                onChange={setSelectedCollection}
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
            <div>
              <Select
                label="Search Type"
                value={searchType}
                onChange={(value) => {
                  setSearchType(value as 'text' | 'vector' | 'hybrid');
                  setError(null);
                }}
              >
                <Select.Option id="text" value="text">Text Search</Select.Option>
                <Select.Option id="vector" value="vector">Vector Search</Select.Option>
                <Select.Option id="hybrid" value="hybrid">Hybrid Search</Select.Option>
              </Select>
            </div>
          </div>

          {searchType === 'text' || searchType === 'hybrid' ? (
            <div>
              <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
                Search Query
              </label>
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !loading) {
                    performSearch();
                  }
                }}
                placeholder="Enter your search query..."
                className="w-full px-3 py-2 border border-neutral-300 dark:border-neutral-800 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>
          ) : (
            <div>
              <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
                Vector (JSON Array)
              </label>
              <textarea
                value={vectorInput}
                onChange={(e) => setVectorInput(e.target.value)}
                placeholder='[0.1, 0.2, 0.3, ...]'
                rows={3}
                className="w-full px-3 py-2 border border-neutral-300 dark:border-neutral-800 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500 font-mono text-sm"
              />
            </div>
          )}

          <div className="flex items-end gap-4">
            <div className="w-32">
              <Select
                label="Limit"
                value={String(searchLimit)}
                onChange={(value) => setSearchLimit(Number(value))}
              >
                <Select.Option id="5" value="5">5</Select.Option>
                <Select.Option id="10" value="10">10</Select.Option>
                <Select.Option id="25" value="25">25</Select.Option>
                <Select.Option id="50" value="50">50</Select.Option>
                <Select.Option id="100" value="100">100</Select.Option>
              </Select>
            </div>
            <Button
              variant="primary"
              onClick={performSearch}
              disabled={loading || !selectedCollection}
              isLoading={loading}
            >
              <SearchLg className="w-4 h-4 mr-2" />
              Search
            </Button>
          </div>
        </div>
      </Card>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Results */}
      {results.length > 0 && (
        <Card>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-neutral-900 dark:text-white">
                Search Results
              </h2>
              <div className="text-sm text-neutral-500 dark:text-neutral-400">
                {results.length} result{results.length !== 1 ? 's' : ''} found
                {searchTime > 0 && ` in ${searchTime}ms`}
              </div>
            </div>

            <div className="space-y-4">
              {results.map((result, index) => (
                <div
                  key={result.id}
                  className="border border-neutral-200 dark:border-neutral-800 rounded-lg p-4 hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors"
                >
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <span className="text-sm font-medium text-neutral-500 dark:text-neutral-400">
                        #{index + 1}
                      </span>
                      <span className="text-sm font-mono text-neutral-900 dark:text-white">
                        {result.id}
                      </span>
                    </div>
                    {result.score !== undefined && (
                      <span className="text-sm font-medium text-neutral-600 dark:text-neutral-300">
                        Score: {result.score.toFixed(4)}
                      </span>
                    )}
                  </div>

                  {result.payload && (
                    <div className="mt-3">
                      <div className="text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-2">
                        Payload:
                      </div>
                      <CodeEditor
                        value={formatJSON(result.payload)}
                        language="json"
                        height="200px"
                        readOnly={true}
                      />
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </Card>
      )}

      {!loading && results.length === 0 && searchQuery && (
        <Card>
          <div className="text-center py-12">
            <SearchLg className="w-16 h-16 mx-auto text-neutral-400 dark:text-neutral-500 mb-4" />
            <h3 className="text-lg font-semibold text-neutral-900 dark:text-white mb-2">
              No Results Found
            </h3>
            <p className="text-neutral-500 dark:text-neutral-400">
              Try adjusting your search query or select a different collection
            </p>
          </div>
        </Card>
      )}
    </div>
  );
}

export default SearchPage;

