import { act, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { resetMemoryEngineCacheForTests } from '../../components/intelligence/useMemoryEngineCapabilities';
import { renderWithProviders } from '../../test/test-utils';
import Brain from '../Brain';

const graphExportMock = vi.hoisted(() => vi.fn());
// The bound memory engine, as `MemoryFamilyGate` reads it. Rejected by default,
// so every gate fails open and the tabs render as they always have.
const engineMock = vi.hoisted(() => ({ list: vi.fn(), get: vi.fn() }));
vi.mock('../../utils/tauriCommands/memoryEngine', () => ({
  memoryEnginesList: (...a: unknown[]) => engineMock.list(...a),
  memoryEngineGet: (...a: unknown[]) => engineMock.get(...a),
}));
// Controllable authenticated identity so we can simulate a logout→login cycle
// (userId null → set) and assert the graph reloads (#4149).
const coreAuthRef = vi.hoisted(() => ({ current: 'user-A' as string | null }));
const navigateSpy = vi.hoisted(() => vi.fn());

vi.mock('react-router-dom', async importOriginal => {
  const actual = await importOriginal<typeof import('react-router-dom')>();
  return { ...actual, useNavigate: () => navigateSpy };
});

vi.mock('../../utils/tauriCommands', () => ({
  memoryTreeGraphExport: graphExportMock,
  isTauri: () => false,
}));

vi.mock('../../providers/CoreStateProvider', () => ({
  useCoreState: () => ({
    snapshot: {
      auth: { userId: coreAuthRef.current, isAuthenticated: coreAuthRef.current != null },
    },
  }),
}));

vi.mock('../../components/intelligence/MemoryGraph', async () => {
  const React = await import('react');
  return {
    MemoryGraph: ({ nodes }: { nodes: unknown[] }) =>
      React.createElement('div', { 'data-testid': 'memory-graph' }, `nodes:${nodes.length}`),
  };
});

vi.mock('../../lib/i18n/I18nContext', () => ({ useT: () => ({ t: (k: string) => k }) }));

vi.mock('../../components/layout/ChipTabs', async () => {
  const React = await import('react');
  return {
    default: ({ children }: { children?: React.ReactNode }) =>
      React.createElement('div', null, children),
  };
});
vi.mock('../../components/ui/BetaBanner', () => ({ default: () => null }));
vi.mock('../../components/intelligence/MemoryControls', () => ({ MemoryControls: () => null }));
vi.mock('../../components/intelligence/MemoryTreeStatusPanel', async () => {
  const React = await import('react');
  return {
    MemoryTreeStatusPanel: () => React.createElement('div', { 'data-testid': 'brain-sync' }),
  };
});
vi.mock('../../components/intelligence/MemorySourcesRegistry', async () => {
  const React = await import('react');
  return {
    MemorySourcesRegistry: () => React.createElement('div', { 'data-testid': 'brain-sources' }),
  };
});
vi.mock('../../components/intelligence/Toast', () => ({ ToastContainer: () => null }));
vi.mock('../../components/intelligence/SyncAuditPanel', async () => {
  const React = await import('react');
  return {
    SyncAuditPanel: () => React.createElement('div', { 'data-testid': 'brain-sync-audit' }),
  };
});
vi.mock('../../components/intelligence/SyncActivityCard', async () => {
  const React = await import('react');
  return {
    SyncActivityCard: () =>
      React.createElement('div', { 'data-testid': 'brain-sync-activity-card' }),
  };
});

const makeGraph = (n: number) => ({
  nodes: Array.from({ length: n }, (_, i) => ({ id: `n${i}`, kind: 'summary', label: `N${i}` })),
  edges: [],
  content_root_abs: '/tmp/content',
});

describe('Brain page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    coreAuthRef.current = 'user-A';
    resetMemoryEngineCacheForTests();
    engineMock.get.mockRejectedValue(new Error('no engine rpc in this test'));
    engineMock.list.mockRejectedValue(new Error('no engine rpc in this test'));
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders the graph once data is fetched', async () => {
    graphExportMock.mockResolvedValue(makeGraph(3));
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=graph'] });
    });
    await waitFor(() => {
      expect(screen.getByTestId('memory-graph')).toHaveTextContent('nodes:3');
    });
  });

  it('shows a loading state, not an empty canvas, until the graph arrives', async () => {
    let resolveGraph!: (graph: ReturnType<typeof makeGraph>) => void;
    graphExportMock.mockReturnValue(
      new Promise(resolve => {
        resolveGraph = resolve;
      })
    );
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=graph'] });
    });
    expect(screen.getByTestId('brain-graph-loading')).toHaveTextContent('workspace.loadingGraph');
    expect(screen.queryByTestId('memory-graph')).not.toBeInTheDocument();

    await act(async () => {
      resolveGraph(makeGraph(2));
    });
    await waitFor(() => {
      expect(screen.getByTestId('memory-graph')).toHaveTextContent('nodes:2');
    });
    expect(screen.queryByTestId('brain-graph-loading')).not.toBeInTheDocument();
  });

  it('renders empty-state graph when there are no nodes', async () => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=graph'] });
    });
    await waitFor(() => {
      expect(screen.getByTestId('memory-graph')).toHaveTextContent('nodes:0');
    });
  });

  it('reloads the memory graph from the store when the user re-authenticates (#4149)', async () => {
    // Start signed-out / mid identity-flip: the first fetch resolves empty.
    coreAuthRef.current = null;
    graphExportMock.mockResolvedValue(makeGraph(0));
    let view!: ReturnType<typeof renderWithProviders>;
    await act(async () => {
      view = renderWithProviders(<Brain />, { initialEntries: ['/?tab=graph'] });
    });
    await waitFor(() => expect(graphExportMock).toHaveBeenCalledTimes(1));
    expect(screen.getByTestId('memory-graph')).toHaveTextContent('nodes:0');

    // Re-login: identity becomes available — the graph must re-pull from the
    // persistent store rather than keep the signed-out empty state.
    coreAuthRef.current = 'user-A';
    graphExportMock.mockResolvedValue(makeGraph(5));
    await act(async () => {
      view.rerender(<Brain />);
    });
    await waitFor(() => expect(graphExportMock).toHaveBeenCalledTimes(2));
    await waitFor(() => {
      expect(screen.getByTestId('memory-graph')).toHaveTextContent('nodes:5');
    });
  });

  it('surfaces an error alert when the fetch fails', async () => {
    graphExportMock.mockRejectedValue(new Error('boom'));
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=graph'] });
    });
    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeInTheDocument();
    });
  });

  // All tabs share the standard scaffold. Drive each via the `?tab=` query
  // param so every per-tab branch is exercised.
  it.each([
    ['sources', 'brain-sources'],
    ['sync', 'brain-sync'],
  ])('renders the %s tab', async (tab, testId) => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: [`/?tab=${tab}`] });
    });
    await waitFor(() => {
      expect(screen.getByTestId(testId)).toBeInTheDocument();
    });
  });

  // Sync has two sub-views reflected in `?view=`: the default "status" view
  // (live status panel + activity card) and "history" (the full-height run
  // history table). They're mutually exclusive panes, not stacked together.
  it('shows the live status and activity panels on the Sync status sub-view (default)', async () => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=sync'] });
    });
    await waitFor(() => {
      expect(screen.getByTestId('brain-sync')).toBeInTheDocument();
      expect(screen.getByTestId('brain-sync-activity')).toBeInTheDocument();
      expect(screen.getByTestId('brain-sync-activity-card')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('brain-sync-history')).toBeNull();
    expect(screen.queryByTestId('brain-sync-audit')).toBeNull();
  });

  it('shows the sync history panel on the Sync history sub-view', async () => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=sync&view=history'] });
    });
    await waitFor(() => {
      expect(screen.getByTestId('brain-sync-history')).toBeInTheDocument();
      expect(screen.getByTestId('brain-sync-audit')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('brain-sync-activity')).toBeNull();
    expect(screen.queryByTestId('brain-sync-activity-card')).toBeNull();
  });

  // A remote engine without the Sources family (a direct CortexDB) has no
  // `memory_sources.*` RPCs at all: the sync panels must say so rather than
  // render and fail.
  it('shows the sync panels as unavailable on an engine without sources', async () => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    engineMock.get.mockResolvedValue({ driver: 'cortex' });
    engineMock.list.mockResolvedValue({
      active: 'cortex',
      engines: [
        {
          id: 'cortex',
          label: 'CortexDB',
          capabilities: ['core', 'recall', 'portability', 'answer'],
        },
      ],
    });
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=sync'] });
    });
    await waitFor(() => {
      expect(screen.getAllByTestId('memory-family-unavailable').length).toBeGreaterThan(0);
    });
    expect(screen.queryByTestId('brain-sync-activity')).toBeNull();
    expect(screen.queryByTestId('brain-sync-activity-card')).toBeNull();
  });

  // Hosted memory serves a tree — the server's understanding — but keeps no
  // local chunk store: the graph draws, and the ingest pipeline's status panel
  // says it is not available rather than reading a store that is not there.
  const hostedWithTree = () => {
    engineMock.get.mockResolvedValue({ driver: 'tinyhumans' });
    engineMock.list.mockResolvedValue({
      active: 'tinyhumans',
      engines: [
        {
          id: 'tinyhumans',
          label: 'CortexDB (via TinyHumans)',
          capabilities: ['core', 'recall', 'portability', 'sources', 'retrieval', 'tree'],
        },
      ],
    });
  };

  it('draws the graph on hosted memory', async () => {
    graphExportMock.mockResolvedValue(makeGraph(2));
    hostedWithTree();
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=graph'] });
    });
    await waitFor(() => {
      expect(screen.getByTestId('memory-graph')).toHaveTextContent('nodes:2');
    });
  });

  it('gates the local pipeline status on hosted memory', async () => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    hostedWithTree();
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=sync'] });
    });
    await waitFor(() => {
      const gated = screen
        .getAllByTestId('memory-family-unavailable')
        .map(node => node.getAttribute('data-family'));
      expect(gated).toContain('chunks');
    });
    expect(screen.queryByTestId('brain-sync')).toBeNull();
  });

  // Hosted memory accepts synced items but reads no local agent transcripts:
  // the sources registry renders, the coding-sessions card says it is not
  // available instead of calling an RPC the engine refuses.
  it('gates the coding sessions card on its own family', async () => {
    graphExportMock.mockResolvedValue(makeGraph(0));
    engineMock.get.mockResolvedValue({ driver: 'tinyhumans' });
    engineMock.list.mockResolvedValue({
      active: 'tinyhumans',
      engines: [
        {
          id: 'tinyhumans',
          label: 'CortexDB (via TinyHumans)',
          capabilities: ['core', 'recall', 'portability', 'answer', 'sources', 'documents'],
        },
      ],
    });
    await act(async () => {
      renderWithProviders(<Brain />, { initialEntries: ['/?tab=sources'] });
    });
    await waitFor(() => {
      const gated = screen
        .getAllByTestId('memory-family-unavailable')
        .map(node => node.getAttribute('data-family'));
      expect(gated).toContain('coding_sessions');
      expect(gated).not.toContain('sources');
    });
  });
});
