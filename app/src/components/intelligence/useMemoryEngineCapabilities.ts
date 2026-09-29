import debug from 'debug';
import { useEffect, useState } from 'react';

import { memoryEngineGet, memoryEnginesList } from '../../utils/tauriCommands/memoryEngine';

const log = debug('brain:memory-engine-capabilities');

export interface MemoryEngineCapabilities {
  /** Id of the bound engine; null until the core answers or if it cannot. */
  id: string | null;
  /** Human label of the bound engine (falls back to the id). */
  label: string | null;
  /** Capability families the bound engine advertises; null while unknown. */
  capabilities: ReadonlySet<string> | null;
}

const UNKNOWN: MemoryEngineCapabilities = { id: null, label: null, capabilities: null };

/**
 * The active memory engine and the capability families it advertises, from
 * `engine_get` + `engines_list`.
 *
 * Fails open: while loading, or if the core does not answer, `capabilities` is
 * null and callers must treat every family as available. Hiding a working
 * feature because a status probe failed would be worse than showing an error.
 */
export function useMemoryEngineCapabilities(): MemoryEngineCapabilities {
  const [state, setState] = useState<MemoryEngineCapabilities>(UNKNOWN);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [list, current] = await Promise.all([memoryEnginesList(), memoryEngineGet()]);
        if (cancelled) return;
        const entry = list.engines.find(e => e.id === current.driver);
        setState({
          id: current.driver,
          label: entry?.label ?? current.driver,
          capabilities: entry ? new Set(entry.capabilities) : null,
        });
      } catch (err) {
        log('capabilities unavailable: %o', err);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return state;
}
