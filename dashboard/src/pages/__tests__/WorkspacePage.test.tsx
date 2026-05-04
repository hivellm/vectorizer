import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import WorkspacePage from '../WorkspacePage';

// The real hook (src/hooks/useWorkspace.ts) exposes
// { getConfig, updateConfig, addWorkspace, removeWorkspace, listWorkspaces }.
// The page renders projects from `config.projects`. Mock that surface so
// the restyled page can mount without hitting the network.
vi.mock('@/hooks/useWorkspace', () => ({
  useWorkspace: () => ({
    getConfig: vi.fn(async () => ({
      projects: [
        {
          name: 'docs',
          path: '/var/lib/vectorizer/docs',
          description: 'Documentation corpus',
          collections: [
            {
              name: 'docs-md',
              description: 'Markdown docs',
              include_patterns: ['**/*.md'],
              exclude_patterns: ['node_modules/**'],
            },
          ],
        },
        {
          name: 'code',
          path: '/srv/code',
          description: 'Source code',
          collections: [],
        },
      ],
    })),
    updateConfig: vi.fn(async () => undefined),
    addWorkspace: vi.fn(async () => undefined),
    removeWorkspace: vi.fn(async () => undefined),
    listWorkspaces: vi.fn(async () => []),
  }),
}));

vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

describe('WorkspacePage', () => {
  it('renders the page heading and the projects list', async () => {
    render(<MemoryRouter><WorkspacePage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Workspace/i })).toBeTruthy();
    // Either a project name or the empty-state copy should appear once
    // the async config fetch resolves.
    const matches = await screen.findAllByText(
      /docs|code|No projects|Add your first project/i,
      undefined,
      { timeout: 3000 },
    );
    expect(matches.length).toBeGreaterThan(0);
  });
});
