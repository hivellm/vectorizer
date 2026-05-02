import { useContext } from 'react';
import { Link, useLocation } from 'react-router-dom';
import AuthContext from '@/contexts/AuthContext';
import { Icons } from './Icons';
import { HexLogo } from './primitives/HexLogo';

interface NavEntry {
  to: string;
  label: string;
  icon: keyof typeof Icons;
  badge?: string;
}

const PRIMARY: NavEntry[] = [
  { to: '/overview', label: 'Overview', icon: 'dashboard' },
  { to: '/collections', label: 'Collections', icon: 'collections' },
  { to: '/search', label: 'Search', icon: 'search' },
  { to: '/vectors', label: 'Vectors', icon: 'vectors' },
  { to: '/monitoring', label: 'Monitoring', icon: 'activity' },
  { to: '/cluster', label: 'Replication', icon: 'globe' },
  { to: '/api-keys', label: 'API Keys', icon: 'keys' },
  { to: '/mcp-tools', label: 'MCP Tools', icon: 'mcp' },
];

const SECONDARY: NavEntry[] = [
  { to: '/configuration', label: 'Settings', icon: 'settings' },
];

interface Props {
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

export function ConsoleSidebar({ collapsed, onToggleCollapsed }: Props) {
  const location = useLocation();
  // Use context directly (not useAuth) so the sidebar can render even when
  // not wrapped in AuthProvider (e.g. in isolated tests). The `user && ...`
  // guard below handles the missing-context / unauthenticated case.
  const auth = useContext(AuthContext);
  const user = auth?.user ?? null;
  const isActive = (to: string) =>
    location.pathname === to || location.pathname.startsWith(to + '/');

  const renderItem = (n: NavEntry) => {
    const Icon = Icons[n.icon];
    return (
      <Link
        key={n.to}
        to={n.to}
        className={`nav-item ${isActive(n.to) ? 'active' : ''}`}
        title={collapsed ? n.label : undefined}
        style={collapsed ? { justifyContent: 'center', padding: '8px 0' } : undefined}
      >
        <Icon className="icon" />
        {!collapsed && <span>{n.label}</span>}
        {!collapsed && n.badge && <span className="badge">{n.badge}</span>}
      </Link>
    );
  };

  return (
    <aside className="sidebar" style={collapsed ? { width: 60 } : undefined}>
      <div
        className="sidebar-brand"
        style={collapsed ? { padding: '16px 0', justifyContent: 'center' } : undefined}
      >
        <HexLogo size={28} />
        {!collapsed && (
          <>
            <div>
              <div className="name">Vectorizer</div>
            </div>
            <span className="ver">v3.0.0</span>
          </>
        )}
      </div>

      <div className="sidebar-section">
        {!collapsed && <div className="sidebar-label">Workspace</div>}
        <nav className="sidebar-nav">{PRIMARY.map(renderItem)}</nav>
      </div>

      <div className="sidebar-section" style={{ marginTop: 'auto' }}>
        <nav className="sidebar-nav">
          {SECONDARY.map(renderItem)}
          <div
            className="nav-item"
            onClick={onToggleCollapsed}
            role="button"
            title="Collapse"
            style={collapsed ? { justifyContent: 'center', padding: '8px 0' } : undefined}
          >
            <Icons.panel2 className="icon" />
            {!collapsed && <span>Collapse sidebar</span>}
          </div>
        </nav>
      </div>

      {!collapsed && user && (
        <div className="sidebar-footer">
          <div className="avatar">{user.username.slice(0, 2).toUpperCase()}</div>
          <div className="info">
            <div className="n">{user.username}</div>
            <div className="e">role: {user.roles?.[0] ?? 'User'}</div>
          </div>
        </div>
      )}
    </aside>
  );
}
