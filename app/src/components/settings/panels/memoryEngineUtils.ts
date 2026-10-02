import type { MemoryEngineDescriptor } from '../../../utils/tauriCommands/memoryEngine';

export type MemoryEngineErrorKind =
  | 'insufficient_credits'
  | 'session_expired'
  | 'memory_forbidden'
  | 'backend_unavailable'
  | 'other';

/** Maps the core's prefixed error strings onto a friendly state. */
export function classifyMemoryEngineError(err: unknown): MemoryEngineErrorKind {
  const message = err instanceof Error ? err.message : String(err ?? '');
  if (message.includes('INSUFFICIENT_CREDITS:')) return 'insufficient_credits';
  if (message.includes('SESSION_EXPIRED:')) return 'session_expired';
  // A refused credential (an API key without the memory scope): not a lapsed
  // session, so it must never read as one and sign the user out.
  if (message.includes('MEMORY_FORBIDDEN:')) return 'memory_forbidden';
  if (message.includes('BACKEND_UNAVAILABLE:') || message.includes('MEMORY_UNREACHABLE:')) {
    return 'backend_unavailable';
  }
  return 'other';
}

/** Capabilities the active engine offers that the target lacks. */
export function missingCapabilities(
  active: MemoryEngineDescriptor | undefined,
  target: MemoryEngineDescriptor | undefined
): string[] {
  if (!active || !target || active.id === target.id) return [];
  const have = new Set(target.capabilities);
  return active.capabilities.filter(cap => !have.has(cap));
}

/** `graph_export` -> `graph export`; capability ids are core-defined. */
export function humanizeCapability(cap: string): string {
  return cap.replace(/[_-]+/g, ' ');
}

/** Fallback label for an engine id the RPC did not describe. */
export function engineLabel(engines: MemoryEngineDescriptor[], id: string): string {
  return engines.find(e => e.id === id)?.label ?? id;
}
