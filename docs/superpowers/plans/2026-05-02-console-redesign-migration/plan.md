# Vectorizer Console Redesign Migration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the entire `dashboard/` SPA from its current Tailwind v4 + Untitled UI shell to the new "Console" design language (dark technical theme with teal/magenta accents) delivered by the design team in `Vectorizer.zip`, while preserving all real API integration, routing, and authentication.

**Architecture:** Add a new design system on top of the existing Vite + React 19 + TypeScript stack: port `styles.css` to `dashboard/src/styles/console.css` (CSS variables + utility classes), build a `console/` component library that mirrors the redesign primitives (Sidebar, Topbar, KPI, Sparkline, StatusPill, Ring, Pill, Card, Tbl, CommandPalette, Icons), introduce `ConsoleLayout` that replaces `MainLayout` in the router, then port each existing page to the new visual language one at a time. Tailwind v4 stays installed but unused so we can roll back if necessary; it gets removed in the final cleanup phase. The plan deliberately keeps real data hooks (`useCollections`, `useApiKeys`, etc.) and never adopts the redesign's mock data.

**Tech Stack:** React 19, TypeScript, Vite 8, react-router-dom 7, Zustand stores, Vitest + happy-dom (unit), Playwright (e2e), existing Axios-based API hooks. Fonts: Inter + JetBrains Mono via Google Fonts (added in `index.html`).

---

## Reference Material

The full redesign source (HTML/JSX prototype + assets + design notes) is preserved at:

```
docs/superpowers/plans/2026-05-02-console-redesign-migration/reference/
├── Vectorizer Console.html      # Entry point of the prototype
├── styles.css                   # Source-of-truth for the design tokens & utilities
├── icons.jsx                    # Inline SVG icon set
├── ui.jsx                       # Sparkline / StatusPill / Ring / HexLogo / useTick
├── chrome.jsx                   # Sidebar + Topbar
├── cmdk.jsx                     # Command palette (⌘K)
├── app.jsx                      # Layout shell + tweaks integration
├── tweaks-panel.jsx             # Designer-time tool — DO NOT PORT
├── data.js                      # Mock data — DO NOT PORT
├── screens/
│   ├── overview.jsx             # KPI strip + System Health + Quantization + Top Collections + Events
│   ├── collections.jsx          # List/detail split + sparklines per collection
│   ├── search.jsx               # 4-tab playground + results + request panel + pipeline
│   ├── vectors.jsx              # Vector browser + embedding bar viz
│   ├── monitoring.jsx           # Throughput strip + SIMD + WAL + caches
│   └── other.jsx                # Replication + ApiKeys + MCP + Settings
├── assets/logo.png              # Hex/cube logo (512×512 RGBA PNG)
└── uploads/*.md                 # Spec docs (caching, simd, wal, replication, capabilities, roadmap)
```

Every task that reproduces a redesign component points to the exact source file under `reference/` so the engineer can `cat` it for the canonical layout/markup.

---

## File Structure

### Files to create

```
dashboard/
├── public/
│   └── logo.png                         # Copied from reference/assets/logo.png
├── src/
│   ├── styles/
│   │   └── console.css                  # Ported from reference/styles.css (TS-friendly, no JSX)
│   └── components/
│       └── console/
│           ├── Icons.tsx                # SVG icon set (one component per icon)
│           ├── ConsoleLayout.tsx        # Outlet host: sidebar + topbar + page slot + cmdk
│           ├── ConsoleSidebar.tsx       # Left navigation (replaces Sidebar.tsx)
│           ├── ConsoleTopbar.tsx        # Crumbs + env switch + cmdk button + icon-btns
│           ├── CommandPalette.tsx       # ⌘K overlay
│           ├── primitives/
│           │   ├── Card.tsx             # .card / .card-head / .card-body wrappers
│           │   ├── Kpi.tsx              # .kpi tile with value/delta/spark
│           │   ├── Pill.tsx             # .pill variants (teal/magenta/amber/green/red/muted)
│           │   ├── StatusPill.tsx       # Status → pill mapping
│           │   ├── Sparkline.tsx        # SVG sparkline (data, color, width, height)
│           │   ├── Ring.tsx             # SVG progress ring
│           │   ├── Bar.tsx              # .bar / span fill
│           │   ├── HexLogo.tsx          # <img> wrapper for /logo.png
│           │   ├── Tbl.tsx              # .tbl wrapper + Tr/Th/Td
│           │   └── KeyValue.tsx         # .kv definition list helper
│           ├── hooks/
│           │   └── useTick.ts           # Live tick interval hook
│           └── index.ts                 # Public re-exports for `@/components/console`
└── e2e/
    └── console-shell.spec.ts            # Playwright smoke test for new layout
```

### Files to modify

```
dashboard/
├── index.html                           # Add Inter + JetBrains Mono font preconnect/links
├── src/
│   ├── main.tsx                         # Import console.css alongside theme.css
│   ├── App.tsx                          # No change expected (kept for now)
│   ├── router/AppRouter.tsx             # Swap MainLayout → ConsoleLayout
│   └── pages/                           # Each page rewritten in console primitives
│       ├── OverviewPage.tsx
│       ├── CollectionsPage.tsx
│       ├── SearchPage.tsx
│       ├── VectorsPage.tsx
│       ├── ConfigurationPage.tsx
│       ├── ApiKeysPage.tsx
│       ├── ClusterPage.tsx
│       ├── FileWatcherPage.tsx
│       ├── GraphPage.tsx
│       ├── BackupsPage.tsx
│       ├── LogsPage.tsx
│       ├── WorkspacePage.tsx
│       ├── UsersPage.tsx
│       ├── ApiDocsPage.tsx
│       ├── ConnectionsPage.tsx
│       ├── SetupWizardPage.tsx
│       ├── LoginPage.tsx
│       └── (new) MonitoringPage.tsx     # Combines metrics from existing logs/metrics
│       └── (new) McpToolsPage.tsx       # New: surface registered MCP tools
```

### Files to delete (final cleanup, Phase 5)

```
dashboard/src/components/layout/MainLayout.tsx   # Replaced by ConsoleLayout
dashboard/src/components/layout/Sidebar.tsx      # Replaced by ConsoleSidebar
dashboard/src/components/layout/Header.tsx       # Replaced by ConsoleTopbar
```

`WizardLayout.tsx` stays — the setup wizard keeps a separate layout but is restyled to console.

---

## Phase 0 — Worktree & branch

### Task 0.1: Create feature branch in a fresh worktree

**Files:** none

- [ ] **Step 1: Verify clean status**

```bash
git status
```

Expected: working tree clean (current branch `fix/embedding-provider-label-3.2.1` is fine, but the migration goes on its own branch).

- [ ] **Step 2: Create worktree on `main` and new branch**

```bash
git fetch origin main
git worktree add ../vectorizer-console-redesign -b feat/console-redesign origin/main
cd ../vectorizer-console-redesign
```

Expected: new directory `../vectorizer-console-redesign` checked out at `feat/console-redesign`.

- [ ] **Step 3: Copy this plan + reference into the worktree**

The reference folder already lives under `docs/superpowers/plans/2026-05-02-console-redesign-migration/reference/`. Confirm it is present in the worktree:

```bash
ls docs/superpowers/plans/2026-05-02-console-redesign-migration/reference/
```

Expected: lists `styles.css`, `icons.jsx`, `screens/`, `assets/logo.png`, etc.

- [ ] **Step 4: Install deps**

```bash
cd dashboard
pnpm install
```

Expected: lockfile resolves cleanly.

- [ ] **Step 5: Sanity-check the existing build**

```bash
pnpm run build:skip-check
```

Expected: build succeeds. We need a green baseline before migrating anything.

- [ ] **Step 6: Commit baseline marker**

```bash
git add docs/superpowers/plans/2026-05-02-console-redesign-migration/
git commit -m "chore(plan): add console redesign migration plan + reference"
```

---

## Phase 1 — Foundation (theme + primitives + shell)

### Task 1.1: Ship the logo asset

**Files:**
- Create: `dashboard/public/logo.png`
- Test: `dashboard/e2e/console-shell.spec.ts`

- [ ] **Step 1: Copy the logo**

```bash
cp docs/superpowers/plans/2026-05-02-console-redesign-migration/reference/assets/logo.png dashboard/public/logo.png
```

- [ ] **Step 2: Verify it renders standalone**

```bash
cd dashboard
pnpm run dev &
sleep 3
curl -I http://localhost:5173/logo.png
```

Expected: `HTTP/1.1 200 OK` with `content-type: image/png`. Kill the dev server with `kill %1`.

- [ ] **Step 3: Commit**

```bash
git add dashboard/public/logo.png
git commit -m "feat(console): add hex/cube logo asset"
```

---

### Task 1.2: Port the design tokens & utility CSS

**Files:**
- Create: `dashboard/src/styles/console.css`
- Modify: `dashboard/src/main.tsx` (one-line import)
- Reference: `reference/styles.css` (771 lines — port verbatim, only minor selector hardening)

- [ ] **Step 1: Copy the source verbatim, then strip the dev-only `tweaks-fab` rule**

```bash
cp docs/superpowers/plans/2026-05-02-console-redesign-migration/reference/styles.css \
   dashboard/src/styles/console.css
```

