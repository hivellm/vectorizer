/**
 * Application router with persistent navigation
 */

import { Routes, Route, Navigate } from 'react-router';
import MainLayout from '@/components/layout/MainLayout';
import OverviewPage from '@/pages/OverviewPage';
import CollectionsPage from '@/pages/CollectionsPage';
import VectorsPage from '@/pages/VectorsPage';
import SearchPage from '@/pages/SearchPage';
import FileWatcherPage from '@/pages/FileWatcherPage';
import GraphPage from '@/pages/GraphPage';
import ConnectionsPage from '@/pages/ConnectionsPage';
import WorkspacePage from '@/pages/WorkspacePage';
import ConfigurationPage from '@/pages/ConfigurationPage';
import LogsPage from '@/pages/LogsPage';
import BackupsPage from '@/pages/BackupsPage';
import TestPage from '@/pages/TestPage';

function AppRouter() {
  return (
    <Routes>
      <Route path="/" element={<MainLayout />}>
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
      </Route>
    </Routes>
  );
}

export default AppRouter;
