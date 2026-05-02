import { useEffect, useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { ToastProvider } from '@/providers/ToastProvider';
import { ConsoleSidebar } from './ConsoleSidebar';
import { ConsoleTopbar } from './ConsoleTopbar';
import { CommandPalette } from './CommandPalette';

const CRUMBS: Record<string, string[]> = {
  '/overview':       ['Vectorizer', 'Overview'],
  '/collections':    ['Vectorizer', 'Collections'],
  '/search':         ['Vectorizer', 'Search Playground'],
  '/vectors':        ['Vectorizer', 'Vector Browser'],
  '/monitoring':     ['Vectorizer', 'Monitoring'],
  '/cluster':        ['Vectorizer', 'Replication'],
  '/api-keys':       ['Vectorizer', 'API Keys'],
  '/mcp-tools':      ['Vectorizer', 'MCP Tools'],
  '/configuration':  ['Vectorizer', 'Settings'],
  '/file-watcher':   ['Vectorizer', 'File Watcher'],
  '/graph':          ['Vectorizer', 'Graph'],
  '/connections':    ['Vectorizer', 'Connections'],
  '/workspace':      ['Vectorizer', 'Workspace'],
  '/logs':           ['Vectorizer', 'Logs'],
  '/backups':        ['Vectorizer', 'Backups'],
  '/users':          ['Vectorizer', 'Users'],
  '/docs':           ['Vectorizer', 'API Docs'],
};

export function ConsoleLayout() {
  const [collapsed, setCollapsed] = useState(false);
  const [cmdOpen, setCmdOpen] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    document.body.dataset.console = '1';
    return () => { delete document.body.dataset.console; };
  }, []);

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCmdOpen((o) => !o);
      } else if (e.key === 'Escape') {
        setCmdOpen(false);
      }
    };
    window.addEventListener('keydown', h);
    return () => window.removeEventListener('keydown', h);
  }, []);

  const crumbs = CRUMBS[location.pathname] ?? ['Vectorizer'];

  return (
    <ToastProvider>
      <div className="app" style={{ gridTemplateColumns: collapsed ? '60px 1fr' : '232px 1fr' }}>
        <ConsoleSidebar collapsed={collapsed} onToggleCollapsed={() => setCollapsed((c) => !c)} />
        <div className="main">
          <ConsoleTopbar crumbs={crumbs} onOpenCmd={() => setCmdOpen(true)} />
          <Outlet />
        </div>
        <CommandPalette
          open={cmdOpen}
          onClose={() => setCmdOpen(false)}
          onNavigate={(to) => navigate(to)}
        />
      </div>
    </ToastProvider>
  );
}
