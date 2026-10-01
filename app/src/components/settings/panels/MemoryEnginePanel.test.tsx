/**
 * MemoryEnginePanel: engine list from the core, switching with and without
 * copying memories (migration polling), error mapping, and signed-out gating
 * of hosted engines.
 */
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import { renderWithProviders } from '../../../test/test-utils';
import type {
  MemoryEngineDescriptor,
  MemoryEngineState,
} from '../../../utils/tauriCommands/memoryEngine';
import { resetMemoryEngineCacheForTests } from '../../intelligence/useMemoryEngineCapabilities';
import MemoryEnginePanel, {
  copyCompletion,
  MIGRATE_JOB_STORAGE_KEY,
  MIGRATE_POLL_INTERVAL_MS,
} from './MemoryEnginePanel';

const hoisted = vi.hoisted(() => ({
  list: vi.fn(),
  get: vi.fn(),
  set: vi.fn(),
  migrate: vi.fn(),
  status: vi.fn(),
  cancel: vi.fn(),
  storage: new Map<string, string>(),
  track: vi.fn(),
  auth: { isAuthenticated: true },
}));

vi.mock('../../../utils/tauriCommands/memoryEngine', () => ({
  memoryEnginesList: (...a: unknown[]) => hoisted.list(...a),
  memoryEngineGet: (...a: unknown[]) => hoisted.get(...a),
  memoryEngineSet: (...a: unknown[]) => hoisted.set(...a),
  memoryEngineMigrate: (...a: unknown[]) => hoisted.migrate(...a),
  memoryEngineMigrateStatus: (...a: unknown[]) => hoisted.status(...a),
  memoryEngineMigrateCancel: (...a: unknown[]) => hoisted.cancel(...a),
}));

vi.mock('../../../store/userScopedStorage', () => ({
  userScopedStorage: {
    getItem: async (k: string) => hoisted.storage.get(k) ?? null,
    setItem: async (k: string, v: string) => {
      hoisted.storage.set(k, v);
    },
    removeItem: async (k: string) => {
      hoisted.storage.delete(k);
    },
  },
}));

vi.mock('../../../services/analytics', () => ({
  trackAnalyticsEvent: (...a: unknown[]) => hoisted.track(...a),
}));

vi.mock('../../../providers/CoreStateProvider', () => ({
  useCoreState: () => ({ snapshot: { auth: hoisted.auth, sessionToken: 'jwt' } }),
}));

vi.mock('../../../utils/localSession', () => ({ isLocalSessionToken: () => false }));

vi.mock('../hooks/useSettingsNavigation', () => ({
  useSettingsNavigation: () => ({ navigateBack: vi.fn(), breadcrumbs: [] }),
}));

function engine(
  id: string,
  overrides: Partial<MemoryEngineDescriptor> = {}
): MemoryEngineDescriptor {
  return {
    id,
    label: id,
    description: `${id} description`,
    needs_endpoint: false,
    needs_key: false,
    key_optional: false,
    deployments: [],
    default_endpoint: null,
    hosted: false,
    capabilities: ['recall', 'graph'],
    ...overrides,
  };
}

const ENGINES: MemoryEngineDescriptor[] = [
  engine('tinymemory'),
  engine('tinyhumans', { hosted: true, capabilities: ['recall'] }),
  engine('supermemory', { needs_key: true }),
  engine('mem0', {
    needs_endpoint: true,
    needs_key: true,
    key_optional: true,
    deployments: ['cloud', 'self_hosted'],
    default_endpoint: 'https://api.mem0.ai',
    capabilities: ['recall'],
  }),
];

function state(overrides: Partial<MemoryEngineState> = {}): MemoryEngineState {
  return {
    driver: 'tinymemory',
    endpoint: null,
    deployment: null,
    has_credential: false,
    class: 'local',
    fell_back_from: null,
    last_error: null,
    ...overrides,
  };
}

