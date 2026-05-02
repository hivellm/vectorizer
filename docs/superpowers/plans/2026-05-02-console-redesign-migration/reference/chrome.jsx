// Sidebar + Topbar
const { Icons } = window;
const { HexLogo } = window.UI;

const NAV = [
  { id: "overview", label: "Overview", icon: "dashboard" },
  { id: "collections", label: "Collections", icon: "collections", badge: "8" },
  { id: "search", label: "Search Playground", icon: "search" },
  { id: "vectors", label: "Vector Browser", icon: "vectors" },
  { id: "monitoring", label: "Monitoring", icon: "activity" },
  { id: "replication", label: "Replication", icon: "globe", badge: "4" },
  { id: "keys", label: "API Keys", icon: "keys" },
  { id: "mcp", label: "MCP Tools", icon: "mcp" },
];
const NAV_BOTTOM = [
  { id: "settings", label: "Settings", icon: "settings" },
];

const Sidebar = ({ active, onNavigate, collapsed, onToggleCollapsed }) => {
  return (
    <aside className="sidebar" style={collapsed ? { width: 60 } : null}>
      <div className="sidebar-brand" style={collapsed ? { padding: "16px 0", justifyContent: "center" } : null}>
        <HexLogo size={28}/>
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
        <nav className="sidebar-nav">
          {NAV.map(n => {
            const Ic = Icons[n.icon];
            return (
              <div key={n.id}
                className={`nav-item ${active===n.id?"active":""}`}
                onClick={() => onNavigate(n.id)}
                title={collapsed ? n.label : ""}
                style={collapsed ? { justifyContent: "center", padding: "8px 0" } : null}>
                <Ic className="icon"/>
                {!collapsed && <span>{n.label}</span>}
                {!collapsed && n.badge && <span className="badge">{n.badge}</span>}
              </div>
            );
          })}
        </nav>
      </div>

      <div className="sidebar-section" style={{ marginTop: "auto" }}>
        <nav className="sidebar-nav">
          {NAV_BOTTOM.map(n => {
            const Ic = Icons[n.icon];
            return (
              <div key={n.id}
                className={`nav-item ${active===n.id?"active":""}`}
                onClick={() => onNavigate(n.id)}
                title={collapsed ? n.label : ""}
                style={collapsed ? { justifyContent: "center", padding: "8px 0" } : null}>
                <Ic className="icon"/>
                {!collapsed && <span>{n.label}</span>}
              </div>
            );
          })}
          <div className="nav-item" onClick={onToggleCollapsed} title="Collapse"
            style={collapsed ? { justifyContent: "center", padding: "8px 0" } : null}>
            <Icons.panel2 className="icon"/>
            {!collapsed && <span>Collapse sidebar</span>}
          </div>
        </nav>
      </div>

      {!collapsed && (
        <div className="sidebar-footer">
          <div className="avatar">VZ</div>
          <div className="info">
            <div className="n">admin@hivellm</div>
            <div className="e">role: Admin</div>
          </div>
        </div>
      )}
    </aside>
  );
};

const Topbar = ({ crumbs, onOpenCmd, onOpenTweaks }) => {
  return (
    <div className="topbar">
      <div className="crumbs">
        {crumbs.map((c, i) => (
          <React.Fragment key={i}>
            {i>0 && <span className="sep">›</span>}
            <span className={i===crumbs.length-1?"now":""}>{c}</span>
          </React.Fragment>
        ))}
      </div>
      <div className="env-switch">
        <span className="dot"/>
        <span>production</span>
        <Icons.chevron size={12}/>
      </div>
      <div className="cmdk" onClick={onOpenCmd}>
        <Icons.search size={13}/>
        <span>Search collections, vectors, keys…</span>
        <span className="kbd">⌘K</span>
      </div>
      <button className="icon-btn" title="Notifications"><Icons.bell/></button>
      <button className="icon-btn" title="Refresh"><Icons.refresh/></button>
      <button className="icon-btn" title="Tweaks" onClick={onOpenTweaks}><Icons.settings/></button>
    </div>
  );
};

window.Chrome = { Sidebar, Topbar, NAV };
