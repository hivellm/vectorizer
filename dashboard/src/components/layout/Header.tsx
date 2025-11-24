/**
 * Header component - Untitled UI style
 */

import { useLocation } from 'react-router';

interface HeaderProps {
  onMenuClick?: () => void;
}

function Header({ onMenuClick }: HeaderProps) {
  const location = useLocation();
  
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
    return 'Dashboard';
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
        {/* Add header actions here */}
      </div>
    </header>
  );
}

export default Header;