async function renderPanel() {
  renderWithProviders(<MemoryEnginePanel />);
  await screen.findByTestId('memory-engine-option-tinymemory');
}

const pick = (id: string) =>
  fireEvent.click(screen.getByRole('radio', { name: new RegExp(id, 'i') }));

beforeEach(() => {
  vi.clearAllMocks();
  resetMemoryEngineCacheForTests();
  hoisted.storage.clear();
  hoisted.auth = { isAuthenticated: true };
  hoisted.list.mockResolvedValue({ engines: ENGINES, active: 'tinymemory' });
  hoisted.get.mockResolvedValue(state());
});

afterEach(() => {
  vi.useRealTimers();
});

describe('MemoryEnginePanel', () => {
  test('renders the engines returned by the core and marks the active one', async () => {
    await renderPanel();
    for (const e of ENGINES) {
      expect(screen.getByTestId(`memory-engine-option-${e.id}`)).toBeInTheDocument();
    }
    const active = screen.getByTestId('memory-engine-option-tinymemory');
    expect(active).toHaveTextContent('Active');
    expect(screen.getByTestId('memory-engine-option-mem0')).not.toHaveTextContent('Active');
    // Nothing to switch to until another engine is picked.
    expect(screen.getByTestId('memory-engine-switch')).toBeDisabled();
  });

  test('switch without copying calls engine_set and tracks only the engine id', async () => {
    hoisted.set.mockResolvedValue(state({ driver: 'supermemory', has_credential: true }));
    await renderPanel();
    pick('supermemory');

    const keyInput = screen.getByLabelText('API key');
    expect(keyInput).toHaveAttribute('type', 'password');
    expect(screen.getByTestId('memory-engine-switch')).toBeDisabled();
    fireEvent.change(keyInput, { target: { value: 'sk-secret' } });
    fireEvent.click(screen.getByTestId('memory-engine-switch'));

    expect(await screen.findByTestId('memory-engine-switch-dialog')).toBeInTheDocument();
    expect(screen.getByText('Copy my existing memories to the new engine?')).toBeInTheDocument();
    // Brain features the target lacks are listed.
    hoisted.get.mockResolvedValue(state({ driver: 'supermemory', has_credential: true }));
    fireEvent.click(screen.getByTestId('memory-engine-switch-only'));

    await waitFor(() =>
      expect(hoisted.set).toHaveBeenCalledWith({ driver: 'supermemory', api_key: 'sk-secret' })
    );
    expect(hoisted.migrate).not.toHaveBeenCalled();
    await waitFor(() =>
      expect(hoisted.track).toHaveBeenCalledWith('memory_engine_switched', {
        engine: 'supermemory',
      })
    );
    await waitFor(() =>
      expect(screen.queryByTestId('memory-engine-switch-dialog')).not.toBeInTheDocument()
    );
    // The stored key is never echoed back into the field.
    expect(screen.getByLabelText('API key')).toHaveValue('');
    expect(screen.getByText('A key is saved. Leave blank to keep it.')).toBeInTheDocument();
  });

  test('prefills the endpoint and offers deployments for engines that need them', async () => {
    await renderPanel();
    pick('mem0');
    expect(screen.getByLabelText('Endpoint')).toHaveValue('https://api.mem0.ai');
    expect(screen.getByLabelText('Deployment')).toHaveValue('cloud');
    // Key is optional for this engine, so switching is allowed without one.
    expect(screen.getByTestId('memory-engine-switch')).toBeEnabled();
  });

  test('copy & switch polls migration progress, then refreshes', async () => {
    vi.useFakeTimers();
    hoisted.migrate.mockResolvedValue({ job_id: 'job-1' });
    hoisted.status
      .mockResolvedValueOnce({ state: 'running', copied: 2, total: 4, error: null })
      .mockResolvedValueOnce({ state: 'done', copied: 4, total: 4, error: null });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    pick('mem0');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByTestId('memory-engine-lacking')).toHaveTextContent('graph');

    hoisted.get.mockResolvedValue(state({ driver: 'mem0', endpoint: 'https://api.mem0.ai' }));
    fireEvent.click(screen.getByTestId('memory-engine-copy-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(hoisted.migrate).toHaveBeenCalledWith(
      { driver: 'mem0', endpoint: 'https://api.mem0.ai', deployment: 'cloud' },
      { replayContent: true }
    );
    expect(screen.getByTestId('memory-engine-progress')).toBeInTheDocument();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
    });
    expect(hoisted.status).toHaveBeenCalledWith('job-1');
    expect(screen.getByTestId('memory-engine-progress')).toHaveTextContent('Copied 2 of 4');

    await act(async () => {
      await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
    });
    expect(hoisted.track).toHaveBeenCalledWith('memory_engine_switched', { engine: 'mem0' });
    expect(screen.queryByTestId('memory-engine-switch-dialog')).not.toBeInTheDocument();
    expect(hoisted.get).toHaveBeenCalledTimes(2);
  });

  test('a failed migration shows an error and keeps the previous engine', async () => {
    vi.useFakeTimers();
    hoisted.migrate.mockResolvedValue({ job_id: 'job-2' });
    hoisted.status.mockResolvedValue({ state: 'failed', copied: 1, total: 4, error: 'boom' });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    pick('mem0');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    fireEvent.click(screen.getByTestId('memory-engine-copy-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
    });
    expect(screen.getByTestId('memory-engine-error-other')).toBeInTheDocument();
    expect(hoisted.track).not.toHaveBeenCalled();
  });

  test('insufficient credits maps to a friendly state with a billing link', async () => {
    hoisted.set.mockRejectedValue(new Error('INSUFFICIENT_CREDITS: balance is zero'));
    await renderPanel();
    pick('tinyhumans');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    fireEvent.click(await screen.findByTestId('memory-engine-switch-only'));

    const alert = await screen.findByTestId('memory-engine-error-insufficient_credits');
    expect(alert).toHaveTextContent('out of OpenHuman credits');
    expect(screen.getByTestId('memory-engine-open-billing')).toBeInTheDocument();
    expect(hoisted.track).not.toHaveBeenCalled();
  });

  test('session expired prompts to sign in', async () => {
    hoisted.set.mockRejectedValue(new Error('SESSION_EXPIRED: token rejected'));
    await renderPanel();
    pick('tinyhumans');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    fireEvent.click(await screen.findByTestId('memory-engine-switch-only'));
    expect(await screen.findByTestId('memory-engine-error-session_expired')).toBeInTheDocument();
    expect(screen.getByTestId('memory-engine-sign-in')).toBeInTheDocument();
  });

  test('hosted engines are disabled when signed out', async () => {
    hoisted.auth = { isAuthenticated: false };
    await renderPanel();
    const hosted = screen.getByTestId('memory-engine-option-tinyhumans');
    expect(hosted).toHaveTextContent('Billed through your OpenHuman plan or credits.');
    expect(hosted).toHaveTextContent('Sign in to use this engine.');
    expect(screen.getByRole('radio', { name: /tinyhumans/i })).toBeDisabled();
    // Non-hosted engines stay selectable.
    expect(screen.getByRole('radio', { name: /supermemory/i })).toBeEnabled();
  });

  test('a failed bind says memory is paused (not local) and shows the reason', async () => {
    hoisted.get.mockResolvedValue(
      state({ driver: 'null', fell_back_from: 'mem0', last_error: 'engine credential unavailable' })
    );
    await renderPanel();
    const warning = screen.getByTestId('memory-engine-fallback');
    expect(warning).toHaveTextContent('Memory is paused');
    expect(warning).toHaveTextContent('mem0');
    expect(warning).toHaveTextContent('engine credential unavailable');
    expect(warning).not.toHaveTextContent(/local memory/i);
  });

  test('cancel stops a running copy and leaves the panel on the previous engine', async () => {
    vi.useFakeTimers();
    hoisted.migrate.mockResolvedValue({ job_id: 'job-c' });
    hoisted.cancel.mockResolvedValue({ cancelled: true });
    hoisted.status
      .mockResolvedValueOnce({ state: 'running', copied: 1, total: null, error: null })
      .mockResolvedValue({ state: 'cancelled', copied: 1, total: null, error: null });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    pick('mem0');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    fireEvent.click(screen.getByTestId('memory-engine-copy-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    fireEvent.click(screen.getByTestId('memory-engine-cancel-migration'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(hoisted.cancel).toHaveBeenCalledWith('job-c');

    for (let i = 0; i < 2; i += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
      });
    }
    expect(screen.queryByTestId('memory-engine-switch-dialog')).not.toBeInTheDocument();
    expect(hoisted.track).not.toHaveBeenCalled();
    expect(hoisted.storage.has(MIGRATE_JOB_STORAGE_KEY)).toBe(false);
  });

  test('the running migration job id is persisted and resumed on return', async () => {
    vi.useFakeTimers();
    hoisted.storage.set(
      MIGRATE_JOB_STORAGE_KEY,
      JSON.stringify({ jobId: 'job-r', driver: 'mem0' })
    );
    hoisted.status.mockResolvedValue({ state: 'running', copied: 3, total: null, error: null });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(hoisted.status).toHaveBeenCalledWith('job-r');
    expect(screen.getByTestId('memory-engine-progress')).toHaveTextContent('Copied 3 memories');
  });

  test('a stored job that already finished is cleared without a dialog', async () => {
    hoisted.storage.set(
      MIGRATE_JOB_STORAGE_KEY,
      JSON.stringify({ jobId: 'job-x', driver: 'mem0' })
    );
    hoisted.status.mockResolvedValue({ state: 'failed', copied: 0, total: null, error: 'x' });
    await renderPanel();
    await waitFor(() => expect(hoisted.storage.has(MIGRATE_JOB_STORAGE_KEY)).toBe(false));
    expect(screen.queryByTestId('memory-engine-switch-dialog')).not.toBeInTheDocument();
  });

  test('starting a copy persists the job id', async () => {
    hoisted.migrate.mockResolvedValue({ job_id: 'job-p' });
    hoisted.status.mockResolvedValue({ state: 'running', copied: 0, total: null, error: null });
    await renderPanel();
    pick('mem0');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    fireEvent.click(await screen.findByTestId('memory-engine-copy-switch'));
    await waitFor(() => expect(hoisted.storage.get(MIGRATE_JOB_STORAGE_KEY)).toContain('job-p'));
    expect(JSON.parse(hoisted.storage.get(MIGRATE_JOB_STORAGE_KEY) ?? '{}')).toEqual({
      jobId: 'job-p',
      driver: 'mem0',
      replayContent: true,
    });
  });

  test('a resumed migration keeps the replay choice it was started with', async () => {
    vi.useFakeTimers();
    hoisted.storage.set(
      MIGRATE_JOB_STORAGE_KEY,
      JSON.stringify({ jobId: 'job-o', driver: 'mem0', replayContent: false })
    );
    const skippedContent = {
      step: 'content',
      skipped_because: 'the caller chose not to re-send content',
      read: 0,
      written: 0,
      unchanged: 0,
      failed: 0,
      errors: [],
    };
    hoisted.status
      .mockResolvedValueOnce({ state: 'running', copied: 1, total: null, error: null })
      .mockResolvedValue({
        state: 'done',
        copied: 1,
        total: null,
        error: null,
        steps: [skippedContent],
      });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByTestId('memory-engine-progress')).toBeInTheDocument();
    hoisted.get.mockResolvedValue(state({ driver: 'mem0', endpoint: 'https://api.mem0.ai' }));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(
      screen.queryByTestId('memory-engine-left-behind'),
      'content the user chose not to re-send is not reported as left behind'
    ).not.toBeInTheDocument();
  });

  test('the copy can leave synced content behind, and the hint shows for hosted engines', async () => {
    vi.useFakeTimers();
    hoisted.migrate.mockResolvedValue({ job_id: 'job-h' });
    hoisted.status.mockResolvedValue({ state: 'running', copied: 0, total: null, error: null });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    pick('tinyhumans');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    const dialog = screen.getByTestId('memory-engine-switch-dialog');
    expect(dialog).toHaveTextContent('conversation history');
    expect(dialog).toHaveTextContent('OpenHuman credits');
    const replay = screen.getByTestId('memory-engine-replay-content');
    expect(replay).toBeChecked();
    fireEvent.click(replay);
    expect(replay).not.toBeChecked();
    fireEvent.click(screen.getByTestId('memory-engine-copy-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(hoisted.migrate).toHaveBeenCalledWith(
      { driver: 'tinyhumans' },
      { replayContent: false }
    );
  });

  test('the progress line names the step under way', async () => {
    vi.useFakeTimers();
    hoisted.migrate.mockResolvedValue({ job_id: 'job-s' });
    hoisted.status.mockResolvedValue({
      state: 'running',
      copied: 12,
      total: null,
      error: null,
      step: 'episodic',
      step_read: 40,
      step_written: 38,
      steps: [],
    });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    pick('mem0');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    fireEvent.click(screen.getByTestId('memory-engine-copy-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
    });
    expect(screen.getByTestId('memory-engine-progress-line')).toHaveTextContent(
      'Copying conversation history… 40 copied so far'
    );
  });

  test('after the switch, what the new engine could not take is named', async () => {
    vi.useFakeTimers();
    hoisted.migrate.mockResolvedValue({ job_id: 'job-l' });
    const skipped = (step: string) => ({
      step,
      skipped_because: 'the target does not serve it',
      read: 0,
      written: 0,
      unchanged: 0,
      failed: 0,
      errors: [],
    });
    hoisted.status.mockResolvedValue({
      state: 'done',
      copied: 4,
      total: null,
      error: null,
      steps: [
        skipped('goals'),
        skipped('profile'),
        { ...skipped('content'), skipped_because: null, read: 5, written: 3, failed: 2 },
      ],
    });
    renderWithProviders(<MemoryEnginePanel />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    pick('mem0');
    fireEvent.click(screen.getByTestId('memory-engine-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    hoisted.get.mockResolvedValue(state({ driver: 'mem0', endpoint: 'https://api.mem0.ai' }));
    fireEvent.click(screen.getByTestId('memory-engine-copy-switch'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(MIGRATE_POLL_INTERVAL_MS);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByTestId('memory-engine-left-behind')).toHaveTextContent(
      'goals, learned profile'
    );
    expect(screen.getByTestId('memory-engine-content-refused')).toHaveTextContent('(2)');
  });
});

describe('copyCompletion', () => {
  const step = (name: 'content' | 'goals', skipped: string | null, failed = 0) => ({
    step: name,
    skipped_because: skipped,
    read: 0,
    written: 0,
    unchanged: 0,
    failed,
    errors: [],
  });
  const done = (steps: ReturnType<typeof step>[]) => ({
    state: 'done' as const,
    copied: 0,
    total: null,
    error: null,
    steps,
  });

  test('content the user chose not to re-send is not left behind', () => {
    const status = done([
      step('content', 'the caller chose not to re-send content'),
      step('goals', null),
    ]);
    expect(copyCompletion(status, false)).toEqual({ leftBehind: [], refused: 0 });
    expect(copyCompletion(status, true)).toEqual({ leftBehind: ['content'], refused: 0 });
  });

  test('content the new engine refused is counted', () => {
    const status = done([
      step('goals', 'the target does not serve goals'),
      step('content', null, 3),
    ]);
    expect(copyCompletion(status, true)).toEqual({ leftBehind: ['goals'], refused: 3 });
  });
});
