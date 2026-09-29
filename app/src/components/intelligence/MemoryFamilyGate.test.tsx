/**
 * MemoryFamilyGate: shows children when the bound engine has the family (or
 * when that is unknown), and a friendly "Not available" state when it does not.
 */
import { screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { renderWithProviders } from '../../test/test-utils';
import MemoryFamilyGate from './MemoryFamilyGate';
import { resetMemoryEngineCacheForTests } from './useMemoryEngineCapabilities';

const hoisted = vi.hoisted(() => ({ list: vi.fn(), get: vi.fn() }));

vi.mock('../../utils/tauriCommands/memoryEngine', () => ({
  memoryEnginesList: (...a: unknown[]) => hoisted.list(...a),
  memoryEngineGet: (...a: unknown[]) => hoisted.get(...a),
}));

function bind(driver: string, label: string, capabilities: string[]) {
  hoisted.get.mockResolvedValue({ driver });
  hoisted.list.mockResolvedValue({
    active: driver,
    engines: [{ id: driver, label, capabilities }],
  });
}

describe('MemoryFamilyGate', () => {
  beforeEach(() => {
    hoisted.list.mockReset();
    hoisted.get.mockReset();
    resetMemoryEngineCacheForTests();
  });

  test('shows the empty state naming the engine when the family is missing', async () => {
    bind('mem0', 'Mem0', ['core', 'recall', 'portability']);
    renderWithProviders(
      <MemoryFamilyGate family="goals">
        <div>goals body</div>
      </MemoryFamilyGate>
    );
    await waitFor(() => expect(screen.getByTestId('memory-family-unavailable')).toBeTruthy());
    expect(screen.getByText(/Not available with/)).toBeTruthy();
    expect(screen.queryByText('goals body')).toBeNull();
  });

  test('renders children when the engine has the family', async () => {
    bind('tinymemory', 'Local', ['core', 'recall', 'goals']);
    renderWithProviders(
      <MemoryFamilyGate family="goals">
        <div>goals body</div>
      </MemoryFamilyGate>
    );
    await waitFor(() => expect(hoisted.list).toHaveBeenCalled());
    expect(screen.getByText('goals body')).toBeTruthy();
    expect(screen.queryByTestId('memory-family-unavailable')).toBeNull();
  });

  test('fails open when the core cannot report the engine', async () => {
    hoisted.get.mockRejectedValue(new Error('offline'));
    hoisted.list.mockRejectedValue(new Error('offline'));
    renderWithProviders(
      <MemoryFamilyGate family="goals">
        <div>goals body</div>
      </MemoryFamilyGate>
    );
    await waitFor(() => expect(hoisted.get).toHaveBeenCalled());
    expect(screen.getByText('goals body')).toBeTruthy();
    expect(screen.queryByTestId('memory-family-unavailable')).toBeNull();
  });

  test('shows a skeleton, not the children, while the engine is loading', async () => {
    let release: (v: unknown) => void = () => {};
    hoisted.get.mockReturnValue(new Promise(r => (release = r)));
    hoisted.list.mockResolvedValue({ active: 'mem0', engines: [] });
    renderWithProviders(
      <MemoryFamilyGate family="goals">
        <div>goals body</div>
      </MemoryFamilyGate>
    );
    expect(screen.getByTestId('memory-family-loading')).toBeTruthy();
    expect(screen.queryByText('goals body')).toBeNull();
    release({ driver: 'tinymemory' });
    await waitFor(() => expect(screen.queryByTestId('memory-family-loading')).toBeNull());
  });

  test('gates every family when memory is paused on the null driver', async () => {
    hoisted.get.mockResolvedValue({
      driver: 'null',
      fell_back_from: 'mem0',
      last_error: 'engine credential unavailable',
    });
    hoisted.list.mockResolvedValue({ active: 'null', engines: [] });
    renderWithProviders(
      <MemoryFamilyGate family="core">
        <div>core body</div>
      </MemoryFamilyGate>
    );
    await waitFor(() => expect(screen.getByTestId('memory-family-unavailable')).toBeTruthy());
    expect(screen.getByText('Memory is paused')).toBeTruthy();
    expect(screen.getByText('engine credential unavailable')).toBeTruthy();
    expect(screen.queryByText('core body')).toBeNull();
  });
});
