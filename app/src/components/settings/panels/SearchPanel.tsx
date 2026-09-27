import { ChevronDownIcon } from 'lucide-react';
import { useEffect, useId, useState } from 'react';

import { useT } from '../../../lib/i18n/I18nContext';
import { useCoreState } from '../../../providers/CoreStateProvider';
import { isLocalSessionToken } from '../../../utils/localSession';
import {
  openhumanGetSearchSettings,
  openhumanUpdateSearchSettings,
  type SearchProviderUpdate,
  type SearchSettings,
  type SearchSettingsUpdate,
} from '../../../utils/tauriCommands/config';
import PanelPage from '../../layout/PanelPage';
import { Alert, AlertDescription } from '../../ui/Alert';
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from '../../ui/Collapsible';
import { CenteredLoadingState } from '../../ui/LoadingState';
import StatusLine from '../../ui/StatusLine';
import Switch from '../../ui/Switch';
import SettingsBackButton from '../components/SettingsBackButton';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';
import SearchPanelAllowedSites from './SearchPanelAllowedSites';
import SearchPanelProviderCard from './SearchPanelProviderCard';
import SearchPanelRoles from './SearchPanelRoles';

type Status =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'saving' }
  | { kind: 'saved' }
  | { kind: 'error'; message: string };

const errorMessage = (err: unknown) => (err instanceof Error ? err.message : String(err));

/**
 * Web search settings. Everything is rendered from the core's
 * `config_get_search_settings` response: the global switch, one card per
 * provider, the per-role provider order, the allowed-websites list and the
 * advanced tool-presentation toggle. Every update returns the full settings
 * object, which replaces local state, so the view never guesses what the core
 * decided (for example which provider now serves a role).
 */
const SearchPanel = ({ embedded = false }: { embedded?: boolean }) => {
  const { t } = useT();
  const { navigateBack } = useSettingsNavigation();
  const { snapshot } = useCoreState();
  const isLocalSession = isLocalSessionToken(snapshot.sessionToken);
  const enabledId = useId();
  const presentationId = useId();

  const [settings, setSettings] = useState<SearchSettings | null>(null);
  const [status, setStatus] = useState<Status>({ kind: 'loading' });
  const saving = status.kind === 'saving';

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
        setStatus({ kind: 'error', message: errorMessage(err) });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  /** Send a patch; on success the returned settings replace local state. */
  const persist = async (update: SearchSettingsUpdate): Promise<boolean> => {
    if (!settings || saving) return false;
    setStatus({ kind: 'saving' });
    try {
      const res = await openhumanUpdateSearchSettings(update);
      if (res?.result) setSettings(res.result);
      setStatus({ kind: 'saved' });
      return true;
    } catch (err) {
      setStatus({ kind: 'error', message: errorMessage(err) });
      return false;
    }
  };

  const updateProvider = (id: string, patch: SearchProviderUpdate) =>
    persist({ providers: { [id]: patch } });

  const managedUnavailable = isLocalSession || (settings ? !settings.managed_available : false);

  return (
    <PanelPage
      className="z-10"
      testId="search-settings-panel"
      contentClassName=""
      description={embedded ? undefined : t('settings.search.menuDesc')}
      leading={embedded ? undefined : <SettingsBackButton onBack={navigateBack} />}>
      <div className={embedded ? 'space-y-5' : 'p-4 space-y-5'}>
        <p className="text-xs text-content-muted leading-relaxed">
          {t('settings.search.description')}
        </p>

        {managedUnavailable && (
          <Alert variant="info">
            <AlertDescription>{t('settings.search.localManagedUnavailable')}</AlertDescription>
          </Alert>
        )}

        {status.kind === 'loading' && <CenteredLoadingState label={t('common.loading')} />}

        {settings && (
          <>
            {/* Search on/off */}
            <div className="flex items-center gap-3 rounded-xl border border-line bg-surface p-3">
              <label htmlFor={enabledId} className="flex-1 min-w-0">
                <span className="block text-sm font-medium text-content">
                  {t('settings.search.enabledLabel')}
                </span>
                <span className="block text-xs text-content-muted">
                  {t('settings.search.enabledDesc')}
                </span>
              </label>
              <Switch
                id={enabledId}
                data-testid="search-enabled-toggle"
                aria-label={t('settings.search.enabledLabel')}
                checked={settings.enabled}
                disabled={saving}
                onCheckedChange={next => void persist({ enabled: next })}
              />
            </div>

            {/* Providers */}
            <section className="space-y-2" aria-labelledby="search-providers-heading">
              <div>
                <h3
                  id="search-providers-heading"
                  className="text-xs font-semibold text-content-secondary">
                  {t('settings.search.providersTitle')}
                </h3>
                <p className="text-[11px] text-content-muted leading-relaxed">
                  {t('settings.search.providersDesc')}
                </p>
              </div>
              {settings.providers.map(provider => (
                <SearchPanelProviderCard
                  key={provider.id}
                  provider={provider}
                  saving={saving}
                  onUpdate={patch => updateProvider(provider.id, patch)}
                  t={t}
                />
              ))}
            </section>

            <SearchPanelRoles settings={settings} saving={saving} persist={persist} t={t} />

            <SearchPanelAllowedSites settings={settings} saving={saving} persist={persist} t={t} />

            {/* Advanced */}
            <CollapsibleRoot variant="card" data-testid="search-advanced">
              <CollapsibleTrigger size="sm">
                {t('settings.search.advancedTitle')}
                <ChevronDownIcon
                  className="size-3.5 transition-transform group-data-[state=open]:rotate-180"
                  aria-hidden="true"
                />
              </CollapsibleTrigger>
              <CollapsibleContent size="sm">
                <div className="flex items-center gap-3 pt-1">
                  <label htmlFor={presentationId} className="flex-1 min-w-0">
                    <span className="block text-xs font-medium text-content">
                      {t('settings.search.exposeProviderTools')}
                    </span>
                    <span className="block text-[11px] text-content-muted leading-relaxed">
                      {t('settings.search.exposeProviderToolsDesc')}
                    </span>
                  </label>
                  <Switch
                    id={presentationId}
                    data-testid="search-presentation-toggle"
                    aria-label={t('settings.search.exposeProviderTools')}
                    checked={settings.presentation === 'all_tools'}
                    disabled={saving}
                    onCheckedChange={next =>
                      void persist({ presentation: next ? 'all_tools' : 'roles' })
                    }
                  />
                </div>
              </CollapsibleContent>
            </CollapsibleRoot>
          </>
        )}

        <StatusLine
          saving={saving}
          savedNote={status.kind === 'saved' ? t('settings.search.statusSaved') : null}
          error={
            status.kind === 'error'
              ? `${t('settings.search.statusError')}: ${status.message}`
              : null
          }
          savingLabel={t('settings.search.statusSaving')}
        />
      </div>
    </PanelPage>
  );
};

export default SearchPanel;
