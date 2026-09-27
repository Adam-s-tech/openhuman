import { useState } from 'react';

import type { SearchSettings, SearchSettingsUpdate } from '../../../utils/tauriCommands/config';
import Button from '../../ui/Button';
import TextArea from '../../ui/TextArea';
import { ToggleGroupItem, ToggleGroupRoot } from '../../ui/ToggleGroup';

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

/**
 * Normalize a user-entered allowed-site entry down to a bare host so it
 * matches `url_guard`'s host-based comparison. Strips a leading scheme and any
 * path/query/fragment — e.g. `https://reuters.com/markets` → `reuters.com` —
 * and trims surrounding whitespace. The `*` allow-all wildcard is preserved.
 */
export const normalizeAllowedHost = (raw: string): string =>
  raw
    .trim()
    .replace(/^[a-z][a-z0-9+.-]*:\/\//i, '')
    .replace(/\/.*$/, '')
    .trim();

interface Props {
  settings: SearchSettings;
  saving: boolean;
  persist: (update: SearchSettingsUpdate) => Promise<boolean>;
  t: (key: string) => string;
}

/**
 * "Allowed websites": the host allowlist shared by web_fetch / curl and (when
 * enabled) the browser tool. Web search is not gated by this list.
 *
 * The editor text and mode are seeded from settings once, on mount, so a later
 * settings refresh (after saving a provider change) can't clobber in-progress
 * host edits or the chosen mode.
 */
const SearchPanelAllowedSites = ({ settings, saving, persist, t }: Props) => {
  const [allowedText, setAllowedText] = useState<string>(() =>
    settings.allowed_domains.filter(d => d !== '*').join('\n')
  );
  const [mode, setMode] = useState<AccessMode>(() => {
    const explicit = settings.allowed_domains.filter(d => d !== '*');
    return settings.allow_all ? 'all' : explicit.length > 0 ? 'custom' : 'block';
  });

  // "Allow all" / "Block all" persist immediately; "Custom" only reveals the
  // host editor (its Save button persists the list), keeping what was typed.
  const selectMode = (next: AccessMode) => {
    if (saving) return;
    setMode(next);
    if (next === 'all') {
      void persist({ allow_all: true });
    } else if (next === 'block') {
      void persist({ allowed_domains: [], allow_all: false });
    }
  };

  const persistAllowedDomains = () => {
    const domains = allowedText.split('\n').map(normalizeAllowedHost).filter(Boolean);
    // Editing the explicit host list implies "not allow-all".
    void persist({ allowed_domains: domains, allow_all: false });
  };

  return (
    <div className="rounded-xl border border-line bg-surface p-3 space-y-2">
      {/* Section heading, not a form label — use a <p> so screen readers
          don't announce an orphan <label> with no htmlFor. */}
      <p className="text-xs font-semibold text-content-secondary">
        {t('settings.search.allowedSitesLabel')}
      </p>
      <ToggleGroupRoot
        type="single"
        aria-label={t('settings.search.accessModeAria')}
        value={mode}
        onValueChange={value => {
          if (value) selectMode(value as AccessMode);
        }}
        disabled={saving}
        className="flex w-full rounded-lg border border-line overflow-hidden gap-0">
        {(
          [
            ['all', t('settings.search.accessAllowAll')],
            ['custom', t('settings.search.accessCustom')],
            ['block', t('settings.search.accessBlockAll')],
          ] as const
        ).map(([value, label]) => (
          <ToggleGroupItem
            key={value}
            value={value}
            variant="tertiary"
            className="flex-1 rounded-none border-0 border-l border-line first:border-l-0 px-3 py-1.5 text-xs data-[state=on]:bg-primary-500 data-[state=on]:text-content-inverted">
            {label}
          </ToggleGroupItem>
        ))}
      </ToggleGroupRoot>
      <p className="text-[11px] text-content-muted leading-relaxed">
        {mode === 'all'
          ? t('settings.search.allowedSitesAllOn')
          : mode === 'block'
            ? t('settings.search.accessBlockAllHint')
            : t('settings.search.allowedSitesHint')}
      </p>
      {mode === 'custom' && (
        <>
          <TextArea
            value={allowedText}
            onChange={e => setAllowedText(e.target.value)}
            rows={4}
            spellCheck={false}
            placeholder={t('settings.search.allowedSitesPlaceholder')}
            className="font-mono text-xs"
            aria-label={t('settings.search.allowedSitesLabel')}
          />
          <Button
            type="button"
            variant="primary"
            size="xs"
            onClick={() => persistAllowedDomains()}
            disabled={saving}>
            {t('settings.search.allowedSitesSave')}
          </Button>
        </>
      )}
    </div>
  );
};

export default SearchPanelAllowedSites;
