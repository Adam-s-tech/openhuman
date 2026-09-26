import { useState } from 'react';

import { useT } from '../../../lib/i18n/I18nContext';
import { channelLuminance } from '../../../lib/theme/color';
import { resolveFamilyVariant } from '../../../lib/theme/presets';
import {
  ACCENT_FAMILIES,
  ACCENT_SHADES,
  COLOR_GROUPS,
  FONT_CHOICES,
  FONT_ROLES,
  fontChoiceForStack,
  type FontRole,
} from '../../../lib/theme/tokens';
import type { BackdropKind, Theme } from '../../../lib/theme/types';
import { useAppDispatch, useAppSelector } from '../../../store/hooks';
import {
  deleteCustomTheme,
  resetActiveTheme,
  resolveTheme,
  selectActiveFamilyId,
  selectActiveThemeId,
  selectCustomThemes,
  selectEffectiveTheme,
  selectThemeFamilies,
  selectThemeVariant,
  setActiveFamily,
  setActiveTheme,
  setFontRole,
  setThemeBackdrop,
  setThemeToken,
  setThemeVariant,
  type ThemeVariant,
  upsertCustomTheme,
} from '../../../store/themeSlice';
import {
  AccordionContent,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
  Alert,
  AlertDescription,
  Button,
  Card,
  TextArea,
  TextField,
  ToggleGroupItem,
  ToggleGroupRoot,
} from '../../ui';
import { SettingsSelect } from '../controls';
import SettingsPanel from '../layout/SettingsPanel';
import ColorTokenField from './theme/ColorTokenField';

/** Minimal base swatch values used only for preview tiles of built-in themes. */
const BASE_SWATCH: Record<'light' | 'dark', Record<string, string>> = {
  light: {
    'surface-canvas': '245 245 245',
    surface: '255 255 255',
    content: '23 23 23',
    'primary-500': '47 110 244',
  },
  dark: {
    'surface-canvas': '0 0 0',
    surface: '23 23 23',
    content: '245 245 245',
    'primary-500': '47 110 244',
  },
};

/** Read the live effective value of a token (override or tokens.css default). */
function readToken(key: string): string {
  if (typeof document === 'undefined') return '0 0 0';
  const v = window.getComputedStyle(document.documentElement).getPropertyValue(`--${key}`).trim();
  return v || '0 0 0';
}

function readFontRole(role: FontRole): string {
  if (typeof document === 'undefined') return '';
  return window
    .getComputedStyle(document.documentElement)
    .getPropertyValue(`--font-${role}`)
    .trim();
}

