import { useEffect, useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { ToastProvider } from '@/providers/ToastProvider';
import { ConsoleSidebar } from './ConsoleSidebar';
import { ConsoleTopbar } from './ConsoleTopbar';
import { CommandPalette } from './CommandPalette';
import { CRUMBS } from './nav';

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
    const inEditableSurface = (target: EventTarget | null): boolean => {
      if (!(target instanceof HTMLElement)) return false;
      const tag = target.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
      if (target.isContentEditable) return true;
      // Monaco editor lazy-loaded by CodeEditor; bail if focus is inside one.
      if (target.closest('.monaco-editor')) return true;
      return false;
    };
    const h = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k' && !inEditableSurface(e.target)) {
        e.preventDefault();
        setCmdOpen((o) => !o);
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
