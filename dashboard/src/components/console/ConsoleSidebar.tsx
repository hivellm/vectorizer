import { Link, useLocation } from 'react-router-dom';
import { useOptionalAuth } from '@/contexts/AuthContext';
import { Icons } from './Icons';
import { HexLogo } from './primitives/HexLogo';
import { NAV, type NavEntry } from './nav';

interface Props {
  collapsed: boolean;
  onToggleCollapsed: () => void;
  /** Server version reported by /health. Falls back to '—' until the
   *  first response lands so the sidebar never shows a stale hardcode. */
  version?: string;
}

export function ConsoleSidebar({ collapsed, onToggleCollapsed, version }: Props) {
  const location = useLocation();
  const auth = useOptionalAuth();
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
      >
        <Icon className="icon" />
        {!collapsed && <span>{n.label}</span>}
        {!collapsed && n.badge && <span className="badge">{n.badge}</span>}
      </Link>
    );
  };

  return (
    <aside className={`sidebar ${collapsed ? 'collapsed' : ''}`}>
      <div
        className="sidebar-brand"
        style={collapsed ? { flexDirection: 'column', gap: 4 } : undefined}
      >
        <HexLogo size={28} />
        {!collapsed && (
          <>
            <div>
              <div className="name">Vectorizer</div>
            </div>
            <span className="ver" style={{ marginLeft: 'auto' }}>
              {version ?? '—'}
            </span>
          </>
        )}
        <button
          type="button"
          onClick={onToggleCollapsed}
          className="icon-btn"
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          style={collapsed ? { marginTop: 8 } : { marginLeft: 6 }}
        >
          <Icons.panel2 />
        </button>
      </div>

      <div className="sidebar-section">
        {!collapsed && <div className="sidebar-label">Workspace</div>}
        <nav className="sidebar-nav">{NAV.map(renderItem)}</nav>
      </div>

      {!collapsed && user && (
        <div className="sidebar-footer" style={{ marginTop: 'auto' }}>
          <div className="avatar">
            {user.username.slice(0, 2).toUpperCase() || 'VZ'}
          </div>
          <div className="info">
            <div className="n">{user.username}</div>
            <div className="e">role: {user.roles?.[0] ?? 'User'}</div>
          </div>
        </div>
      )}
    </aside>
  );
}
