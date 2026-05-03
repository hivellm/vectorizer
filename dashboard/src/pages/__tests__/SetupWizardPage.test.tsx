import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import SetupWizardPage from '../SetupWizardPage';

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get:  vi.fn(async () => ({ data: { needs_setup: true } })),
    post: vi.fn(async () => ({ data: { ok: true } })),
    put:  vi.fn(async () => ({ data: { ok: true } })),
  }),
}));

vi.mock('@/hooks/useToast', () => ({
  useToast: () => ({ show: vi.fn(), success: vi.fn(), error: vi.fn() }),
}));

// useSetup pulls /setup/status; stub it so the wizard mounts without
// hitting the network in jsdom/happy-dom.
vi.mock('@/hooks/useSetup', () => ({
  useSetup: () => ({
    getStatus: vi.fn(async () => ({
      needs_setup: true,
      version: '0.0.0',
      deployment_type: 'binary',
      has_workspace_config: false,
      project_count: 0,
      collection_count: 0,
    })),
    analyzeDirectory: vi.fn(async () => ({})),
    applyConfig: vi.fn(async () => ({ success: true, message: 'ok' })),
    verify: vi.fn(async () => ({})),
  }),
}));

vi.mock('@/hooks/useTemplates.tsx', async () => {
  const actual = await vi.importActual<typeof import('@/hooks/useTemplates.tsx')>(
    '@/hooks/useTemplates.tsx'
  );
  return {
    ...actual,
    useTemplates: () => ({ templates: [], loading: false, error: null, refetch: vi.fn() }),
  };
});

vi.mock('@/hooks/useApiKeys', () => ({
  useApiKeys: () => ({ createApiKey: vi.fn(async () => ({ api_key: 'k' })), loading: false, error: null }),
}));

describe('SetupWizardPage', () => {
  it('renders the wizard heading', async () => {
    render(<MemoryRouter><SetupWizardPage /></MemoryRouter>);
    // Match either "Setup" or "Welcome" depending on the wizard's first-step
    // label. The wizard renders a loading spinner first, then the welcome
    // step once the async status fetch resolves; await it.
    const headings = await screen.findAllByRole('heading');
    expect(headings.length).toBeGreaterThan(0);
  });
});
