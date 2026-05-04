import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';

// Skip the Monaco mount in the test environment — it's network/canvas heavy
// and not what this test is verifying. Same pattern as ConfigurationPage.test.
vi.mock('@/components/ui/CodeEditor', () => ({
  default: () => <div data-testid="code-editor-stub">[CodeEditor]</div>,
}));

// The ApiDocs sandbox uses raw `fetch` for `/api-keys` and the live
// "Try it" requests, plus `useSandboxHistory` for persistence — both run
// only when the user clicks "Try it in Sandbox" / executes a request.
// No useApiClient / useToast mock is needed here because the static
// catalog renders without those hooks at all.

import ApiDocsPage from '../ApiDocsPage';

describe('ApiDocsPage', () => {
  it('renders the page heading', () => {
    render(
      <MemoryRouter>
        <ApiDocsPage />
      </MemoryRouter>,
    );
    expect(
      screen.getByRole('heading', {
        name: /API Docs|API Documentation|API Reference/i,
      }),
    ).toBeTruthy();
  });

  it('lists at least one well-known REST endpoint from the static catalog', () => {
    render(
      <MemoryRouter>
        <ApiDocsPage />
      </MemoryRouter>,
    );
    // The static catalog always includes GET /collections — the path
    // appears verbatim in the endpoint row regardless of expanded state.
    const matches = screen.getAllByText('/collections');
    expect(matches.length).toBeGreaterThan(0);
  });
});
