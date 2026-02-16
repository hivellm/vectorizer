/**
 * Overview page - Dashboard home with dark mode support
 * Matches dashboard v1 functionality
 */

import { useEffect, useRef } from 'react';
import { Link } from 'react-router-dom';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import LoadingState from '@/components/LoadingState';
import StatCard from '@/components/ui/StatCard';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import WelcomeBanner from '@/components/WelcomeBanner';
import { formatNumber, formatDate } from '@/utils/formatters';
import { SearchLg, BarChart01, Code01 } from '@untitledui/icons';
import type { Collection } from '@/hooks/useCollections';

function OverviewPage() {
  const { listCollections } = useCollections();
  const { collections, loading, error, setCollections, setLoading, setError } = useCollectionsStore();
  const autoRefreshIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const fetchCollections = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listCollections();
      console.log('[OverviewPage] Received data:', data);
      
      // Ensure data is an array
      if (Array.isArray(data)) {
        console.log('[OverviewPage] Data is array, setting collections:', data.length);
        setCollections(data);
      } else {
        // Handle case where API returns { collections: [...] }
        const apiResponse = data as unknown as { collections?: Collection[] };
        const collectionsArray = Array.isArray(apiResponse?.collections) ? apiResponse.collections : [];
        console.log('[OverviewPage] Extracted collections from object:', collectionsArray.length);
        setCollections(collectionsArray);
      }
    } catch (err) {
      console.error('[OverviewPage] Error fetching collections:', err);
      setError(err instanceof Error ? err.message : 'Failed to load collections');
      setCollections([]); // Set empty array on error
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    // Initial fetch
    fetchCollections();

    // Auto-refresh every 30 seconds
    autoRefreshIntervalRef.current = setInterval(() => {
      fetchCollections();
    }, 30000);

    // Cleanup interval on unmount
    return () => {
      if (autoRefreshIntervalRef.current) {
        clearInterval(autoRefreshIntervalRef.current);
      }
    };
  }, [listCollections, setCollections, setLoading, setError]);

  // Ensure collections is always an array
  const collectionsArray = Array.isArray(collections) ? collections : [];
  
  const totalVectors = collectionsArray.reduce((sum, col) => sum + (col.vector_count || 0), 0);
  const avgDimension = collectionsArray.length > 0
    ? Math.round(collectionsArray.reduce((sum, col) => sum + (col.dimension || 0), 0) / collectionsArray.length)
    : 0;

  if (loading) {
    return <LoadingState message="Loading dashboard..." />;
  }

  const topCollections = collectionsArray.slice(0, 5);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl sm:text-2xl font-bold text-neutral-900 dark:text-white">Overview</h1>
        <p className="text-sm sm:text-base text-neutral-600 dark:text-neutral-400 mt-1">Welcome to Vectorizer Dashboard</p>
      </div>

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      {/* Welcome Banner - Show for first-time users */}
      <WelcomeBanner />
      
      {/* Stats Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
        <StatCard
          title="Collections"
          value={formatNumber(collectionsArray.length)}
          subtitle={`${collectionsArray.length} active collection${collectionsArray.length !== 1 ? 's' : ''}`}
        />
        <StatCard
          title="Total Vectors"
          value={formatNumber(totalVectors)}
          subtitle="Across all collections"
        />
        <StatCard
          title="Avg Dimension"
          value={formatNumber(avgDimension)}
          subtitle="Average vector dimension"
        />
        <StatCard
          title="Server Status"
          value="Online"
          subtitle="All systems operational"
        />
      </div>

      {/* Collections Overview */}
      <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-neutral-900 dark:text-white">Collections Overview</h2>
          <Link to="/collections">
            <Button variant="primary" size="sm">
              View All
            </Button>
          </Link>
        </div>
        {topCollections.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-neutral-200 dark:divide-neutral-700">
              <thead className="bg-neutral-50 dark:bg-neutral-800/50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">Name</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">Vectors</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">Dimension</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">Metric</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">Created</th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-neutral-900 divide-y divide-neutral-200 dark:divide-neutral-800/50">
                {topCollections.map((col) => (
                  <tr key={col.name} className="hover:bg-neutral-50 dark:hover:bg-neutral-800/50">
                    <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-neutral-900 dark:text-white">{col.name}</td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-neutral-600 dark:text-neutral-400">{formatNumber(col.vector_count || 0)}</td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-neutral-600 dark:text-neutral-400">{col.dimension}</td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-neutral-600 dark:text-neutral-400">{col.metric}</td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-neutral-600 dark:text-neutral-400">
                      {col.created_at ? formatDate(col.created_at) : 'N/A'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="text-center py-8 text-neutral-500 dark:text-neutral-400">
            <p>No collections found</p>
          </div>
        )}
      </Card>

      {/* Quick Actions */}
      <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50">
        <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">Quick Actions</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Link to="/search">
            <div className="p-4 border border-neutral-200 dark:border-neutral-800/50 rounded-lg hover:bg-neutral-50 dark:hover:bg-neutral-800/50 cursor-pointer transition-colors">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-neutral-100 dark:bg-neutral-800 rounded-lg flex items-center justify-center">
                  <SearchLg className="w-5 h-5 text-neutral-600 dark:text-neutral-300" />
                </div>
                <div>
                  <h3 className="font-medium text-neutral-900 dark:text-white">Search Vectors</h3>
                  <p className="text-sm text-neutral-500 dark:text-neutral-400">Search across collections</p>
                </div>
              </div>
            </div>
          </Link>
          <Link to="/vectors">
            <div className="p-4 border border-neutral-200 dark:border-neutral-800/50 rounded-lg hover:bg-neutral-50 dark:hover:bg-neutral-800/50 cursor-pointer transition-colors">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-neutral-100 dark:bg-neutral-800 rounded-lg flex items-center justify-center">
                  <BarChart01 className="w-5 h-5 text-neutral-600 dark:text-neutral-300" />
                </div>
                <div>
                  <h3 className="font-medium text-neutral-900 dark:text-white">Browse Vectors</h3>
                  <p className="text-sm text-neutral-500 dark:text-neutral-400">Browse vector data</p>
                </div>
              </div>
            </div>
          </Link>
          <Link to="/docs">
            <div className="p-4 border border-neutral-200 dark:border-neutral-800/50 rounded-lg hover:bg-neutral-50 dark:hover:bg-neutral-800/50 cursor-pointer transition-colors">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-neutral-100 dark:bg-neutral-800 rounded-lg flex items-center justify-center">
                  <Code01 className="w-5 h-5 text-neutral-600 dark:text-neutral-300" />
                </div>
                <div>
                  <h3 className="font-medium text-neutral-900 dark:text-white">API Playground</h3>
                  <p className="text-sm text-neutral-500 dark:text-neutral-400">Test API endpoints</p>
                </div>
              </div>
            </div>
          </Link>
        </div>
      </Card>

      {/* System Information */}
      <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50">
        <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-4">System Information</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <span className="text-sm text-neutral-500 dark:text-neutral-400">Version:</span>
            <span className="ml-2 text-sm font-medium text-neutral-900 dark:text-white">1.0.0</span>
          </div>
          <div>
            <span className="text-sm text-neutral-500 dark:text-neutral-400">Uptime:</span>
            <span className="ml-2 text-sm font-medium text-neutral-900 dark:text-white">N/A</span>
          </div>
          <div>
            <span className="text-sm text-neutral-500 dark:text-neutral-400">Memory Usage:</span>
            <span className="ml-2 text-sm font-medium text-neutral-900 dark:text-white">N/A</span>
          </div>
        </div>
      </Card>
    </div>
  );
}

export default OverviewPage;
