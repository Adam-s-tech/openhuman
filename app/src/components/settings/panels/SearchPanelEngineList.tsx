import { Ban, KeyRound, type LucideIcon, Sparkles } from 'lucide-react';

import { cn } from '../../../lib/cn';
import type { SearchEngineId } from '../../../utils/tauriCommands/config';
import Badge from '../../ui/Badge';
import { ToggleGroupItem, ToggleGroupRoot } from '../../ui/ToggleGroup';

export interface EngineOption {
  id: SearchEngineId;
  label: string;
  description: string;
  requiresKey: boolean;
}

interface SearchPanelEngineListProps {
  engines: EngineOption[];
  selectedEngine: SearchEngineId;
  ariaLabel: string;
  isConfigured: (engine: SearchEngineId) => boolean;
  onSelect: (engine: SearchEngineId) => void;
  t: (key: string) => string;
}

const engineIcon = (engine: EngineOption): LucideIcon => {
  if (engine.id === 'disabled') return Ban;
  if (engine.id === 'managed') return Sparkles;
  return KeyRound;
};

/**
 * The search-engine picker as a grid of icon tiles. A single-select
 * `ToggleGroup` gives the real `role="radiogroup"` / `role="radio"` /
 * `aria-checked` pairing (Radix sets these for `type="single"`) plus
 * roving-focus arrow-key navigation, so each tile is the radio itself.
 */
const SearchPanelEngineList = ({
  engines,
  selectedEngine,
  ariaLabel,
  isConfigured,
  onSelect,
  t,
}: SearchPanelEngineListProps) => (
  <ToggleGroupRoot
    type="single"
    aria-label={ariaLabel}
    value={selectedEngine}
    onValueChange={value => {
      if (value) onSelect(value as SearchEngineId);
    }}
    className="grid w-full gap-2 p-4 sm:grid-cols-2 xl:grid-cols-3">
    {engines.map(opt => {
      const selected = opt.id === selectedEngine;
      const configured = isConfigured(opt.id);
      const blocked = opt.requiresKey && !configured && selected;
      const Icon = engineIcon(opt);
      return (
        <ToggleGroupItem
          key={opt.id}
          value={opt.id}
          data-testid={`search-engine-${opt.id}`}
          variant="tertiary"
          className={cn(
            'h-auto w-full items-start justify-start gap-3 whitespace-normal rounded-xl border px-3.5 py-3 text-left font-normal transition-colors',
            selected
              ? 'border-primary-500 bg-primary-50 ring-1 ring-primary-500 hover:bg-primary-50 dark:bg-primary-500/10 dark:hover:bg-primary-500/10'
              : 'border-line bg-surface hover:border-line-strong hover:bg-surface-hover'
          )}>
          <span
            className={cn(
              'flex h-9 w-9 shrink-0 items-center justify-center rounded-lg',
              selected
                ? 'bg-primary-500 text-content-inverted'
                : 'bg-surface-muted text-content-secondary'
            )}>
            <Icon className="h-4.5 w-4.5" aria-hidden />
          </span>
          <span className="min-w-0 flex-1">
            <span className="flex flex-wrap items-center gap-2">
              <span className="text-sm font-semibold text-content">{opt.label}</span>
              {opt.requiresKey && (
                <Badge variant={configured ? 'success' : 'warning'}>
                  {configured
                    ? t('settings.search.statusConfigured')
                    : t('settings.search.statusNeedsKey')}
                </Badge>
              )}
            </span>
            <span className="mt-0.5 block text-xs leading-relaxed text-content-muted">
              {opt.description}
            </span>
            {blocked && (
              <span className="mt-1 block text-xs text-amber-700 dark:text-amber-300">
                {t('settings.search.fallbackToManaged')}
              </span>
            )}
          </span>
        </ToggleGroupItem>
      );
    })}
  </ToggleGroupRoot>
);

export default SearchPanelEngineList;
