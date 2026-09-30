/**
 * The shared engine cache: every reader in one view shares one
 * `engines_list` + `engine_get`, and a forced refresh re-fetches.
 */
import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { resetMemoryEngineCacheForTests, useMemoryEngine } from './useMemoryEngineCapabilities';

const hoisted = vi.hoisted(() => ({ list: vi.fn(), get: vi.fn() }));

vi.mock('../../utils/tauriCommands/memoryEngine', () => ({
  memoryEnginesList: (...a: unknown[]) => hoisted.list(...a),
  memoryEngineGet: (...a: unknown[]) => hoisted.get(...a),
}));

beforeEach(() => {
  hoisted.list.mockReset();
  hoisted.get.mockReset();
  resetMemoryEngineCacheForTests();
  hoisted.get.mockResolvedValue({ driver: 'mem0' });
  hoisted.list.mockResolvedValue({
    active: 'mem0',
    engines: [{ id: 'mem0', label: 'Mem0', capabilities: ['core'] }],
  });
});

describe('useMemoryEngine', () => {
  test('several readers in one view share a single list + get', async () => {
    const a = renderHook(() => useMemoryEngine());
    const b = renderHook(() => useMemoryEngine());
    const c = renderHook(() => useMemoryEngine());
    await waitFor(() => expect(a.result.current.id).toBe('mem0'));
    await waitFor(() => expect(b.result.current.id).toBe('mem0'));
    await waitFor(() => expect(c.result.current.label).toBe('Mem0'));
    expect(hoisted.list).toHaveBeenCalledTimes(1);
    expect(hoisted.get).toHaveBeenCalledTimes(1);
  });

  test('a reader mounting within the TTL reuses the result', async () => {
    const first = renderHook(() => useMemoryEngine());
    await waitFor(() => expect(first.result.current.id).toBe('mem0'));
    const later = renderHook(() => useMemoryEngine());
    expect(later.result.current.id).toBe('mem0');
    expect(hoisted.get).toHaveBeenCalledTimes(1);
  });

  test('refresh forces a new fetch and fans out to every reader', async () => {
    const a = renderHook(() => useMemoryEngine());
    const b = renderHook(() => useMemoryEngine());
    await waitFor(() => expect(a.result.current.id).toBe('mem0'));
    hoisted.get.mockResolvedValue({ driver: 'null' });
    await act(async () => {
      await a.result.current.refresh();
    });
    await waitFor(() => expect(b.result.current.id).toBe('null'));
    expect(hoisted.get).toHaveBeenCalledTimes(2);
  });

  test('a failed fetch leaves capabilities unknown (fail open) and reports the error', async () => {
    hoisted.get.mockRejectedValue(new Error('offline'));
    const a = renderHook(() => useMemoryEngine());
    await waitFor(() => expect(a.result.current.loading).toBe(false));
    expect(a.result.current.capabilities).toBeNull();
    expect(a.result.current.error).toBeTruthy();
  });
});
