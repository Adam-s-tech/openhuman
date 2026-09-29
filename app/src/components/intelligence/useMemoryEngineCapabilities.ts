import debug from 'debug';
import { useCallback, useEffect, useState } from 'react';

import {
  type MemoryEngineDescriptor,
  memoryEngineGet,
  memoryEnginesList,
  type MemoryEngineState,
} from '../../utils/tauriCommands/memoryEngine';

const log = debug('brain:memory-engine');

/** How long a fetched engine snapshot is reused by views mounting close together. */
export const MEMORY_ENGINE_CACHE_TTL_MS = 5000;

export interface MemoryEngineView {
  /** True until the first answer (or failure) arrives. */
  loading: boolean;
  /** Id of the bound engine; null until the core answers or if it cannot. */
  id: string | null;
  /** Human label of the bound engine (falls back to the id). */
  label: string | null;
  /** Capability families the bound engine advertises; null while unknown. */
  capabilities: ReadonlySet<string> | null;
  /** Every selectable engine (empty until known). */
  engines: MemoryEngineDescriptor[];
  /** The raw `engine_get` state (fell_back_from, last_error, ...); null until known. */
  current: MemoryEngineState | null;
  /** Error from the last fetch, when it failed. */
  error: unknown;
  /** Re-fetch now (deduplicated with any fetch already in flight). */
  refresh: () => Promise<void>;
}

interface Snapshot {
  loading: boolean;
  engines: MemoryEngineDescriptor[];
  current: MemoryEngineState | null;
  error: unknown;
  fetchedAt: number;
}

const INITIAL: Snapshot = { loading: true, engines: [], current: null, error: null, fetchedAt: 0 };

let snapshot: Snapshot = INITIAL;
let inflight: Promise<void> | null = null;
const listeners = new Set<(s: Snapshot) => void>();

function publish(next: Snapshot) {
  snapshot = next;
  listeners.forEach(fn => fn(next));
}

/**
 * One `engines_list` + `engine_get` per view: concurrent callers share the
 * in-flight request and callers within the TTL share its result.
 */
function fetchSnapshot(force: boolean): Promise<void> {
  if (inflight) return inflight;
  const fresh =
    snapshot.fetchedAt > 0 && Date.now() - snapshot.fetchedAt < MEMORY_ENGINE_CACHE_TTL_MS;
  if (!force && fresh && snapshot.error === null) return Promise.resolve();
  inflight = (async () => {
    try {
      const [list, current] = await Promise.all([memoryEnginesList(), memoryEngineGet()]);
      log('loaded engines=%d active=%s', list.engines.length, current.driver);
      publish({
        loading: false,
        engines: list.engines,
        current,
        error: null,
        fetchedAt: Date.now(),
      });
    } catch (err) {
      log('unavailable: %o', err);
      publish({ ...snapshot, loading: false, error: err, fetchedAt: Date.now() });
    } finally {
      inflight = null;
    }
  })();
  return inflight;
}

/** Drop the cached snapshot (after a switch) so the next reader re-fetches. */
export function invalidateMemoryEngine(): void {
  snapshot = { ...snapshot, fetchedAt: 0 };
}

/** Test seam: forget everything, including subscribers' last value. */
export function resetMemoryEngineCacheForTests(): void {
  snapshot = INITIAL;
  inflight = null;
}

/**
 * The active memory engine, its capability families and the engine list — one
 * shared cache behind the Brain row, the family gates and the settings panel.
 *
 * Fails open: while loading, or if the core does not answer, `capabilities` is
 * null and callers must treat every family as available. Hiding a working
 * feature because a status probe failed would be worse than showing an error.
 */
export function useMemoryEngine(): MemoryEngineView {
  const [snap, setSnap] = useState<Snapshot>(snapshot);

  useEffect(() => {
    listeners.add(setSnap);
    setSnap(snapshot);
    void fetchSnapshot(false);
    return () => {
      listeners.delete(setSnap);
    };
  }, []);

  const refresh = useCallback(() => fetchSnapshot(true), []);
  const entry = snap.current ? snap.engines.find(e => e.id === snap.current?.driver) : undefined;
  return {
    loading: snap.loading,
    id: snap.current?.driver ?? null,
    label: snap.current ? (entry?.label ?? snap.current.driver) : null,
    capabilities: entry ? new Set(entry.capabilities) : null,
    engines: snap.engines,
    current: snap.current,
    error: snap.error,
    refresh,
  };
}

/** Back-compat alias for callers that only need the id / label / capabilities. */
export const useMemoryEngineCapabilities = useMemoryEngine;
