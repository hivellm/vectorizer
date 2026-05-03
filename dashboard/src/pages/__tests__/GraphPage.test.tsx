import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';

vi.mock('vis-network', () => ({
  Network: vi.fn(),
}));
vi.mock('vis-data', () => ({
  DataSet: vi.fn(),
}));
// vis-network ships its own CSS via 'vis-network/styles/vis-network.css';
// happy-dom can't parse it, so neutralise the import.
vi.mock('vis-network/styles/vis-network.css', () => ({}));

vi.mock('@/hooks/useGraph', () => ({
  useGraph: () => ({
    listNodes:           vi.fn(async () => ({ nodes: [], count: 0 })),
    listEdges:           vi.fn(async () => ({ edges: [], count: 0 })),
    getNeighbors:        vi.fn(async () => []),
    findRelated:         vi.fn(async () => []),
    findPath:            vi.fn(async () => ({ found: false, path: [] })),
    createEdge:          vi.fn(async () => 'edge-1'),
    deleteEdge:          vi.fn(async () => undefined),
    discoverEdges:       vi.fn(async () => ({ success: true, edges_created: 0, message: '' })),
    discoverEdgesForNode:vi.fn(async () => ({ success: true, edges_created: 0, message: '' })),
    getDiscoveryStatus:  vi.fn(async () => ({ total_nodes: 0, nodes_with_edges: 0, total_edges: 0 })),
    enableGraph:         vi.fn(async () => ({ success: true, collection: 'docs', message: '', node_count: 0 })),
    getGraphStatus:      vi.fn(async () => ({ collection: 'docs', enabled: false, node_count: 0, edge_count: 0 })),
  }),
}));

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get:    vi.fn(async () => ({ data: { nodes: [], edges: [] } })),
    post:   vi.fn(async () => ({ data: {} })),
    delete: vi.fn(async () => ({ data: {} })),
  }),
}));

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy' },
    ],
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [{ name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy' }],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
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

import GraphPage from '../GraphPage';

describe('GraphPage', () => {
  it('renders the page heading', () => {
    render(<MemoryRouter><GraphPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Graph/i })).toBeTruthy();
  });
});
