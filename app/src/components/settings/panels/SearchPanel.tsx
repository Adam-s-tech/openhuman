import { Save } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

import { useT } from '../../../lib/i18n/I18nContext';
import { useCoreState } from '../../../providers/CoreStateProvider';
import { isLocalSessionToken } from '../../../utils/localSession';
import {
  openhumanGetSearchSettings,
  openhumanUpdateSearchSettings,
  type SearchEngineId,
  type SearchSettings,
  type SearchSettingsUpdate,
} from '../../../utils/tauriCommands/config';
import PanelPage from '../../layout/PanelPage';
import { Alert, AlertDescription } from '../../ui/Alert';
import Button from '../../ui/Button';
import Card from '../../ui/Card';
import { CenteredLoadingState } from '../../ui/LoadingState';
import StatusLine from '../../ui/StatusLine';
import TextArea from '../../ui/TextArea';
import { ToggleGroupItem, ToggleGroupRoot } from '../../ui/ToggleGroup';
import SettingsBackButton from '../components/SettingsBackButton';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';
import SearchPanelEngineList, { type EngineOption } from './SearchPanelEngineList';
import KeyEditor from './SearchPanelKeyEditor';

type Status =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'saving' }
  | { kind: 'saved' }
  | { kind: 'error'; message: string };

/**
 * Tri-state web-access mode for the unified fetch + browser allowlist.
 * - `all`    → `allow_all: true` (the `"*"` wildcard)
 * - `custom` → `allow_all: false` + an explicit host list (textarea)
 * - `block`  → `allow_all: false` + an empty host list (no web access)
 *
 * `block` and an empty `custom` are indistinguishable once persisted (both are
 * `allow_all: false` + `[]`); the distinction only matters locally while
 * editing.
 */
type AccessMode = 'all' | 'custom' | 'block';

/** Search engines that route directly from this machine with the user's own key. */
type ByokEngine = 'parallel' | 'brave' | 'querit' | 'exa' | 'tavily';

/** Patch field that carries each BYOK engine's key. Empty string clears it. */
const BYOK_KEY_FIELD: Record<ByokEngine, keyof SearchSettingsUpdate> = {
  parallel: 'parallel_api_key',
  brave: 'brave_api_key',
  querit: 'querit_api_key',
  exa: 'exa_api_key',
  tavily: 'tavily_api_key',
};

/**
 * Normalize a user-entered allowed-site entry down to a bare host so it
 * matches `url_guard`'s host-based comparison. Strips a leading scheme and any
 * path/query/fragment — e.g. `https://reuters.com/markets` → `reuters.com` —
 * and trims surrounding whitespace. The `*` allow-all wildcard is preserved.
 */
