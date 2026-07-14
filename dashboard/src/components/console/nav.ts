import type { Icons } from './Icons';

export interface NavEntry {
  to: string;
  label: string;
  icon: keyof typeof Icons;
  /** Set true to keep this entry out of the primary sidebar (e.g. Settings goes to the secondary section). */
  secondary?: boolean;
  /** Optional command palette hint (e.g. "G O"). */
  hint?: string;
  /** Optional badge shown in the sidebar item. */
  badge?: string;
}

export const NAV: NavEntry[] = [
  { to: '/overview',      label: 'Overview',         icon: 'dashboard',   hint: 'G O' },
  { to: '/collections',   label: 'Collections',      icon: 'collections', hint: 'G C' },
  { to: '/search',        label: 'Search',           icon: 'search',      hint: 'G S' },
  { to: '/vectors',       label: 'Vectors',          icon: 'vectors',     hint: 'G V' },
  { to: '/monitoring',    label: 'Monitoring',       icon: 'activity',    hint: 'G M' },
  { to: '/api-keys',      label: 'API Keys',         icon: 'keys',        hint: 'G K' },
  { to: '/mcp-tools',     label: 'MCP Tools',        icon: 'mcp' },
  { to: '/configuration', label: 'Settings',         icon: 'settings',    secondary: true },
];

/** Crumbs map for ConsoleLayout. Routes not in NAV (file-watcher, graph, etc.) are listed
 *  as plain strings here for completeness. */
export const CRUMBS: Record<string, string[]> = {
  '/overview':       ['Vectorizer', 'Overview'],
  '/collections':    ['Vectorizer', 'Collections'],
  '/search':         ['Vectorizer', 'Search Playground'],
  '/vectors':        ['Vectorizer', 'Vector Browser'],
  '/monitoring':     ['Vectorizer', 'Monitoring'],
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
  '/test':           ['Vectorizer', 'Test'],
};
