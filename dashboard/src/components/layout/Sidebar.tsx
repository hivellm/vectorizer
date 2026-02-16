/**
 * Sidebar navigation component - Untitled UI style
 */

import { Link, useLocation } from 'react-router-dom';
import { useTheme } from '@/providers/ThemeProvider';
import { Moon01, Sun } from '@untitledui/icons';

interface NavItem {
  path: string;
  label: string;
  icon?: string;
}

const navItems: NavItem[] = [
  { path: '/overview', label: 'Overview' },
  { path: '/collections', label: 'Collections' },
  { path: '/search', label: 'Search' },
  { path: '/vectors', label: 'Vectors' },
  { path: '/file-watcher', label: 'File Watcher' },
  { path: '/graph', label: 'Graph Relationships' },
  { path: '/connections', label: 'Connections' },
  { path: '/workspace', label: 'Workspace' },
  { path: '/configuration', label: 'Configuration' },
  { path: '/logs', label: 'Logs' },
  { path: '/backups', label: 'Backups' },
  { path: '/users', label: 'Users' },
  { path: '/api-keys', label: 'API Keys' },
  { path: '/docs', label: 'API Docs' },
];

interface SidebarProps {
  onClose?: () => void;
}

function Sidebar({ onClose }: SidebarProps) {
  const location = useLocation();
  const { theme, toggleTheme } = useTheme();

  return (
    <aside className="w-64 h-full bg-white dark:bg-neutral-900 border-r border-neutral-200 dark:border-neutral-800/50 flex flex-col">
      {/* Logo/Header */}
      <div className="h-16 flex items-center justify-between px-6 border-b border-neutral-200 dark:border-neutral-800">
        <div>
          <h1 className="text-xl font-semibold text-neutral-900 dark:text-white leading-none">Vectorizer</h1>
          <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-1 leading-none">Vector Database</p>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="lg:hidden p-2 text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        )}
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-4">
        <ul className="space-y-1">
          {navItems.map((item) => {
            const isActive = location.pathname === item.path;
            return (
              <li key={item.path}>
                <Link
                  to={item.path}
                  onClick={onClose}
                  className={`
                    flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors
                    ${isActive
                      ? 'bg-neutral-100 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100'
                      : 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/50'
                    }
                  `}
                >
                  <span className="ml-3">{item.label}</span>
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Theme Toggle */}
      <div className="p-4 border-t border-neutral-200 dark:border-neutral-800">
        <button
          onClick={toggleTheme}
          className="w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm font-medium text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        >
          <span>Theme</span>
          {theme === 'dark' ? (
            <Moon01 className="w-4 h-4 text-neutral-600 dark:text-neutral-300" />
          ) : (
            <Sun className="w-4 h-4 text-neutral-600 dark:text-neutral-300" />
          )}
        </button>
      </div>
    </aside>
  );
}

export default Sidebar;
