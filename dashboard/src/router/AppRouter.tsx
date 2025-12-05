/**
 * Application router with persistent navigation and code splitting
 */

import { lazy, Suspense } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import MainLayout from '@/components/layout/MainLayout';
import LoadingSpinner from '@/components/LoadingSpinner';
import ProtectedRoute from '@/components/ProtectedRoute';
import { useAuth } from '@/contexts/AuthContext';

// Lazy load pages for code splitting (smaller initial bundle)
const LoginPage = lazy(() => import('@/pages/LoginPage'));
const OverviewPage = lazy(() => import('@/pages/OverviewPage'));
const CollectionsPage = lazy(() => import('@/pages/CollectionsPage'));
const VectorsPage = lazy(() => import('@/pages/VectorsPage'));
const SearchPage = lazy(() => import('@/pages/SearchPage'));
const FileWatcherPage = lazy(() => import('@/pages/FileWatcherPage'));
const GraphPage = lazy(() => import('@/pages/GraphPage'));
const ConnectionsPage = lazy(() => import('@/pages/ConnectionsPage'));
const WorkspacePage = lazy(() => import('@/pages/WorkspacePage'));
const ConfigurationPage = lazy(() => import('@/pages/ConfigurationPage'));
const LogsPage = lazy(() => import('@/pages/LogsPage'));
const BackupsPage = lazy(() => import('@/pages/BackupsPage'));
const TestPage = lazy(() => import('@/pages/TestPage'));
const UsersPage = lazy(() => import('@/pages/UsersPage'));
const ApiKeysPage = lazy(() => import('@/pages/ApiKeysPage'));

// Loading fallback component
const PageLoader = () => (
  <div className="flex items-center justify-center min-h-screen">
    <LoadingSpinner size="lg" />
  </div>
);

function AppRouter() {
  const { isAuthenticated } = useAuth();

  return (
    <Suspense fallback={<PageLoader />}>
      <Routes>
        {/* Public routes */}
        <Route
          path="/login"
          element={
            isAuthenticated ? <Navigate to="/overview" replace /> : <LoginPage />
          }
        />

        {/* Protected routes */}
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <MainLayout />
            </ProtectedRoute>
          }
        >
          <Route index element={<Navigate to="/overview" replace />} />
          <Route path="overview" element={<OverviewPage />} />
          <Route path="collections" element={<CollectionsPage />} />
          <Route path="search" element={<SearchPage />} />
          <Route path="vectors" element={<VectorsPage />} />
          <Route path="file-watcher" element={<FileWatcherPage />} />
          <Route path="graph" element={<GraphPage />} />
          <Route path="connections" element={<ConnectionsPage />} />
          <Route path="workspace" element={<WorkspacePage />} />
          <Route path="configuration" element={<ConfigurationPage />} />
          <Route path="logs" element={<LogsPage />} />
          <Route path="backups" element={<BackupsPage />} />
          <Route path="test" element={<TestPage />} />
          <Route path="users" element={<UsersPage />} />
          <Route path="api-keys" element={<ApiKeysPage />} />
        </Route>
      </Routes>
    </Suspense>
  );
}

export default AppRouter;