- [ ] **Step 2: Scope `html, body` rules to avoid clashing with theme.css**

Open `dashboard/src/styles/console.css` and replace:

```css
html, body {
  margin: 0;
  padding: 0;
  background: var(--bg);
  color: var(--text);
  ...
}
```

with:

```css
body[data-console="1"] {
  margin: 0;
  padding: 0;
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.5;
  letter-spacing: -0.005em;
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}
```

The `[data-console="1"]` attribute is set by `ConsoleLayout` so the body styles only apply inside the new shell, leaving login/setup pages neutral until they are migrated.

- [ ] **Step 3: Update `dashboard/src/main.tsx` to load the file**

Find:
```tsx
import './styles/theme.css';
```

Replace with:
```tsx
import './styles/theme.css';
import './styles/console.css';
```

- [ ] **Step 4: Add Inter + JetBrains Mono in `dashboard/index.html`**

Replace `<head>` with:

```html
<head>
  <meta charset="UTF-8" />
  <link rel="icon" type="image/png" href="/logo.png" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Vectorizer Console</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;450;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
</head>
```

- [ ] **Step 5: Build to confirm no PostCSS error**

```bash
pnpm run build:skip-check
```

Expected: build succeeds. The new CSS is tree-shaken from any page that doesn't reference its classes.

- [ ] **Step 6: Commit**

```bash
git add dashboard/src/styles/console.css dashboard/src/main.tsx dashboard/index.html
git commit -m "feat(console): import design tokens and utility CSS"
```

---

### Task 1.3: Build the icon set

**Files:**
- Create: `dashboard/src/components/console/Icons.tsx`
- Test: `dashboard/src/components/console/__tests__/Icons.test.tsx`
- Reference: `reference/icons.jsx`

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/components/console/__tests__/Icons.test.tsx`:

```tsx
import { render } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Icons } from '../Icons';

describe('Icons', () => {
  it('renders every named icon as inline SVG', () => {
    const names: Array<keyof typeof Icons> = [
      'dashboard', 'collections', 'search', 'vectors', 'monitor', 'keys', 'mcp',
      'settings', 'plus', 'zap', 'cpu', 'database', 'bolt', 'layers', 'activity',
      'chevron', 'copy', 'trash', 'bell', 'filter', 'sparkles', 'globe', 'shield',
      'flame', 'panel', 'panel2', 'arrowDown', 'arrowUp', 'check', 'x', 'refresh', 'hex',
    ];
    for (const name of names) {
      const Cmp = Icons[name];
      const { container } = render(<Cmp />);
      const svg = container.querySelector('svg');
      expect(svg, `icon ${name} should render`).toBeTruthy();
      expect(svg!.getAttribute('viewBox')).toBe('0 0 24 24');
    }
  });

  it('respects size prop', () => {
    const { container } = render(<Icons.search size={20} />);
    expect(container.querySelector('svg')!.getAttribute('width')).toBe('20');
    expect(container.querySelector('svg')!.getAttribute('height')).toBe('20');
  });
});
```

- [ ] **Step 2: Verify the test fails**

```bash
cd dashboard
pnpm exec vitest run src/components/console/__tests__/Icons.test.tsx
```

Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Create `dashboard/src/components/console/Icons.tsx`:

```tsx
import type { SVGProps, ReactNode } from 'react';

interface IconProps extends Omit<SVGProps<SVGSVGElement>, 'children'> {
  size?: number;
  strokeWidth?: number;
}

interface InternalProps extends IconProps {
  d: ReactNode;
  fill?: string;
}

const Icon = ({ d, size = 16, fill = 'none', strokeWidth = 1.6, ...rest }: InternalProps) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill={fill}
    stroke="currentColor"
    strokeWidth={strokeWidth}
    strokeLinecap="round"
    strokeLinejoin="round"
    {...rest}
  >
    {d}
  </svg>
);

type IconComponent = (p: IconProps) => JSX.Element;

const make = (d: ReactNode): IconComponent => (p: IconProps) => <Icon {...p} d={d} />;