const normalizeAllowedHost = (raw: string): string =>
  raw
    .trim()
    .replace(/^[a-z][a-z0-9+.-]*:\/\//i, '')
    .replace(/\/.*$/, '')
    .trim();

const SearchPanel = ({ embedded = false }: { embedded?: boolean }) => {
  const { t } = useT();
  const { navigateBack } = useSettingsNavigation();
  const { snapshot } = useCoreState();
  const isLocalSession = isLocalSessionToken(snapshot.sessionToken);

  const [settings, setSettings] = useState<SearchSettings | null>(null);
  const [status, setStatus] = useState<Status>({ kind: 'loading' });
  const [parallelKey, setParallelKey] = useState<string>('');
  const [braveKey, setBraveKey] = useState<string>('');
  const [queritKey, setQueritKey] = useState<string>('');
  const [exaKey, setExaKey] = useState<string>('');
  const [tavilyKey, setTavilyKey] = useState<string>('');
  const [showParallel, setShowParallel] = useState(false);
  const [showBrave, setShowBrave] = useState(false);
  const [showQuerit, setShowQuerit] = useState(false);
  const [showExa, setShowExa] = useState(false);
  const [showTavily, setShowTavily] = useState(false);
  // Editor text for the allowed-websites host list (one host per line). The
  // "*" wildcard is represented by the access mode, not shown here.
  const [allowedText, setAllowedText] = useState<string>('');
  // Tri-state web-access mode for the unified fetch + browser allowlist.
  const [mode, setMode] = useState<AccessMode>('all');
  // Sync editor + mode from settings exactly once, so a later settings refresh
  // (e.g. after saving an engine change) can't clobber the user's in-progress
  // host edits or chosen mode.
  const initializedRef = useRef(false);

  const ENGINES: EngineOption[] = [
    {
      id: 'disabled',
      label: t('settings.search.engineDisabledLabel'),
      description: t('settings.search.engineDisabledDesc'),
      requiresKey: false,
    },
    {
      id: 'managed',
      label: t('settings.search.engineManagedLabel'),
      description: t('settings.search.engineManagedDesc'),
      requiresKey: false,
    },
    {
      id: 'parallel',
      label: t('settings.search.engineParallelLabel'),
      description: t('settings.search.engineParallelDesc'),
      requiresKey: true,
    },
    {
      id: 'brave',
      label: t('settings.search.engineBraveLabel'),
      description: t('settings.search.engineBraveDesc'),
      requiresKey: true,
    },
    {
      id: 'querit',
      label: t('settings.search.engineQueritLabel'),
      description: t('settings.search.engineQueritDesc'),
      requiresKey: true,
    },
    {
      id: 'exa',
      label: t('settings.search.engineExaLabel'),
      description: t('settings.search.engineExaDesc'),
      requiresKey: true,
    },
    {
      id: 'tavily',
      label: t('settings.search.engineTavilyLabel'),
      description: t('settings.search.engineTavilyDesc'),
      requiresKey: true,
    },
  ];
  const visibleEngines = isLocalSession
    ? ENGINES.filter(engine => engine.id !== 'managed')
    : ENGINES;

  useEffect(() => {
    let cancelled = false;
    openhumanGetSearchSettings()
      .then(res => {
        if (cancelled) return;
        setSettings(res.result);
        setStatus({ kind: 'idle' });
      })
      .catch(err => {
        if (cancelled) return;
        setStatus({ kind: 'error', message: err instanceof Error ? err.message : String(err) });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Reflect the loaded allowlist into the editor + mode, exactly once.
  useEffect(() => {
    if (!settings || initializedRef.current) return;
    initializedRef.current = true;
    const explicit = settings.allowed_domains.filter(d => d !== '*');
    setAllowedText(explicit.join('\n'));
    setMode(settings.allow_all ? 'all' : explicit.length > 0 ? 'custom' : 'block');
  }, [settings]);

  const selectedEngine = (settings?.engine as SearchEngineId | undefined) ?? 'managed';

  const persistEngine = async (next: SearchEngineId) => {
    if (!settings || status.kind === 'saving') return;
    const previous = settings;
    setSettings({ ...settings, engine: next });
    setStatus({ kind: 'saving' });
    try {
      await openhumanUpdateSearchSettings({ engine: next });
      const refreshed = await openhumanGetSearchSettings();
      setSettings(refreshed.result);
      setStatus({ kind: 'saved' });
    } catch (err) {
      setSettings(previous);
      setStatus({ kind: 'error', message: err instanceof Error ? err.message : String(err) });
    }
  };

  // Clear the local draft input once its key round-trips to the core.
  const clearDraftKey: Record<ByokEngine, () => void> = {
    parallel: () => setParallelKey(''),
    brave: () => setBraveKey(''),
    querit: () => setQueritKey(''),
    exa: () => setExaKey(''),
    tavily: () => setTavilyKey(''),
  };

  const persistKey = async (engine: ByokEngine, rawKey: string) => {
    if (!settings) return;
    setStatus({ kind: 'saving' });
    try {
      await openhumanUpdateSearchSettings({ [BYOK_KEY_FIELD[engine]]: rawKey });
      const refreshed = await openhumanGetSearchSettings();
      setSettings(refreshed.result);
      clearDraftKey[engine]();
      setStatus({ kind: 'saved' });
    } catch (err) {
      setStatus({ kind: 'error', message: err instanceof Error ? err.message : String(err) });
    }
  };

  const persistSearchUpdate = async (update: SearchSettingsUpdate) => {
    if (!settings || status.kind === 'saving') return;
    setStatus({ kind: 'saving' });
    try {
      await openhumanUpdateSearchSettings(update);
      const refreshed = await openhumanGetSearchSettings();
      setSettings(refreshed.result);
      setStatus({ kind: 'saved' });
    } catch (err) {
      setStatus({ kind: 'error', message: err instanceof Error ? err.message : String(err) });
    }
  };

  // Switch web-access mode. "Allow all" / "Block all" persist immediately;
  // "Custom" only reveals the host editor (its Save button persists the list),
  // and we keep whatever the user has already typed.
  const selectMode = (next: AccessMode) => {
    if (status.kind === 'saving') return;
    setMode(next);
    if (next === 'all') {
      void persistSearchUpdate({ allow_all: true });
    } else if (next === 'block') {
      void persistSearchUpdate({ allowed_domains: [], allow_all: false });
    }
  };

  const persistAllowedDomains = () => {
    const domains = allowedText.split('\n').map(normalizeAllowedHost).filter(Boolean);
    // Editing the explicit host list implies "not allow-all".
    void persistSearchUpdate({ allowed_domains: domains, allow_all: false });
  };

  const isConfigured = (engine: SearchEngineId): boolean => {
    if (!settings) return false;
    if (engine === 'disabled') return true;
    if (engine === 'managed') return true;
    if (engine === 'parallel') return settings.parallel_configured;
    if (engine === 'brave') return settings.brave_configured;
    if (engine === 'querit') return settings.querit_configured;
    if (engine === 'exa') return settings.exa_configured;
    if (engine === 'tavily') return settings.tavily_configured;
    return false;
  };

  // One row per BYOK engine in the API keys card.
  const KEY_ROWS: {
    engine: ByokEngine;
    labelKey: string;
    placeholderKey: string;
    configured: boolean;
    value: string;
    setValue: (v: string) => void;
    show: boolean;
    toggleShow: () => void;
    docUrl: string;
  }[] = settings
    ? [
        {
          engine: 'parallel',
          labelKey: 'settings.search.parallelKeyLabel',
          placeholderKey: 'settings.search.placeholderParallel',
          configured: settings.parallel_configured,
          value: parallelKey,
          setValue: setParallelKey,
          show: showParallel,
          toggleShow: () => setShowParallel(s => !s),
          docUrl: 'https://parallel.ai/',
        },
        {
          engine: 'brave',
          labelKey: 'settings.search.braveKeyLabel',
          placeholderKey: 'settings.search.placeholderBrave',
          configured: settings.brave_configured,
          value: braveKey,
          setValue: setBraveKey,
          show: showBrave,
          toggleShow: () => setShowBrave(s => !s),
          docUrl: 'https://brave.com/search/api/',
        },
        {
          engine: 'querit',
          labelKey: 'settings.search.queritKeyLabel',
          placeholderKey: 'settings.search.placeholderQuerit',
          configured: settings.querit_configured,
          value: queritKey,
          setValue: setQueritKey,
          show: showQuerit,
          toggleShow: () => setShowQuerit(s => !s),
          docUrl: 'https://www.querit.ai/en/docs/reference/post',
        },
        {
          engine: 'exa',
          labelKey: 'settings.search.exaKeyLabel',
          placeholderKey: 'settings.search.placeholderExa',
          configured: settings.exa_configured,
          value: exaKey,
          setValue: setExaKey,
          show: showExa,
          toggleShow: () => setShowExa(s => !s),
          docUrl: 'https://exa.ai',
        },
        {
          engine: 'tavily',
          labelKey: 'settings.search.tavilyKeyLabel',
          placeholderKey: 'settings.search.placeholderTavily',
          configured: settings.tavily_configured,
          value: tavilyKey,
          setValue: setTavilyKey,
          show: showTavily,
          toggleShow: () => setShowTavily(s => !s),
          docUrl: 'https://tavily.com',
        },
      ]
    : [];

  return (
    <PanelPage
      className="z-10"
      testId="search-settings-panel"
      contentClassName=""
      description={embedded ? undefined : t('settings.search.menuDesc')}
      leading={embedded ? undefined : <SettingsBackButton onBack={navigateBack} />}>
      <div className={embedded ? 'space-y-5' : 'space-y-5 p-4'}>
        {isLocalSession && (
          <Alert variant="info">
            <AlertDescription>{t('settings.search.localManagedUnavailable')}</AlertDescription>
          </Alert>
        )}

        {status.kind === 'loading' && <CenteredLoadingState label={t('common.loading')} />}

        {!settings && status.kind === 'error' && (
          <Alert variant="destructive" density="compact" data-testid="search-settings-load-error">
            <AlertDescription>{`${t('settings.search.statusError')}: ${status.message}`}</AlertDescription>
          </Alert>
        )}

        {settings && (
          <>
            {/* ── Engine ─────────────────────────────────────────────── */}
            <Card
              title={t('settings.search.engineAria')}
              description={t('settings.search.description')}>
              <SearchPanelEngineList
                engines={visibleEngines}
                selectedEngine={selectedEngine}
                ariaLabel={t('settings.search.engineAria')}
                isConfigured={isConfigured}
                onSelect={engine => void persistEngine(engine)}
                t={t}
              />
            </Card>

            {/* ── BYO API keys, one row per direct provider ───────────── */}
            <Card
              title={t('settings.search.apiKeysHeading')}
              description={t('settings.search.apiKeysDesc')}>
              {KEY_ROWS.map(row => (
                <KeyEditor
                  key={row.engine}
                  label={t(row.labelKey)}
                  placeholder={
                    row.configured ? t('settings.search.placeholderStored') : t(row.placeholderKey)
                  }
                  show={row.show}
                  onToggleShow={row.toggleShow}
                  value={row.value}
                  onChange={row.setValue}
                  onSave={() => void persistKey(row.engine, row.value)}
                  onClear={() => void persistKey(row.engine, '')}
                  configured={row.configured}
                  docUrl={row.docUrl}
                  t={t}
                />
              ))}
            </Card>

            {/* ── Allowed websites: the unified host allowlist shared by
                web_fetch / curl and (when enabled) the browser tool. Web
                search is not gated by this list. ────────────────────── */}
            <Card
              title={t('settings.search.allowedSitesLabel')}
              description={
                mode === 'all'
                  ? t('settings.search.allowedSitesAllOn')
                  : mode === 'block'
                    ? t('settings.search.accessBlockAllHint')
                    : t('settings.search.allowedSitesHint')
              }
              headerRight={
                <ToggleGroupRoot
                  type="single"
                  variant="secondary"
                  size="xs"
                  aria-label={t('settings.search.accessModeAria')}
                  value={mode}
                  onValueChange={value => {
                    if (value) selectMode(value as AccessMode);
                  }}
                  disabled={status.kind === 'saving'}
                  className="gap-0 overflow-hidden rounded-lg border border-line *:rounded-none *:border-0">
                  {(
                    [
                      ['all', 'settings.search.accessAllowAll'],
                      ['custom', 'settings.search.accessCustom'],
                      ['block', 'settings.search.accessBlockAll'],
                    ] as const
                  ).map(([value, labelKey]) => (
                    <ToggleGroupItem
                      key={value}
                      value={value}
                      className="h-auto px-2.5 py-1 text-xs font-medium data-[state=on]:bg-primary-500 data-[state=on]:text-content-inverted">
                      {t(labelKey)}
                    </ToggleGroupItem>
                  ))}
                </ToggleGroupRoot>
              }>
              {mode === 'custom' && (
                <div className="space-y-3 p-4">
                  <TextArea
                    value={allowedText}
                    onChange={e => setAllowedText(e.target.value)}
                    rows={5}
                    spellCheck={false}
                    placeholder={t('settings.search.allowedSitesPlaceholder')}
                    className="font-mono text-xs"
                    aria-label={t('settings.search.allowedSitesLabel')}
                  />
                  <div className="flex justify-end">
                    <Button
                      type="button"
                      variant="primary"
                      size="sm"
                      leadingIcon={<Save className="h-3.5 w-3.5" aria-hidden />}
                      onClick={() => persistAllowedDomains()}
                      disabled={status.kind === 'saving'}>
                      {t('settings.search.allowedSitesSave')}
                    </Button>
                  </div>
                </div>
              )}
            </Card>

            <StatusLine
              saving={status.kind === 'saving'}
              savedNote={status.kind === 'saved' ? t('settings.search.statusSaved') : null}
              error={
                status.kind === 'error'
                  ? `${t('settings.search.statusError')}: ${status.message}`
                  : null
              }
              savingLabel={t('settings.search.statusSaving')}
            />
          </>
        )}
      </div>
    </PanelPage>
  );
};

export default SearchPanel;
