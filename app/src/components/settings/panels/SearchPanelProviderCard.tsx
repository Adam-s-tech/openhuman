import { useId, useState } from 'react';

import type {
  SearchProviderInfo,
  SearchProviderStatus,
  SearchProviderUpdate,
  SearchRoute,
} from '../../../utils/tauriCommands/config';
import Badge, { type BadgeVariant } from '../../ui/Badge';
import Button from '../../ui/Button';
import Input from '../../ui/Input';
import Switch from '../../ui/Switch';
import { ToggleGroupItem, ToggleGroupRoot } from '../../ui/ToggleGroup';
import KeyEditor from './SearchPanelKeyEditor';

type Translate = (key: string) => string;

const STATUS_VARIANT: Record<SearchProviderStatus, BadgeVariant> = {
  ready: 'success',
  needs_key: 'warning',
  sign_in_required: 'warning',
  disabled: 'neutral',
  search_off: 'neutral',
};

/** Badge text for a provider status. Literal keys so the i18n scanner sees them. */
export function statusLabel(status: SearchProviderStatus, t: Translate): string {
  switch (status) {
    case 'ready':
      return t('settings.search.statusReady');
    case 'needs_key':
      return t('settings.search.statusNeedsKey');
    case 'sign_in_required':
      return t('settings.search.statusSignInRequired');
    default:
      return t('settings.search.statusOff');
  }
}

const withProvider = (text: string, provider: string) => text.replace('{provider}', provider);

interface Props {
  provider: SearchProviderInfo;
  saving: boolean;
  /** Persist a patch for this provider; resolves true when the core accepted it. */
  onUpdate: (patch: SearchProviderUpdate) => Promise<boolean>;
  t: Translate;
}

/**
 * One search provider: enable switch, status badge, route choice (when the
 * provider supports more than one), its key editor and any extra field the
 * core reports for it (SearXNG's instance URL). Everything shown is driven by
 * the provider entry the core returned; nothing here knows a provider by id.
 */
const SearchPanelProviderCard = ({ provider, saving, onUpdate, t }: Props) => {
  const switchId = useId();
  const [draftKey, setDraftKey] = useState('');
  const [showKey, setShowKey] = useState(false);
  const [draftBaseUrl, setDraftBaseUrl] = useState(provider.base_url ?? '');
  const testId = `search-provider-${provider.id}`;

  // A key is needed for the direct route. A provider that reports
  // `deep_research_available` (Gemini) also takes an optional key on the
  // managed route, because the key is what unlocks deep research.
  const deepResearchCapable = provider.deep_research_available !== undefined;
  const showKeyEditor =
    provider.takes_key && (provider.route === 'direct' || deepResearchCapable);
  const hasBaseUrl = provider.base_url !== undefined;

  const saveKey = async (value: string) => {
    if (await onUpdate({ api_key: value })) setDraftKey('');
  };

  return (
    <div
      data-testid={testId}
      data-status={provider.status}
      className="rounded-xl border border-line bg-surface p-3 space-y-3">
      <div className="flex items-center gap-3">
        <label htmlFor={switchId} className="flex-1 min-w-0 flex items-center gap-2">
          <span className="text-sm font-medium text-content truncate">{provider.label}</span>
          <Badge variant={STATUS_VARIANT[provider.status]} data-testid={`${testId}-status`}>
            {statusLabel(provider.status, t)}
          </Badge>
        </label>
        <Switch
          id={switchId}
          data-testid={`${testId}-toggle`}
          aria-label={withProvider(t('settings.search.providerToggleAria'), provider.label)}
          checked={provider.enabled}
          disabled={saving}
          onCheckedChange={next => void onUpdate({ enabled: next })}
        />
      </div>

      {provider.routes.length > 1 && (
        <ToggleGroupRoot
          type="single"
          aria-label={withProvider(t('settings.search.routeAria'), provider.label)}
          value={provider.route}
          onValueChange={value => {
            if (value && value !== provider.route) void onUpdate({ route: value as SearchRoute });
          }}
          disabled={saving}
          className="flex w-full rounded-lg border border-line overflow-hidden gap-0">
          {provider.routes.map(route => (
            <ToggleGroupItem
              key={route}
              value={route}
              data-testid={`${testId}-route-${route}`}
              disabled={route === 'managed' && !provider.managed_available}
              variant="tertiary"
              className="flex-1 rounded-none border-0 border-l border-line first:border-l-0 px-3 py-1.5 text-xs data-[state=on]:bg-primary-500 data-[state=on]:text-content-inverted">
              {route === 'managed'
                ? t('settings.search.routeManaged')
                : t('settings.search.routeDirect')}
            </ToggleGroupItem>
          ))}
        </ToggleGroupRoot>
      )}

      {showKeyEditor && (
        <KeyEditor
          label={withProvider(t('settings.search.apiKeyLabel'), provider.label)}
          placeholder={
            provider.key_configured
              ? t('settings.search.placeholderStored')
              : withProvider(t('settings.search.placeholderKey'), provider.label)
          }
          show={showKey}
          onToggleShow={() => setShowKey(s => !s)}
          value={draftKey}
          onChange={setDraftKey}
          onSave={() => void saveKey(draftKey)}
          onClear={() => void saveKey('')}
          configured={provider.key_configured}
          docUrl={provider.docs_url}
          disabled={saving}
          testId={`${testId}-key`}
          t={t}
        />
      )}

      {deepResearchCapable && (
        <p
          data-testid={`${testId}-deep-research`}
          className="text-[11px] text-content-muted leading-relaxed">
          {provider.deep_research_available
            ? withProvider(t('settings.search.deepResearchAvailable'), provider.label)
            : withProvider(t('settings.search.deepResearchHint'), provider.label)}
        </p>
      )}

      {hasBaseUrl && (
        <div className="space-y-1">
          <label
            htmlFor={`${switchId}-base-url`}
            className="block text-xs font-semibold text-content">
            {t('settings.search.baseUrlLabel')}
          </label>
          <div className="flex items-center gap-2">
            <Input
              id={`${switchId}-base-url`}
              data-testid={`${testId}-base-url`}
              inputSize="sm"
              value={draftBaseUrl}
              onChange={e => setDraftBaseUrl(e.target.value)}
              spellCheck={false}
              className="flex-1 min-w-0 font-mono"
            />
            <Button
              type="button"
              variant="primary"
              size="xs"
              disabled={
                saving ||
                draftBaseUrl.trim().length === 0 ||
                draftBaseUrl.trim() === (provider.base_url ?? '')
              }
              onClick={() => void onUpdate({ base_url: draftBaseUrl.trim() })}>
              {t('settings.search.baseUrlSave')}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
};

export default SearchPanelProviderCard;