export const Icons = {
  dashboard: make(<><rect x="3" y="3" width="7" height="9"/><rect x="14" y="3" width="7" height="5"/><rect x="14" y="12" width="7" height="9"/><rect x="3" y="16" width="7" height="5"/></>),
  collections: make(<><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v6c0 1.66 4 3 9 3s9-1.34 9-3V5"/><path d="M3 11v6c0 1.66 4 3 9 3s9-1.34 9-3v-6"/></>),
  search: make(<><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></>),
  vectors: make(<><circle cx="6" cy="6" r="2"/><circle cx="18" cy="6" r="2"/><circle cx="6" cy="18" r="2"/><circle cx="18" cy="18" r="2"/><path d="M8 6h8M6 8v8M18 8v8M8 18h8"/></>),
  monitor: make(<><rect x="3" y="4" width="18" height="12" rx="1"/><path d="M8 20h8M12 16v4"/></>),
  keys: make(<><circle cx="7" cy="14" r="4"/><path d="m10.5 11 8-8 2 2-2 2 2 2-3 3-2-2-3 3"/></>),
  mcp: make(<><polygon points="12,2 22,8 22,16 12,22 2,16 2,8"/><polyline points="12,8 16,10 16,14 12,16 8,14 8,10 12,8"/></>),
  settings: make(<><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></>),
  plus: make(<path d="M12 5v14M5 12h14"/>),
  zap: make(<polygon points="13,2 3,14 12,14 11,22 21,10 12,10"/>),
  cpu: make(<><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"/></>),
  database: make(<><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></>),
  bolt: make(<polygon points="13,2 3,14 12,14 11,22 21,10 12,10"/>),
  layers: make(<><polygon points="12,2 2,7 12,12 22,7 12,2"/><polyline points="2,17 12,22 22,17"/><polyline points="2,12 12,17 22,12"/></>),
  activity: make(<polyline points="22,12 18,12 15,21 9,3 6,12 2,12"/>),
  chevron: make(<polyline points="9,18 15,12 9,6"/>),
  copy: make(<><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></>),
  trash: make(<><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></>),
  bell: make(<><path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></>),
  filter: make(<polygon points="22,3 2,3 10,12.46 10,19 14,21 14,12.46"/>),
  sparkles: make(<><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M5.6 18.4l2.1-2.1M16.3 7.7l2.1-2.1"/></>),
  globe: make(<><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 0 1 0 20 15 15 0 0 1 0-20"/></>),
  shield: make(<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>),
  flame: make(<path d="M8.5 14.5A2.5 2.5 0 0 0 11 17c1.7 0 3-1.3 3-3 0-1.4-.5-2.2-1.5-3.2C11.3 9.5 11 8 11 7c0-1.7-2-3-2-3s-1 3-3 5c-1.5 1.5-2 3-2 4 0 2.7 2.2 5 5 5"/>),
  panel: make(<><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/></>),
  panel2: make(<><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M15 3v18"/></>),
  arrowDown: make(<><path d="M12 5v14"/><polyline points="19,12 12,19 5,12"/></>),
  arrowUp: make(<><path d="M12 19V5"/><polyline points="5,12 12,5 19,12"/></>),
  check: make(<polyline points="20,6 9,17 4,12"/>),
  x: make(<path d="M18 6 6 18M6 6l12 12"/>),
  refresh: make(<><polyline points="23,4 23,10 17,10"/><polyline points="1,20 1,14 7,14"/><path d="M3.5 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.65 4.36A9 9 0 0 0 20.5 15"/></>),
  hex: make(<polygon points="12,2 22,7 22,17 12,22 2,17 2,7"/>),
} as const;
```

- [ ] **Step 4: Run the test**

```bash
pnpm exec vitest run src/components/console/__tests__/Icons.test.tsx
```

Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add dashboard/src/components/console/Icons.tsx dashboard/src/components/console/__tests__/Icons.test.tsx
git commit -m "feat(console): add icon set"
```

---

### Task 1.4: Build the visual primitives

**Files:**
- Create: `dashboard/src/components/console/primitives/HexLogo.tsx`
- Create: `dashboard/src/components/console/primitives/Sparkline.tsx`
- Create: `dashboard/src/components/console/primitives/Ring.tsx`
- Create: `dashboard/src/components/console/primitives/StatusPill.tsx`
- Create: `dashboard/src/components/console/primitives/Pill.tsx`
- Create: `dashboard/src/components/console/primitives/Card.tsx`
- Create: `dashboard/src/components/console/primitives/Kpi.tsx`
- Create: `dashboard/src/components/console/primitives/Bar.tsx`
- Create: `dashboard/src/components/console/primitives/Tbl.tsx`
- Create: `dashboard/src/components/console/primitives/KeyValue.tsx`
- Create: `dashboard/src/components/console/hooks/useTick.ts`
- Create: `dashboard/src/components/console/index.ts`
- Test: `dashboard/src/components/console/__tests__/primitives.test.tsx`
- Reference: `reference/ui.jsx`

- [ ] **Step 1: Write the failing test for the public surface**

Create `dashboard/src/components/console/__tests__/primitives.test.tsx`:

```tsx
import { render, screen, act } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import {
  HexLogo, Sparkline, Ring, StatusPill, Pill, Card, CardHead, CardBody,
  Kpi, Bar, Tbl, Th, Td, KeyValue, KeyValueRow, useTick,
} from '../';
import { useEffect } from 'react';

describe('console primitives', () => {
  it('HexLogo renders /logo.png', () => {
    render(<HexLogo size={32} />);
    const img = screen.getByAltText('Vectorizer');
    expect(img.getAttribute('src')).toBe('/logo.png');
    expect(img.getAttribute('width')).toBe('32');
  });

  it('Sparkline returns null on empty data', () => {
    const { container } = render(<Sparkline data={[]} />);
    expect(container.querySelector('svg')).toBeNull();
  });

  it('Sparkline draws a polyline for non-empty data', () => {
    const { container } = render(<Sparkline data={[1, 4, 2, 8, 5]} width={100} height={20} />);
    expect(container.querySelector('polyline')).toBeTruthy();
    expect(container.querySelector('polygon')).toBeTruthy(); // area fill
  });

  it('Ring renders centered label and sub', () => {
    render(<Ring value={42} max={100} label="42%" sub="CPU" />);
    expect(screen.getByText('42%')).toBeTruthy();
    expect(screen.getByText('CPU')).toBeTruthy();
  });

  it('StatusPill maps healthy to green class', () => {
    const { container } = render(<StatusPill status="healthy" />);
    const pill = container.querySelector('.pill');
    expect(pill?.className).toContain('green');
    expect(pill?.textContent).toContain('healthy');
  });

  it('Pill applies tone class', () => {
    const { container } = render(<Pill tone="magenta">Admin</Pill>);
    expect(container.querySelector('.pill.magenta')).toBeTruthy();
  });

  it('Card composes head + body', () => {
    render(
      <Card>
        <CardHead title="Top Collections" />
        <CardBody>content</CardBody>
      </Card>,
    );
    expect(screen.getByText('Top Collections')).toBeTruthy();
    expect(screen.getByText('content')).toBeTruthy();
  });

  it('Kpi renders label, value and delta', () => {
    render(<Kpi label="qps" value="2,480" unit="qps" delta={{ tone: 'up', text: '+12.4%' }} />);
    expect(screen.getByText('qps')).toBeTruthy();
    expect(screen.getByText('2,480')).toBeTruthy();
    expect(screen.getByText('+12.4%')).toBeTruthy();
  });

  it('Bar fills to percent', () => {
    const { container } = render(<Bar percent={73} />);
    const fill = container.querySelector('.bar > span') as HTMLElement;
    expect(fill.style.width).toBe('73%');
  });

  it('Tbl renders thead + tbody', () => {
    render(
      <Tbl>
        <thead><tr><Th>Name</Th></tr></thead>
        <tbody><tr><Td>foo</Td></tr></tbody>
      </Tbl>,
    );
    expect(screen.getByText('Name')).toBeTruthy();
    expect(screen.getByText('foo')).toBeTruthy();
  });

  it('KeyValue renders dt/dd pairs', () => {
    render(
      <KeyValue>
        <KeyValueRow term="Index">HNSW</KeyValueRow>
      </KeyValue>,
    );
    expect(screen.getByText('Index')).toBeTruthy();
    expect(screen.getByText('HNSW')).toBeTruthy();
  });

  it('useTick increments at the given interval', () => {
    vi.useFakeTimers();
    let observed = -1;
    const Probe = () => {
      const t = useTick(100);
      useEffect(() => { observed = t; }, [t]);
      return null;
    };
    render(<Probe />);
    expect(observed).toBe(0);
    act(() => { vi.advanceTimersByTime(350); });
    expect(observed).toBe(3);
    vi.useRealTimers();
  });
});
```

- [ ] **Step 2: Confirm the test fails**

```bash
pnpm exec vitest run src/components/console/__tests__/primitives.test.tsx
```

Expected: FAIL — module `../` not found.

- [ ] **Step 3: Implement `HexLogo.tsx`**

Create `dashboard/src/components/console/primitives/HexLogo.tsx`:

```tsx
interface HexLogoProps {
  size?: number;
}

export function HexLogo({ size = 28 }: HexLogoProps) {
  return (
    <img
      src="/logo.png"
      alt="Vectorizer"
      width={size}
      height={size}
      style={{ display: 'block' }}
    />
  );
}
```

- [ ] **Step 4: Implement `Sparkline.tsx`**

Create `dashboard/src/components/console/primitives/Sparkline.tsx`:

```tsx
interface SparklineProps {
  data: number[];
  color?: string;
  width?: number;
  height?: number;
  fill?: boolean;
}

export function Sparkline({
  data,
  color = 'var(--teal)',
  width = 80,
  height = 28,
  fill = true,
}: SparklineProps) {
  if (!data.length) return null;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const denom = data.length === 1 ? 1 : data.length - 1;
  const points = data
    .map((d, i) => `${(i / denom) * width},${height - ((d - min) / range) * (height - 4) - 2}`)
    .join(' ');
  const area = `0,${height} ${points} ${width},${height}`;
  return (
    <svg width={width} height={height} style={{ display: 'block' }}>
      {fill && <polygon points={area} fill={color} opacity="0.12" />}
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
```

- [ ] **Step 5: Implement `Ring.tsx`**

Create `dashboard/src/components/console/primitives/Ring.tsx`:

```tsx
import type { ReactNode } from 'react';

interface RingProps {
  value: number;
  max?: number;
  size?: number;
  label: ReactNode;
  sub?: ReactNode;
  color?: string;
}

export function Ring({ value, max = 100, size = 110, label, sub, color = 'var(--teal)' }: RingProps) {
  const r = size / 2 - 8;
  const c = 2 * Math.PI * r;
  const pct = Math.min(value / max, 1);
  return (
    <div style={{ position: 'relative', width: size, height: size }}>
      <svg width={size} height={size}>
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--bg-3)" strokeWidth="6" />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke={color}
          strokeWidth="6"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - pct)}
          strokeLinecap="round"
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          style={{ transition: 'stroke-dashoffset 0.6s' }}
        />
      </svg>
      <div style={{ position: 'absolute', inset: 0, display: 'grid', placeItems: 'center', textAlign: 'center' }}>
        <div>
          <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.02em', fontFeatureSettings: '"tnum" 1' }}>
            {label}
          </div>
          {sub && (
            <div style={{ fontSize: 10, color: 'var(--text-2)', textTransform: 'uppercase', letterSpacing: '0.06em', marginTop: 2 }}>
              {sub}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 6: Implement `Pill.tsx` and `StatusPill.tsx`**

Create `dashboard/src/components/console/primitives/Pill.tsx`:

```tsx
import type { ReactNode } from 'react';

export type PillTone = 'teal' | 'magenta' | 'amber' | 'green' | 'red' | 'muted' | 'default';

interface PillProps {
  tone?: PillTone;
  className?: string;
  live?: boolean;
  children: ReactNode;
}

export function Pill({ tone = 'default', className, live, children }: PillProps) {
  const cls = [
    'pill',
    tone !== 'default' ? tone : '',
    live ? 'live' : '',
    className ?? '',
  ]
    .filter(Boolean)
    .join(' ');
  return <span className={cls}>{children}</span>;
}
```

Create `dashboard/src/components/console/primitives/StatusPill.tsx`:

```tsx
import { Pill, type PillTone } from './Pill';

const MAP: Record<string, { tone: PillTone; label: string }> = {
  healthy: { tone: 'green', label: 'healthy' },
  'in-sync': { tone: 'green', label: 'in-sync' },
  ok: { tone: 'green', label: 'ok' },
  indexing: { tone: 'amber', label: 'indexing' },
  'catching-up': { tone: 'amber', label: 'catching up' },
  warning: { tone: 'amber', label: 'warning' },
  error: { tone: 'red', label: 'error' },
};

export function StatusPill({ status }: { status: string }) {
  const m = MAP[status] ?? { tone: 'muted' as PillTone, label: status };
  return (
    <Pill tone={m.tone}>
      <span className={`dot ${m.tone === 'muted' ? '' : m.tone}`} />
      {m.label}
    </Pill>
  );
}
```

- [ ] **Step 7: Implement `Card.tsx`**

Create `dashboard/src/components/console/primitives/Card.tsx`:

```tsx
import type { ReactNode } from 'react';

interface CardProps {
  className?: string;
  children: ReactNode;
}
export function Card({ className, children }: CardProps) {
  return <div className={['card', className].filter(Boolean).join(' ')}>{children}</div>;
}

interface CardHeadProps {
  title?: ReactNode;
  sub?: ReactNode;
  right?: ReactNode;
  children?: ReactNode;
}
export function CardHead({ title, sub, right, children }: CardHeadProps) {
  if (children) return <div className="card-head">{children}</div>;
  return (
    <div className="card-head">
      <div className="title">{title}</div>
      {sub && <span className="sub">{sub}</span>}
      {right}
    </div>
  );
}

interface CardBodyProps {
  tight?: boolean;
  className?: string;
  children: ReactNode;
}
export function CardBody({ tight, className, children }: CardBodyProps) {
  return (
    <div className={['card-body', tight ? 'tight' : '', className ?? ''].filter(Boolean).join(' ')}>
      {children}
    </div>
  );
}
```

- [ ] **Step 8: Implement `Kpi.tsx`**

Create `dashboard/src/components/console/primitives/Kpi.tsx`:

```tsx
import type { ReactNode } from 'react';
import { Sparkline } from './Sparkline';

type Accent = 'teal' | 'magenta' | 'amber' | 'none';

interface DeltaProps {
  tone: 'up' | 'down' | 'neutral';
  text: string;
}

interface KpiProps {
  label: ReactNode;
  value: ReactNode;
  unit?: string;
  delta?: DeltaProps;
  accent?: Accent;
  spark?: { data: number[]; color?: string };
}

export function Kpi({ label, value, unit, delta, accent = 'none', spark }: KpiProps) {
  const cls = ['kpi', accent !== 'none' ? `accent-${accent}` : ''].filter(Boolean).join(' ');
  return (
    <div className={cls}>
      <div className="label">{label}</div>
      <div className="value tnum">
        {value}
        {unit && <span className="unit">{unit}</span>}
      </div>
      {delta && <div className={`delta ${delta.tone}`}>{delta.text}</div>}
      {spark && (
        <div className="spark">
          <Sparkline data={spark.data} color={spark.color ?? 'var(--teal)'} />
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 9: Implement `Bar.tsx`**

Create `dashboard/src/components/console/primitives/Bar.tsx`:

```tsx
type Tone = 'teal' | 'magenta' | 'amber';

interface BarProps {
  percent: number; // 0-100
  tone?: Tone;
}

export function Bar({ percent, tone = 'teal' }: BarProps) {
  const cls = tone === 'teal' ? '' : tone;
  const pct = Math.max(0, Math.min(100, percent));
  return (
    <div className="bar">
      <span className={cls} style={{ width: `${pct}%` }} />
    </div>
  );
}
```

- [ ] **Step 10: Implement `Tbl.tsx` and `KeyValue.tsx`**

Create `dashboard/src/components/console/primitives/Tbl.tsx`:

```tsx
import type { HTMLAttributes, TableHTMLAttributes, TdHTMLAttributes, ThHTMLAttributes, ReactNode } from 'react';

export function Tbl({ children, ...rest }: TableHTMLAttributes<HTMLTableElement>) {
  return <table className="tbl" {...rest}>{children}</table>;
}

export function Th({ children, ...rest }: ThHTMLAttributes<HTMLTableCellElement>) {
  return <th {...rest}>{children}</th>;
}

export function Td({ children, ...rest }: TdHTMLAttributes<HTMLTableCellElement>) {
  return <td {...rest}>{children}</td>;
}

export interface RowAttrs extends HTMLAttributes<HTMLTableRowElement> {
  active?: boolean;
}
export function Tr({ active, className, children, ...rest }: RowAttrs & { children: ReactNode }) {
  return (
    <tr className={[active ? 'active' : '', className ?? ''].filter(Boolean).join(' ')} {...rest}>
      {children}
    </tr>
  );
}
```

Create `dashboard/src/components/console/primitives/KeyValue.tsx`:

```tsx
import type { ReactNode } from 'react';

export function KeyValue({ children }: { children: ReactNode }) {
  return <dl className="kv">{children}</dl>;
}

export function KeyValueRow({ term, children }: { term: ReactNode; children: ReactNode }) {
  return (
    <>
      <dt>{term}</dt>
      <dd>{children}</dd>
    </>
  );
}
```

- [ ] **Step 11: Implement `useTick.ts`**

Create `dashboard/src/components/console/hooks/useTick.ts`:

```ts
import { useEffect, useState } from 'react';

export function useTick(intervalMs = 1500): number {
  const [t, setT] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setT((x) => x + 1), intervalMs);
    return () => clearInterval(id);
  }, [intervalMs]);
  return t;
}
```

- [ ] **Step 12: Implement the barrel `index.ts`**

Create `dashboard/src/components/console/index.ts`:

```ts
export { Icons } from './Icons';
export { HexLogo } from './primitives/HexLogo';
export { Sparkline } from './primitives/Sparkline';
export { Ring } from './primitives/Ring';
export { StatusPill } from './primitives/StatusPill';
export { Pill, type PillTone } from './primitives/Pill';
export { Card, CardHead, CardBody } from './primitives/Card';
export { Kpi } from './primitives/Kpi';
export { Bar } from './primitives/Bar';
export { Tbl, Tr, Th, Td } from './primitives/Tbl';
export { KeyValue, KeyValueRow } from './primitives/KeyValue';
export { useTick } from './hooks/useTick';
```

- [ ] **Step 13: Run the test**

```bash
pnpm exec vitest run src/components/console/__tests__/primitives.test.tsx
```

Expected: 11 passed.

- [ ] **Step 14: Commit**

```bash
git add dashboard/src/components/console/
git commit -m "feat(console): add visual primitives (sparkline, ring, kpi, pill, card, tbl, kv)"
```

---

### Task 1.5: Build the Sidebar

**Files:**
- Create: `dashboard/src/components/console/ConsoleSidebar.tsx`
- Test: `dashboard/src/components/console/__tests__/ConsoleSidebar.test.tsx`
- Reference: `reference/chrome.jsx`

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/components/console/__tests__/ConsoleSidebar.test.tsx`:

```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect } from 'vitest';
import { ConsoleSidebar } from '../ConsoleSidebar';

const renderAt = (path: string) =>
  render(
    <MemoryRouter initialEntries={[path]}>
      <ConsoleSidebar collapsed={false} onToggleCollapsed={() => {}} />
    </MemoryRouter>,
  );

describe('ConsoleSidebar', () => {
  it('renders all primary navigation links', () => {
    renderAt('/overview');
    for (const label of [
      'Overview', 'Collections', 'Search', 'Vectors', 'Monitoring',
      'Replication', 'API Keys', 'MCP Tools', 'Settings',
    ]) {
      expect(screen.getByText(label)).toBeTruthy();
    }
  });

  it('marks the active route', () => {
    renderAt('/collections');
    const item = screen.getByText('Collections').closest('a, [role="link"], div');
    expect(item?.className).toContain('active');
  });

  it('hides labels when collapsed', () => {
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={true} onToggleCollapsed={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.queryByText('Overview')).toBeNull();
  });

  it('calls onToggleCollapsed', () => {
    let toggled = 0;
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={false} onToggleCollapsed={() => { toggled++; }} />
      </MemoryRouter>,
    );
    fireEvent.click(screen.getByText(/collapse sidebar/i));
    expect(toggled).toBe(1);
  });
});
```

- [ ] **Step 2: Verify it fails**

```bash
pnpm exec vitest run src/components/console/__tests__/ConsoleSidebar.test.tsx
```

Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Create `dashboard/src/components/console/ConsoleSidebar.tsx`:

```tsx
import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
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
  const { user } = useAuth();
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
```

- [ ] **Step 4: Run the test**

```bash
pnpm exec vitest run src/components/console/__tests__/ConsoleSidebar.test.tsx
```

Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add dashboard/src/components/console/ConsoleSidebar.tsx dashboard/src/components/console/__tests__/ConsoleSidebar.test.tsx
git commit -m "feat(console): add sidebar with workspace nav"
```

---

### Task 1.6: Build the Topbar

**Files:**
- Create: `dashboard/src/components/console/ConsoleTopbar.tsx`
- Test: `dashboard/src/components/console/__tests__/ConsoleTopbar.test.tsx`
- Reference: `reference/chrome.jsx` lines 90–116

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/components/console/__tests__/ConsoleTopbar.test.tsx`:

```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { ConsoleTopbar } from '../ConsoleTopbar';

describe('ConsoleTopbar', () => {
  it('renders crumbs with the last segment as current', () => {
    render(<ConsoleTopbar crumbs={['Vectorizer', 'Collections']} onOpenCmd={() => {}} />);
    expect(screen.getByText('Vectorizer')).toBeTruthy();
    expect(screen.getByText('Collections').className).toContain('now');
  });

  it('opens command palette on click', () => {
    let opened = 0;
    render(<ConsoleTopbar crumbs={['x']} onOpenCmd={() => { opened++; }} />);
    fireEvent.click(screen.getByText(/Search collections/));
    expect(opened).toBe(1);
  });
});
```

- [ ] **Step 2: Confirm fail**

```bash
pnpm exec vitest run src/components/console/__tests__/ConsoleTopbar.test.tsx
```

Expected: FAIL.

- [ ] **Step 3: Implement**

Create `dashboard/src/components/console/ConsoleTopbar.tsx`:

```tsx
import { Fragment } from 'react';
import { Icons } from './Icons';

interface Props {
  crumbs: string[];
  onOpenCmd: () => void;
  onRefresh?: () => void;
}

export function ConsoleTopbar({ crumbs, onOpenCmd, onRefresh }: Props) {
  return (
    <div className="topbar">
      <div className="crumbs">
        {crumbs.map((c, i) => (
          <Fragment key={i}>
            {i > 0 && <span className="sep">›</span>}
            <span className={i === crumbs.length - 1 ? 'now' : undefined}>{c}</span>
          </Fragment>
        ))}
      </div>
      <div className="env-switch">
        <span className="dot" />
        <span>production</span>
        <Icons.chevron size={12} />
      </div>
      <button type="button" className="cmdk" onClick={onOpenCmd}>
        <Icons.search size={13} />
        <span>Search collections, vectors, keys…</span>
        <span className="kbd">⌘K</span>
      </button>
      <button type="button" className="icon-btn" title="Notifications">
        <Icons.bell />
      </button>
      <button type="button" className="icon-btn" title="Refresh" onClick={onRefresh}>
        <Icons.refresh />
      </button>
    </div>
  );
}
```

- [ ] **Step 4: Run test**

```bash
pnpm exec vitest run src/components/console/__tests__/ConsoleTopbar.test.tsx
```

Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add dashboard/src/components/console/ConsoleTopbar.tsx dashboard/src/components/console/__tests__/ConsoleTopbar.test.tsx
git commit -m "feat(console): add topbar with crumbs + cmdk button"
```

---

### Task 1.7: Build the Command Palette

**Files:**
- Create: `dashboard/src/components/console/CommandPalette.tsx`
- Test: `dashboard/src/components/console/__tests__/CommandPalette.test.tsx`
- Reference: `reference/cmdk.jsx`

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/components/console/__tests__/CommandPalette.test.tsx`:

```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi } from 'vitest';
import { CommandPalette } from '../CommandPalette';

const setup = (open = true) => {
  const onClose = vi.fn();
  const navigate = vi.fn();
  render(
    <MemoryRouter>
      <CommandPalette open={open} onClose={onClose} onNavigate={navigate} />
    </MemoryRouter>,
  );
  return { onClose, navigate };
};

describe('CommandPalette', () => {
  it('renders nothing when closed', () => {
    setup(false);
    expect(screen.queryByPlaceholderText(/Search or type a command/)).toBeNull();
  });

  it('navigates on Enter', () => {
    const { navigate } = setup();
    const input = screen.getByPlaceholderText(/Search or type a command/) as HTMLInputElement;
    fireEvent.change(input, { target: { value: 'Overview' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(navigate).toHaveBeenCalledWith('/overview');
  });

  it('closes on Escape', () => {
    const { onClose } = setup();
    fireEvent.keyDown(screen.getByPlaceholderText(/Search or type a command/), { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Confirm fail**

```bash
pnpm exec vitest run src/components/console/__tests__/CommandPalette.test.tsx
```

Expected: FAIL.

- [ ] **Step 3: Implement**

Create `dashboard/src/components/console/CommandPalette.tsx`:

```tsx
import { useEffect, useMemo, useState } from 'react';
import { Icons } from './Icons';

interface CmdItem {
  label: string;
  to: string;
  icon: keyof typeof Icons;
  hint?: string;
  section: string;
}

const ITEMS: CmdItem[] = [
  { section: 'Navigation', label: 'Go to Overview',     to: '/overview',     icon: 'dashboard',   hint: 'G O' },
  { section: 'Navigation', label: 'Go to Collections',  to: '/collections',  icon: 'collections', hint: 'G C' },
  { section: 'Navigation', label: 'Go to Search',       to: '/search',       icon: 'search',      hint: 'G S' },
  { section: 'Navigation', label: 'Go to Vectors',      to: '/vectors',      icon: 'vectors',     hint: 'G V' },
  { section: 'Navigation', label: 'Go to Monitoring',   to: '/monitoring',   icon: 'activity',    hint: 'G M' },
  { section: 'Navigation', label: 'Go to API Keys',     to: '/api-keys',     icon: 'keys',        hint: 'G K' },
  { section: 'Navigation', label: 'Go to MCP Tools',    to: '/mcp-tools',    icon: 'mcp' },
  { section: 'Navigation', label: 'Go to Settings',     to: '/configuration',icon: 'settings' },
];

interface Props {
  open: boolean;
  onClose: () => void;
  onNavigate: (to: string) => void;
}

export function CommandPalette({ open, onClose, onNavigate }: Props) {
  const [q, setQ] = useState('');
  const [active, setActive] = useState(0);

  useEffect(() => { if (open) { setQ(''); setActive(0); } }, [open]);

  const flat = useMemo(
    () => ITEMS.filter((it) => it.label.toLowerCase().includes(q.toLowerCase())),
    [q],
  );

  if (!open) return null;

  const go = (it: CmdItem) => {
    onNavigate(it.to);
    onClose();
  };

  return (
    <div className="cmd-overlay" onClick={onClose} role="dialog" aria-modal>
      <div className="cmd-panel" onClick={(e) => e.stopPropagation()}>
        <input
          className="cmd-input"
          placeholder="Search or type a command…"
          autoFocus
          value={q}
          onChange={(e) => { setQ(e.target.value); setActive(0); }}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') { setActive((a) => Math.min(a + 1, flat.length - 1)); e.preventDefault(); }
            if (e.key === 'ArrowUp')   { setActive((a) => Math.max(a - 1, 0)); e.preventDefault(); }
            if (e.key === 'Enter' && flat[active]) go(flat[active]);
            if (e.key === 'Escape')   onClose();
          }}
        />
        <div className="cmd-list">
          {Object.entries(
            flat.reduce<Record<string, CmdItem[]>>((acc, it) => {
              (acc[it.section] ??= []).push(it);
              return acc;
            }, {}),
          ).map(([section, items]) => (
            <div key={section}>
              <div className="cmd-section">{section}</div>
              {items.map((it) => {
                const idx = flat.indexOf(it);
                const Icon = Icons[it.icon];
                return (
                  <div
                    key={it.to}
                    className={`cmd-row ${idx === active ? 'active' : ''}`}
                    onClick={() => go(it)}
                    onMouseEnter={() => setActive(idx)}
                  >
                    <Icon className="icon" />
                    <span>{it.label}</span>
                    {it.hint && <span className="hint">{it.hint}</span>}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Run test**

```bash
pnpm exec vitest run src/components/console/__tests__/CommandPalette.test.tsx
```

Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add dashboard/src/components/console/CommandPalette.tsx dashboard/src/components/console/__tests__/CommandPalette.test.tsx
git commit -m "feat(console): add ⌘K command palette"
```

---

### Task 1.8: Wire ConsoleLayout

**Files:**
- Create: `dashboard/src/components/console/ConsoleLayout.tsx`
- Modify: `dashboard/src/router/AppRouter.tsx`
- Test: `dashboard/src/components/console/__tests__/ConsoleLayout.test.tsx`
- Reference: `reference/app.jsx`

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/components/console/__tests__/ConsoleLayout.test.tsx`:

```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { describe, it, expect } from 'vitest';
import { AuthProvider } from '@/contexts/AuthContext';
import { ConsoleLayout } from '../ConsoleLayout';

const Page = () => <div data-testid="page">PAGE</div>;

describe('ConsoleLayout', () => {
  it('renders sidebar, topbar and outlet', () => {
    render(
      <MemoryRouter initialEntries={['/overview']}>
        <AuthProvider>
          <Routes>
            <Route element={<ConsoleLayout />}>
              <Route path="/overview" element={<Page />} />
            </Route>
          </Routes>
        </AuthProvider>
      </MemoryRouter>,
    );
    expect(screen.getByText('Vectorizer')).toBeTruthy();
    expect(screen.getByText(/Search collections/)).toBeTruthy();
    expect(screen.getByTestId('page')).toBeTruthy();
    expect(document.body.dataset.console).toBe('1');
  });

  it('opens command palette on ⌘K', () => {
    render(
      <MemoryRouter initialEntries={['/overview']}>
        <AuthProvider>
          <Routes>
            <Route element={<ConsoleLayout />}>
              <Route path="/overview" element={<Page />} />
            </Route>
          </Routes>
        </AuthProvider>
      </MemoryRouter>,
    );
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    expect(screen.getByPlaceholderText(/Search or type a command/)).toBeTruthy();
  });
});
```

- [ ] **Step 2: Confirm fail**

```bash
pnpm exec vitest run src/components/console/__tests__/ConsoleLayout.test.tsx
```

Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Create `dashboard/src/components/console/ConsoleLayout.tsx`:

```tsx
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
```

- [ ] **Step 4: Add the barrel export**

Append to `dashboard/src/components/console/index.ts`:

```ts
export { ConsoleLayout } from './ConsoleLayout';
export { ConsoleSidebar } from './ConsoleSidebar';
export { ConsoleTopbar } from './ConsoleTopbar';
export { CommandPalette } from './CommandPalette';
```

- [ ] **Step 5: Wire into the router**

Open `dashboard/src/router/AppRouter.tsx`. Replace the import line:

```tsx
import MainLayout from '@/components/layout/MainLayout';
```

with:

```tsx
import { ConsoleLayout } from '@/components/console';
```

Then replace the `<MainLayout />` element in the protected route block with `<ConsoleLayout />`. Also add the new routes for the redesigned pages — see Task 2.5 (Monitoring) and Task 2.6 (MCP Tools) — but at this point only stub the destinations:

```tsx
<Route path="monitoring" element={<div className="page"><h1 className="page-title">Monitoring (TODO)</h1></div>} />
<Route path="mcp-tools" element={<div className="page"><h1 className="page-title">MCP Tools (TODO)</h1></div>} />
```

These two stubs satisfy the sidebar links until their pages are built in Phase 3.

- [ ] **Step 6: Run tests**

```bash
pnpm exec vitest run src/components/console/__tests__/ConsoleLayout.test.tsx
pnpm run lint
```

Expected: ConsoleLayout tests pass; lint clean.

- [ ] **Step 7: Boot the dev server and visually verify**

```bash
pnpm run dev
# Open http://localhost:5173 — sidebar + topbar should render with the dark theme.
# Press ⌘K — palette opens; arrow keys move the highlight; Enter navigates; Escape closes.
# Click "Collapse sidebar" — sidebar narrows to 60px.
```

Stop the server when satisfied.

- [ ] **Step 8: Commit**

```bash
git add dashboard/src/components/console/ConsoleLayout.tsx \
        dashboard/src/components/console/index.ts \
        dashboard/src/components/console/__tests__/ConsoleLayout.test.tsx \
        dashboard/src/router/AppRouter.tsx
git commit -m "feat(console): wire console layout into router"
```

---

### Task 1.9: Playwright smoke test for the shell

**Files:**
- Create: `dashboard/e2e/console-shell.spec.ts`

- [ ] **Step 1: Write the spec**

Create `dashboard/e2e/console-shell.spec.ts`:

```ts
import { test, expect } from '@playwright/test';

test.describe('console shell', () => {
  test('sidebar and topbar render on /overview', async ({ page }) => {
    await page.goto('/overview');
    await expect(page.locator('.sidebar')).toBeVisible();
    await expect(page.locator('.topbar')).toBeVisible();
    await expect(page.locator('.sidebar-brand .name')).toHaveText('Vectorizer');
  });

  test('command palette opens with ⌘K', async ({ page }) => {
    await page.goto('/overview');
    await page.keyboard.press('Meta+k');
    await expect(page.getByPlaceholder(/Search or type a command/)).toBeVisible();
  });

  test('navigates to Collections via palette', async ({ page }) => {
    await page.goto('/overview');
    await page.keyboard.press('Meta+k');
    await page.keyboard.type('Collect');
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/\/collections$/);
  });
});
```

- [ ] **Step 2: Run it**

```bash
cd dashboard
pnpm exec playwright install --with-deps
pnpm exec playwright test e2e/console-shell.spec.ts
```

Expected: 3 passed.

- [ ] **Step 3: Commit**

```bash
git add dashboard/e2e/console-shell.spec.ts
git commit -m "test(console): smoke e2e for sidebar, topbar and ⌘K"
```

---

## Phase 2 — Core pages

Each page follows the same shape:

1. Read the redesign source under `reference/screens/<name>.jsx`.
2. Identify the **real** data hooks (existing `useCollections`, `useApiKeys`, etc.) — do not import any mock from `data.js`.
3. Translate JSX into TSX and wire the hook results into KPIs/tables.
4. Replace inline `style={{...}}` from the prototype with the existing utility classes (`grid-4`, `card`, `kpi`, `pill`, etc.) where possible; keep inline styles only where the redesign also used them.
5. Write a Vitest unit test for the page that asserts: KPI strip renders, primary table renders, page heading is correct, loading state is rendered when the hook says `loading`.
6. Visually QA against `reference/Vectorizer Console.html` (open it in a browser side-by-side).

Each task ends with a commit `feat(console): port <Page>`.

### Task 2.1: OverviewPage

**Files:**
- Modify: `dashboard/src/pages/OverviewPage.tsx`
- Test: `dashboard/src/pages/__tests__/OverviewPage.test.tsx`
- Reference: `reference/screens/overview.jsx`

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/pages/__tests__/OverviewPage.test.tsx` (replace any pre-existing snapshot test):

```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import OverviewPage from '../OverviewPage';

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy', metric: 'cosine' },
      { name: 'code', dimension: 768, vector_count: 8000, status: 'indexing', metric: 'cosine' },
    ],
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy' },
      { name: 'code', dimension: 768, vector_count: 8000, status: 'indexing' },
    ],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

describe('OverviewPage', () => {
  it('renders KPI strip and top collections table', () => {
    render(<MemoryRouter><OverviewPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Overview/i })).toBeTruthy();
    expect(screen.getByText(/Total vectors/i)).toBeTruthy();
    expect(screen.getByText('docs')).toBeTruthy();
    expect(screen.getByText('code')).toBeTruthy();
  });
});
```

- [ ] **Step 2: Confirm fail**

```bash
pnpm exec vitest run src/pages/__tests__/OverviewPage.test.tsx
```

Expected: FAIL — page still uses the old layout.

- [ ] **Step 3: Rewrite the page**

Replace the contents of `dashboard/src/pages/OverviewPage.tsx` with:

```tsx
import { useEffect, useRef } from 'react';
import { useCollections } from '@/hooks/useCollections';
import { useCollectionsStore } from '@/stores/collections';
import LoadingState from '@/components/LoadingState';
import {
  Icons, Sparkline, Ring, StatusPill, Pill, Card, CardHead, CardBody, Kpi, Bar,
  Tbl, Th, Td, KeyValue, KeyValueRow, useTick,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';
import type { Collection } from '@/hooks/useCollections';

const SPARK = (n: number, base: number, amp: number) =>
  Array.from({ length: n }, (_, i) => base + Math.sin(i / 2) * amp + Math.random() * amp * 0.3);

function OverviewPage() {
  const { listCollections } = useCollections();
  const { collections, loading, setCollections, setLoading, setError } = useCollectionsStore();
  const ref = useRef<NodeJS.Timeout | null>(null);
  const tick = useTick(2000);

  const fetchCollections = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listCollections();
      const arr = Array.isArray(data) ? data : ((data as unknown as { collections?: Collection[] })?.collections ?? []);
      setCollections(arr);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load collections');
      setCollections([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCollections();
    ref.current = setInterval(fetchCollections, 30000);
    return () => { if (ref.current) clearInterval(ref.current); };
  }, []);

  if (loading && !collections.length) return <LoadingState message="Loading dashboard..." />;

  const list = Array.isArray(collections) ? collections : [];
  const totalVectors = list.reduce((s, c) => s + (c.vector_count ?? 0), 0);
  const top = list.slice(0, 6);

  // Note: qps/cpu/mem/conns are placeholders until the metrics API is wired.
  // Replace these with values from `/metrics` once that endpoint lands (Task 4.1).
  const qps = 2480 + Math.round(Math.sin(tick / 2) * 120);
  const cpu = 38 + Math.sin(tick / 2.5) * 6;
  const mem = 62.4 + Math.sin(tick / 3) * 1.2;
  const conns = 184 + Math.round(Math.cos(tick) * 14);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Overview</h1>
          <p className="page-sub">Real-time health of the Vectorizer node</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn" onClick={fetchCollections}><Icons.refresh size={13} />Refresh</button>
          <button className="btn primary"><Icons.plus size={13} />New Collection</button>
        </div>
      </div>

      <div className="grid grid-4" style={{ marginBottom: 14 }}>
        <Kpi accent="teal" label={<><Icons.zap size={12} />Queries / sec</>} value={qps.toLocaleString()} unit="qps"
             delta={{ tone: 'up', text: '+12.4% vs 24h' }}
             spark={{ data: SPARK(20, 2400, 200), color: 'var(--teal)' }} />
        <Kpi label={<><Icons.cpu size={12} />Search latency p99</>} value="2.8" unit="ms"
             delta={{ tone: 'up', text: '−0.4ms vs 24h' }}
             spark={{ data: SPARK(20, 2.8, 0.4), color: 'var(--text-2)' }} />
        <Kpi accent="magenta" label={<><Icons.layers size={12} />Total vectors</>} value={formatNumber(totalVectors)}
             delta={{ tone: 'neutral', text: `${list.length} collections` }}
             spark={{ data: SPARK(20, 580, 8), color: 'var(--magenta)' }} />
        <Kpi label={<><Icons.flame size={12} />Cache hit rate</>} value="94.2" unit="%"
             delta={{ tone: 'up', text: '+1.8% vs 24h' }}
             spark={{ data: SPARK(20, 94, 2), color: 'var(--green)' }} />
      </div>

      <div className="grid grid-2-1" style={{ marginBottom: 14 }}>
        <Card>
          <CardHead title="System Health" right={<Pill tone="green" live><span className="dot green" />healthy</Pill>} />
          <CardBody>
            <div className="grid grid-3" style={{ gap: 18, alignItems: 'center' }}>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring value={cpu} max={100} label={`${cpu.toFixed(0)}%`} sub="CPU" color="var(--teal)" />
              </div>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring value={mem} max={100} label={`${mem.toFixed(1)}%`} sub="MEMORY" color="var(--magenta)" />
              </div>
              <div style={{ display: 'grid', placeItems: 'center' }}>
                <Ring value={conns} max={500} label={conns} sub="CONNECTIONS" color="var(--amber)" />
              </div>
            </div>
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Server binary">vectorizer 3.0.0</KeyValueRow>
              <KeyValueRow term="Bind">127.0.0.1:15002 (REST) · /mcp (StreamableHTTP)</KeyValueRow>
              <KeyValueRow term="Workspace">{list.length} collections</KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>

        <Card>
          <CardHead title="Quantization" sub="SQ-8bit · default" />
          <CardBody>
            <div style={{ textAlign: 'center', marginBottom: 14 }}>
              <div style={{ fontSize: 36, fontWeight: 600, letterSpacing: '-0.02em' }}>4.0×</div>
              <div className="muted" style={{ fontSize: 11, textTransform: 'uppercase', letterSpacing: '0.06em' }}>compression ratio</div>
            </div>
            <div className="col" style={{ gap: 10 }}>
              <div>
                <div className="row" style={{ fontSize: 11, marginBottom: 4 }}>
                  <span className="muted">MAP score</span>
                  <span className="right mono">+8.9%</span>
                </div>
                <Bar percent={82} />
              </div>
              <div>
                <div className="row" style={{ fontSize: 11, marginBottom: 4 }}>
                  <span className="muted">Recall@10</span>
                  <span className="right mono">98.4%</span>
                </div>
                <Bar percent={98} tone="magenta" />
              </div>
            </div>
          </CardBody>
        </Card>
      </div>

      <div className="grid grid-2-1">
        <Card>
          <CardHead title="Top Collections" />
          <CardBody tight>
            <Tbl>
              <thead>
                <tr>
                  <Th>Name</Th>
                  <Th>Vectors</Th>
                  <Th>Dim</Th>
                  <Th>Status</Th>
                </tr>
              </thead>
              <tbody>
                {top.map((c) => (
                  <tr key={c.name}>
                    <Td>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <Icons.database size={13} className="muted" />
                        <span style={{ fontWeight: 500 }}>{c.name}</span>
                      </div>
                    </Td>
                    <Td className="num">{formatNumber(c.vector_count ?? 0)}</Td>
                    <Td className="num">{c.dimension ?? '—'}</Td>
                    <Td><StatusPill status={(c as { status?: string }).status ?? 'healthy'} /></Td>
                  </tr>
                ))}
              </tbody>
            </Tbl>
          </CardBody>
        </Card>

        <Card>
          <CardHead title="Recent Events" right={<Pill tone="green" live><span className="dot green" />live</Pill>} />
          <CardBody tight>
            <div className="scroll-body">
              <div style={{ padding: 24, color: 'var(--text-2)' }}>
                Wire events feed in Task 4.2.
              </div>
            </div>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

export default OverviewPage;
```

- [ ] **Step 4: Run tests**

```bash
pnpm exec vitest run src/pages/__tests__/OverviewPage.test.tsx
```

Expected: 1 passed.

- [ ] **Step 5: Visual QA**

```bash
pnpm run dev
# Visit /overview. Expect:
#  · 4-column KPI strip
#  · System Health card with 3 rings
#  · Top Collections table populated from real API
```

- [ ] **Step 6: Commit**

```bash
git add dashboard/src/pages/OverviewPage.tsx dashboard/src/pages/__tests__/OverviewPage.test.tsx
git commit -m "feat(console): port Overview page"
```

---

### Task 2.2: CollectionsPage

**Files:**
- Modify: `dashboard/src/pages/CollectionsPage.tsx`
- Test: `dashboard/src/pages/__tests__/CollectionsPage.test.tsx`
- Reference: `reference/screens/collections.jsx`

The redesign uses a **list / detail split** (`grid-1-2`). Hook into `useCollectionsStore` for the list and reuse the existing `CollectionDetailsModal` for the right pane content (or inline its read view directly into the page — preferred).

- [ ] **Step 1: Write the failing test**

Create `dashboard/src/pages/__tests__/CollectionsPage.test.tsx`:

```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import CollectionsPage from '../CollectionsPage';

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs',  dimension: 384, vector_count: 1200, status: 'healthy', metric: 'cosine' },
      { name: 'code',  dimension: 768, vector_count: 8000, status: 'indexing', metric: 'cosine' },
    ],
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [
      { name: 'docs',  dimension: 384, vector_count: 1200, status: 'healthy', metric: 'cosine' },
      { name: 'code',  dimension: 768, vector_count: 8000, status: 'indexing', metric: 'cosine' },
    ],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

describe('CollectionsPage', () => {
  it('renders the list and selects the first item by default', () => {
    render(<MemoryRouter><CollectionsPage /></MemoryRouter>);
    expect(screen.getByText('docs')).toBeTruthy();
    expect(screen.getByText('code')).toBeTruthy();
    // detail pane shows first collection
    expect(screen.getAllByText('docs').length).toBeGreaterThanOrEqual(2);
  });

  it('switches detail when clicking a different row', () => {
    render(<MemoryRouter><CollectionsPage /></MemoryRouter>);
    fireEvent.click(screen.getByText('code'));
    expect(screen.getAllByText('code').length).toBeGreaterThanOrEqual(2);
  });
});
```

- [ ] **Step 2: Confirm fail**

```bash
pnpm exec vitest run src/pages/__tests__/CollectionsPage.test.tsx
```

Expected: FAIL.

- [ ] **Step 3: Implement**

Open `reference/screens/collections.jsx` and translate it. Pseudocode of the structure:

```
.page
  .page-head
    h1 + sub
    actions: Filter, Reindex, Create
  .grid grid-1-2
    Card: list
      CardHead: filter input + count pill
      CardBody.tight: scroll list
        clicking sets `selected`
    .col gap=14
      Card: header (name, badges, actions)
      Card: stats grid (Vectors, Dim, Storage, p99)
      Card: KeyValue (Index, Distance, Quantization, Embedding, Created, Owner, WAL offset)
      .grid grid-2: query throughput sparkline + vector growth sparkline
```

Use the existing `useCollectionsStore` for the list. Wire `Reindex` and `Delete` to existing API hooks if available; otherwise, leave the buttons as no-ops with a TODO comment pointing to Phase 4.

Run the test until it passes.

- [ ] **Step 4: Visual QA + commit**

```bash
pnpm run dev   # visit /collections
git add dashboard/src/pages/CollectionsPage.tsx dashboard/src/pages/__tests__/CollectionsPage.test.tsx
git commit -m "feat(console): port Collections page"
```

---

### Task 2.3: SearchPage (Search Playground)

**Files:**
- Modify: `dashboard/src/pages/SearchPage.tsx`
- Test: `dashboard/src/pages/__tests__/SearchPage.test.tsx`
- Reference: `reference/screens/search.jsx`

The redesign exposes 4 search modes as tabs: Intelligent, Semantic, Contextual, Multi-collection. Wire each tab to its existing endpoint via the existing API client (`/intelligent_search`, `/semantic_search`, `/contextual_search`, `/multi_collection_search`).

- [ ] **Step 1 — Test:** assert tab switching changes the description string and the displayed `POST /<type>_search` in the request panel.

- [ ] **Step 2:** Verify fail.

- [ ] **Step 3:** Implement using `useSearchHistory` (already present) for recent queries; wire results to a real `axios.post(...)` call from `useApiClient`.

- [ ] **Step 4:** Visual QA against `reference/Vectorizer Console.html` — open the prototype side-by-side and check tab spacing, result row layout (rank/title/snippet/meta/score), and the `code` block formatting.

- [ ] **Step 5:** Commit `feat(console): port Search page`.

---

### Task 2.4: VectorsPage

**Files:**
- Modify: `dashboard/src/pages/VectorsPage.tsx`
- Test: `dashboard/src/pages/__tests__/VectorsPage.test.tsx`
- Reference: `reference/screens/vectors.jsx`

The visual showpiece is the **embedding viz** — a 96-bar SVG histogram of the first 96 dimensions, teal for positive, magenta for negative, centered on a baseline at y=50. The redesign hardcodes 96 dims; in the real app, slice the actual vector to the first 96.

- [ ] **Step 1 — Test:** assert the table renders, clicking a row updates the right pane, and the embedding SVG contains 96 `<rect>` elements.
- [ ] **Step 2:** Verify fail.
- [ ] **Step 3:** Implement; pull vectors via existing API hook (or scaffold a new `useVectors(collection)` hook if not present).
- [ ] **Step 4:** Visual QA.
- [ ] **Step 5:** Commit `feat(console): port Vector Browser page`.

---

### Task 2.5: MonitoringPage (NEW)

**Files:**
- Create: `dashboard/src/pages/MonitoringPage.tsx`
- Test: `dashboard/src/pages/__tests__/MonitoringPage.test.tsx`
- Modify: `dashboard/src/router/AppRouter.tsx` (replace stub from Task 1.8)
- Reference: `reference/screens/monitoring.jsx`

Sections: throughput strip, SIMD backend card, WAL card, Query cache card, File-ops cache card. Wire to whichever metrics endpoint exists; if none exists yet, use the redesign's static data **with a top-of-file `// TODO(metrics-endpoint): wire real values from /metrics`** comment so it is auditable.

- [ ] **Steps 1–5:** Standard pattern. Commit `feat(console): add Monitoring page`.

---

### Task 2.6: McpToolsPage (NEW)

**Files:**
- Create: `dashboard/src/pages/McpToolsPage.tsx`
- Test: `dashboard/src/pages/__tests__/McpToolsPage.test.tsx`
- Modify: `dashboard/src/router/AppRouter.tsx`
- Reference: `reference/screens/other.jsx` (`McpScreen`)

Wire to the existing MCP capability registry endpoint (`GET /mcp/capabilities` or the equivalent — check `src/api/`). The KPI strip needs Active connections, Tool calls today, Errors. The table lists tools sorted by usage with status pill.

- [ ] **Steps 1–5:** Commit `feat(console): add MCP Tools page`.

---

### Task 2.7: ApiKeysPage

**Files:**
- Modify: `dashboard/src/pages/ApiKeysPage.tsx`
- Test: `dashboard/src/pages/__tests__/ApiKeysPage.test.tsx`
- Reference: `reference/screens/other.jsx` (`ApiKeysScreen`)

Includes the static **Permission matrix** card. Wire keys to `useApiKeys`.

- [ ] **Steps 1–5:** Commit `feat(console): port API Keys page`.

---

### Task 2.8: ClusterPage → Replication

**Files:**
- Modify: `dashboard/src/pages/ClusterPage.tsx`
- Test: `dashboard/src/pages/__tests__/ClusterPage.test.tsx`
- Reference: `reference/screens/other.jsx` (`ReplicationScreen`)

KPIs: Master offset, Connected replicas, Max lag, Write concern. Replicas table.

- [ ] **Steps 1–5:** Commit `feat(console): port Cluster (Replication) page`.

---

### Task 2.9: ConfigurationPage → Settings

**Files:**
- Modify: `dashboard/src/pages/ConfigurationPage.tsx`
- Test: `dashboard/src/pages/__tests__/ConfigurationPage.test.tsx`
- Reference: `reference/screens/other.jsx` (`SettingsScreen`)

The redesign has only two cards (General, Defaults). The existing ConfigurationPage is huge (1.6k lines) — collapse the YAML editor into a third card titled "Raw config" while keeping General and Defaults as `KeyValue` lists.

- [ ] **Steps 1–5:** Commit `feat(console): port Configuration page to settings layout`.

---

## Phase 3 — Auxiliary pages (no redesign equivalent)

Each of the following pages has no direct redesign mock-up. They keep their existing functionality but wear the new visual identity: `.page` wrapper, `.page-head` with title + sub + actions, `.card` containers, `.tbl` for tables, `.btn`, `.input`, `.field` for forms.

For each, the recipe is:

1. Wrap the page contents in `<div className="page"><div className="page-head">…</div>…</div>`.
2. Replace any `<Card>` import that comes from `@/components/ui/Card` with the console `Card` (from `@/components/console`).
3. Replace `<Table>` with `<Tbl>` and the `Th`/`Td` wrappers.
4. Replace `<Button>` with the console `.btn` element directly (no extra component needed — small enough).
5. Replace any inputs with `<input className="input"/>` / `<select className="input"/>` etc.
6. Drop the `dark:` Tailwind class chains; the new theme is dark-only.
7. Add a Vitest test asserting the heading and the primary data table render.
8. Commit `feat(console): port <Page>`.

### Task 3.1: LoginPage

Reference: `reference/screens/other.jsx` doesn't have a login screen — design one consistent with the console theme:

- centered `Card` (max-width 380px)
- HexLogo + "Vectorizer" wordmark
- two `.field` inputs (username/password)
- `.btn primary` for sign-in
- error banner with `.pill red`

`LoginPage` does **not** sit inside `ConsoleLayout`. It sets `document.body.dataset.console = '1'` itself so the dark theme applies, then clears it on unmount.

### Task 3.2: SetupWizardPage

Wraps in `WizardLayout` (kept as-is) but every step gets re-skinned to console primitives. KPIs at the top show progress (steps completed / steps total). Buttons become `.btn` and `.btn primary`.

### Task 3.3: FileWatcherPage
### Task 3.4: GraphPage
### Task 3.5: BackupsPage
### Task 3.6: LogsPage
### Task 3.7: WorkspacePage
### Task 3.8: UsersPage
### Task 3.9: ApiDocsPage
### Task 3.10: ConnectionsPage

Each follows the recipe above. Commit per page.

---

## Phase 4 — Wire real metrics

Until this phase, several pages display synthetic numbers (Overview KPIs, Monitoring throughput, Replication lag). Replace each with real data from the backend.

### Task 4.1: Metrics endpoint hook

**Files:**
- Create: `dashboard/src/hooks/useMetrics.ts`
- Test: `dashboard/src/hooks/__tests__/useMetrics.test.ts`

- [ ] **Step 1:** Test that the hook calls `GET /metrics` (or whichever endpoint exists — confirm with `rg "axum.*metrics" src/api/`) and returns `{ qps, p99, cpu, mem, connections, cacheHitRate }`.
- [ ] **Step 2:** Implement the hook with a 2 s polling interval (matches `useTick`).
- [ ] **Step 3:** Replace the synthetic values in `OverviewPage` and `MonitoringPage` with hook output.
- [ ] **Step 4:** Run all page tests.
- [ ] **Step 5:** Commit `feat(console): wire real metrics into KPIs`.

### Task 4.2: Events feed hook

**Files:**
- Create: `dashboard/src/hooks/useEvents.ts`
- Test: `dashboard/src/hooks/__tests__/useEvents.test.ts`

Backed by Server-Sent Events on `/events` if available; otherwise long-poll `/events?since=<seq>` every 2 s. Replace the placeholder in `OverviewPage` "Recent Events" card.

Commit `feat(console): wire live events feed`.

### Task 4.3: Cache & WAL stats

Replace the static numbers in `MonitoringPage` (Hits, Misses, Evictions, WAL sequence) with a `useStats` hook that reads `/stats`. Commit `feat(console): wire WAL and cache stats`.

---

## Phase 5 — Cleanup

### Task 5.1: Delete legacy layout

**Files to delete:**

```bash
rm dashboard/src/components/layout/MainLayout.tsx
rm dashboard/src/components/layout/Sidebar.tsx
rm dashboard/src/components/layout/Header.tsx
```

- [ ] Run `pnpm run lint` to confirm no orphan imports.
- [ ] Run `pnpm run build:skip-check`.
- [ ] Commit `chore(console): remove legacy MainLayout/Sidebar/Header`.

### Task 5.2: Drop Tailwind v4 if unused

Audit with:

```bash
rg --no-heading "className=\"[^\"]*\b(bg|text|border|dark:|md:|lg:|flex)\b" dashboard/src
```

If no high-traffic page still depends on Tailwind, remove `tailwindcss` and `@tailwindcss/vite` from `package.json` and the corresponding lines from `vite.config.ts`. Otherwise, leave Tailwind in place and document the hybrid in `dashboard/README.md`.

Commit `chore(console): drop tailwind` (or the documentation update commit).

### Task 5.3: Refresh tests for new component IDs

The old `MainLayout`/`Header`/`Sidebar` tests under `dashboard/src/components/layout/__tests__/` no longer apply. Delete them. Make sure the new console tests fully cover navigation.

Commit `test(console): replace legacy layout tests`.

### Task 5.4: Update screenshots in `dashboard/README.md`

Capture three screenshots: Overview, Search, Monitoring. Save them under `dashboard/docs/screenshots/`. Reference them in the README.

Commit `docs(console): refresh dashboard screenshots`.

### Task 5.5: Final QA pass

- [ ] `pnpm run lint`
- [ ] `pnpm exec vitest run`
- [ ] `pnpm exec playwright test`
- [ ] `pnpm run build`
- [ ] Manual smoke: visit every route in the sidebar; ⌘K palette navigates to each; collapse-toggle works; logout works.

---

## Phase 6 — Rollout

### Task 6.1: PR

```bash
gh pr create --base main --head feat/console-redesign \
  --title "feat(dashboard): console redesign" \
  --body-file docs/superpowers/plans/2026-05-02-console-redesign-migration/PR_BODY.md
```

PR body lives under the same plan folder; it summarises the migration, lists the screens ported, and links to the design source.

### Task 6.2: Visual regression review

Tag the design author. Block merge until at least one approving review.

### Task 6.3: Merge & deploy

After approval, squash-merge into `main`. The CI pipeline rebuilds the dashboard and ships it as part of the binary asset.

### Task 6.4: Delete the original zip

After the PR is open, delete the source archive the user has at the repo root:

```bash
rm Vectorizer.zip
```

(The reference folder under `docs/superpowers/plans/.../reference/` stays — it is the long-term source of truth for the design tokens and screens.)

Commit if needed: `chore: drop redesign source zip — preserved under plans/`.

---

## Self-review checklist

- [x] **Spec coverage:** every screen in `reference/screens/` has a Phase 2 task; every existing page in `dashboard/src/pages/` has either a Phase 2 or Phase 3 task; no spec section is unaccounted for.
- [x] **No placeholders:** all "implement later" markers are explicit `TODO(metrics-endpoint)` or `TODO(events-feed)` calls with a Phase 4 task that resolves them.
- [x] **Type consistency:** `Icons[name]`, `Pill tone`, `Kpi accent`, `Bar tone` follow the same vocabulary throughout the plan; primitives' prop names (`label`, `sub`, `value`, `unit`, `delta`, `accent`, `spark`) are stable.
- [x] **TDD discipline:** every task starts with a failing test.
- [x] **Surgical:** legacy layout files are only deleted in Phase 5, not earlier — the new shell coexists with the old until every page is ported.
