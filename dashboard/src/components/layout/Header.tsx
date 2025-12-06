/**
 * Header component - Untitled UI style
 */

import { useLocation } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';

interface HeaderProps {
  onMenuClick?: () => void;
}

function Header({ onMenuClick }: HeaderProps) {
  const location = useLocation();
  const { user, logout, authRequired } = useAuth();

  // Get page title from path
  const getPageTitle = () => {
    const path = location.pathname;
    if (path === '/overview' || path === '/') return 'Overview';
    if (path === '/collections') return 'Collections';
    if (path === '/search') return 'Search';
    if (path === '/vectors') return 'Vectors';
    if (path === '/file-watcher') return 'File Watcher';
    if (path === '/graph') return 'Graph Relationships';
    if (path === '/connections') return 'Connections';
    if (path === '/workspace') return 'Workspace';
    if (path === '/configuration') return 'Configuration';
    if (path === '/logs') return 'Logs';
    if (path === '/backups') return 'Backups';
    if (path === '/users') return 'User Management';
    if (path === '/api-keys') return 'API Keys';
    return 'Dashboard';
  };

  const handleLogout = async () => {
    await logout();
  };

  return (
    <header className="h-16 bg-white dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800/50 flex items-center justify-between px-3 sm:px-4 md:px-6">
      <div className="flex items-center gap-3">
        {/* Mobile Menu Button */}
        {onMenuClick && (
          <button
            onClick={onMenuClick}
            className="lg:hidden p-2 text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        )}
        <h2 className="text-base sm:text-lg font-semibold text-neutral-900 dark:text-white leading-none">
          {getPageTitle()}
        </h2>
      </div>
      <div className="flex items-center gap-4">
        {/* User info and logout */}
        {authRequired && user && (
          <div className="flex items-center gap-3">
            <div className="hidden sm:flex items-center gap-2">
              <div className="w-8 h-8 rounded-full bg-indigo-600 flex items-center justify-center">
                <span className="text-sm font-medium text-white">
                  {user.username.charAt(0).toUpperCase()}
                </span>
              </div>
              <span className="text-sm text-neutral-700 dark:text-neutral-300">
                {user.username}
              </span>
              {user.roles.includes('Admin') && (
                <span className="px-2 py-0.5 text-xs font-medium bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400 rounded">
                  Admin
                </span>
              )}
            </div>
            <button
              onClick={handleLogout}
              className="p-2 text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
              title="Logout"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
            </button>
          </div>
        )}
      </div>
    </header>
  );
}

export default Header;
