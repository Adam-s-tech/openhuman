/**
 * Typed wrappers for the memory-engine RPCs (`openhuman.memory_engine*`).
 *
 * The core owns engine selection, credentials and migration; these wrappers
 * only carry the wire shapes. Errors may be prefixed `INSUFFICIENT_CREDITS:`,
 * `SESSION_EXPIRED:` or `BACKEND_UNAVAILABLE:` — see `memoryEngineErrors`.
 */
import { callCoreRpc } from '../../services/coreRpcClient';
import { CORE_RPC_METHODS } from '../../services/rpcMethods';

export interface MemoryEngineDescriptor {
  id: string;
  label: string;
  description: string;
  needs_endpoint: boolean;
  needs_key: boolean;
  key_optional: boolean;
  /** e.g. `["cloud", "self_hosted"]`, or empty when the engine has one shape. */
  deployments: string[];
  default_endpoint: string | null;
  /** True when billed through OpenHuman credits and the signed-in session. */
  hosted: boolean;
  capabilities: string[];
}

export interface MemoryEnginesList {
  engines: MemoryEngineDescriptor[];
  active: string;
}

export interface MemoryEngineState {
  driver: string;
  endpoint: string | null;
  deployment: string | null;
  has_credential: boolean;
  class: string;
  fell_back_from: string | null;
  last_error: string | null;
}

export interface MemoryEngineTarget {
  driver: string;
  endpoint?: string;
  deployment?: string;
  api_key?: string;
}

export type MemoryEngineMigrateState = 'running' | 'done' | 'failed' | 'cancelled';

export interface MemoryEngineMigrateStatus {
  state: MemoryEngineMigrateState;
  copied: number;
  total: number | null;
  error: string | null;
  /** Caveat on a finished job (writes made while the copy ran); absent otherwise. */
  note?: string | null;
}

export async function memoryEnginesList(): Promise<MemoryEnginesList> {
  return await callCoreRpc<MemoryEnginesList>({ method: CORE_RPC_METHODS.memoryEnginesList });
}

export async function memoryEngineGet(): Promise<MemoryEngineState> {
  return await callCoreRpc<MemoryEngineState>({ method: CORE_RPC_METHODS.memoryEngineGet });
}

export async function memoryEngineSet(target: MemoryEngineTarget): Promise<MemoryEngineState> {
  return await callCoreRpc<MemoryEngineState>({
    method: CORE_RPC_METHODS.memoryEngineSet,
    params: target,
  });
}

/** Copies every memory from the active engine into `to`, then switches to it. */
export async function memoryEngineMigrate(to: MemoryEngineTarget): Promise<{ job_id: string }> {
  return await callCoreRpc<{ job_id: string }>({
    method: CORE_RPC_METHODS.memoryEngineMigrate,
    params: { to },
  });
}

export async function memoryEngineMigrateStatus(jobId: string): Promise<MemoryEngineMigrateStatus> {
  return await callCoreRpc<MemoryEngineMigrateStatus>({
    method: CORE_RPC_METHODS.memoryEngineMigrateStatus,
    params: { job_id: jobId },
  });
}

/** Stops a running migration; the active engine is left unchanged. */
export async function memoryEngineMigrateCancel(jobId: string): Promise<{ cancelled: boolean }> {
  return await callCoreRpc<{ cancelled: boolean }>({
    method: CORE_RPC_METHODS.memoryEngineMigrateCancel,
    params: { job_id: jobId },
  });
}
