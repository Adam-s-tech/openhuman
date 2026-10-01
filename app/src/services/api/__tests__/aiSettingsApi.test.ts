/**
 * Unit tests for aiSettingsApi.ts
 *
 * All external deps (tauriCommands/auth, tauriCommands/config, coreRpcClient,
 * tauriCommands/common) are mocked so no Tauri runtime is needed.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';

// ─── Import SUT after mocks ───────────────────────────────────────────────────

import {
  type AISettings,
  classifyProviderVerificationFailure,
  clearCloudProviderKey,
  completeOpenAiCodexOAuth,
  describeProviderVerificationFailure,
  flushCloudProviders,
  importOpenAiCodexCliAuth,
  listProviderModels,
  loadAISettings,
  loadLocalProviderSnapshot,
  loadProviderAuthErrors,
  modelRegistryVision,
  OPENAI_CODEX_OAUTH_MISSING_AUTH_URL,
  OPENAI_CODEX_OAUTH_MISSING_CALLBACK_URL,
  parseProviderString,
  type ProviderRef,
  saveAISettings,
  serializeProviderRef,
  setCloudProviderKey,
  startOpenAiCodexOAuth,
  testProviderModel,
  upsertModelRegistryVision,
  verifyCloudProviderConnection,
} from '../aiSettingsApi';

// ─── Mock declarations (must be hoisted before imports) ───────────────────────

const mockOpenhumanGetClientConfig = vi.fn();
const mockAuthListProviderCredentials = vi.fn();
const mockOpenhumanUpdateModelSettings = vi.fn();
const mockOpenhumanUpdateLocalAiSettings = vi.fn();
const mockAuthStoreProviderCredentials = vi.fn();
const mockAuthRemoveProviderCredentials = vi.fn();
const mockCallCoreRpc = vi.fn();
const mockIsTauri = vi.fn(() => true);
const mockOpenhumanLocalAiStatus = vi.fn();
const mockOpenhumanLocalAiDiagnostics = vi.fn();

vi.mock('../../coreRpcClient', () => ({ callCoreRpc: (a: unknown) => mockCallCoreRpc(a) }));

vi.mock('../../../utils/tauriCommands/common', () => ({
  isTauri: () => mockIsTauri(),
  CommandResponse: {},
}));

vi.mock('../../../utils/tauriCommands/auth', () => ({
  authListProviderCredentials: (a?: unknown) => mockAuthListProviderCredentials(a),
  authStoreProviderCredentials: (a: unknown) => mockAuthStoreProviderCredentials(a),
  authRemoveProviderCredentials: (a: unknown) => mockAuthRemoveProviderCredentials(a),
}));

vi.mock('../../../utils/tauriCommands/config', () => ({
  openhumanGetClientConfig: () => mockOpenhumanGetClientConfig(),
  openhumanUpdateModelSettings: (a: unknown) => mockOpenhumanUpdateModelSettings(a),
  openhumanUpdateLocalAiSettings: (a: unknown) => mockOpenhumanUpdateLocalAiSettings(a),
}));

vi.mock('../../../utils/tauriCommands/localAi', () => ({
  openhumanLocalAiStatus: (...args: unknown[]) => mockOpenhumanLocalAiStatus(...args),
  openhumanLocalAiDiagnostics: (...args: unknown[]) => mockOpenhumanLocalAiDiagnostics(...args),
}));

// ─── Helpers ─────────────────────────────────────────────────────────────────

function makeClientConfigResult(overrides: Record<string, unknown> = {}) {
  return {
    result: {
      api_url: null,
      inference_url: null,
      default_model: null,
      app_version: '0.0.0-test',
      api_key_set: false,
      model_routes: [],
      cloud_providers: [],
      model_registry: [],
      primary_cloud: null,
      reasoning_provider: null,
      agentic_provider: null,
      coding_provider: null,
      memory_provider: null,
      embeddings_provider: null,
      learning_provider: null,
      ...overrides,
    },
  };
}

function makeAuthProfileResult(profiles: Array<{ id: string; provider: string }> = []) {
  return { result: profiles.map(p => ({ ...p, profile_name: 'default', kind: 'token' })) };
}

// ─── parseProviderString ─────────────────────────────────────────────────────

describe('parseProviderString', () => {
  it('returns default for empty string', () => {
    expect(parseProviderString('')).toEqual({ kind: 'default' });
  });

  it('returns default for null/undefined', () => {
    expect(parseProviderString(null)).toEqual({ kind: 'default' });
    expect(parseProviderString(undefined)).toEqual({ kind: 'default' });
  });

  it('returns default for the "cloud" sentinel', () => {
    expect(parseProviderString('cloud')).toEqual({ kind: 'default' });
  });

  it('returns openhuman for the "openhuman" literal', () => {
    expect(parseProviderString('openhuman')).toEqual({ kind: 'openhuman' });
  });

  it('returns openhuman for "openhuman:<anything>"', () => {
    expect(parseProviderString('openhuman:gpt-4o')).toEqual({ kind: 'openhuman' });
  });

  it('parses ollama provider strings', () => {
    expect(parseProviderString('ollama:llama3.1:8b')).toEqual({
      kind: 'local',
      model: 'llama3.1:8b',
    });
  });

  it('parses cloud slug:model strings', () => {
    expect(parseProviderString('openai:gpt-4o')).toEqual({
      kind: 'cloud',
      providerSlug: 'openai',
      model: 'gpt-4o',
    });
    expect(parseProviderString('anthropic:claude-3-5-sonnet-20241022')).toEqual({
      kind: 'cloud',
      providerSlug: 'anthropic',
      model: 'claude-3-5-sonnet-20241022',
    });
  });

  it('falls back to openhuman for unrecognised bare strings', () => {
    expect(parseProviderString('unknown-provider')).toEqual({ kind: 'openhuman' });
  });

  // The `@<temp>` suffix is the per-workload temperature override added with
  // the LLM-routing UI redesign. It must round-trip through parse/serialize
  // and degrade gracefully when the tail isn't a finite number.
  describe('temperature suffix grammar', () => {
    it('parses @temp suffix on cloud strings', () => {
      expect(parseProviderString('openai:gpt-4o@0.7')).toEqual({
        kind: 'cloud',
        providerSlug: 'openai',
        model: 'gpt-4o',
        temperature: 0.7,
      });
    });

    it('parses @temp suffix on ollama strings (including model ids with colons)', () => {
      expect(parseProviderString('ollama:llama3.1:8b@0.2')).toEqual({
        kind: 'local',
        model: 'llama3.1:8b',
        temperature: 0.2,
      });
    });

    it('treats a non-numeric @tail as part of the model id', () => {
      // Guards against silently dropping a chunk of the model id when the
      // user happens to pick a tag like `:beta` after an `@`.
      expect(parseProviderString('openai:gpt@beta')).toEqual({
        kind: 'cloud',
        providerSlug: 'openai',
        model: 'gpt@beta',
      });
    });

    it('drops the temperature key when not configured (toEqual contract)', () => {
      // Existing call sites compare with toEqual — emitting an extra
      // `temperature: null` would break unrelated snapshots.
      const ref = parseProviderString('openai:gpt-4o');
      expect(ref).toEqual({ kind: 'cloud', providerSlug: 'openai', model: 'gpt-4o' });
    });
  });
});

// ─── serializeProviderRef ─────────────────────────────────────────────────────

describe('serializeProviderRef', () => {
  it('serializes openhuman refs', () => {
    const ref: ProviderRef = { kind: 'openhuman' };
    expect(serializeProviderRef(ref)).toBe('openhuman');
  });

  it('serializes default refs', () => {
    const ref: ProviderRef = { kind: 'default' };
    expect(serializeProviderRef(ref)).toBe('cloud');
  });

  it('serializes cloud refs to slug:model', () => {
    const ref: ProviderRef = { kind: 'cloud', providerSlug: 'openai', model: 'gpt-4o' };
    expect(serializeProviderRef(ref)).toBe('openai:gpt-4o');
  });

  it('serializes local refs to ollama:model', () => {
    const ref: ProviderRef = { kind: 'local', model: 'llama3.1:8b' };
    expect(serializeProviderRef(ref)).toBe('ollama:llama3.1:8b');
  });

  it('round-trips through parseProviderString', () => {
    const cases: ProviderRef[] = [
      { kind: 'openhuman' },
      { kind: 'default' },
      { kind: 'cloud', providerSlug: 'anthropic', model: 'claude-3-haiku-20240307' },
      { kind: 'local', model: 'llama3:latest' },
    ];
    for (const ref of cases) {
      expect(parseProviderString(serializeProviderRef(ref))).toEqual(ref);
    }
  });

  it('appends @temp suffix when temperature is set, omits when not', () => {
    expect(serializeProviderRef({ kind: 'cloud', providerSlug: 'openai', model: 'gpt-4o' })).toBe(
      'openai:gpt-4o'
    );
    expect(
      serializeProviderRef({
        kind: 'cloud',
        providerSlug: 'openai',
        model: 'gpt-4o',
        temperature: 0.7,
      })
    ).toBe('openai:gpt-4o@0.7');
    expect(serializeProviderRef({ kind: 'local', model: 'llama3', temperature: 1.25 })).toBe(
      'ollama:llama3@1.25'
    );
  });

  it('rounds temperature to 2 decimal places on the wire', () => {
    // Stops floating-point drift (0.7 + 0.0000001) from leaking into the
    // persisted provider string and confusing the Rust factory.
    expect(
      serializeProviderRef({
        kind: 'cloud',
        providerSlug: 'openai',
        model: 'gpt-4o',
        temperature: 0.7000001,
      })
    ).toBe('openai:gpt-4o@0.7');
  });

  it('treats non-finite temperatures as unset', () => {
    expect(serializeProviderRef({ kind: 'local', model: 'llama3', temperature: Number.NaN })).toBe(
      'ollama:llama3'
    );
  });

  it('round-trips temperature through parse + serialize', () => {
    const ref: ProviderRef = {
      kind: 'cloud',
      providerSlug: 'openai',
      model: 'gpt-4o',
      temperature: 0.2,
    };
    expect(parseProviderString(serializeProviderRef(ref))).toEqual(ref);
  });
});

// ─── loadAISettings ──────────────────────────────────────────────────────────

describe('loadAISettings', () => {
  beforeEach(() => {
    mockOpenhumanGetClientConfig.mockReset();
    mockAuthListProviderCredentials.mockReset();
    mockOpenhumanUpdateLocalAiSettings.mockReset();
    mockOpenhumanLocalAiStatus.mockReset();
    mockOpenhumanLocalAiDiagnostics.mockReset();
  });

  it('returns cloudProviders with has_api_key=false when no profiles stored', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [
          {
            id: 'p_openai_1',
            slug: 'openai',
            label: 'OpenAI',
            endpoint: 'https://api.openai.com/v1',
            auth_style: 'bearer',
          },
        ],
      })
    );
    mockAuthListProviderCredentials.mockResolvedValue(makeAuthProfileResult([]));

    const settings = await loadAISettings();

    expect(settings.cloudProviders).toHaveLength(1);
    expect(settings.cloudProviders[0].slug).toBe('openai');
    expect(settings.cloudProviders[0].auth_style).toBe('bearer');
    expect(settings.cloudProviders[0].has_api_key).toBe(false);
  });

  it('parses per-tier credits_bypass into creditsBypass, defaulting to false (#3767)', async () => {
    mockAuthListProviderCredentials.mockResolvedValue(makeAuthProfileResult([]));

    // Absent in an older snapshot → both tiers conservative false.
    mockOpenhumanGetClientConfig.mockResolvedValue(makeClientConfigResult({}));
    expect((await loadAISettings()).creditsBypass).toEqual({ chat: false, reasoning: false });

    // Per-tier: chat true, reasoning absent → chat true, reasoning false.
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({ credits_bypass: { chat: true } })
    );
    expect((await loadAISettings()).creditsBypass).toEqual({ chat: true, reasoning: false });

    // Both present.
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({ credits_bypass: { chat: true, reasoning: true } })
    );
    expect((await loadAISettings()).creditsBypass).toEqual({ chat: true, reasoning: true });
  });

  it('sets has_api_key=true when a matching provider:<slug> profile is stored', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [
          {
            id: 'p_anthropic_1',
            slug: 'anthropic',
            label: 'Anthropic',
            endpoint: 'https://api.anthropic.com/v1',
            auth_style: 'anthropic',
          },
        ],
      })
    );
    // New-style key format: "provider:<slug>"
    mockAuthListProviderCredentials.mockResolvedValue(
      makeAuthProfileResult([{ id: 'prof-1', provider: 'provider:anthropic' }])
    );

    const settings = await loadAISettings();

    expect(settings.cloudProviders[0].has_api_key).toBe(true);
    // auth_style must survive the round-trip unmodified.
    expect(settings.cloudProviders[0].auth_style).toBe('anthropic');
  });

  it('also accepts legacy bare-slug auth profiles', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [
          {
            id: 'p_openai_2',
            slug: 'openai',
            label: 'OpenAI',
            endpoint: 'https://api.openai.com/v1',
            auth_style: 'bearer',
          },
        ],
      })
    );
    // Legacy format: bare slug, no "provider:" prefix
    mockAuthListProviderCredentials.mockResolvedValue(
      makeAuthProfileResult([{ id: 'prof-2', provider: 'openai' }])
    );

    const settings = await loadAISettings();
    expect(settings.cloudProviders[0].has_api_key).toBe(true);
  });

  it('parses non-default per-workload routing strings correctly', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [],
        reasoning_provider: 'openai:gpt-4o',
        agentic_provider: 'anthropic:claude-3-5-sonnet-20241022',
        coding_provider: 'ollama:codellama:13b',
        memory_provider: null,
        embeddings_provider: null,
        learning_provider: null,
      })
    );
    mockAuthListProviderCredentials.mockResolvedValue(makeAuthProfileResult([]));

    const settings = await loadAISettings();

    expect(settings.routing.reasoning).toEqual({
      kind: 'cloud',
      providerSlug: 'openai',
      model: 'gpt-4o',
    });
    expect(settings.routing.agentic).toEqual({
      kind: 'cloud',
      providerSlug: 'anthropic',
      model: 'claude-3-5-sonnet-20241022',
    });
    expect(settings.routing.coding).toEqual({ kind: 'local', model: 'codellama:13b' });
    expect(settings.routing.memory).toEqual({ kind: 'default' });
  });

  it('degrades gracefully when authListProviderCredentials throws', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [
          {
            id: 'p_openai_3',
            slug: 'openai',
            label: 'OpenAI',
            endpoint: 'https://api.openai.com/v1',
            auth_style: 'bearer',
          },
        ],
      })
    );
    mockAuthListProviderCredentials.mockRejectedValue(new Error('no profiles file'));

    const settings = await loadAISettings();

    // Should not throw; has_api_key should default to false.
    expect(settings.cloudProviders[0].has_api_key).toBe(false);
  });

  it('keeps local runtime endpoint providers so the AI panel can edit them', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [
          {
            id: 'p_ollama_1',
            slug: 'ollama',
            label: 'Ollama',
            endpoint: 'http://127.0.0.1:11434/v1',
            auth_style: 'none',
          },
        ],
      })
    );
    mockAuthListProviderCredentials.mockResolvedValue(makeAuthProfileResult([]));

    const settings = await loadAISettings();

    expect(settings.cloudProviders).toHaveLength(1);
    expect(settings.cloudProviders[0].slug).toBe('ollama');
    expect(settings.cloudProviders[0].endpoint).toBe('http://127.0.0.1:11434/v1');
  });

  it('includes two cloud providers with correct labels and endpoints', async () => {
    mockOpenhumanGetClientConfig.mockResolvedValue(
      makeClientConfigResult({
        cloud_providers: [
          {
            id: 'p_openai_4',
            slug: 'openai',
            label: 'OpenAI',
            endpoint: 'https://api.openai.com/v1',
            auth_style: 'bearer',
          },
          {
            id: 'p_anthropic_4',
            slug: 'anthropic',
            label: 'Anthropic',
            endpoint: 'https://api.anthropic.com/v1',
            auth_style: 'anthropic',
          },
        ],
        reasoning_provider: 'openai:gpt-4o',
        agentic_provider: 'anthropic:claude-3-5-sonnet-20241022',
      })
    );
    mockAuthListProviderCredentials.mockResolvedValue(
      makeAuthProfileResult([
        { id: 'prof-openai', provider: 'provider:openai' },
        { id: 'prof-anthropic', provider: 'provider:anthropic' },
      ])
    );

    const settings = await loadAISettings();

    expect(settings.cloudProviders).toHaveLength(2);
    const openai = settings.cloudProviders.find(p => p.slug === 'openai')!;
    const anthropic = settings.cloudProviders.find(p => p.slug === 'anthropic')!;

    expect(openai.label).toBe('OpenAI');
    expect(openai.endpoint).toBe('https://api.openai.com/v1');
    expect(openai.auth_style).toBe('bearer');
    expect(openai.has_api_key).toBe(true);

    expect(anthropic.label).toBe('Anthropic');
    expect(anthropic.endpoint).toBe('https://api.anthropic.com/v1');
    expect(anthropic.auth_style).toBe('anthropic');
    expect(anthropic.has_api_key).toBe(true);

    expect(settings.routing.reasoning).toEqual({
      kind: 'cloud',
      providerSlug: 'openai',
      model: 'gpt-4o',
    });
    expect(settings.routing.agentic).toEqual({
      kind: 'cloud',
      providerSlug: 'anthropic',
      model: 'claude-3-5-sonnet-20241022',
    });
  });
});

describe('local provider snapshot', () => {
  beforeEach(() => {
    mockOpenhumanLocalAiStatus.mockReset();
    mockOpenhumanLocalAiDiagnostics.mockReset();
  });

  it('loadLocalProviderSnapshot joins status and diagnostics', async () => {
    mockOpenhumanLocalAiStatus.mockResolvedValue({ result: { state: 'ready' } });
    mockOpenhumanLocalAiDiagnostics.mockResolvedValue({
      installed_models: [{ name: 'gemma3:1b-it-qat', size: 123 }],
    });

    const snapshot = await loadLocalProviderSnapshot();

    expect(snapshot.status).toEqual({ state: 'ready' });
    expect(snapshot.installedModels).toEqual([{ name: 'gemma3:1b-it-qat', size: 123 }]);
    expect(snapshot).not.toHaveProperty('presets');
  });

  it('loadLocalProviderSnapshot tolerates an unreachable endpoint', async () => {
    mockOpenhumanLocalAiStatus.mockRejectedValue(new Error('connection refused'));
    mockOpenhumanLocalAiDiagnostics.mockRejectedValue(new Error('connection refused'));

    const snapshot = await loadLocalProviderSnapshot();

    expect(snapshot).toEqual({ status: null, diagnostics: null, installedModels: [] });
  });
});