/** "surface-canvas" → "Surface canvas". */
function humanize(key: string): string {
  const s = key.replace(/-/g, ' ');
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function swatchChannels(theme: Theme, key: string): string {
  return theme.colors[key] ?? BASE_SWATCH[theme.isDark ? 'dark' : 'light'][key] ?? '128 128 128';
}

function channelsToCss(channels: string): string {
  return `rgb(${channels.trim().split(/\s+/).join(' ')} / 1)`;
}

/** Tile background: the theme's canvas gradient if any, else its flat canvas. */
function tileCanvas(theme: Theme): string {
  return theme.gradient?.canvas ?? channelsToCss(swatchChannels(theme, 'surface-canvas'));
}

/**
 * Is this a usable colour map — an object of `token -> "r g b"` strings?
 *
 * The bug this exists for (#5901): the old check was
 * `typeof parsed.colors !== 'object'`, which passes for `null` AND for an
 * array, since `typeof null` and `typeof []` are both `'object'`. Execution
 * then reached `colors: { ...(parsed.colors) }`; spreading either yields `{}`
 * silently, so a malformed paste was accepted as a theme.
 *
 * An EMPTY object is deliberately allowed. `CLASSIC_LIGHT` and `CLASSIC_DARK`
 * both carry `colors: {}` on purpose (`lib/theme/presets.ts:63-78`) — they
 * inherit the base stylesheet tokens and carry their meaning in `isDark`, which
 * `applyTheme` applies independently of any colour
 * (`providers/ThemeProvider.tsx:48-50`). The panel's own export serialises the
 * effective theme, so rejecting `{}` would break its export -> import round trip
 * for the two most common themes, and would also refuse legitimate
 * font-, gradient- or backdrop-only themes.
 *
 * Every value must be a string. `swatchChannels` falls back only on
 * `null`/`undefined` (`??`), so a non-string like `{"surface": 42}` reaches
 * `channelsToCss`, which calls `.trim()` on it and throws — crashing the panel
 * on a theme that was already stored.
 */
function isValidColorMap(colors: unknown): colors is Record<string, string> {
  if (typeof colors !== 'object' || colors === null || Array.isArray(colors)) {
    return false;
  }
  return Object.values(colors).every(value => typeof value === 'string');
}

function importedGradient(parsed: Partial<Theme>): Theme['gradient'] {
  if (!parsed.gradient || typeof parsed.gradient !== 'object') return undefined;
  return typeof parsed.gradient.canvas === 'string' ? { canvas: parsed.gradient.canvas } : {};
}

function importedBackdrop(parsed: Partial<Theme>): Theme['backdrop'] {
  if (!parsed.backdrop || typeof parsed.backdrop !== 'object') return undefined;
  const { kind } = parsed.backdrop;
  if (kind !== 'mesh' && kind !== 'solid' && kind !== 'image') return undefined;
  return {
    kind,
    imageUrl: typeof parsed.backdrop.imageUrl === 'string' ? parsed.backdrop.imageUrl : undefined,
  };
}

interface ThemeStudioPanelProps {
  /** Render the sections only — the host draws the page header. */
  embedded?: boolean;
  /**
   * Which half to render when embedded. The Appearance page puts the theme
   * gallery at the top and the customizer at the bottom, with text size and
   * language between them, so it mounts this component twice. Omit for both.
   */
  part?: 'gallery' | 'customize';
}

const ThemeStudioPanel = ({ embedded = false, part }: ThemeStudioPanelProps = {}) => {
  const { t } = useT();
  const dispatch = useAppDispatch();
  const families = selectThemeFamilies();
  const customThemes = useAppSelector(selectCustomThemes);
  const activeThemeId = useAppSelector(selectActiveThemeId);
  const activeFamilyId = useAppSelector(selectActiveFamilyId);
  const variant = useAppSelector(selectThemeVariant);
  const effectiveTheme = useAppSelector(selectEffectiveTheme);

  const isActiveCustom = customThemes.some(th => th.id === activeThemeId);
  // Which variant to render in family preview tiles (Auto → resolved OS variant).
  const previewVariant: 'light' | 'dark' = variant === 'system' ? resolveTheme('system') : variant;

  const VARIANT_OPTIONS: { id: ThemeVariant; label: string }[] = [
    { id: 'light', label: t('settings.theme.variantLight', 'Light') },
    { id: 'dark', label: t('settings.theme.variantDark', 'Dark') },
    { id: 'system', label: t('settings.theme.variantAuto', 'Auto') },
  ];
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [importText, setImportText] = useState('');
  const [importError, setImportError] = useState('');
  const [copied, setCopied] = useState(false);

  const handleExport = async () => {
    const active = customThemes.find(th => th.id === activeThemeId) ?? effectiveTheme;
    const json = JSON.stringify(active, null, 2);
    try {
      await navigator.clipboard?.writeText(json);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard blocked — the textarea below still shows the JSON to copy.
    }
  };

  const handleImport = () => {
    setImportError('');
    try {
      const parsed = JSON.parse(importText) as Partial<Theme>;
      if (!parsed || typeof parsed !== 'object' || !isValidColorMap(parsed.colors)) {
        throw new Error('shape');
      }
      const theme: Theme = {
        id: `custom-${Date.now()}`,
        name: parsed.name
          ? String(parsed.name)
          : t('settings.theme.importedName', 'Imported theme'),
        isDark: Boolean(parsed.isDark),
        builtIn: false,
        colors: { ...(parsed.colors as Record<string, string>) },
        fonts: { ...(parsed.fonts ?? {}) },
        gradient: importedGradient(parsed),
        backdrop: importedBackdrop(parsed),
      };
      dispatch(upsertCustomTheme(theme));
      setImportText('');
    } catch {
      setImportError(t('settings.theme.importError', 'Could not parse that theme JSON.'));
    }
  };

  const activeMeta = customThemes.find(th => th.id === activeThemeId);
  const exportJson = JSON.stringify(activeMeta ?? effectiveTheme, null, 2);

  // Contrast guard: warn if the editable theme's primary text on its canvas is low-contrast.
  const contrastRisk =
    isActiveCustom &&
    Math.abs(
      channelLuminance(readToken('content')) - channelLuminance(readToken('surface-canvas'))
    ) < 0.2;

  const tileClass = (selected: boolean) =>
    `flex flex-col gap-2 rounded-xl border p-3 text-left transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/25 ${
      selected ? 'border-primary-500 ring-1 ring-primary-500' : 'border-line hover:bg-surface-hover'
    }`;

  const swatch = (theme: Theme) => (
    <span
      className="flex h-10 items-center gap-1 rounded-lg px-2"
      style={{ background: tileCanvas(theme) }}>
      <span
        className="h-5 w-5 rounded-full border border-line-subtle"
        style={{ background: channelsToCss(swatchChannels(theme, 'surface')) }}
      />
      <span
        className="h-3 w-8 rounded-full"
        style={{ background: channelsToCss(swatchChannels(theme, 'content')) }}
      />
      <span
        className="ml-auto h-4 w-4 rounded-full"
        style={{ background: channelsToCss(swatchChannels(theme, 'primary-500')) }}
      />
    </span>
  );

  // ── Theme gallery: family tiles + one Light/Dark/Auto toggle ────────
  const gallery = (
    <Card
      title={t('settings.theme.presetsHeading', 'Themes')}
      padded
      divided={false}
      data-testid="theme-gallery"
      headerRight={
        <ToggleGroupRoot
          type="single"
          variant="secondary"
          size="xs"
          value={variant}
          onValueChange={next => {
            if (next) dispatch(setThemeVariant(next as ThemeVariant));
          }}
          aria-label={t('settings.theme.variantAria', 'Theme variant')}
          className="overflow-hidden rounded-lg border border-line gap-0 *:rounded-none *:border-0">
          {VARIANT_OPTIONS.map(opt => (
            <ToggleGroupItem
              key={opt.id}
              value={opt.id}
              className="h-auto px-2.5 py-1 text-xs font-medium data-[state=on]:bg-primary-500 data-[state=on]:text-content-inverted">
              {opt.label}
            </ToggleGroupItem>
          ))}
        </ToggleGroupRoot>
      }>
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
        {families.map(fam => {
          const selected = !isActiveCustom && fam.id === activeFamilyId;
          return (
            <button
              key={fam.id}
              type="button"
              aria-pressed={selected}
              onClick={() => dispatch(setActiveFamily(fam.id))}
              className={tileClass(selected)}>
              {swatch(resolveFamilyVariant(fam, previewVariant))}
              <span className="truncate text-sm font-medium text-content">{fam.name}</span>
            </button>
          );
        })}
        {customThemes.map(th => {
          const selected = th.id === activeThemeId;
          return (
            <button
              key={th.id}
              type="button"
              aria-pressed={selected}
              onClick={() => dispatch(setActiveTheme(th.id))}
              className={tileClass(selected)}>
              {swatch(th)}
              <span className="flex items-center justify-between gap-1">
                <span className="truncate text-sm font-medium text-content">{th.name}</span>
                <span className="text-[11px] text-content-faint">
                  {t('settings.theme.customBadge', 'Custom')}
                </span>
              </span>
            </button>
          );
        })}
      </div>
    </Card>
  );

  const colorFields = (keys: readonly string[], label: (key: string) => string) => (
    <div className="divide-y divide-line-subtle">
      {keys.map(key => (
        <ColorTokenField
          key={key}
          tokenKey={key}
          label={label(key)}
          value={effectiveTheme.colors[key] ?? readToken(key)}
          disabled={false}
          onChange={channels => dispatch(setThemeToken({ key, value: channels }))}
        />
      ))}
    </div>
  );

  // ── Customizer: every fine-grained control, collapsed by default ─────
  // These used to be eight always-open cards (four colour groups of up to a
  // dozen rows each, fonts, background, import) stacked above the font size
  // and language settings, so the everyday controls sat a long scroll down.
  const customize = (
    <section className="space-y-2" data-testid="theme-customize">
      {!isActiveCustom && (
        <p className="px-1 text-xs text-content-muted">
          {t(
            'settings.theme.autoForkHint',
            'Editing a preset automatically saves your changes as a new custom theme.'
          )}
        </p>
      )}

      {isActiveCustom && contrastRisk && (
        <Alert variant="warning" density="compact">
          <AlertDescription>
            {t(
              'settings.theme.contrastWarn',
              'Low contrast between text and background — this theme may be hard to read.'
            )}
          </AlertDescription>
        </Alert>
      )}

      <AccordionRoot type="multiple" variant="contained">
        {COLOR_GROUPS.map(group => (
          <AccordionItem key={group.id} value={`colors-${group.id}`}>
            <AccordionTrigger>{t(group.i18nKey, humanize(group.id))}</AccordionTrigger>
            <AccordionContent>
              {colorFields(group.keys, humanize)}
              {group.id === 'accents' && (
                <div className="pt-2">
                  <Button variant="tertiary" size="xs" onClick={() => setShowAdvanced(v => !v)}>
                    {showAdvanced
                      ? t('settings.theme.hideShades', 'Hide all accent shades')
                      : t('settings.theme.showShades', 'Show all accent shades')}
                  </Button>
                  {showAdvanced &&
                    ACCENT_FAMILIES.map(fam => (
                      <div key={fam} className="pt-3">
                        <div className="text-xs font-semibold text-content-muted">
                          {humanize(fam)}
                        </div>
                        {colorFields(
                          ACCENT_SHADES.map(shade => `${fam}-${shade}`),
                          humanize
                        )}
                      </div>
                    ))}
                </div>
              )}
            </AccordionContent>
          </AccordionItem>
        ))}

        <AccordionItem value="fonts">
          <AccordionTrigger>{t('settings.theme.fontsHeading', 'Fonts')}</AccordionTrigger>
          <AccordionContent>
            <div className="space-y-2">
              {FONT_ROLES.map(role => {
                const current = fontChoiceForStack(
                  effectiveTheme.fonts[role] ?? readFontRole(role)
                );
                return (
                  <div key={role} className="flex items-center justify-between gap-3">
                    <span className="text-sm text-content">
                      {t(`settings.theme.fontRole.${role}`, humanize(role))}
                    </span>
                    <SettingsSelect
                      inputSize="sm"
                      value={current?.id ?? '__current__'}
                      disabled={false}
                      aria-label={t(`settings.theme.fontRole.${role}`, humanize(role))}
                      onChange={e => {
                        const choice = FONT_CHOICES.find(c => c.id === e.target.value);
                        if (choice) dispatch(setFontRole({ role, stack: choice.stack }));
                      }}>
                      {!current && (
                        <option value="__current__" disabled>
                          {t('settings.theme.fontCurrent', 'Current')}
                        </option>
                      )}
                      {FONT_CHOICES.map(c => (
                        <option key={c.id} value={c.id}>
                          {c.label}
                        </option>
                      ))}
                    </SettingsSelect>
                  </div>
                );
              })}
            </div>
          </AccordionContent>
        </AccordionItem>

        <AccordionItem value="background">
          <AccordionTrigger>{t('settings.theme.backdropHeading', 'Background')}</AccordionTrigger>
          <AccordionContent>
            <div className="space-y-2">
              <ToggleGroupRoot
                type="single"
                variant="secondary"
                size="xs"
                value={effectiveTheme.backdrop?.kind ?? 'solid'}
                onValueChange={next => {
                  if (next)
                    dispatch(
                      setThemeBackdrop({
                        kind: next as BackdropKind,
                        imageUrl: effectiveTheme.backdrop?.imageUrl,
                      })
                    );
                }}
                aria-label={t('settings.theme.backdropHeading', 'Background')}
                className="overflow-hidden rounded-lg border border-line gap-0 *:rounded-none *:border-0">
                {(['mesh', 'solid', 'image'] as BackdropKind[]).map(kind => (
                  <ToggleGroupItem
                    key={kind}
                    value={kind}
                    className="h-auto px-3 py-1.5 text-xs font-medium data-[state=on]:bg-primary-500 data-[state=on]:text-content-inverted">
                    {t(`settings.theme.backdrop.${kind}`, kind)}
                  </ToggleGroupItem>
                ))}
              </ToggleGroupRoot>
              {effectiveTheme.backdrop?.kind === 'image' && (
                <TextField
                  type="url"
                  inputSize="sm"
                  disabled={false}
                  value={effectiveTheme.backdrop?.imageUrl ?? ''}
                  placeholder="https://…/background.jpg"
                  aria-label={t('settings.theme.backdropImageUrl', 'Background image URL')}
                  onChange={e =>
                    dispatch(setThemeBackdrop({ kind: 'image', imageUrl: e.target.value }))
                  }
                  className="text-xs"
                />
              )}
              <p className="text-[11px] text-content-faint">
                {t(
                  'settings.theme.backdropHint',
                  'Mesh shows the animated gradient; Solid uses a flat background; Image paints your own.'
                )}
              </p>
            </div>
          </AccordionContent>
        </AccordionItem>

        {isActiveCustom && (
          <AccordionItem value="manage">
            <AccordionTrigger>{t('settings.theme.actions', 'Manage theme')}</AccordionTrigger>
            <AccordionContent>
              <div className="flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" onClick={() => dispatch(resetActiveTheme())}>
                  {t('settings.theme.reset', 'Reset overrides')}
                </Button>
                <Button
                  variant="secondary"
                  tone="danger"
                  size="sm"
                  onClick={() => dispatch(deleteCustomTheme(activeThemeId))}>
                  {t('settings.theme.delete', 'Delete theme')}
                </Button>
                <Button variant="secondary" size="sm" onClick={handleExport}>
                  {copied
                    ? t('settings.theme.copied', 'Copied!')
                    : t('settings.theme.export', 'Copy JSON')}
                </Button>
              </div>
              <TextArea
                readOnly
                value={exportJson}
                rows={4}
                aria-label={t('settings.theme.export', 'Copy JSON')}
                className="mt-2 resize-none bg-surface-muted p-2 font-mono text-[11px] text-content-secondary"
              />
            </AccordionContent>
          </AccordionItem>
        )}

        <AccordionItem value="import">
          <AccordionTrigger>{t('settings.theme.import', 'Import theme')}</AccordionTrigger>
          <AccordionContent>
            <div className="space-y-2">
              <p className="text-xs text-content-muted">
                {t(
                  'settings.theme.importHint',
                  'Paste exported theme JSON to add it as a custom theme.'
                )}
              </p>
              <TextArea
                value={importText}
                onChange={e => setImportText(e.target.value)}
                rows={4}
                placeholder='{ "name": "...", "isDark": false, "colors": { ... } }'
                aria-label={t('settings.theme.import', 'Import theme')}
                className="resize-none p-2 font-mono text-[11px]"
              />
              {importError && (
                <p className="text-xs text-coral-600 dark:text-coral-300">{importError}</p>
              )}
              <Button size="sm" onClick={handleImport} disabled={!importText.trim()}>
                {t('settings.theme.importApply', 'Import')}
              </Button>
            </div>
          </AccordionContent>
        </AccordionItem>
      </AccordionRoot>
    </section>
  );

  const body =
    part === 'gallery' ? (
      gallery
    ) : part === 'customize' ? (
      customize
    ) : (
      <>
        {gallery}
        {customize}
      </>
    );

  // Embedded: the Appearance page owns the header and renders these sections
  // among its own. That is the only host today — `/settings/theme` redirects to
  // `/settings/appearance`, because a separate "Theme studio" page split one
  // subject across two sidebar rows whose light/dark toggles wrote the same two
  // slice fields (`setThemeMode` and `setThemeVariant` are identical). The
  // unembedded branch is kept for a standalone host.
  if (embedded) return body;

  return (
    <SettingsPanel description={t('settings.theme.menuDesc', 'Customize colours and fonts.')}>
      {body}
    </SettingsPanel>
  );
};

export default ThemeStudioPanel;
