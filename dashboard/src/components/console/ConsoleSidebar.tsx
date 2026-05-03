import { Link, useLocation } from 'react-router-dom';
import { useOptionalAuth } from '@/contexts/AuthContext';
import { Icons } from './Icons';
import { HexLogo } from './primitives/HexLogo';
import { NAV, type NavEntry } from './nav';

const PRIMARY = NAV.filter((n) => !n.secondary);
const SECONDARY = NAV.filter((n) => n.secondary);

interface Props {
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

export function ConsoleSidebar({ collapsed, onToggleCollapsed }: Props) {
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
      <div className="sidebar-brand">
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
          <button
            type="button"
            className="nav-item"
            onClick={onToggleCollapsed}
            title="Collapse"
            style={{
              background: 'none',
              border: '1px solid transparent',
              width: '100%',
              textAlign: 'inherit',
              cursor: 'pointer',
              font: 'inherit',
              color: 'inherit',
            }}
          >
            <Icons.panel2 className="icon" />
            {!collapsed && <span>Collapse sidebar</span>}
          </button>
        </nav>
      </div>

      {!collapsed && user && (
        <div className="sidebar-footer">
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
